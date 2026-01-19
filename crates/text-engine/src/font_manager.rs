//! Font management and loading

use std::sync::Arc;

/// Font manager for loading and caching fonts
pub struct FontManager {
    default_font_data: Arc<Vec<u8>>,
}

impl FontManager {
    /// Create a new font manager
    pub fn new() -> Self {
        // Load default system font
        let default_font_data = Self::load_system_font();

        Self {
            default_font_data: Arc::new(default_font_data),
        }
    }

    /// Get the default font data
    pub fn default_font(&self) -> &[u8] {
        &self.default_font_data
    }

    /// Load a system font (platform-specific)
    #[cfg(target_os = "windows")]
    fn load_system_font() -> Vec<u8> {
        // Try to load Segoe UI (Windows default)
        let font_paths = vec![
            r"C:\Windows\Fonts\segoeui.ttf",
            r"C:\Windows\Fonts\arial.ttf",
            r"C:\Windows\Fonts\verdana.ttf",
        ];

        for path in font_paths {
            if let Ok(data) = std::fs::read(path) {
                return data;
            }
        }

        // Fallback: create minimal embedded font data
        Self::embedded_fallback_font()
    }

    #[cfg(not(target_os = "windows"))]
    fn load_system_font() -> Vec<u8> {
        // For non-Windows platforms, use embedded fallback for now
        Self::embedded_fallback_font()
    }

    /// Minimal embedded font fallback
    /// This is a placeholder - in production, we'd embed a real font like Noto Sans
    fn embedded_fallback_font() -> Vec<u8> {
        // For now, try to load DejaVu Sans from common locations
        let fallback_paths = vec![
            "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
            "/usr/share/fonts/TTF/DejaVuSans.ttf",
            "/System/Library/Fonts/Helvetica.ttc",
        ];

        for path in fallback_paths {
            if let Ok(data) = std::fs::read(path) {
                return data;
            }
        }

        // If all else fails, return empty vec
        // Tests will need to handle this gracefully
        Vec::new()
    }
}

impl Default for FontManager {
    fn default() -> Self {
        Self::new()
    }
}
