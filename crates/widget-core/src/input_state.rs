//! Input state types
//!
//! This module defines the state structures used by widgets to manage user input and reactive updates.
//! These states are typically stored in the ECS (Entity Component System) as components attached to widget entities.
//!
//! # Main Types
//!
//! - [`TextInputState`]: Manages the state of a text input field (cursor position, text value, etc.).
//! - [`ReactiveTextState`]: Manages text content that updates reactively from a signal.
//! - [`ComputedTextState`]: Manages text content derived from a computed value.
//! - [`ReactiveColorState`]: Manages color updates from a signal.

use flux_state::ReadSignal;
use render_engine::Color;

pub use input_engine::text::{ComputedTextState, ReactiveTextState, TextInputState};

/// Reactive color state for a node.
///
/// Used by widgets that need to update their visual color based on a signal
/// (e.g., hover states, theming, or data-driven coloring).
#[derive(Clone)]
pub struct ReactiveColorState {
    /// The source signal for the color.
    pub read_signal: ReadSignal<Color>,
}
