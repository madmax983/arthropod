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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::Runtime;
    use crate::signal::Signal;

    #[test]
    fn test_computed_basic() {
        let runtime = Runtime::new();
        let signal = Signal::new(runtime.clone(), 1);
        let (read, write) = signal.split();

        let read_clone = read.clone();
        let computed = Computed::new(runtime, move || read_clone.get() * 2);

        assert_eq!(computed.get(), 2);

        write.set(10);
        assert_eq!(computed.get(), 20);
    }

    #[test]
    fn test_diamond_pull() {
        let runtime = Runtime::new();
        let a = Signal::new(runtime.clone(), 1);
        let (r_a, w_a) = a.split();

        let r_a_1 = r_a.clone();
        let b = Computed::new(runtime.clone(), move || r_a_1.get() * 2);

        let r_a_2 = r_a.clone();
        let c = Computed::new(runtime.clone(), move || r_a_2.get() + 1);

        // D = B + C
        let b_c = b.clone();
        let c_c = c.clone();
        let d = Computed::new(runtime, move || b_c.get() + c_c.get());

        // Initial: a=1, b=2, c=2, d=4
        assert_eq!(d.get(), 4);

        w_a.set(2);
        // a=2, b=4, c=3, d=7
        assert_eq!(d.get(), 7);
    }

    #[test]
    fn test_dynamic_deps() {
        let runtime = Runtime::new();
        let toggle = Signal::new(runtime.clone(), true);
        let (r_toggle, w_toggle) = toggle.split();

        let a = Signal::new(runtime.clone(), 10);
        let (r_a, _w_a) = a.split();

        let b = Signal::new(runtime.clone(), 20);
        let (r_b, w_b) = b.split();

        let r_toggle_c = r_toggle.clone();
        let r_a_c = r_a.clone();
        let r_b_c = r_b.clone();

        let computed = Computed::new(runtime.clone(), move || {
            if r_toggle_c.get() {
                r_a_c.get()
            } else {
                r_b_c.get()
            }
        });

        assert_eq!(computed.get(), 10);

        // Update B. A is still active. Computed should not change.
        w_b.set(30);
        assert_eq!(computed.get(), 10);

        // Switch to B.
        w_toggle.set(false);
        assert_eq!(computed.get(), 30);
    }
}
