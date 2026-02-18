//! Reactive runtime with dependency tracking.

use std::any::Any;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use std::thread;

/// Unique identifier for reactive nodes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "nova", derive(serde::Serialize, serde::Deserialize))]
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

    // Reusable buffer for graph traversal to avoid allocations
    traversal_buffer: Vec<NodeId>,
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

        // If it's an effect, schedule it.
        // Optimization: We don't need to check `!self.pending_effects.contains(&id)`
        // because the `stale` check above guarantees we only enter this block once
        // per update cycle for a given effect. If it was already pending, it would
        // be in `stale`, and we would have returned early.
        if self.effects.contains_key(&id) {
            self.pending_effects.push(id);
        }

        // We should recurse for computed values
        self.computeds.contains_key(&id)
    }

    fn mark_subscribers_stale(&mut self, source: NodeId) {
        // Optimization: Avoid allocating intermediate Vecs by iterating HashSet refs directly.
        // We use a reusable buffer for DFS traversal to avoid repeated allocations.
        self.traversal_buffer.clear();

        if let Some(subs) = self.subscribers.get(&source) {
            self.traversal_buffer.extend(subs);
        }

        while let Some(node) = self.traversal_buffer.pop() {
            // mark_stale borrows &mut self, but returns bool.
            // The borrow ends after the if condition check.
            #[allow(clippy::collapsible_if)]
            if self.mark_stale(node) {
                if let Some(subs) = self.subscribers.get(&node) {
                    self.traversal_buffer.extend(subs);
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

    fn prepare_execution(&mut self, id: NodeId) -> Result<(), String> {
        self.cleanup_dependencies(id);
        self.stale.remove(&id);
        self.push_context(id)
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
                traversal_buffer: Vec::new(),
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

    fn run_with_context<R>(&self, id: NodeId, f: impl FnOnce() -> R) -> R {
        {
            let mut inner = self.inner.lock().unwrap();
            if let Err(e) = inner.prepare_execution(id) {
                // Drop lock before panicking to prevent mutex poisoning,
                // which would cause double-panics during unwinding cleanup.
                drop(inner);
                panic!("{}", e);
            }
        }

        // SAFETY: The context guard ensures that `pop_context` is called
        // even if the closure panics.
        let _guard = ContextGuard { runtime: self, id };

        f()
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
        let effect_fn = {
            let inner = self.inner.lock().unwrap();
            inner.effects.get(&id).cloned()
        };

        // We run in context regardless of whether the effect exists,
        // because we need to clear dependencies for zombie nodes.
        // run_with_context handles prepare_execution (and cleaning deps).
        self.run_with_context(id, || {
            if let Some(f) = effect_fn {
                f();
            }
        });
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
        let compute_fn = {
            let inner = self.inner.lock().unwrap();
            inner
                .computeds
                .get(&id)
                .unwrap_or_else(|| panic!("Computed not found for id {:?}", id))
                .compute
                .clone()
        };

        // Run computation in context
        let new_value = self.run_with_context(id, || compute_fn());

        // Store new value
        {
            let mut inner = self.inner.lock().unwrap();
            if let Some(computed) = inner.computeds.get_mut(&id) {
                computed.value = Some(new_value);
            }
        }
    }

    pub(crate) fn dispose_effect(&self, id: NodeId) {
        let mut inner = self.inner.lock().unwrap();

        // Remove from effects
        inner.effects.remove(&id);

        // Clean up dependencies
        inner.cleanup_dependencies(id);
    }
}

#[cfg(feature = "nova")]
#[derive(Debug, Clone)]
#[cfg_attr(feature = "nova", derive(serde::Serialize, serde::Deserialize))]
pub struct GraphSnapshot {
    pub nodes: Vec<NodeInfo>,
    pub dependencies: Vec<(NodeId, NodeId)>,
    pub stale_nodes: Vec<NodeId>,
}

#[cfg(feature = "nova")]
#[derive(Debug, Clone)]
#[cfg_attr(feature = "nova", derive(serde::Serialize, serde::Deserialize))]
pub struct NodeInfo {
    pub id: NodeId,
    pub node_type: NodeType,
    pub label: String,
}

#[cfg(feature = "nova")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "nova", derive(serde::Serialize, serde::Deserialize))]
pub enum NodeType {
    Signal,
    Computed,
    Effect,
}

#[cfg(feature = "nova")]
impl Runtime {
    /// Snapshots the current dependency graph for debugging/devtools.
    pub fn inspect_graph(&self) -> GraphSnapshot {
        let inner = self.inner.lock().unwrap();
        let mut nodes = Vec::new();
        let mut dependencies = Vec::new();
        let mut stale_nodes = Vec::new();

        // Collect Signals
        for id in inner.signals.keys() {
            nodes.push(NodeInfo {
                id: *id,
                node_type: NodeType::Signal,
                label: format!("Signal({:?})", id),
            });
        }

        // Collect Computeds
        for id in inner.computeds.keys() {
            nodes.push(NodeInfo {
                id: *id,
                node_type: NodeType::Computed,
                label: format!("Computed({:?})", id),
            });
        }

        // Collect Effects
        for id in inner.effects.keys() {
            nodes.push(NodeInfo {
                id: *id,
                node_type: NodeType::Effect,
                label: format!("Effect({:?})", id),
            });
        }

        // Collect Dependencies
        // dependencies: node -> its dependencies.
        // We want to represent edges as (Subscriber, Source) or (Source, Subscriber)?
        // Usually graph is (Source, Target).
        // inner.dependencies maps Observer -> {Sources}.
        // So Source -> Observer is the flow of data.
        for (observer, sources) in &inner.dependencies {
            for source in sources {
                // Edge from Source to Observer
                dependencies.push((*source, *observer));
            }
        }

        // Collect Stale Nodes
        for id in &inner.stale {
            stale_nodes.push(*id);
        }

        GraphSnapshot {
            nodes,
            dependencies,
            stale_nodes,
        }
    }
}
