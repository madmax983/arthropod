//! Reactive state management for Arthropod GUI framework.
//!
//! Provides fine-grained reactivity with Signals, Computed values, and Effects.

mod runtime;
mod signal;
mod computed;
mod effect;

pub use runtime::{Runtime, NodeId};
pub use signal::{Signal, ReadSignal, WriteSignal};
pub use computed::Computed;
pub use effect::Effect;
