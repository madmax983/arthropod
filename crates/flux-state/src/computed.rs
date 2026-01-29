//! Computed values - derived reactive state.
//!
//! Computed values are signals that are derived from other signals. They automatically
//! update when their dependencies change.
//!
//! They are lazy: they only re-execute their computation when their dependencies change
//! and their value is requested.

use crate::runtime::{NodeId, Runtime};
use std::sync::{Arc, Mutex};

/// A derived/computed value that automatically tracks dependencies.
///
/// Computed values are useful for deriving state from other signals without manually
/// syncing them. They memoize their result and only recompute when dependencies change.
///
/// # Example
///
/// ```
/// use flux_state::{Runtime, Signal, Computed};
///
/// let runtime = Runtime::new();
/// let count = Signal::new(runtime.clone(), 1);
/// let (read_count, _) = count.split();
///
/// // Create a computed value that doubles the count
/// let read_count_clone = read_count.clone();
/// let double_count = Computed::new(runtime, move || {
///     read_count_clone.get() * 2
/// });
///
/// assert_eq!(double_count.get(), 2);
/// ```
#[derive(Clone)]
pub struct Computed<T> {
    id: NodeId,
    runtime: Arc<Runtime>,
    _marker: std::marker::PhantomData<T>,
}

impl<T: Clone + 'static + Send> Computed<T> {
    /// Create a new computed value.
    ///
    /// The `compute` closure will be called to calculate the initial value,
    /// and then automatically called again whenever any signal it reads changes.
    pub fn new<F>(runtime: Arc<Runtime>, compute: F) -> Self
    where
        F: Fn() -> T + 'static + Send + Sync,
    {
        let id = runtime.create_computed(Arc::new(move || {
            Box::new(Mutex::new(compute())) as Box<dyn std::any::Any + Send>
        }));

        // Initialize the value by computing it once
        let result = Self {
            id,
            runtime: Arc::clone(&runtime),
            _marker: std::marker::PhantomData,
        };

        // Compute initial value
        result.runtime.recompute(result.id);

        result
    }

    /// Get the current value.
    ///
    /// This method:
    /// 1. Tracks the dependency if called inside an Effect or another Computed.
    /// 2. Recomputes the value if any dependencies have changed (lazy evaluation).
    /// 3. Returns the memoized value otherwise.
    pub fn get(&self) -> T {
        self.runtime.track(self.id);

        // Check if value is stale and recompute if needed
        if self.runtime.is_stale(self.id) {
            self.runtime.recompute(self.id);
        }

        self.runtime
            .with_computed_value(self.id, |v: &dyn std::any::Any| {
                v.downcast_ref::<Mutex<T>>()
                    .expect("Type mismatch")
                    .lock()
                    .unwrap()
                    .clone()
            })
    }
}
