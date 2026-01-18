# Theming and Styling Implementation Guide

**Status:** Draft
**Version:** 0.1.0
**Date:** 2026-01-18
**Related ADR:** [ADR 0009: Three-Layer Theming Architecture](../adr/0009-three-layer-theming-architecture.md)

## Overview

This document provides implementation details for Arthropod's three-layer theming system. It covers API design, platform-specific code, macro implementation, and rendering integration.

## Table of Contents

1. [Architecture Recap](#architecture-recap)
2. [Layer 1: Platform Native Defaults](#layer-1-platform-native-defaults)
3. [Layer 2: Semantic Design Tokens](#layer-2-semantic-design-tokens)
4. [Layer 3: CSS-like Custom Styling](#layer-3-css-like-custom-styling)
5. [Rendering Integration](#rendering-integration)
6. [Implementation Examples](#implementation-examples)
7. [Testing Strategy](#testing-strategy)

## Architecture Recap

```
User Code (style! macro)
    ↓
Design Tokens (platform-agnostic)
    ↓
System Theme (platform-specific)
    ↓
Render Engine (GPU)
```

**Data Flow**:
1. OS provides native materials/colors → `SystemTheme`
2. Tokens resolve to platform values → `DesignTokens`
3. Developer styles reference tokens → `Style`
4. Styles resolve to concrete values → Render instructions
5. Render engine draws pixels

## Layer 1: Platform Native Defaults

### File Structure

```
crates/plat-core/src/
├── theme/
│   ├── mod.rs              # Public API
│   ├── system_theme.rs     # Platform-agnostic types
│   └── platform/
│       ├── windows.rs      # Windows implementation
│       ├── macos.rs        # macOS implementation
│       ├── linux.rs        # Linux implementation
│       └── android.rs      # Android implementation
```

### Core Types (`theme/system_theme.rs`)

```rust
use crate::Color;

/// System theme queried from the operating system
#[derive(Debug, Clone)]
pub struct SystemTheme {
    /// Primary accent color from OS
    pub accent_color: Color,

    /// Background material appropriate for this platform
    pub background_material: BackgroundMaterial,

    /// Primary text color
    pub text_color: Color,

    /// Secondary text color (dimmed)
    pub text_color_secondary: Color,

    /// Whether platform supports transparency/blur
    pub supports_transparency: bool,

    /// Whether user prefers dark mode
    pub prefers_dark_mode: bool,

    /// Whether reduced motion is enabled (accessibility)
    pub prefers_reduced_motion: bool,

    /// Platform-specific capabilities
    pub capabilities: PlatformCapabilities,
}

impl SystemTheme {
    /// Query system theme from the OS
    pub fn query() -> Result<Self, PlatformError> {
        platform::query_system_theme()
    }

    /// Subscribe to system theme changes (dark mode, accent color, etc.)
    pub fn subscribe_changes(callback: impl Fn(SystemTheme) + Send + 'static) {
        platform::subscribe_theme_changes(callback);
    }
}

/// Platform-specific background materials
#[derive(Debug, Clone)]
pub enum BackgroundMaterial {
    #[cfg(target_os = "windows")]
    Windows(WindowsMaterial),

    #[cfg(target_os = "macos")]
    MacOS(MacOSVibrancy),

    #[cfg(target_os = "android")]
    Android(MaterialYou),

    #[cfg(target_os = "linux")]
    Linux(LinuxMaterial),

    /// Fallback for platforms without native materials
    Solid(Color),
}

/// Windows-specific materials
#[cfg(target_os = "windows")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowsMaterial {
    /// Windows 11 Mica backdrop (translucent, tinted)
    Mica,
    /// Windows 11 Mica Alt (more opaque variant)
    MicaAlt,
    /// Windows 10+ Acrylic (blur + transparency)
    Acrylic,
    /// Solid color fallback
    Solid(Color),
}

/// macOS-specific vibrancy materials
#[cfg(target_os = "macos")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MacOSVibrancy {
    /// Sidebar material (NSVisualEffectMaterial.sidebar)
    Sidebar,
    /// Main content area material
    Content,
    /// Menu/popover material
    Menu,
    /// Tooltip/HUD material
    HUD,
    /// Sheet material
    Sheet,
    /// Popover material
    Popover,
}

/// Android Material You colors
#[cfg(target_os = "android")]
#[derive(Debug, Clone)]
pub struct MaterialYou {
    /// Primary color from wallpaper
    pub primary: Color,
    /// Secondary color
    pub secondary: Color,
    /// Tertiary color
    pub tertiary: Color,
    /// Surface color
    pub surface: Color,
    /// Surface variant
    pub surface_variant: Color,
    /// Elevation level (for shadows)
    pub elevation: f32,
}

/// Linux theme colors (GTK/Qt)
#[cfg(target_os = "linux")]
#[derive(Debug, Clone)]
pub struct LinuxMaterial {
    /// Theme name (e.g., "Adwaita-dark")
    pub theme_name: String,
    /// Base background color
    pub base: Color,
    /// Window background
    pub background: Color,
    /// Whether theme supports transparency
    pub supports_transparency: bool,
}

/// Platform capabilities
#[derive(Debug, Clone, Default)]
pub struct PlatformCapabilities {
    /// Supports blur effects (Mica, Acrylic, Vibrancy)
    pub blur: bool,
    /// Supports dynamic wallpaper colors (Material You)
    pub dynamic_colors: bool,
    /// Supports custom window chrome
    pub custom_chrome: bool,
    /// Supports shadow effects
    pub shadows: bool,
}
```

### Windows Implementation (`theme/platform/windows.rs`)

```rust
use super::*;
use windows::Win32::{
    Foundation::*,
    Graphics::Dwm::*,
    System::Registry::*,
    UI::WindowsAndMessaging::*,
};

/// Query Windows system theme
pub fn query_system_theme() -> Result<SystemTheme, PlatformError> {
    unsafe {
        Ok(SystemTheme {
            accent_color: query_accent_color()?,
            background_material: query_background_material(),
            text_color: query_text_color(),
            text_color_secondary: query_text_color_secondary(),
            supports_transparency: supports_transparency(),
            prefers_dark_mode: query_dark_mode(),
            prefers_reduced_motion: query_reduced_motion(),
            capabilities: PlatformCapabilities {
                blur: supports_mica() || supports_acrylic(),
                dynamic_colors: false, // Windows doesn't have Material You
                custom_chrome: true,
                shadows: true,
            },
        })
    }
}

/// Query Windows accent color from registry
unsafe fn query_accent_color() -> Result<Color, PlatformError> {
    // HKEY_CURRENT_USER\Software\Microsoft\Windows\DWM\AccentColor
    let mut hkey = HKEY::default();
    let result = RegOpenKeyExW(
        HKEY_CURRENT_USER,
        w!("Software\\Microsoft\\Windows\\DWM"),
        0,
        KEY_READ,
        &mut hkey,
    );

    if result != 0 {
        return Ok(Color::rgb(0.0, 0.47, 0.84)); // Default Windows blue
    }

    let mut data: u32 = 0;
    let mut size = std::mem::size_of::<u32>() as u32;
    let result = RegQueryValueExW(
        hkey,
        w!("AccentColor"),
        None,
        None,
        Some(&mut data as *mut u32 as *mut u8),
        Some(&mut size),
    );

    RegCloseKey(hkey);

    if result != 0 {
        return Ok(Color::rgb(0.0, 0.47, 0.84)); // Default
    }

    // AccentColor is ABGR format
    let a = ((data >> 24) & 0xFF) as f32 / 255.0;
    let b = ((data >> 16) & 0xFF) as f32 / 255.0;
    let g = ((data >> 8) & 0xFF) as f32 / 255.0;
    let r = (data & 0xFF) as f32 / 255.0;

    Ok(Color::rgba(r, g, b, a))
}

/// Determine appropriate background material
unsafe fn query_background_material() -> BackgroundMaterial {
    if supports_mica() {
        BackgroundMaterial::Windows(WindowsMaterial::Mica)
    } else if supports_acrylic() {
        BackgroundMaterial::Windows(WindowsMaterial::Acrylic)
    } else {
        // Fallback to solid color
        let dark_mode = query_dark_mode();
        let color = if dark_mode {
            Color::rgb(0.12, 0.12, 0.12)
        } else {
            Color::rgb(0.96, 0.96, 0.96)
        };
        BackgroundMaterial::Windows(WindowsMaterial::Solid(color))
    }
}

/// Check if Mica is supported (Windows 11 22000+)
fn supports_mica() -> bool {
    let version = windows_version::OsVersion::current();
    version.major >= 10 && version.build >= 22000
}

/// Check if Acrylic is supported (Windows 10 1803+)
fn supports_acrylic() -> bool {
    let version = windows_version::OsVersion::current();
    version.major >= 10 && version.build >= 17134
}

/// Check if dark mode is enabled
unsafe fn query_dark_mode() -> bool {
    // HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Themes\Personalize\AppsUseLightTheme
    let mut hkey = HKEY::default();
    let result = RegOpenKeyExW(
        HKEY_CURRENT_USER,
        w!("Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize"),
        0,
        KEY_READ,
        &mut hkey,
    );

    if result != 0 {
        return false; // Default to light mode
    }

    let mut data: u32 = 0;
    let mut size = std::mem::size_of::<u32>() as u32;
    let result = RegQueryValueExW(
        hkey,
        w!("AppsUseLightTheme"),
        None,
        None,
        Some(&mut data as *mut u32 as *mut u8),
        Some(&mut size),
    );

    RegCloseKey(hkey);

    if result != 0 {
        return false;
    }

    data == 0 // 0 = dark mode, 1 = light mode
}

/// Apply background material to window
pub fn apply_background_material(
    hwnd: HWND,
    material: WindowsMaterial,
) -> Result<(), PlatformError> {
    unsafe {
        match material {
            WindowsMaterial::Mica => {
                let backdrop_type = 2i32; // DWMSBT_MAINWINDOW
                DwmSetWindowAttribute(
                    hwnd,
                    DWMWA_SYSTEMBACKDROP_TYPE,
                    &backdrop_type as *const i32 as *const _,
                    std::mem::size_of::<i32>() as u32,
                )
                .map_err(|e| PlatformError::WindowOperation(format!("Mica failed: {}", e)))?;
            }
            WindowsMaterial::MicaAlt => {
                let backdrop_type = 4i32; // DWMSBT_TABBEDWINDOW
                DwmSetWindowAttribute(
                    hwnd,
                    DWMWA_SYSTEMBACKDROP_TYPE,
                    &backdrop_type as *const i32 as *const _,
                    std::mem::size_of::<i32>() as u32,
                )
                .map_err(|e| PlatformError::WindowOperation(format!("MicaAlt failed: {}", e)))?;
            }
            WindowsMaterial::Acrylic => {
                // Windows 10 Acrylic requires SetWindowCompositionAttribute
                // (Undocumented API, requires manual definition)
                apply_acrylic(hwnd)?;
            }
            WindowsMaterial::Solid(color) => {
                // Standard window background (no special API needed)
                // Just paint with solid color
            }
        }
        Ok(())
    }
}

// Subscribe to theme changes via WM_SETTINGCHANGE
pub fn subscribe_theme_changes(callback: impl Fn(SystemTheme) + Send + 'static) {
    // Register for WM_SETTINGCHANGE with wParam = 0, lParam = "ImmersiveColorSet"
    // This fires when user changes accent color or dark/light mode
    // Implementation requires adding WM_SETTINGCHANGE to wndproc
}
```

### macOS Implementation (`theme/platform/macos.rs`)

```rust
use super::*;
use objc2::runtime::NSObject;
use objc2_foundation::{NSColor, NSString};
use objc2_app_kit::{NSAppearance, NSVisualEffectView, NSVisualEffectMaterial};

pub fn query_system_theme() -> Result<SystemTheme, PlatformError> {
    unsafe {
        Ok(SystemTheme {
            accent_color: query_accent_color(),
            background_material: query_background_material(),
            text_color: query_text_color(),
            text_color_secondary: query_text_color_secondary(),
            supports_transparency: true,
            prefers_dark_mode: query_dark_mode(),
            prefers_reduced_motion: query_reduced_motion(),
            capabilities: PlatformCapabilities {
                blur: true,
                dynamic_colors: false,
                custom_chrome: true,
                shadows: true,
            },
        })
    }
}

unsafe fn query_accent_color() -> Color {
    let ns_color = NSColor::controlAccentColor();
    ns_color_to_color(ns_color)
}

unsafe fn query_background_material() -> BackgroundMaterial {
    BackgroundMaterial::MacOS(MacOSVibrancy::Content)
}

unsafe fn query_dark_mode() -> bool {
    let appearance = NSAppearance::currentDrawingAppearance();
    let name = appearance.name();
    name.to_string().contains("Dark")
}

pub fn apply_vibrancy(
    ns_view: *mut NSObject,
    material: MacOSVibrancy,
) -> Result<(), PlatformError> {
    unsafe {
        let effect_view = NSVisualEffectView::alloc();
        effect_view.initWithFrame(/* view bounds */);

        let ns_material = match material {
            MacOSVibrancy::Sidebar => NSVisualEffectMaterial::Sidebar,
            MacOSVibrancy::Content => NSVisualEffectMaterial::UnderWindowBackground,
            MacOSVibrancy::Menu => NSVisualEffectMaterial::Menu,
            MacOSVibrancy::HUD => NSVisualEffectMaterial::HUDWindow,
            MacOSVibrancy::Sheet => NSVisualEffectMaterial::Sheet,
            MacOSVibrancy::Popover => NSVisualEffectMaterial::Popover,
        };

        effect_view.setMaterial(ns_material);
        // Add effect_view to window content view
        Ok(())
    }
}
```

### Public API (`theme/mod.rs`)

```rust
mod system_theme;
mod platform;

pub use system_theme::*;

// Re-export platform-specific types
#[cfg(target_os = "windows")]
pub use system_theme::WindowsMaterial;

#[cfg(target_os = "macos")]
pub use system_theme::MacOSVibrancy;

#[cfg(target_os = "android")]
pub use system_theme::MaterialYou;

/// Extension trait for applying background materials to windows
pub trait WindowMaterialExt {
    fn set_background_material(&self, material: BackgroundMaterial) -> Result<(), PlatformError>;
}

impl WindowMaterialExt for crate::Window {
    fn set_background_material(&self, material: BackgroundMaterial) -> Result<(), PlatformError> {
        platform::apply_background_material(&self.inner, material)
    }
}
```

## Layer 2: Semantic Design Tokens

### File Structure

```
crates/arthropod/src/
├── theme/
│   ├── mod.rs              # Public API
│   ├── tokens.rs           # DesignTokens struct
│   └── token_value.rs      # TokenValue enum
```

### Core Types (`theme/tokens.rs`)

```rust
use plat_core::SystemTheme;
use crate::Color;

/// Platform-agnostic design tokens
#[derive(Debug, Clone)]
pub struct DesignTokens {
    // === Surfaces ===
    /// Primary background (window background)
    pub surface_primary: TokenValue,
    /// Elevated surfaces (cards, panels)
    pub surface_elevated: TokenValue,
    /// Overlay surfaces (tooltips, dropdowns)
    pub surface_overlay: TokenValue,
    /// Disabled surface
    pub surface_disabled: TokenValue,

    // === Text ===
    pub text_primary: TokenValue,
    pub text_secondary: TokenValue,
    pub text_tertiary: TokenValue,
    pub text_disabled: TokenValue,
    pub text_on_accent: TokenValue,

    // === Accent ===
    pub accent: TokenValue,
    pub accent_hover: TokenValue,
    pub accent_pressed: TokenValue,
    pub accent_disabled: TokenValue,

    // === Semantic Colors ===
    pub success: TokenValue,
    pub warning: TokenValue,
    pub error: TokenValue,
    pub info: TokenValue,

    // === Spacing (rem-like, scales with base size) ===
    pub space_xs: f32,   // 4px
    pub space_sm: f32,   // 8px
    pub space_md: f32,   // 16px
    pub space_lg: f32,   // 24px
    pub space_xl: f32,   // 32px
    pub space_2xl: f32,  // 48px

    // === Border Radius ===
    pub radius_sm: f32,  // 4px
    pub radius_md: f32,  // 8px
    pub radius_lg: f32,  // 12px
    pub radius_xl: f32,  // 16px
    pub radius_full: f32, // 9999px (pill shape)

    // === Shadows ===
    pub shadow_sm: Shadow,
    pub shadow_md: Shadow,
    pub shadow_lg: Shadow,
    pub shadow_xl: Shadow,

    // === Typography ===
    pub font_size_xs: f32,   // 12px
    pub font_size_sm: f32,   // 14px
    pub font_size_md: f32,   // 16px
    pub font_size_lg: f32,   // 18px
    pub font_size_xl: f32,   // 24px
    pub font_size_2xl: f32,  // 32px

    pub font_weight_normal: u16,  // 400
    pub font_weight_medium: u16,  // 500
    pub font_weight_bold: u16,    // 700

    // === Animation ===
    pub transition_fast: Duration,    // 100ms
    pub transition_normal: Duration,  // 200ms
    pub transition_slow: Duration,    // 300ms
}

impl DesignTokens {
    /// Create design tokens from system theme (native defaults)
    pub fn from_system(system_theme: &SystemTheme) -> Self {
        let dark_mode = system_theme.prefers_dark_mode;

        Self {
            // Surfaces resolve to platform materials
            surface_primary: TokenValue::Material(system_theme.background_material.clone()),
            surface_elevated: if dark_mode {
                TokenValue::Color(Color::rgba(0.18, 0.18, 0.18, 1.0))
            } else {
                TokenValue::Color(Color::rgba(1.0, 1.0, 1.0, 1.0))
            },
            surface_overlay: TokenValue::Color(
                if dark_mode {
                    Color::rgba(0.22, 0.22, 0.22, 0.95)
                } else {
                    Color::rgba(1.0, 1.0, 1.0, 0.95)
                }
            ),

            // Text colors from system theme
            text_primary: TokenValue::Color(system_theme.text_color),
            text_secondary: TokenValue::Color(system_theme.text_color_secondary),
            text_tertiary: TokenValue::Color(
                system_theme.text_color.with_alpha(0.6)
            ),

            // Accent from system
            accent: TokenValue::Color(system_theme.accent_color),
            accent_hover: TokenValue::Color(system_theme.accent_color.lighten(0.1)),
            accent_pressed: TokenValue::Color(system_theme.accent_color.darken(0.1)),

            // Static values (platform-independent)
            space_md: 16.0,
            radius_md: 8.0,
            font_size_md: 16.0,
            // ... (all other values with defaults)
        }
    }

    /// Override accent color (for custom branding)
    pub fn with_custom_accent(mut self, color: Color) -> Self {
        self.accent = TokenValue::Color(color);
        self.accent_hover = TokenValue::Color(color.lighten(0.1));
        self.accent_pressed = TokenValue::Color(color.darken(0.1));
        self.accent_disabled = TokenValue::Color(color.with_alpha(0.4));
        self
    }

    /// Override all surface colors (full custom theme)
    pub fn with_custom_surfaces(
        mut self,
        primary: Color,
        elevated: Color,
        overlay: Color,
    ) -> Self {
        self.surface_primary = TokenValue::Color(primary);
        self.surface_elevated = TokenValue::Color(elevated);
        self.surface_overlay = TokenValue::Color(overlay);
        self
    }
}
```

### Token Value Type (`theme/token_value.rs`)

```rust
use plat_core::BackgroundMaterial;
use crate::{Color, Gradient};

/// Value that a design token can resolve to
#[derive(Debug, Clone)]
pub enum TokenValue {
    /// Native platform material (Mica, Vibrancy, etc.)
    Material(BackgroundMaterial),
    /// Solid color
    Color(Color),
    /// Gradient (linear or radial)
    Gradient(Gradient),
}

impl TokenValue {
    /// Resolve token to a concrete rendering value
    pub fn resolve(&self) -> ResolvedValue {
        match self {
            TokenValue::Material(m) => ResolvedValue::Material(m.clone()),
            TokenValue::Color(c) => ResolvedValue::Color(*c),
            TokenValue::Gradient(g) => ResolvedValue::Gradient(g.clone()),
        }
    }

    /// Extract as color (for backends that don't support materials)
    pub fn as_color(&self, fallback: Color) -> Color {
        match self {
            TokenValue::Color(c) => *c,
            TokenValue::Material(BackgroundMaterial::Solid(c)) => *c,
            _ => fallback,
        }
    }
}

/// Resolved value ready for rendering
#[derive(Debug, Clone)]
pub enum ResolvedValue {
    Material(BackgroundMaterial),
    Color(Color),
    Gradient(Gradient),
}
```

## Layer 3: CSS-like Custom Styling

### File Structure

```
crates/arthropod/src/
├── style/
│   ├── mod.rs              # Public API
│   ├── style_sheet.rs      # Style struct
│   ├── properties.rs       # Style properties
│   └── macros.rs           # style! macro
crates/arthropod-macros/src/
└── style.rs                # Proc macro implementation
```

### Style Struct (`style/style_sheet.rs`)

```rust
use crate::theme::{DesignTokens, TokenValue};
use crate::{Color, Gradient};

/// Complete style definition for a widget
#[derive(Debug, Clone, Default)]
pub struct Style {
    pub background: Option<Background>,
    pub border: Option<Border>,
    pub border_radius: Option<BorderRadius>,
    pub padding: Option<Padding>,
    pub margin: Option<Margin>,
    pub box_shadow: Option<Shadow>,
    pub opacity: Option<f32>,
    pub width: Option<Dimension>,
    pub height: Option<Dimension>,
}

#[derive(Debug, Clone)]
pub enum Background {
    Material(BackgroundMaterial),
    Color(Color),
    Gradient(Gradient),
    Token(TokenValue),
}

#[derive(Debug, Clone)]
pub struct Border {
    pub width: f32,
    pub color: Color,
    pub style: BorderStyle,
}

#[derive(Debug, Clone, Copy)]
pub enum BorderStyle {
    Solid,
    Dashed,
    Dotted,
    None,
}

#[derive(Debug, Clone, Copy)]
pub struct BorderRadius {
    pub top_left: f32,
    pub top_right: f32,
    pub bottom_right: f32,
    pub bottom_left: f32,
}

impl BorderRadius {
    pub fn uniform(radius: f32) -> Self {
        Self {
            top_left: radius,
            top_right: radius,
            bottom_right: radius,
            bottom_left: radius,
        }
    }
}

impl Style {
    /// Resolve all token references to concrete values
    pub fn resolve(&self, tokens: &DesignTokens) -> ResolvedStyle {
        ResolvedStyle {
            background: self.background.as_ref().map(|bg| match bg {
                Background::Token(token) => token.resolve(),
                Background::Material(m) => ResolvedValue::Material(m.clone()),
                Background::Color(c) => ResolvedValue::Color(*c),
                Background::Gradient(g) => ResolvedValue::Gradient(g.clone()),
            }),
            border: self.border.clone(),
            border_radius: self.border_radius,
            // ... resolve all properties
        }
    }
}

/// Style with all tokens resolved to concrete values
#[derive(Debug, Clone)]
pub struct ResolvedStyle {
    pub background: Option<ResolvedValue>,
    pub border: Option<Border>,
    pub border_radius: Option<BorderRadius>,
    pub padding: Option<Padding>,
    pub margin: Option<Margin>,
    pub box_shadow: Option<Shadow>,
    pub opacity: Option<f32>,
}
```

### Macro Implementation (`arthropod-macros/src/style.rs`)

```rust
use proc_macro::TokenStream;
use syn::{parse_macro_input, Expr};
use quote::quote;

/// Parse CSS-like syntax and generate Style struct
pub fn style_impl(input: TokenStream) -> TokenStream {
    // Parse CSS-like syntax
    let style_decl = parse_macro_input!(input as StyleDeclaration);

    // Generate Rust code
    let generated = quote! {
        Style {
            background: #background,
            border_radius: #border_radius,
            padding: #padding,
            // ... all properties
        }
    };

    generated.into()
}

// Example parsing (simplified):
// Input:  background: tokens.surface_elevated;
// Output: Some(Background::Token(tokens.surface_elevated))
```

## Rendering Integration

### render-engine Integration

```rust
// In render-engine/src/backend/wgpu_backend.rs

impl WgpuBackend {
    /// Apply resolved style to a window
    pub fn apply_window_style(
        &mut self,
        window: &Window,
        style: &ResolvedStyle,
    ) -> Result<()> {
        // Apply background material
        if let Some(ResolvedValue::Material(material)) = &style.background {
            window.set_background_material(material.clone())?;
        }

        // Set clear color for non-material backgrounds
        if let Some(ResolvedValue::Color(color)) = &style.background {
            self.set_clear_color(*color);
        }

        // Other properties render as geometry
        // (borders, shadows, etc.)

        Ok(())
    }
}
```

## Implementation Examples

### Example 1: System Default Theme

```rust
use arthropod::prelude::*;

fn main() {
    let app = Application::new();
    let window = app.create_window();

    // Get system theme
    let system_theme = window.get_system_theme();

    // Create tokens from system (native appearance)
    let tokens = DesignTokens::from_system(&system_theme);

    // On Windows 11: tokens.surface_primary = Mica
    // On macOS: tokens.surface_primary = Vibrancy::Content
    // On Linux: tokens.surface_primary = GTK theme background

    // Apply to window
    window.set_background_material(tokens.surface_primary);
}
```

### Example 2: Custom Branded Theme

```rust
let system_theme = window.get_system_theme();
let tokens = DesignTokens::from_system(&system_theme)
    .with_custom_accent(Color::rgb(0.2, 0.6, 1.0)) // Brand blue
    .with_custom_surfaces(
        Color::rgb(0.98, 0.98, 0.98), // Light gray
        Color::rgb(1.0, 1.0, 1.0),    // White
        Color::rgba(0.0, 0.0, 0.0, 0.8), // Dark overlay
    );

// Now tokens use custom colors but keep native materials where appropriate
```

### Example 3: Complete Style Definition

```rust
let card_style = style! {
    background: tokens.surface_elevated;
    border_radius: tokens.radius_lg;
    padding: tokens.space_md;
    box_shadow: tokens.shadow_md;

    &:hover {
        box_shadow: tokens.shadow_lg;
        transform: translateY(-2px);
    }

    @media (prefers_color_scheme: dark) {
        border: (1.0, rgba(255, 255, 255, 0.1));
    }
};

// Resolve and apply
let resolved = card_style.resolve(&tokens);
render_engine.render_card(resolved);
```

## Testing Strategy

### Unit Tests

```rust
#[test]
fn test_token_resolution() {
    let system_theme = SystemTheme::query().unwrap();
    let tokens = DesignTokens::from_system(&system_theme);

    // Verify accent color is valid
    assert!(tokens.accent.as_color(Color::BLACK).r >= 0.0);

    // Verify spacing makes sense
    assert!(tokens.space_sm < tokens.space_md);
    assert!(tokens.space_md < tokens.space_lg);
}

#[test]
fn test_custom_accent() {
    let system_theme = SystemTheme::query().unwrap();
    let custom_color = Color::rgb(1.0, 0.0, 0.0); // Red
    let tokens = DesignTokens::from_system(&system_theme)
        .with_custom_accent(custom_color);

    assert_eq!(tokens.accent.as_color(Color::BLACK), custom_color);
}
```

### Integration Tests

```rust
#[test]
#[cfg(target_os = "windows")]
fn test_mica_application() {
    let app = Application::new();
    let window = app.create_window();

    let material = BackgroundMaterial::Windows(WindowsMaterial::Mica);
    window.set_background_material(material).unwrap();

    // Verify via DWM API that Mica is actually enabled
}
```

### Visual Regression Tests

Use screenshot comparison tools to verify:
- System theme correctly detected
- Materials render correctly on each platform
- Dark mode switches work
- Custom themes override correctly

## Performance Considerations

- **Token Resolution**: Cache resolved styles (don't resolve every frame)
- **System Theme Queries**: Query once at startup, update on WM_SETTINGCHANGE
- **Material Application**: Apply once per window, not every frame
- **Style Macro**: Expands at compile-time (zero runtime cost)

## Future Enhancements

- **Animation**: `transition: background 0.2s ease`
- **Responsive**: `@media (max-width: 600px)`
- **Theme Hot-Reload**: Developer mode for instant theme updates
- **CSS Variables**: `var(--custom-color)`
- **Theme Editor**: Visual tool for designing custom themes

## References

- [ADR 0009: Three-Layer Theming Architecture](../adr/0009-three-layer-theming-architecture.md)
- [ADR 0005: Platform Abstraction Strategy](../adr/0005-platform-abstraction-strategy.md)
- [Windows Mica Documentation](https://learn.microsoft.com/en-us/windows/apps/design/style/mica)
- [macOS NSVisualEffectView](https://developer.apple.com/documentation/appkit/nsvisualeffectview)
- [Material Design 3](https://m3.material.io/)
