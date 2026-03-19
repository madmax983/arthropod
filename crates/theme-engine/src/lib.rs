//! Theme Engine - Three-layer theming system for Arthropod
//!
//! Implements ADR 0009's three-layer architecture:
//! - Layer 1: SystemTheme (platform-native materials and colors)
//! - Layer 2: DesignTokens (semantic design system)
//! - Layer 3: Style API (CSS-like component styling)
//!
//! # Architecture
//!
//! ```text
//! Layer 3: style! { background: tokens.surface_primary; }
//!            ↓
//! Layer 2: DesignTokens { surface_primary: Material(Mica) }
//!            ↓
//! Layer 1: SystemTheme { mica_available: true, accent: Color }
//! ```

pub mod design_tokens;

pub mod system_theme;

pub use design_tokens::{DesignTokens, TokenValue};

pub use system_theme::SystemTheme;

// Re-export common styling primitives for convenience

// Re-export BackdropMaterial for convenience
pub use plat_core::BackdropMaterial;

/// Common color type
pub type Color = glam::Vec4;

/// Error types for theme operations
#[derive(Debug, thiserror::Error)]
pub enum ThemeError {
    #[error("Platform theme query failed: {0}")]
    PlatformError(String),

    #[error("Unsupported platform")]
    UnsupportedPlatform,

    #[error("Invalid token value: {0}")]
    InvalidToken(String),
}

pub type Result<T> = std::result::Result<T, ThemeError>;
