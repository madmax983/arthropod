//! Reactive runtime with dependency tracking.

use std::any::Any;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Condvar, Mutex};
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
/// // Runtime::new() returns an Arc<Runtime>
/// let runtime = Runtime::new();
///
/// // Pass runtime.clone() to signals/effects.
/// // See Signal::new() for more details.
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
    condvar: Condvar,
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

    // Nodes currently being computed (to prevent concurrent recomputation)
    // Maps NodeId -> ThreadId of the thread computing it.
    computing: HashMap<NodeId, std::thread::ThreadId>,

    // Tracks which node a thread is waiting for (for deadlock detection).
    waiting_for: HashMap<std::thread::ThreadId, NodeId>,

    // Pending effects to run
    pending_effects: Vec<NodeId>,

    // Reusable buffer to avoid allocations during effect flushing
    spare_pending_effects: Vec<NodeId>,

    // Reusable buffer for graph traversal to avoid allocations
    traversal_buffer: Vec<NodeId>,

    #[cfg(feature = "nova")]
    labels: HashMap<NodeId, String>,
}

/// Helper struct to ensure pending effects are restored if a panic occurs during flush.
struct PanicRestorer<'a> {
    runtime: &'a Runtime,
    remaining_effects: &'a mut Vec<NodeId>,
}

impl<'a> Drop for PanicRestorer<'a> {
    fn drop(&mut self) {
        if std::thread::panicking() {
            // Restore unexecuted effects to the pending queue so they aren't lost.
            // If empty, extend_from_slice does nothing, avoiding an extra conditional branch.
            let mut inner = match self.runtime.inner.lock() {
                Ok(guard) => guard,
                Err(poisoned) => poisoned.into_inner(),
            };
            inner
                .pending_effects
                .extend_from_slice(self.remaining_effects);
        }
    }
}

struct ComputingGuard<'a> {
    runtime: &'a Runtime,
    id: NodeId,
}

impl<'a> Drop for ComputingGuard<'a> {
    fn drop(&mut self) {
        if std::thread::panicking() {
            let mut inner = match self.runtime.inner.lock() {
                Ok(g) => g,
                Err(p) => p.into_inner(),
            };
            inner.computing.remove(&self.id);
            self.runtime.condvar.notify_all();
        }
    }
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
            // Only restore stale for Computeds. Effects should be left "clean" (not stale)
            // so they can be re-triggered by dependencies. If we mark an Effect stale here,
            // mark_stale() will skip it in the future (thinking it's already scheduled),
            // effectively creating a zombie effect that never runs again.
            if inner.computeds.contains_key(&self.id) {
                inner.stale.insert(self.id);
            }
        }
    }
}

impl RuntimeInner {
    fn check_recursion(&self, id: NodeId) -> bool {
        self.tracking_context
            .get(&thread::current().id())
            .is_some_and(|stack| stack.contains(&id))
    }

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
            if self.mark_stale(node)
                && let Some(subs) = self.subscribers.get(&node)
            {
                self.traversal_buffer.extend(subs);
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

    fn prepare_execution(&mut self, id: NodeId, keep_stale: bool) -> Result<(), String> {
        self.cleanup_dependencies(id);
        if !keep_stale {
            self.stale.remove(&id);
        }
        self.push_context(id)
    }

    fn take_pending_effects(&mut self, buffer: &mut Vec<NodeId>) -> bool {
        if self.pending_effects.is_empty() {
            // Donate our buffer back to the runtime if it has capacity
            // and the runtime's spare buffer is smaller.
            if buffer.capacity() > self.spare_pending_effects.capacity() {
                self.spare_pending_effects = std::mem::take(buffer);
            }
            return false;
        }

        // Swap out pending_effects with an empty buffer.
        // Prefer reusing spare_pending_effects if available.
        let empty_buf = if self.spare_pending_effects.capacity() > 0 {
            std::mem::take(&mut self.spare_pending_effects)
        } else {
            std::mem::take(buffer)
        };

        // buffer gets the full buffer, pending_effects gets the empty one
        *buffer = std::mem::replace(&mut self.pending_effects, empty_buf);
        true
    }

    fn detect_deadlock(
        &self,
        target_node: NodeId,
        current_thread: std::thread::ThreadId,
    ) -> Result<(), String> {
        let mut current_target = target_node;

        // Trace the dependency chain: Me -> Node -> Owner -> WaitingFor -> Node...
        loop {
            // Who owns the lock for the target node?
            if let Some(owner_thread) = self.computing.get(&current_target) {
                if *owner_thread == current_thread {
                    // Cycle detected!
                    // We are waiting for a node that is ultimately held by us (or a chain leading to us).
                    return Err(
                        "Deadlock detected: Cyclic dependency in computed values across threads."
                            .to_string(),
                    );
                }

                // What is that thread waiting for?
                if let Some(next_node) = self.waiting_for.get(owner_thread) {
                    current_target = *next_node;
                    continue;
                }
            }
            // Chain ends (owner is running but not waiting)
            break;
        }
        Ok(())
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
                computing: HashMap::new(),
                waiting_for: HashMap::new(),
                pending_effects: Vec::new(),
                spare_pending_effects: Vec::new(),
                traversal_buffer: Vec::new(),
                #[cfg(feature = "nova")]
                labels: HashMap::new(),
            }),
            condvar: Condvar::new(),
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

    fn run_with_context<R>(&self, id: NodeId, keep_stale: bool, f: impl FnOnce() -> R) -> R {
        {
            let mut inner = self.inner.lock().unwrap();
            if let Err(e) = inner.prepare_execution(id, keep_stale) {
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
        // Optimization: Process effects in batches to reduce lock contention.
        // Instead of locking for every single effect (N locks), we lock once
        // to grab all pending effects, then run them.
        let mut local_effects = Vec::new();

        while self
            .inner
            .lock()
            .unwrap()
            .take_pending_effects(&mut local_effects)
        {
            self.process_effect_batch(&mut local_effects);
        }
    }

    fn process_effect_batch(&self, batch: &mut Vec<NodeId>) {
        // Process batch in reverse order (LIFO) to match original behavior.
        // Note: run_effect() might trigger more effects recursively via notify(),
        // or if run from another thread, pending_effects might be populated again.
        // The outer loop handles these cases.
        let restorer = PanicRestorer {
            runtime: self,
            remaining_effects: batch,
        };

        while let Some(id) = restorer.remaining_effects.pop() {
            self.run_effect(id);
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
        self.run_with_context(id, false, || {
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
    #[allow(dead_code)]
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

    /// Combined operation to track a dependency and get the signal handle in one lock.
    #[allow(dead_code)]
    pub(crate) fn track_and_get_signal(&self, id: NodeId) -> Arc<dyn Any + Send + Sync> {
        let mut inner = self.inner.lock().unwrap();
        if let Some(observer) = inner.current_context() {
            inner.dependencies.entry(observer).or_default().insert(id);
            inner.subscribers.entry(id).or_default().insert(observer);
        }
        inner
            .signals
            .get(&id)
            .cloned()
            .unwrap_or_else(|| panic!("Signal not found for id {:?}", id))
    }

    /// Combined operation to track a dependency and get the computed handle if fresh.
    /// Returns None if the value is stale or uninitialized, indicating recompute is needed.
    pub(crate) fn track_and_get_computed_if_fresh(
        &self,
        id: NodeId,
    ) -> Option<Arc<dyn Any + Send + Sync>> {
        let mut inner = self.inner.lock().unwrap();

        if let Some(observer) = inner.current_context() {
            inner.dependencies.entry(observer).or_default().insert(id);
            inner.subscribers.entry(id).or_default().insert(observer);
        }

        if inner.stale.contains(&id) {
            return None;
        }

        let computed = inner
            .computeds
            .get(&id)
            .unwrap_or_else(|| panic!("Computed not found for id {:?}", id));
        computed.value.clone()
    }

    /// Get the computed handle if fresh, without tracking.
    pub(crate) fn get_computed_if_fresh(&self, id: NodeId) -> Option<Arc<dyn Any + Send + Sync>> {
        let inner = self.inner.lock().unwrap();

        if inner.stale.contains(&id) {
            return None;
        }

        let computed = inner
            .computeds
            .get(&id)
            .unwrap_or_else(|| panic!("Computed not found for id {:?}", id));
        computed.value.clone()
    }

    pub(crate) fn is_stale(&self, id: NodeId) -> bool {
        self.inner.lock().unwrap().stale.contains(&id)
    }

    fn wait_for_computation<'a>(
        &'a self,
        mut inner: std::sync::MutexGuard<'a, RuntimeInner>,
        id: NodeId,
    ) -> std::sync::MutexGuard<'a, RuntimeInner> {
        // Wait if currently computing (prevent concurrent recomputation)
        while inner.computing.contains_key(&id) {
            // Deadlock detection
            let current_thread = std::thread::current().id();
            if let Err(e) = inner.detect_deadlock(id, current_thread) {
                // Unlock mutex before panicking to avoid poisoning other threads
                drop(inner);
                panic!("{}", e);
            }

            // Register that we are waiting
            inner.waiting_for.insert(current_thread, id);
            inner = self.condvar.wait(inner).unwrap();
            inner.waiting_for.remove(&current_thread);
        }
        inner
    }

    fn finish_computation(&self, id: NodeId, new_value: Arc<dyn Any + Send + Sync>) {
        let mut inner = self.inner.lock().unwrap();
        if let Some(computed) = inner.computeds.get_mut(&id) {
            computed.value = Some(new_value);
        }
        inner.stale.remove(&id);
        inner.computing.remove(&id);
        self.condvar.notify_all();
    }

    /// Recompute the value of a computed node.
    ///
    /// # Panics
    ///
    /// Panics if the computed node does not exist.
    pub(crate) fn recompute(&self, id: NodeId) {
        let compute_fn = {
            let mut inner = self.inner.lock().unwrap();

            // Check for recursion (cycle detection) - return stale value if we are already computing this
            if inner.check_recursion(id) {
                return;
            }

            // Wait if currently computing (prevent concurrent recomputation)
            inner = self.wait_for_computation(inner, id);

            let (is_uninit, compute) = {
                let computed = inner
                    .computeds
                    .get(&id)
                    .unwrap_or_else(|| panic!("Computed not found for id {:?}", id));
                (computed.value.is_none(), computed.compute.clone())
            };

            let is_stale = inner.stale.contains(&id);

            // Check if still stale (someone else might have recomputed it while we waited)
            if !is_stale && !is_uninit {
                return;
            }

            // Mark as computing
            inner.computing.insert(id, std::thread::current().id());

            compute
        };

        // Guard to ensure `computing` is cleaned up if panic occurs
        let _guard = ComputingGuard { runtime: self, id };

        // Run computation in context, keeping stale flag until we are done
        // so concurrent readers see it as stale and wait on computing.
        let new_value = self.run_with_context(id, true, || compute_fn());

        // Store new value and clear stale/computing
        self.finish_computation(id, new_value);
    }

    pub(crate) fn dispose_effect(&self, id: NodeId) {
        let mut inner = self.inner.lock().unwrap();

        // Remove from effects
        inner.effects.remove(&id);

        // Clean up dependencies
        inner.cleanup_dependencies(id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Effect, Signal};

    #[test]
    fn test_buffer_capacity_reused() {
        let runtime = Runtime::new();

        // Check initial capacity
        assert_eq!(
            runtime
                .inner
                .lock()
                .unwrap()
                .spare_pending_effects
                .capacity(),
            0
        );

        let signal = Signal::new(runtime.clone(), 0);
        let (read, write) = signal.split();

        // Create 50 effects that depend on `signal`
        let mut effects = Vec::new();
        for _ in 0..50 {
            let read_clone = read.clone();
            effects.push(Effect::new(runtime.clone(), move || {
                let _ = read_clone.get();
            }));
        }

        // `write.set` notifies subscribers, increasing `pending_effects` capacity
        write.set(1);

        // After flush, the capacity from `pending_effects` is moved to `spare_pending_effects`
        let spare_capacity = runtime
            .inner
            .lock()
            .unwrap()
            .spare_pending_effects
            .capacity();
        assert!(spare_capacity >= 50);

        // Run again to ensure it re-uses the buffer
        write.set(2);

        let new_spare_capacity = runtime
            .inner
            .lock()
            .unwrap()
            .spare_pending_effects
            .capacity();
        assert_eq!(spare_capacity, new_spare_capacity); // Capacity shouldn't have changed
    }

    #[test]
    fn test_panic_restorer() {
        let runtime = Runtime::new();

        // Populate pending effects
        {
            let mut inner = runtime.inner.lock().unwrap();
            inner.pending_effects.push(NodeId(1));
            inner.pending_effects.push(NodeId(2));
            inner.pending_effects.push(NodeId(3));
        }

        // Catch panic
        let result = std::panic::catch_unwind(|| {
            let mut batch = vec![NodeId(1), NodeId(2), NodeId(3)];
            let restorer = super::PanicRestorer {
                runtime: &runtime,
                remaining_effects: &mut batch,
            };

            // Pop one effect (simulate running it)
            restorer.remaining_effects.pop();

            // Trigger panic
            panic!("test panic");
        });

        assert!(result.is_err());

        // Check that remaining effects were restored
        let inner = runtime.inner.lock().unwrap();
        assert_eq!(
            inner.pending_effects,
            vec![NodeId(1), NodeId(2), NodeId(3), NodeId(1), NodeId(2)]
        );
    }

    #[test]
    fn test_buffer_donation() {
        let runtime = Runtime::new();

        // 1. Initial state (empty)
        let mut buffer = Vec::with_capacity(10);
        let taken = runtime
            .inner
            .lock()
            .unwrap()
            .take_pending_effects(&mut buffer);
        assert!(!taken);
        // The empty buffer was donated
        assert_eq!(
            runtime
                .inner
                .lock()
                .unwrap()
                .spare_pending_effects
                .capacity(),
            10
        );

        // 2. Buffer donation should only happen if buffer.capacity() > spare_capacity
        let mut small_buffer = Vec::with_capacity(5);
        let taken = runtime
            .inner
            .lock()
            .unwrap()
            .take_pending_effects(&mut small_buffer);
        assert!(!taken);
        // capacity shouldn't change
        assert_eq!(
            runtime
                .inner
                .lock()
                .unwrap()
                .spare_pending_effects
                .capacity(),
            10
        );

        // 3. Buffer donation should not swap if equal capacity
        let mut equal_buffer = Vec::with_capacity(10);
        let equal_buffer_ptr = equal_buffer.as_ptr();
        // We verify that if capacity is equal, the spare buffer isn't unnecessarily replaced
        // (which would be the case if > was changed to >=).
        let taken = runtime
            .inner
            .lock()
            .unwrap()
            .take_pending_effects(&mut equal_buffer);
        assert!(!taken);

        let spare_ptr = {
            let inner = runtime.inner.lock().unwrap();
            assert_eq!(inner.spare_pending_effects.capacity(), 10);
            inner.spare_pending_effects.as_ptr()
        };
        // The original spare_pending_effects was donated by `buffer` in step 1.
        // It should NOT be the same as `equal_buffer_ptr`.
        assert_ne!(spare_ptr, equal_buffer_ptr);
    }

    #[test]
    fn test_spare_buffer_reuse() {
        let runtime = Runtime::new();

        // Create a spare buffer
        {
            let mut inner = runtime.inner.lock().unwrap();
            inner.spare_pending_effects = Vec::with_capacity(20);
            inner.pending_effects.push(NodeId(1));
        }

        // Take pending effects. The spare buffer should be used as the new pending_effects.
        let mut buffer = Vec::new();
        let taken = runtime
            .inner
            .lock()
            .unwrap()
            .take_pending_effects(&mut buffer);

        assert!(taken);
        assert_eq!(buffer, vec![NodeId(1)]);

        // Check that spare_pending_effects is now empty (was moved)
        let inner = runtime.inner.lock().unwrap();
        assert_eq!(inner.spare_pending_effects.capacity(), 0);
        assert_eq!(inner.pending_effects.capacity(), 20); // Reused spare
    }

    #[test]
    fn test_take_pending_effects_no_spare() {
        let runtime = Runtime::new();

        {
            let mut inner = runtime.inner.lock().unwrap();
            inner.spare_pending_effects = Vec::new(); // capacity 0
            inner.pending_effects.push(NodeId(2));
        }

        let mut buffer = Vec::with_capacity(5);
        let taken = runtime
            .inner
            .lock()
            .unwrap()
            .take_pending_effects(&mut buffer);

        assert!(taken);
        assert_eq!(buffer, vec![NodeId(2)]);

        // Because spare_pending_effects was empty (capacity 0), take_pending_effects
        // should have fallen back to returning the provided buffer.
        let inner = runtime.inner.lock().unwrap();
        assert_eq!(inner.pending_effects.capacity(), 5);
    }

    #[test]
    fn test_get_computed_if_fresh() {
        let runtime = Runtime::new();
        let signal = Signal::new(runtime.clone(), 10);
        let (read, write) = signal.split();

        // Create computed and run once to init
        let read_clone = read.clone();
        let computed = crate::Computed::new(runtime.clone(), move || read_clone.get() * 2);

        // Value is fresh right after init/read
        assert_eq!(computed.get(), 20);

        // Get the internal node ID of the computed
        // (we find it by looking at the inner state directly for the test)
        let computed_id = *runtime
            .inner
            .lock()
            .unwrap()
            .computeds
            .keys()
            .next()
            .unwrap();

        // 1. Should return Some when fresh
        let fresh_val = runtime.get_computed_if_fresh(computed_id).unwrap();
        let val_guard = fresh_val
            .downcast_ref::<std::sync::RwLock<i32>>()
            .unwrap()
            .read()
            .unwrap();
        assert_eq!(*val_guard, 20);
        drop(val_guard);

        // Update dependency to make it stale
        write.set(15);

        // 2. Should return None when stale
        assert!(runtime.get_computed_if_fresh(computed_id).is_none());
    }

    #[test]
    fn test_is_stale() {
        let runtime = Runtime::new();
        let signal = Signal::new(runtime.clone(), 100);
        let (read, write) = signal.split();

        let read_clone = read.clone();
        let computed = crate::Computed::new(runtime.clone(), move || read_clone.get() + 5);

        // Get the internal node ID of the computed
        let computed_id = *runtime
            .inner
            .lock()
            .unwrap()
            .computeds
            .keys()
            .next()
            .unwrap();

        // After creation and initial evaluation, it's fresh
        assert_eq!(computed.get(), 105);
        assert!(!runtime.is_stale(computed_id));

        // Update signal
        write.set(200);

        // Now it's stale
        assert!(runtime.is_stale(computed_id));

        // Reading it recomputes it, making it fresh again
        assert_eq!(computed.get(), 205);
        assert!(!runtime.is_stale(computed_id));
    }

    #[test]
    #[cfg(feature = "nova")]
    fn test_inspect_graph_coverage() {
        let runtime = Runtime::new();
        let signal = Signal::new(runtime.clone(), 1).with_label("sig");
        let (read, _) = signal.split();

        let read_clone = read.clone();
        let comp =
            crate::Computed::new(runtime.clone(), move || read_clone.get() + 1).with_label("comp");

        let read_comp_clone = comp.clone();
        let _effect = crate::Effect::new(runtime.clone(), move || {
            let _ = read_comp_clone.get();
        })
        .with_label("eff");

        let snap = runtime.inspect_graph();

        // We should have 3 nodes: Signal, Computed, Effect
        assert_eq!(snap.nodes.len(), 3);

        let sig_node = snap
            .nodes
            .iter()
            .find(|n| n.node_type == NodeType::Signal)
            .unwrap();
        assert_eq!(sig_node.label, "sig");

        let comp_node = snap
            .nodes
            .iter()
            .find(|n| n.node_type == NodeType::Computed)
            .unwrap();
        assert_eq!(comp_node.label, "comp");

        let eff_node = snap
            .nodes
            .iter()
            .find(|n| n.node_type == NodeType::Effect)
            .unwrap();
        assert_eq!(eff_node.label, "eff");

        // We should have 2 edges: Signal -> Computed, Computed -> Effect
        assert_eq!(snap.dependencies.len(), 2);

        // Assert sorting is correct
        assert!(snap.nodes.windows(2).all(|w| w[0].id.0 <= w[1].id.0));
        assert!(
            snap.dependencies
                .windows(2)
                .all(|w| (w[0].0.0, w[0].1.0) <= (w[1].0.0, w[1].1.0))
        );
    }
}

/// A snapshot of the reactive dependency graph at a specific point in time.
///
/// This is typically used for debugging or building developer tools (like `flux-devtools`
/// or `flux-radar`) to visualize how state is connected and what is currently stale.
///
/// The fields in this struct are explicitly sorted by ID to provide a stable snapshot
/// across multiple frames, preventing UI jitter in diagnostic tools.
///
/// # Examples
///
/// ```
/// # use flux_state::{Runtime, Signal};
/// # let runtime = Runtime::new();
/// # let count = Signal::new(runtime.clone(), 0);
/// # let (read, write) = count.split();
/// # #[cfg(feature = "nova")]
/// # {
/// let snapshot = runtime.inspect_graph();
/// assert_eq!(snapshot.nodes.len(), 1); // Only the signal exists
/// # }
/// ```
#[cfg(feature = "nova")]
#[derive(Debug, Clone)]
#[cfg_attr(feature = "nova", derive(serde::Serialize, serde::Deserialize))]
pub struct GraphSnapshot {
    /// A list of all reactive nodes (`Signal`, `Computed`, `Effect`) currently alive in the graph.
    pub nodes: Vec<NodeInfo>,
    /// A list of directed edges representing dependencies.
    /// The format is `(Source, Subscriber)`. For example, if a `Computed` depends on a `Signal`,
    /// the tuple will be `(SignalId, ComputedId)`.
    pub dependencies: Vec<(NodeId, NodeId)>,
    /// A list of node IDs that are currently marked as "stale" and need recomputation
    /// the next time they are read.
    pub stale_nodes: Vec<NodeId>,
}

/// Metadata about a specific node in the reactive graph.
#[cfg(feature = "nova")]
#[derive(Debug, Clone)]
#[cfg_attr(feature = "nova", derive(serde::Serialize, serde::Deserialize))]
pub struct NodeInfo {
    /// The unique identifier for this node.
    pub id: NodeId,
    /// The kind of reactive primitive this node represents.
    pub node_type: NodeType,
    /// A human-readable debug label assigned to the node, or a fallback string
    /// like `Signal(NodeId(1))` if no label was provided.
    pub label: String,
}

/// The type of a reactive node.
#[cfg(feature = "nova")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "nova", derive(serde::Serialize, serde::Deserialize))]
pub enum NodeType {
    /// A root state container created via `Signal::new`.
    Signal,
    /// Derived state created via `Computed::new`.
    Computed,
    /// A side-effect closure created via `Effect::new`.
    Effect,
}

#[cfg(feature = "nova")]
impl Runtime {
    pub(crate) fn set_label(&self, id: NodeId, label: String) {
        let mut inner = self.inner.lock().unwrap();
        inner.labels.insert(id, label);
    }

    /// Snapshots the current dependency graph for debugging/devtools.
    ///
    /// This method is gated behind the `nova` feature and is primarily meant
    /// for developer tools to introspect the state of reactivity within `flux-state`.
    ///
    /// The returned `GraphSnapshot` represents an instantaneous state, mapping all
    /// signals, computed values, and effects, their current relationships (edges),
    /// and whether they are marked as stale.
    ///
    /// # Thread Safety and Performance
    ///
    /// `inspect_graph` briefly acquires the runtime's internal `Mutex`, blocking other
    /// reactive reads and writes while it creates a deep copy of the structural nodes.
    /// Because of this lock contention and the allocation overhead, this method is intended
    /// for dev/diagnostic loops (e.g., rendering an overlay UI tree) rather than
    /// high-frequency production code.
    ///
    /// # Stability
    ///
    /// The values in the returned `GraphSnapshot` (nodes, dependencies, stale_nodes)
    /// are explicitly sorted by `NodeId` so that snapshots are stable across
    /// multiple UI frames and do not jitter when visualized.
    ///
    /// # Examples
    ///
    /// ```
    /// # use flux_state::{Runtime, Signal, Computed};
    /// # let runtime = Runtime::new();
    /// # let count = Signal::new(runtime.clone(), 10);
    /// # let (read, write) = count.split();
    /// # let read_clone = read.clone();
    /// # let double = Computed::new(runtime.clone(), move || read_clone.get() * 2);
    /// #
    /// // ...
    /// # #[cfg(feature = "nova")]
    /// # {
    /// let snapshot = runtime.inspect_graph();
    /// assert_eq!(snapshot.nodes.len(), 2); // 1 Signal + 1 Computed
    /// assert_eq!(snapshot.dependencies.len(), 1); // Edge: Signal -> Computed
    /// # }
    /// ```
    pub fn inspect_graph(self: &Arc<Self>) -> GraphSnapshot {
        let inner = self.inner.lock().unwrap();
        let mut nodes = Vec::new();
        let mut dependencies = Vec::new();
        let mut stale_nodes = Vec::new();

        // Collect Signals
        for id in inner.signals.keys() {
            let label = inner
                .labels
                .get(id)
                .cloned()
                .unwrap_or_else(|| format!("Signal({:?})", id));
            nodes.push(NodeInfo {
                id: *id,
                node_type: NodeType::Signal,
                label,
            });
        }

        // Collect Computeds
        for id in inner.computeds.keys() {
            let label = inner
                .labels
                .get(id)
                .cloned()
                .unwrap_or_else(|| format!("Computed({:?})", id));
            nodes.push(NodeInfo {
                id: *id,
                node_type: NodeType::Computed,
                label,
            });
        }

        // Collect Effects
        for id in inner.effects.keys() {
            let label = inner
                .labels
                .get(id)
                .cloned()
                .unwrap_or_else(|| format!("Effect({:?})", id));
            nodes.push(NodeInfo {
                id: *id,
                node_type: NodeType::Effect,
                label,
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

        nodes.sort_by_key(|n| n.id.0);
        dependencies.sort_by_key(|(src, tgt)| (src.0, tgt.0));
        stale_nodes.sort_by_key(|n| n.0);

        GraphSnapshot {
            nodes,
            dependencies,
            stale_nodes,
        }
    }
}

#[cfg(test)]
mod tests_sentry {
    use super::*;
    use std::sync::RwLock;

    #[test]
    #[should_panic(expected = "Signal not found for id NodeId(9999)")]
    fn test_get_signal_handle_panics_on_missing_signal() {
        let runtime = Runtime::new();
        runtime.get_signal_handle(NodeId(9999));
    }

    #[test]
    #[should_panic(expected = "Computed not found for id NodeId(9999)")]
    fn test_get_computed_handle_panics_on_missing_computed() {
        let runtime = Runtime::new();
        runtime.get_computed_handle(NodeId(9999));
    }

    #[test]
    #[should_panic(expected = "Computed value not initialized for id")]
    fn test_get_computed_handle_panics_on_uninitialized_computed() {
        let runtime = Runtime::new();

        let mut inner = runtime.inner.lock().unwrap();
        let id = NodeId(inner.next_id);
        inner.next_id += 1;
        inner.computeds.insert(
            id,
            ComputedNode {
                value: None,
                compute: Arc::new(|| Arc::new(RwLock::new(42))),
            },
        );
        drop(inner);

        runtime.get_computed_handle(id);
    }

    #[test]
    #[should_panic(expected = "Signal not found for id NodeId(9999)")]
    fn test_track_and_get_signal_panics_on_missing_signal() {
        let runtime = Runtime::new();
        runtime.track_and_get_signal(NodeId(9999));
    }

    #[test]
    #[should_panic(expected = "Computed not found for id NodeId(9999)")]
    fn test_track_and_get_computed_if_fresh_panics_on_missing_computed() {
        let runtime = Runtime::new();
        runtime.track_and_get_computed_if_fresh(NodeId(9999));
    }

    #[test]
    #[should_panic(expected = "Computed not found for id NodeId(9999)")]
    fn test_get_computed_if_fresh_panics_on_missing_computed() {
        let runtime = Runtime::new();
        runtime.get_computed_if_fresh(NodeId(9999));
    }

    #[test]
    #[should_panic(expected = "Computed not found for id NodeId(9999)")]
    fn test_recompute_panics_on_missing_computed() {
        let runtime = Runtime::new();
        runtime.recompute(NodeId(9999));
    }
}
