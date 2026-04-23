//! Terminal User Interface (TUI) renderer.
//!
//! This crate provides a terminal-based rendering backend for the `render-engine`.
//! It translates UI primitives (like styled rectangles and text) into terminal character blocks
//! using the `ratatui` crate, allowing interfaces designed for graphical windows to be
//! visualized directly in the terminal.

/// Core terminal rendering backend implementation.
pub mod backend;

/// Terminal lifecycle and event loop management.
pub mod runner;

pub use backend::TuiBackend;
pub use runner::TuiRunner;
