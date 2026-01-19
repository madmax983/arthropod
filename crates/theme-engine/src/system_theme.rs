//! Layer 1: System Theme - Platform-native materials and colors
//!
//! Queries the operating system for native theme information:
//! - Windows: Accent color, Mica/Acrylic support, dark mode
//! - macOS: System colors, vibrancy materials (future)
//! - Linux: GTK theme colors (future)

use crate::{Color, Result};

/// Platform-native background materials
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BackgroundMaterial {
    /// Windows-specific materials
    Windows(WindowsMaterial),

    /// macOS-specific materials (future)
    #[allow(dead_code)]
    MacOS(MacOSMaterial),

    /// Solid color fallback
    Solid(Color),
}

/// Windows background materials (Windows 10+)
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WindowsMaterial {
    /// Mica material (Windows 11+)
    Mica,

    /// Acrylic material (Windows 10+)
    Acrylic,

    /// Mica Alt material (Windows 11+)
    MicaAlt,
}

/// macOS vibrancy materials (future)
#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(dead_code)]
pub enum MacOSMaterial {
    Sidebar,
    HeaderView,
    Menu,
    Popover,
    Selection,
}

/// System theme information queried from the OS
#[derive(Debug, Clone)]
pub struct SystemTheme {
    /// System accent color (RGBA, 0.0-1.0)
    pub accent_color: Color,

    /// Whether the system is in dark mode
    pub is_dark_mode: bool,

    /// Whether the system supports transparent materials
    pub supports_transparency: bool,

    /// Available background materials for this platform/OS version
    pub available_materials: Vec<BackgroundMaterial>,

    /// Text color for the current theme
    pub text_color: Color,

    /// Secondary text color (lower contrast)
    pub text_secondary_color: Color,
}

impl SystemTheme {
    /// Query the system theme from the operating system
    ///
    /// # Example
    ///
    /// ```no_run
    /// use theme_engine::SystemTheme;
    ///
    /// let theme = SystemTheme::query()?;
    /// println!("Accent color: {:?}", theme.accent_color);
    /// println!("Dark mode: {}", theme.is_dark_mode);
    /// # Ok::<(), theme_engine::ThemeError>(())
    /// ```
    pub fn query() -> Result<Self> {
        #[cfg(target_os = "windows")]
        {
            Self::query_windows()
        }

        #[cfg(not(target_os = "windows"))]
        {
            // Fallback for unsupported platforms
            Ok(Self::default_theme())
        }
    }

    /// Default theme for unsupported platforms
    #[allow(dead_code)]
    pub(crate) fn default_theme() -> Self {
        Self {
            accent_color: Color::new(0.0, 0.47, 0.84, 1.0), // Default blue
            is_dark_mode: false,
            supports_transparency: false,
            available_materials: vec![BackgroundMaterial::Solid(Color::new(1.0, 1.0, 1.0, 1.0))],
            text_color: Color::new(0.0, 0.0, 0.0, 1.0),
            text_secondary_color: Color::new(0.4, 0.4, 0.4, 1.0),
        }
    }

    #[cfg(target_os = "windows")]
    fn query_windows() -> Result<Self> {
        use windows::Win32::Foundation::BOOL;
        use windows::Win32::Graphics::Dwm::DwmGetColorizationColor;

        // Query accent color from DWM
        let accent_color = unsafe {
            let mut color: u32 = 0;
            let mut opaque_blend: BOOL = BOOL(0);

            match DwmGetColorizationColor(&mut color as *mut u32, &mut opaque_blend as *mut BOOL) {
                Ok(_) => {
                    // DWM returns ARGB, convert to RGBA
                    let a = ((color >> 24) & 0xFF) as f32 / 255.0;
                    let r = ((color >> 16) & 0xFF) as f32 / 255.0;
                    let g = ((color >> 8) & 0xFF) as f32 / 255.0;
                    let b = (color & 0xFF) as f32 / 255.0;
                    Color::new(r, g, b, a)
                }
                Err(_) => {
                    // Fallback: query registry for accent color
                    Self::query_accent_color_from_registry()
                        .unwrap_or(Color::new(0.0, 0.47, 0.84, 1.0))
                }
            }
        };

        // Query dark mode
        let is_dark_mode = Self::query_windows_dark_mode();

        // Detect Windows version for material support
        let os_version = Self::get_windows_version();
        let supports_mica = os_version >= (10, 0, 22000); // Windows 11 build 22000+
        let supports_acrylic = os_version >= (10, 0, 16299); // Windows 10 Fall Creators Update

        let mut available_materials = Vec::new();

        if supports_mica {
            available_materials.push(BackgroundMaterial::Windows(WindowsMaterial::Mica));
            available_materials.push(BackgroundMaterial::Windows(WindowsMaterial::MicaAlt));
        }

        if supports_acrylic {
            available_materials.push(BackgroundMaterial::Windows(WindowsMaterial::Acrylic));
        }

        // Always provide solid fallback
        let bg_color = if is_dark_mode {
            Color::new(0.12, 0.12, 0.12, 1.0)
        } else {
            Color::new(0.95, 0.95, 0.95, 1.0)
        };
        available_materials.push(BackgroundMaterial::Solid(bg_color));

        let (text_color, text_secondary_color) = if is_dark_mode {
            (
                Color::new(1.0, 1.0, 1.0, 1.0),
                Color::new(0.7, 0.7, 0.7, 1.0),
            )
        } else {
            (
                Color::new(0.0, 0.0, 0.0, 1.0),
                Color::new(0.4, 0.4, 0.4, 1.0),
            )
        };

        Ok(Self {
            accent_color,
            is_dark_mode,
            supports_transparency: supports_acrylic,
            available_materials,
            text_color,
            text_secondary_color,
        })
    }

    #[cfg(target_os = "windows")]
    fn query_accent_color_from_registry() -> Option<Color> {
        use windows::core::HSTRING;
        use windows::Win32::System::Registry::{
            RegCloseKey, RegOpenKeyExW, RegQueryValueExW, HKEY_CURRENT_USER, KEY_READ,
            REG_VALUE_TYPE,
        };

        unsafe {
            let subkey = HSTRING::from("SOFTWARE\\Microsoft\\Windows\\DWM");
            let mut hkey = Default::default();

            if RegOpenKeyExW(HKEY_CURRENT_USER, &subkey, 0, KEY_READ, &mut hkey).is_ok() {
                let value_name = HSTRING::from("AccentColor");
                let mut data: u32 = 0;
                let mut data_size = std::mem::size_of::<u32>() as u32;
                let mut reg_type = REG_VALUE_TYPE::default();

                let result = RegQueryValueExW(
                    hkey,
                    &value_name,
                    None,
                    Some(&mut reg_type as *mut _),
                    Some(std::ptr::addr_of_mut!(data) as *mut u8),
                    Some(&mut data_size),
                );

                let _ = RegCloseKey(hkey);

                if result.is_ok() {
                    // Registry stores as ABGR
                    let a = ((data >> 24) & 0xFF) as f32 / 255.0;
                    let b = ((data >> 16) & 0xFF) as f32 / 255.0;
                    let g = ((data >> 8) & 0xFF) as f32 / 255.0;
                    let r = (data & 0xFF) as f32 / 255.0;
                    return Some(Color::new(r, g, b, a));
                }
            }
        }

        None
    }

    #[cfg(target_os = "windows")]
    fn query_windows_dark_mode() -> bool {
        use windows::core::HSTRING;
        use windows::Win32::System::Registry::{
            RegCloseKey, RegOpenKeyExW, RegQueryValueExW, HKEY_CURRENT_USER, KEY_READ,
            REG_VALUE_TYPE,
        };

        unsafe {
            let subkey =
                HSTRING::from("SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize");
            let mut hkey = Default::default();

            if RegOpenKeyExW(HKEY_CURRENT_USER, &subkey, 0, KEY_READ, &mut hkey).is_ok() {
                let value_name = HSTRING::from("AppsUseLightTheme");
                let mut data: u32 = 0;
                let mut data_size = std::mem::size_of::<u32>() as u32;
                let mut reg_type = REG_VALUE_TYPE::default();

                let result = RegQueryValueExW(
                    hkey,
                    &value_name,
                    None,
                    Some(&mut reg_type as *mut _),
                    Some(std::ptr::addr_of_mut!(data) as *mut u8),
                    Some(&mut data_size),
                );

                let _ = RegCloseKey(hkey);

                if result.is_ok() {
                    // 0 = dark mode, 1 = light mode
                    return data == 0;
                }
            }
        }

        false // Default to light mode
    }

    #[cfg(target_os = "windows")]
    fn get_windows_version() -> (u32, u32, u32) {
        // Use RtlGetVersion for accurate version detection
        // For now, use a simple heuristic based on available APIs
        // This would need proper implementation for production

        // Simple heuristic: Check if we're on Windows 11 by checking build number
        // This is a placeholder - proper implementation would use RtlGetVersion
        (10, 0, 22000) // Assume Windows 11 for now
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_theme() {
        let theme = SystemTheme::default_theme();

        assert!(theme.accent_color.w > 0.0);
        assert!(!theme.is_dark_mode);
        assert!(!theme.available_materials.is_empty());
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn test_query_windows() {
        let result = SystemTheme::query_windows();
        assert!(result.is_ok());

        let theme = result.unwrap();
        assert!(theme.accent_color.w > 0.0);
        assert!(!theme.available_materials.is_empty());
    }
}
