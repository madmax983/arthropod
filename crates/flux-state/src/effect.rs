//! Effects - side effects that run when dependencies change.
//!
//! Effects are the "sinks" of the reactive graph. They are used to perform side effects
//! (like updating the DOM, logging, or making network requests) in response to state changes.

use crate::runtime::{NodeId, Runtime};
use std::sync::Arc;

/// An effect that runs automatically when its dependencies change.
///
/// Effects auto-track signals accessed within their closure. When any of those
/// signals change, the effect re-runs.
///
/// # Lifecycle
///
/// 1. **Creation**: The effect runs immediately to detect dependencies.
/// 2. **Update**: When a dependency changes, the effect is scheduled to run again.
/// 3. **Cleanup**: When the `Effect` struct is dropped, it unsubscribes from all signals.
///
/// # Example
///
/// ```
/// use flux_state::{Runtime, Signal, Effect};
///
/// let runtime = Runtime::new();
/// let count = Signal::new(runtime.clone(), 0);
/// let (read, write) = count.split();
///
/// // Create an effect that prints whenever count changes
/// let _effect = Effect::new(runtime, move || {
///     println!("Count changed to: {}", read.get());
/// });
///
/// write.set(1); // Prints "Count changed to: 1"
/// ```
pub struct Effect {
    id: NodeId,
    runtime: Arc<Runtime>,
}

impl Effect {
    /// Create a new effect.
    ///
    /// The `effect_fn` will run immediately to establish initial dependencies.
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
