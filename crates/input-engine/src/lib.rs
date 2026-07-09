//! Input and gesture recognition engine for Arthropod GUI framework.
//!
//! This crate implements the "Input Fusion" system (Spec 005), providing:
//! - High-level gesture recognition
//! - Input state management
//! - Event propagation (bubbling/capturing)
//! - Hit testing against the scene graph
//!
//! # Architecture
//!
//! `input-engine` sits between `plat-core` (raw events) and `widget-core` (UI logic).
//! It consumes `plat-core::WindowEvent`s and produces high-level gestures or updates
//! input state components.

pub(crate) mod focus;
pub(crate) mod form;
pub(crate) mod gestures;
#[cfg(feature = "nova")]
pub(crate) mod mouse_gestures;
pub(crate) mod text;
pub(crate) mod validation;

pub use gestures::{
    ChordMatcher, GestureSignal, InputPattern, SequenceMatcher, create_gesture_signal,
};
#[cfg(feature = "nova")]
pub use mouse_gestures::{MouseGesture, StrokeMatcher};

/// Unique identifier for an input node, decoupled from the rendering engine.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct InputNodeId(pub u64);

// Explicit re-exports for the flattened public API Facade
pub use focus::{focus_next, focus_prev};
pub use form::{
    FormData, FormState, SubmitCallback, get_form_field_errors, revalidate_form, trigger_submit,
};
pub use text::{
    TextInputState, send_backspace, send_char, send_delete, send_key_left, send_key_right,
};
pub use validation::{ValidationResult, ValidationState, Validator};
