//! Native backdrop materials for platform-specific window effects.
//!
//! Windows 11: Mica, MicaAlt, Acrylic
//! Windows 10: Acrylic only
//! macOS: (future) NSVisualEffectView materials

/// Backdrop material for window backgrounds
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BackdropMaterial {
    /// No special material (solid color background)
    #[default]
    None,
    /// Windows 11 Mica - subtle tinted blur based on desktop wallpaper
    Mica,
    /// Windows 11 Mica Alt - stronger tint variant
    MicaAlt,
    /// Windows 10/11 Acrylic - translucent blur effect
    Acrylic,
}
