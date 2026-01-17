//! Reactive state management for Arthropod GUI framework.
//!
//! Provides fine-grained reactivity with Signals, Computed values, and Effects.

mod computed;
mod effect;
mod runtime;
mod signal;

pub use computed::Computed;
pub use effect::Effect;
pub use runtime::{NodeId, Runtime};
pub use signal::{ReadSignal, Signal, WriteSignal};
