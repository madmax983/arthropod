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

use flux_state::{Computed, ReadSignal};
use render_engine::Color;

pub use input_engine::TextInputState;

/// Reactive text state for a node.
///
/// Used by widgets that display text which can change over time based on a `Signal`.
#[derive(Clone)]
pub struct ReactiveTextState {
    /// The source signal for the text.
    pub read_signal: ReadSignal<String>,
}

/// Computed text state for a node.
///
/// Used by widgets that display text derived from other state via a `Computed` value.
#[derive(Clone)]
pub struct ComputedTextState {
    /// The computed value source.
    pub computed: Computed<String>,
}

/// Reactive color state for a node.
///
/// Used by widgets that need to update their visual color based on a signal
/// (e.g., hover states, theming, or data-driven coloring).
#[derive(Clone)]
pub struct ReactiveColorState {
    /// The source signal for the color.
    pub read_signal: ReadSignal<Color>,
}

/// Reactive layout width state for a node.
///
/// Used by widgets that need to update their width based on a signal
/// (e.g., progress bars, volume indicators).
#[derive(Clone)]
pub struct ReactiveLayoutWidthState {
    /// The source signal for the width.
    pub read_signal: ReadSignal<f32>,
    /// Optional handle to keep Computed alive (if the signal came from a Computed)
    pub handle: Option<std::sync::Arc<dyn std::any::Any + Send + Sync>>,
}

/// Reactive layout flex grow state for a node.
#[derive(Clone)]
pub struct ReactiveLayoutFlexGrowState {
    /// The source signal for the flex grow factor.
    pub read_signal: ReadSignal<f32>,
}
