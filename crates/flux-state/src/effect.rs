//! Effects - side effects that run when dependencies change.

use crate::runtime::{NodeId, Runtime};
use std::sync::Arc;

/// An effect that runs when its dependencies change.
pub struct Effect {
    id: NodeId,
    runtime: Arc<Runtime>,
}

impl Effect {
    /// Create a new effect.
    pub fn new<F>(runtime: Arc<Runtime>, effect_fn: F) -> Self
    where
        F: Fn() + 'static + Send + Sync,
    {
        let id = runtime.create_effect(Arc::new(effect_fn));

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
