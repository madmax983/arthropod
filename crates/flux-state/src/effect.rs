//! Effects - side effects that run when dependencies change.

use crate::runtime::{NodeId, Runtime};
use std::rc::Rc;

/// An effect that runs when its dependencies change.
pub struct Effect {
    id: NodeId,
    runtime: Rc<Runtime>,
}

impl Effect {
    /// Create a new effect.
    pub fn new<F>(runtime: Rc<Runtime>, effect_fn: F) -> Self
    where
        F: Fn() + 'static,
    {
        let id = runtime.create_effect(Box::new(effect_fn));

        // Run the effect immediately to establish dependencies
        runtime.run_effect(id);

        Self { id, runtime }
    }
}

impl Drop for Effect {
    fn drop(&mut self) {
        self.runtime.dispose_effect(self.id);
    }
}
