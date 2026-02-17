//! Reactive runtime with dependency tracking.

use std::any::Any;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use std::thread;

/// Unique identifier for reactive nodes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(pub u64);

/// The reactive runtime - manages the dependency graph.
///
/// The `Runtime` is the heart of the reactivity system. It maintains:
/// - Storage for signals, computeds, and effects
/// - The dependency graph (who depends on whom)
/// - The current execution context (which effect/computed is running)
///
/// It is typically shared via `Arc<Runtime>` between all reactive primitives.
///
/// # Thread Safety
///
/// `Runtime` is thread-safe (`Send + Sync`) and uses internal `Mutex` locking
/// to allow signals to be read/written from any thread.
///
/// # Example
///
/// ```
/// use flux_state::Runtime;
/// use std::sync::Arc;
///
/// let runtime = Runtime::new();
/// // Pass runtime.clone() to signals/effects
/// ```
///
/// # Limits
///
/// To prevent stack overflows from infinite reactive loops (e.g., Effect A triggers Signal B,
/// which triggers Effect A), the runtime enforces a **recursion limit of 100**.
///
/// If this limit is exceeded, the runtime will **panic** with a descriptive message.
///
/// This limit applies to the depth of the dependency chain (e.g., Computed A -> Computed B -> ...).
pub struct Runtime {
    inner: Mutex<RuntimeInner>,
}

struct RuntimeInner {
    next_id: u64,

    // Signal storage
    signals: HashMap<NodeId, Arc<dyn Any + Send + Sync>>,

    // Computed storage
    computeds: HashMap<NodeId, ComputedNode>,

    // Effect storage
    effects: HashMap<NodeId, std::sync::Arc<dyn Fn() + Send + Sync>>,

    // Dependency graph
    dependencies: HashMap<NodeId, HashSet<NodeId>>, // node -> its dependencies
    subscribers: HashMap<NodeId, HashSet<NodeId>>,  // node -> nodes that depend on it

    // Current tracking context (per thread stack)
    tracking_context: HashMap<thread::ThreadId, Vec<NodeId>>,

    // Stale tracking
    stale: HashSet<NodeId>,

    // Pending effects to run
    pending_effects: Vec<NodeId>,
}

struct ComputedNode {
    compute: std::sync::Arc<dyn Fn() -> Arc<dyn Any + Send + Sync> + Send + Sync>,
    value: Option<Arc<dyn Any + Send + Sync>>,
}

struct ContextGuard<'a> {
    runtime: &'a Runtime,
    id: NodeId,
}

impl<'a> Drop for ContextGuard<'a> {
    fn drop(&mut self) {
        // Handle poisoned mutex gracefully to avoid double panic during unwinding
        let mut inner = match self.runtime.inner.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        inner.pop_context();

        // If we are unwinding due to a panic, restore the stale status of the node.
        // This ensures that the node is recomputed on the next access, rather than
        // being left in a zombie state with no dependencies.
        if std::thread::panicking() {
            inner.stale.insert(self.id);
        }
    }
}

impl RuntimeInner {
    fn cleanup_dependencies(&mut self, id: NodeId) {
        if let Some(deps) = self.dependencies.remove(&id) {
            for dep in deps {
                if let Some(subs) = self.subscribers.get_mut(&dep) {
                    subs.remove(&id);
                }
            }
        }
    }

    fn mark_stale(&mut self, id: NodeId) -> bool {
        // Only process if not already stale (avoid infinite loops)
        if self.stale.contains(&id) {
            return false;
        }

        self.stale.insert(id);

        // If it's an effect, schedule it
        if self.effects.contains_key(&id) && !self.pending_effects.contains(&id) {
            self.pending_effects.push(id);
        }

        // We should recurse for computed values
        self.computeds.contains_key(&id)
    }

    fn mark_subscribers_stale(&mut self, source: NodeId) {
        // Optimization: Avoid allocating intermediate Vecs by iterating HashSet refs directly.
        // We use a stack for DFS traversal.
        let mut stack = Vec::new();

        if let Some(subs) = self.subscribers.get(&source) {
            stack.extend(subs);
        }

        while let Some(node) = stack.pop() {
            // mark_stale borrows &mut self, but returns bool.
            // The borrow ends after the if condition check.
            #[allow(clippy::collapsible_if)]
            if self.mark_stale(node) {
                if let Some(subs) = self.subscribers.get(&node) {
                    stack.extend(subs);
                }
            }
        }
    }

    fn push_context(&mut self, id: NodeId) -> Result<(), String> {
        let stack = self
            .tracking_context
            .entry(thread::current().id())
            .or_default();

        // Prevent stack overflow from infinite recursion
        if stack.len() >= 100 {
            return Err(
                "Reactive recursion limit exceeded (100). Infinite loop in effects?".to_string(),
            );
        }

        stack.push(id);
        Ok(())
    }

    fn pop_context(&mut self) {
        if let Some(stack) = self.tracking_context.get_mut(&thread::current().id()) {
            stack.pop();
            if stack.is_empty() {
                self.tracking_context.remove(&thread::current().id());
            }
        }
    }

    fn current_context(&self) -> Option<NodeId> {
        self.tracking_context
            .get(&thread::current().id())
            .and_then(|stack| stack.last().copied())
    }
}

impl Runtime {
    /// Create a new reactive runtime.
    ///
    /// Returns an `Arc<Runtime>` because the runtime must be shared between
    /// all signals, effects, and computed values it manages.
    pub fn new() -> std::sync::Arc<Self> {
        std::sync::Arc::new(Self {
            inner: Mutex::new(RuntimeInner {
                next_id: 0,
                signals: HashMap::new(),
                computeds: HashMap::new(),
                effects: HashMap::new(),
                dependencies: HashMap::new(),
                subscribers: HashMap::new(),
                tracking_context: HashMap::new(),
                stale: HashSet::new(),
                pending_effects: Vec::new(),
            }),
        })
    }

    pub(crate) fn create_signal(&self, value: Arc<dyn Any + Send + Sync>) -> NodeId {
        let mut inner = self.inner.lock().unwrap();
        let id = NodeId(inner.next_id);
        inner.next_id += 1;
        inner.signals.insert(id, value);
        id
    }

    pub(crate) fn create_computed(
        &self,
        compute: std::sync::Arc<dyn Fn() -> Arc<dyn Any + Send + Sync> + Send + Sync>,
    ) -> NodeId {
        let mut inner = self.inner.lock().unwrap();
        let id = NodeId(inner.next_id);
        inner.next_id += 1;
        inner.computeds.insert(
            id,
            ComputedNode {
                compute,
                value: None,
            },
        );
        id
    }

    pub(crate) fn create_effect(
        &self,
        effect_fn: std::sync::Arc<dyn Fn() + Send + Sync>,
    ) -> NodeId {
        let mut inner = self.inner.lock().unwrap();
        let id = NodeId(inner.next_id);
        inner.next_id += 1;
        inner.effects.insert(id, effect_fn);
        id
    }

    /// Track a dependency (called during signal/computed reads).
    pub(crate) fn track(&self, source: NodeId) {
        let mut inner = self.inner.lock().unwrap();
        if let Some(observer) = inner.current_context() {
            inner
                .dependencies
                .entry(observer)
                .or_default()
                .insert(source);
            inner
                .subscribers
                .entry(source)
                .or_default()
                .insert(observer);
        }
    }

    /// Notify subscribers that a source changed.
    pub(crate) fn notify(&self, source: NodeId) {
        // Mark all transitive subscribers as stale
        self.inner.lock().unwrap().mark_subscribers_stale(source);

        // Flush pending effects (synchronous for now)
        self.flush_effects();
    }

    fn flush_effects(&self) {
        loop {
            let effect_id = {
                let mut inner = self.inner.lock().unwrap();
                inner.pending_effects.pop()
            };

            match effect_id {
                Some(id) => self.run_effect(id),
                None => break,
            }
        }
    }

    pub(crate) fn run_effect(&self, id: NodeId) {
        // Clear old dependencies and set tracking context
        let (effect_fn, error) = {
            let mut inner = self.inner.lock().unwrap();
            inner.cleanup_dependencies(id);
            inner.stale.remove(&id);
            match inner.push_context(id) {
                Ok(_) => (inner.effects.get(&id).cloned(), None),
                Err(e) => (None, Some(e)),
            }
        };

        if let Some(e) = error {
            panic!("{}", e);
        }

        // SAFETY: The context guard ensures that `pop_context` is called
        // even if the effect closure panics.
        let _guard = ContextGuard { runtime: self, id };

        // Run effect (will re-establish dependencies)
        if let Some(f) = effect_fn {
            f();
        }

        // _guard drops here, calling pop_context()
    }

    /// Get a handle to the signal value (Arc) without holding the runtime lock.
    ///
    /// # Panics
    ///
    /// Panics if the signal does not exist.
    pub(crate) fn get_signal_handle(&self, id: NodeId) -> Arc<dyn Any + Send + Sync> {
        let inner = self.inner.lock().unwrap();
        inner
            .signals
            .get(&id)
            .cloned()
            .unwrap_or_else(|| panic!("Signal not found for id {:?}", id))
    }

    /// Get a handle to the computed value (Arc) without holding the runtime lock.
    ///
    /// # Panics
    ///
    /// Panics if the computed value does not exist or has not been initialized.
    pub(crate) fn get_computed_handle(&self, id: NodeId) -> Arc<dyn Any + Send + Sync> {
        let inner = self.inner.lock().unwrap();
        let computed = inner
            .computeds
            .get(&id)
            .unwrap_or_else(|| panic!("Computed not found for id {:?}", id));
        computed
            .value
            .clone()
            .unwrap_or_else(|| panic!("Computed value not initialized for id {:?}", id))
    }

    pub(crate) fn is_stale(&self, id: NodeId) -> bool {
        self.inner.lock().unwrap().stale.contains(&id)
    }

    /// Recompute the value of a computed node.
    ///
    /// # Panics
    ///
    /// Panics if the computed node does not exist.
    pub(crate) fn recompute(&self, id: NodeId) {
        // Clear old dependencies and set tracking context, then get compute function
        let (compute_fn, error) = {
            let mut inner = self.inner.lock().unwrap();
            inner.cleanup_dependencies(id);
            inner.stale.remove(&id);

            match inner.push_context(id) {
                Ok(_) => {
                    let computed = inner
                        .computeds
                        .get(&id)
                        .unwrap_or_else(|| panic!("Computed not found for id {:?}", id));
                    (Some(computed.compute.clone()), None)
                }
                Err(e) => (None, Some(e)),
            }
        };

        if let Some(e) = error {
            panic!("{}", e);
        }

        let compute_fn = compute_fn.unwrap();

        // SAFETY: The context guard ensures that `pop_context` is called
        // even if the compute closure panics.
        let _guard = ContextGuard { runtime: self, id };

        // Call without holding borrow
        let new_value = compute_fn();

        // Store new value
        {
            let mut inner = self.inner.lock().unwrap();
            if let Some(computed) = inner.computeds.get_mut(&id) {
                computed.value = Some(new_value);
            }
        }

        // _guard drops here, calling pop_context()
    }

    pub(crate) fn dispose_effect(&self, id: NodeId) {
        let mut inner = self.inner.lock().unwrap();

        // Remove from effects
        inner.effects.remove(&id);

        // Clean up dependencies
        inner.cleanup_dependencies(id);
    }
}
