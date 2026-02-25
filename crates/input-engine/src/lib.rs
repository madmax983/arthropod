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

pub mod gestures;
pub mod input_manager;

pub use gestures::{
    ChordMatcher, GestureSignal, InputPattern, SequenceMatcher, create_gesture_signal,
};
pub use input_manager::InputManager;
