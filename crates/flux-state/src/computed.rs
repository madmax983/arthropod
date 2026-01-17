//! Computed values - derived reactive state.

use std::cell::RefCell;
use std::rc::Rc;
use crate::runtime::{Runtime, NodeId};

/// A derived/computed value that automatically tracks dependencies.
#[derive(Clone)]
pub struct Computed<T> {
    id: NodeId,
    runtime: Rc<Runtime>,
    _marker: std::marker::PhantomData<T>,
}

impl<T: Clone + 'static> Computed<T> {
    /// Create a new computed value.
    pub fn new<F>(runtime: Rc<Runtime>, compute: F) -> Self
    where
        F: Fn() -> T + 'static,
    {
        let id = runtime.create_computed(Box::new(move || {
            Box::new(RefCell::new(compute())) as Box<dyn std::any::Any>
        }));

        // Initialize the value by computing it once
        let result = Self {
            id,
            runtime: Rc::clone(&runtime),
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
                v.downcast_ref::<RefCell<T>>()
                    .expect("Type mismatch")
                    .borrow()
                    .clone()
            })
    }
}
