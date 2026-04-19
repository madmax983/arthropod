//! Material UI — Material Design 3 component library for Arthropod.
//!
//! Provides MD3-themed widgets built on top of `widget-core`.
//! Use `MaterialTheme::from_seed` to generate a complete MD3 theme
//! from a single seed color, then store it in `WidgetContext` via
//! the extension mechanism.

/// Core widget components such as buttons, icons, and text.
pub mod components;

/// Components for displaying data, such as lists and tables.
pub mod data_display;

/// Components for providing feedback to the user, like progress indicators and dialogs.
pub mod feedback;

/// Components for receiving input from the user, like text fields and sliders.
pub mod inputs;

/// Components for navigating between different views, like drawers and tabs.
pub mod navigation;

/// Components that act as surfaces for other content, like cards and sheets.
pub mod surfaces;

/// Material Design 3 theme system, including color palettes, typography, and shapes.
pub mod theme;
