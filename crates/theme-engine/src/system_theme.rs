//! Layer 1: System Theme - Platform-native materials and colors
//!
//! Queries the operating system for native theme information:
//! - Windows: Accent color, Mica/Acrylic support, dark mode
//! - macOS: System colors, vibrancy materials (future)
//! - Linux: GTK theme colors (future)

use crate::{Color, Result};
use plat_core::BackdropMaterial;

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
    pub available_materials: Vec<BackdropMaterial>,

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
            available_materials: vec![BackdropMaterial::None],
            text_color: Color::new(0.0, 0.0, 0.0, 1.0),
            text_secondary_color: Color::new(0.4, 0.4, 0.4, 1.0),
        }
    }

    #[cfg(target_os = "windows")]
    fn query_windows() -> Result<Self> {
        use windows::Win32::Graphics::Dwm::DwmGetColorizationColor;
        use windows_core::BOOL;

        // Query accent color from DWM
        // SAFETY: The pointers passed to DwmGetColorizationColor are valid stack-allocated variables.
        // The function writes to these addresses and returns an error code if it fails.
        // We handle the error result appropriately.
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
            available_materials.push(BackdropMaterial::Mica);
            available_materials.push(BackdropMaterial::MicaAlt);
        }

        if supports_acrylic {
            available_materials.push(BackdropMaterial::Acrylic);
        }

        // Always provide fallback
        available_materials.push(BackdropMaterial::None);

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

        // SAFETY: FFI calls to Windows Registry API.
        // - Pointers to stack variables (hkey, data, data_size) are valid.
        // - We check return codes.
        // - Buffer size is handled correctly.
        unsafe {
            let subkey = HSTRING::from("SOFTWARE\\Microsoft\\Windows\\DWM");
            let mut hkey = Default::default();

            if RegOpenKeyExW(HKEY_CURRENT_USER, &subkey, None, KEY_READ, &mut hkey).is_ok() {
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

                if result.is_ok()
                    && reg_type == windows::Win32::System::Registry::REG_DWORD
                    && data_size == std::mem::size_of::<u32>() as u32
                {
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

        // SAFETY: Standard Registry API usage with stack-allocated buffers.
        // Checked against Microsoft docs for RegQueryValueExW.
        unsafe {
            let subkey =
                HSTRING::from("SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize");
            let mut hkey = Default::default();

            if RegOpenKeyExW(HKEY_CURRENT_USER, &subkey, None, KEY_READ, &mut hkey).is_ok() {
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

                if result.is_ok()
                    && reg_type == windows::Win32::System::Registry::REG_DWORD
                    && data_size == std::mem::size_of::<u32>() as u32
                {
                    // 0 = dark mode, 1 = light mode
                    return data == 0;
                }
            }
        }

        false // Default to light mode
    }

    #[cfg(target_os = "windows")]
    fn get_windows_version() -> (u32, u32, u32) {
        // Prefer registry method as GetVersionExW is subject to compatibility shims
        // (returns 6.x for Windows 8+ without a proper app manifest)
        // The registry always contains the real build number
        Self::detect_windows_version_from_registry()
    }

    #[cfg(target_os = "windows")]
    fn detect_windows_version_from_registry() -> (u32, u32, u32) {
        use windows::core::HSTRING;
        use windows::Win32::System::Registry::{
            RegCloseKey, RegOpenKeyExW, RegQueryValueExW, HKEY_LOCAL_MACHINE, KEY_READ,
            REG_VALUE_TYPE,
        };

        // SAFETY: Reading CurrentBuildNumber from HKLM.
        // Buffer `data` is 64 bytes, sufficient for a version string.
        // `data_size` tracks buffer length.
        // `RegQueryValueExW` writes up to `data_size` bytes.
        unsafe {
            let subkey = HSTRING::from("SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion");
            let mut hkey = Default::default();

            if RegOpenKeyExW(HKEY_LOCAL_MACHINE, &subkey, None, KEY_READ, &mut hkey).is_ok() {
                // Read CurrentBuildNumber (stored as string)
                let value_name = HSTRING::from("CurrentBuildNumber");
                let mut data = [0u8; 64];
                let mut data_size = data.len() as u32;
                let mut reg_type = REG_VALUE_TYPE::default();

                let result = RegQueryValueExW(
                    hkey,
                    &value_name,
                    None,
                    Some(&mut reg_type as *mut _),
                    Some(data.as_mut_ptr()),
                    Some(&mut data_size),
                );

                let _ = RegCloseKey(hkey);

                if result.is_ok() && reg_type == windows::Win32::System::Registry::REG_SZ {
                    // Safe parsing with bounds checking
                    // Use min(data_size, 64) to prevent reading uninitialized memory
                    // if registry somehow claimed to write more than buffer size
                    let len = (data_size as usize).min(data.len());

                    if let Some(build) = Self::parse_build_number(&data[..len]) {
                        return (10, 0, build);
                    }
                }
            }
        }

        // Ultimate fallback: assume Windows 10 version 1809 (conservative)
        // This ensures we don't enable Mica on systems that don't support it
        (10, 0, 17763)
    }

    /// Safely parse a build number from UTF-16 bytes (little-endian)
    /// Handles null termination and invalid lengths gracefully
    #[allow(dead_code)]
    fn parse_build_number(data: &[u8]) -> Option<u32> {
        if data.is_empty() {
            return None;
        }

        // Convert bytes to u16s (UTF-16)
        // chunks_exact ensures we only process complete u16s (2 bytes)
        // ignoring any trailing odd byte
        let chars: Vec<u16> = data
            .chunks_exact(2)
            .map(|chunk| u16::from_ne_bytes([chunk[0], chunk[1]]))
            .collect();

        if chars.is_empty() {
            return None;
        }

        // Remove null terminator if present
        let chars = if let Some(&0) = chars.last() {
            &chars[..chars.len() - 1]
        } else {
            &chars[..]
        };

        // Convert to string and parse
        let build_str = String::from_utf16_lossy(chars);
        build_str.trim().parse::<u32>().ok()
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

    #[test]
    fn test_registry_parsing_exploit() {
        // Simulates an exploit where a malicious or broken registry entry claims to be a string
        // but provides an incomplete or malformed buffer.
        let empty_buf: &[u8] = &[];
        assert!(SystemTheme::parse_build_number(empty_buf).is_none());

        let malformed_buf: &[u8] = &[0, 255, 12]; // Odd length, unaligned
        assert!(SystemTheme::parse_build_number(malformed_buf).is_none());

        // Simulate an attack trying to read beyond bounds of a small buffer.
        let oob_buf: &[u8] = &[b'1', 0, b'0', 0, b'.', 0, b'0', 0, b'.', 0, b'1', 0]; // 10.0.1
        assert_eq!(SystemTheme::parse_build_number(oob_buf), None); // Parse returns None if it fails to convert to u32

        let num_buf: &[u8] = &[b'2', 0, b'2', 0, b'0', 0, b'0', 0, b'0', 0]; // 22000
        assert_eq!(SystemTheme::parse_build_number(num_buf), Some(22000));
    }

    #[test]
    fn test_registry_parsing_safety() {
        // Test empty
        assert!(SystemTheme::parse_build_number(&[]).is_none());

        // Test 1 byte (previously caused panic)
        assert!(SystemTheme::parse_build_number(&[0]).is_none());

        // Test odd bytes (should be safe, ignores last byte)
        assert!(SystemTheme::parse_build_number(&[0, 0, 0]).is_none()); // "\0" -> empty string -> parse fail

        // Test valid string "22000"
        let valid_str: Vec<u8> = "22000"
            .encode_utf16()
            .flat_map(|u| u.to_ne_bytes())
            .collect();
        assert_eq!(SystemTheme::parse_build_number(&valid_str), Some(22000));

        // Test valid string with null terminator "22000\0"
        let mut valid_with_null = valid_str.clone();
        valid_with_null.extend(0u16.to_ne_bytes());
        assert_eq!(
            SystemTheme::parse_build_number(&valid_with_null),
            Some(22000)
        );

        // Test valid string with garbage at end (if data len > string len)
        // parse_build_number takes slice, so it parses strictly.
        // If "22000\0garbage", it will try to parse "22000\0garb..." as string?
        // No, it converts ALL bytes to string.
        // If null terminator is at the very end, it removes it.
        // If null terminator is in middle, string contains \0. "22000\0garbage".
        // Rust string parse might fail or stop? "22000\0garbage".parse::<u32>() fails.
        // So it's safe.

        // Test invalid utf16
        // 0xD800 is a lone surrogate
        let invalid_utf16 = 0xD800u16.to_ne_bytes();
        // String::from_utf16_lossy replaces with replacement char, which fails parsing u32
        assert!(SystemTheme::parse_build_number(&invalid_utf16).is_none());
    }
}
