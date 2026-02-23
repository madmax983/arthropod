//! Reactive state management for the Arthropod GUI framework.
//!
//! `flux-state` provides fine-grained reactivity using a Signal/Effect architecture,
//! similar to SolidJS or Preact Signals, but adapted for Rust's ownership model
//! and thread safety requirements.
//!
//! # Core Concepts
//!
//! - **[`Signal`]**: The atomic unit of state. It holds a value and notifies dependents when it changes.
//! - **[`Computed`]**: A derived value that automatically updates when its dependencies change.
//!   It is initialized *eagerly* but updates *lazily*.
//! - **[`Effect`]**: A side effect that runs automatically when its dependencies change.
//! - **[`Runtime`]**: The coordinator that manages the dependency graph and propagates updates.
//!
//! # Thread Safety
//!
//! Core primitives ([`Signal`], [`ReadSignal`], [`WriteSignal`], [`Computed`], [`Effect`]) are
//! thread-safe (`Send + Sync`) when their inner state types are `Send + Sync`. They are
//! designed to be shared across threads (e.g., via `Arc` or simply by cloning the handle).
//! The [`Runtime`] uses internal locking to ensure consistency.
//!
//! # Quick Start
//!
//! ```
//! use flux_state::{Runtime, Signal, Effect, Computed};
//!
//! // 1. Create a runtime (shared via Arc)
//! // Runtime::new() returns an Arc<Runtime>, which is cheap to clone.
//! let runtime = Runtime::new();
//!
//! // 2. Create a signal
//! // Signal::new takes ownership of the runtime handle.
//! let count = Signal::new(runtime.clone(), 0);
//! let (read_count, write_count) = count.split();
//!
//! // 3. Create a computed value (derived state)
//! // Computed values track dependencies automatically when .get() is called.
//! let read_count_computed = read_count.clone();
//! let double_count = Computed::new(runtime.clone(), move || {
//!     read_count_computed.get() * 2
//! });
//!
//! // 4. Create an effect (side effect)
//! // Effects run immediately, and then re-run whenever dependencies change.
//! // IMPORTANT: Keep the return value (`_effect`) to keep the effect alive!
//! let read_count_effect = read_count.clone();
//! let double_count_effect = double_count.clone();
//! let _effect = Effect::new(runtime.clone(), move || {
//!     println!("Count: {}, Double: {}", read_count_effect.get(), double_count_effect.get());
//! });
//!
//! // 5. Update state
//! write_count.set(1); // Output: Count: 1, Double: 2
//! write_count.set(5); // Output: Count: 5, Double: 10
//! ```
//!
//! # Debugging (Nova)
//!
//! When the `nova` feature is enabled, `flux-state` provides tools to inspect the reactive graph.
//! This is useful for building developer tools or debugging complex dependency chains.
//!
//! ```rust
//! # use flux_state::Runtime;
//! # let runtime = Runtime::new();
//! // Take a snapshot of the current dependency graph
//! #[cfg(feature = "nova")]
//! let snapshot = runtime.inspect_graph();
//!
//! #[cfg(feature = "nova")]
//! println!("Graph has {} nodes", snapshot.nodes.len());
//! ```
//!
//! # Common Pitfalls
//!
//! ## Recursion Limit
//! The runtime enforces a recursion limit (default: 100) to prevent infinite loops (e.g.,
//! Effect A triggers Signal B, which triggers Effect A). If you hit this limit, the runtime
//! will panic. Check for circular dependencies in your effects.
//!
//! ## Deadlocks
//! While `flux-state` is thread-safe, be careful not to hold a lock on a non-reactive
//! resource inside an effect if that resource is also needed to update a signal.
//! The runtime's internal locks are fine-grained, but external locks are your responsibility.
//!
//! ## Zombie Effects
//! If you drop the `Effect` handle returned by `Effect::new`, the effect is "disposed" and
//! will stop running. Ensure you store the handle in a struct or a long-lived variable
//! if you want the effect to persist.
//!
//! ```rust
//! # use flux_state::{Runtime, Effect};
//! # let runtime = Runtime::new();
//! {
//!     let _e = Effect::new(runtime.clone(), || println!("I run once!"));
//! } // _e is dropped here, effect is cleaned up
//! ```

mod computed;
mod effect;
mod runtime;
mod signal;

pub use computed::Computed;
pub use effect::Effect;
#[cfg(feature = "nova")]
pub use runtime::{GraphSnapshot, NodeInfo, NodeType};
pub use runtime::{NodeId, Runtime};
pub use signal::{ReadSignal, Signal, WriteSignal};
