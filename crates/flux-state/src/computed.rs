//! Computed values - derived reactive state.

use crate::runtime::{NodeId, Runtime};
use std::sync::{Arc, Mutex};

/// A derived/computed value that automatically tracks dependencies.
#[derive(Clone)]
pub struct Computed<T> {
    id: NodeId,
    runtime: Arc<Runtime>,
    _marker: std::marker::PhantomData<T>,
}

impl<T: Clone + 'static + Send> Computed<T> {
    /// Create a new computed value.
    pub fn new<F>(runtime: Arc<Runtime>, compute: F) -> Self
    where
        F: Fn() -> T + 'static + Send,
    {
        let id = runtime.create_computed(Box::new(move || {
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

    /// Get the current value (tracks dependency, recomputes if stale).
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
