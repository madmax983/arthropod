# ADR 0009: Three-Layer Theming and Styling Architecture

**Status:** Accepted

**Date:** 2026-01-18

**Deciders:** Architecture Team

## Context

Arthropod needs a theming and styling system that:

1. **Feels Native**: Automatically adopts OS-native materials and colors (Windows 11 Mica, macOS Vibrancy, Material You)
2. **Platform-Agnostic Development**: Developers write once, looks appropriate everywhere
3. **Full Control When Needed**: Designers can override system defaults with pixel-perfect control
4. **Type-Safe**: Compile-time checks for style properties
5. **Performance**: Zero runtime cost for static styles, minimal overhead for dynamic theming
6. **Familiar**: CSS-like syntax with Rust ergonomics

Challenges:
- Each platform has different native materials (Mica vs Vibrancy vs Material You)
- Design tokens need to resolve to platform-appropriate values
- Must support both "system default" and "custom branded" apps
- Avoid runtime overhead when styles are static

## Decision

We implement a **three-layer theming architecture** that separates platform capabilities, semantic design, and custom styling.

### Architecture Overview

```
┌─────────────────────────────────────────────────────┐
│  Layer 3: CSS-like Custom Styling (arthropod)      │
│  - style! macro with Rust type safety              │
│  - Familiar CSS syntax (padding, border-radius)    │
│  - Media queries, pseudo-states (:hover)           │
│  - Resolves to concrete rendering instructions     │
└───────────────────┬─────────────────────────────────┘
                    ↓
┌─────────────────────────────────────────────────────┐
│  Layer 2: Semantic Design Tokens (arthropod)       │
│  - Platform-agnostic tokens (surface_primary)      │
│  - Resolves to Layer 1 platform values             │
│  - Can be overridden for custom branding           │
│  - pub struct DesignTokens { ... }                 │
└───────────────────┬─────────────────────────────────┘
                    ↓
┌─────────────────────────────────────────────────────┐
│  Layer 1: Platform Native Defaults (plat-core)     │
│  - SystemTheme queries OS for native values        │
│  - Windows: Mica, Acrylic, accent color            │
│  - macOS: Vibrancy, accent color                   │
│  - Android: Material You dynamic colors            │
│  - Linux: GTK/Qt theme colors                      │
└───────────────────┬─────────────────────────────────┘
                    ↓
┌─────────────────────────────────────────────────────┐
│  render-engine (GPU rendering)                      │
│  - Receives resolved styles (concrete values)      │
│  - Renders backgrounds, borders, shadows           │
│  - No knowledge of tokens or materials             │
└─────────────────────────────────────────────────────┘
```

### Layer 1: Platform Native Defaults (plat-core)

**Responsibility**: Query OS APIs for native materials, colors, and capabilities.

**Location**: `crates/plat-core/src/theme.rs`

```rust
/// Platform-agnostic theme queried from the operating system
pub struct SystemTheme {
    pub accent_color: Color,
    pub background_material: BackgroundMaterial,
    pub text_color: Color,
    pub text_color_secondary: Color,
    pub supports_transparency: bool,
    pub prefers_dark_mode: bool,
}

/// Platform-specific background materials
#[derive(Debug, Clone)]
pub enum BackgroundMaterial {
    Windows(WindowsMaterial),
    MacOS(MacOSVibrancy),
    Android(MaterialYou),
    Linux(GtkTheme),
    Fallback(Color),
}

#[cfg(target_os = "windows")]
pub enum WindowsMaterial {
    Mica,           // Windows 11 backdrop
    Acrylic,        // Windows 10+ blur
    Solid(Color),   // Fallback
}

#[cfg(target_os = "macos")]
pub enum MacOSVibrancy {
    Sidebar,
    Content,
    Menu,
    Popover,
    HUD,
}

#[cfg(target_os = "android")]
pub struct MaterialYou {
    pub primary: Color,
    pub secondary: Color,
    pub tertiary: Color,
    pub surface: Color,
    pub elevation: f32,
}
```

**Platform-Specific Implementation Examples**:

```rust
// Windows: Query DWM for accent color and Mica support
impl WindowImpl {
    pub fn get_system_accent_color() -> Color {
        // HKEY_CURRENT_USER\Software\Microsoft\Windows\DWM\AccentColor
        unsafe { query_registry_accent_color() }
    }

    pub fn supports_mica() -> bool {
        // Check Windows 11 build number
        is_windows_11_or_later()
    }

    pub fn set_background_material(&self, material: WindowsMaterial) -> Result<()> {
        unsafe {
            match material {
                WindowsMaterial::Mica => {
                    DwmSetWindowAttribute(
                        self.hwnd,
                        DWMWA_SYSTEMBACKDROP_TYPE,
                        &2i32, // DWMSBT_MAINWINDOW
                        4
                    )?;
                }
                WindowsMaterial::Acrylic => {
                    // SetWindowCompositionAttribute for acrylic
                }
                WindowsMaterial::Solid(color) => {
                    // Standard window background
                }
            }
        }
        Ok(())
    }
}

// macOS: Query NSAppearance and user accent color
#[cfg(target_os = "macos")]
impl WindowImpl {
    pub fn get_system_accent_color() -> Color {
        unsafe {
            let color = NSColor::controlAccentColor();
            color.to_rgba()
        }
    }

    pub fn set_vibrancy(&self, material: MacOSVibrancy) -> Result<()> {
        unsafe {
            let effect = NSVisualEffectView::alloc();
            effect.setMaterial(material.to_ns_material());
            // Add to window content view
        }
        Ok(())
    }
}
```

**API Surface**:

```rust
// Exposed by plat-core EventLoop
impl EventLoop {
    pub fn get_system_theme(&self) -> SystemTheme { ... }
    pub fn subscribe_theme_changes(&self, callback: impl Fn(SystemTheme)) { ... }
}

// Exposed by plat-core Window
impl Window {
    pub fn set_background_material(&self, material: BackgroundMaterial) -> Result<()> { ... }
}
```

### Layer 2: Semantic Design Tokens (arthropod)

**Responsibility**: Platform-agnostic token system that resolves to platform values.

**Location**: `crates/arthropod/src/theme/tokens.rs`

```rust
/// Design tokens that resolve to platform-appropriate values
pub struct DesignTokens {
    // Surfaces
    pub surface_primary: TokenValue,   // Main background
    pub surface_elevated: TokenValue,  // Cards, modals
    pub surface_overlay: TokenValue,   // Tooltips, dropdowns

    // Text
    pub text_primary: TokenValue,
    pub text_secondary: TokenValue,
    pub text_disabled: TokenValue,

    // Accent
    pub accent: TokenValue,            // From OS or custom
    pub accent_hover: TokenValue,
    pub accent_pressed: TokenValue,

    // Spacing (static values)
    pub space_xs: f32,   // 4.0
    pub space_sm: f32,   // 8.0
    pub space_md: f32,   // 16.0
    pub space_lg: f32,   // 24.0
    pub space_xl: f32,   // 32.0

    // Border radius
    pub radius_sm: f32,  // 4.0
    pub radius_md: f32,  // 8.0
    pub radius_lg: f32,  // 12.0

    // Shadows
    pub shadow_sm: Shadow,
    pub shadow_md: Shadow,
    pub shadow_lg: Shadow,
}

#[derive(Debug, Clone)]
pub enum TokenValue {
    Material(BackgroundMaterial),  // Native OS material
    Color(Color),                  // Solid color
    Gradient(Gradient),            // Linear/radial gradient
}

impl DesignTokens {
    /// Create tokens from system theme (native defaults)
    pub fn from_system(event_loop: &EventLoop) -> Self {
        let system_theme = event_loop.get_system_theme();

        Self {
            surface_primary: TokenValue::Material(system_theme.background_material),
            accent: TokenValue::Color(system_theme.accent_color),
            text_primary: TokenValue::Color(system_theme.text_color),
            // ... resolve all tokens from system
            space_md: 16.0,
            radius_md: 8.0,
            // ...
        }
    }

    /// Override with custom values (for branded apps)
    pub fn with_custom_accent(mut self, color: Color) -> Self {
        self.accent = TokenValue::Color(color);
        self.accent_hover = TokenValue::Color(color.lighten(0.1));
        self.accent_pressed = TokenValue::Color(color.darken(0.1));
        self
    }
}
```

**Usage Example**:

```rust
// System default theme (adopts OS appearance)
let tokens = DesignTokens::from_system(&event_loop);
// On Windows 11: surface_primary = Mica
// On macOS: surface_primary = Vibrancy::Content
// On Android: surface_primary = MaterialYou.surface

// Custom branded theme
let tokens = DesignTokens::from_system(&event_loop)
    .with_custom_accent(Color::rgb(0.2, 0.6, 1.0));
// Override accent but keep native materials
```

### Layer 3: CSS-like Custom Styling (arthropod)

**Responsibility**: Developer-facing styling API with CSS familiarity and Rust type safety.

**Location**: `crates/arthropod/src/style/mod.rs`

```rust
/// Compile-time checked style definitions
pub struct Style {
    pub background: Option<Background>,
    pub border: Option<Border>,
    pub border_radius: Option<f32>,
    pub padding: Option<Padding>,
    pub margin: Option<Margin>,
    pub box_shadow: Option<Shadow>,
    pub opacity: Option<f32>,
    // ... more properties
}

pub enum Background {
    Material(BackgroundMaterial),
    Color(Color),
    Gradient(Gradient),
}

pub struct Border {
    pub width: f32,
    pub color: Color,
    pub style: BorderStyle,
}

pub enum BorderStyle {
    Solid,
    Dashed,
    Dotted,
}

// CSS-like macro for ergonomic styling
macro_rules! style {
    ($($tt:tt)*) => {
        // Proc macro parses CSS-like syntax at compile time
    };
}
```

**Usage Examples**:

```rust
// Using design tokens
let card = style! {
    background: tokens.surface_elevated;
    border_radius: tokens.radius_lg;
    padding: tokens.space_md;
    box_shadow: tokens.shadow_md;

    &:hover {
        box_shadow: tokens.shadow_lg;
    }
};

// Custom values (pixel-perfect control)
let hero = style! {
    background: linear_gradient(
        180deg,
        rgba(0, 0, 0, 0.8),
        rgba(0, 0, 0, 0.0)
    );
    padding: (32.0, 64.0); // vertical, horizontal
    border_radius: 16.0;
};

// Media queries (platform/theme detection)
let button = style! {
    background: tokens.accent;
    padding: tokens.space_md;

    @media (prefers_color_scheme: dark) {
        border: (1.0, rgba(255, 255, 255, 0.1));
    }

    @platform(windows) {
        // Windows-specific tweaks
        border_radius: 4.0; // Sharper corners on Windows
    }
};

// State variants
let input = style! {
    background: tokens.surface_primary;
    border: (1.0, tokens.text_secondary);

    &:focus {
        border: (2.0, tokens.accent);
        box_shadow: (0.0, 0.0, 0.0, 3.0, tokens.accent.with_alpha(0.3));
    }

    &:disabled {
        opacity: 0.5;
        background: tokens.surface_primary;
    }
};
```

### Style Resolution Flow

```rust
// 1. Developer defines style using tokens
let card_style = style! {
    background: tokens.surface_elevated;
    border_radius: tokens.radius_lg;
};

// 2. Compile-time: Macro parses to AST
// 3. Runtime: Resolve tokens to concrete values
let resolved = card_style.resolve(&tokens);
// On Windows 11: background = Material(Mica)
// On macOS: background = Material(Vibrancy::Content)
// On Android: background = Color(MaterialYou.surface)

// 4. Pass to render-engine
backend.set_background_material(&window, resolved.background);
backend.set_border_radius(resolved.border_radius);
```

## Rationale

### Why Three Layers?

**Layer 1 (Platform Native)**:
- Each OS has different capabilities (Mica ≠ Vibrancy ≠ Material You)
- Direct OS API access needed for native feel
- Can't be abstracted without losing platform features

**Layer 2 (Design Tokens)**:
- Developers want "button background" not "Mica vs Vibrancy"
- Tokens resolve to appropriate platform values
- Enables write-once, looks-native-everywhere

**Layer 3 (Custom Styling)**:
- Designers need pixel-perfect control for branding
- CSS familiarity reduces learning curve
- Type safety prevents runtime errors

### Why plat-core for Layer 1?

**Pros**:
- Direct platform API access (Win32, Cocoa, etc.)
- Already handles platform abstraction
- Can export platform-agnostic types (SystemTheme)

**Cons**:
- Couples theming to windowing (acceptable - themes are window-scoped)

### Why Not CSS-in-Rust Libraries?

**Considered Alternatives**:

1. **stylist** (CSS-in-Rust)
   - **Pros**: Full CSS support, familiar syntax
   - **Cons**: Runtime parsing overhead, no platform material support
   - **Rejected**: Can't represent native materials (Mica, Vibrancy)

2. **iced::style** (Iced's styling)
   - **Pros**: Theme support, type-safe
   - **Cons**: Opinionated theme structure, no platform materials
   - **Rejected**: Too coupled to Iced's widget system

3. **Plain Rust structs** (no macro)
   - **Pros**: Zero magic, compile-time checking
   - **Cons**: Verbose, unfamiliar to web developers
   - **Rejected**: Ergonomics matter for adoption

**Our Approach**: Hybrid of CSS familiarity + Rust type safety + platform material support.

## Consequences

### Positive

- **Native Feel**: Apps automatically adopt OS appearance (Mica on Win11, Vibrancy on macOS)
- **Developer Ergonomics**: CSS-like syntax familiar to web developers
- **Type Safety**: Compile-time checks prevent invalid styles
- **Flexibility**: Can use system defaults or full custom control
- **Performance**: Static styles have zero runtime cost
- **Future-Proof**: Easy to add new platform materials (Windows 12, iOS 18, etc.)

### Negative

- **Complexity**: Three layers to understand
- **Macro Magic**: style! macro obscures implementation (mitigated by good docs)
- **Platform Testing**: Need to test appearance on each OS
- **Material Parity**: Some materials don't map 1:1 across platforms

### Mitigations

- **Documentation**: Clear guide showing layer flow with examples
- **Testing**: Visual regression tests on each platform
- **Fallbacks**: Graceful degradation (Mica → Acrylic → Solid Color)
- **Type System**: Compile errors guide developers to correct usage

## Implementation Plan

### Phase 1: Foundation (Current Sprint)
- ✅ ADR and design doc
- 🚧 Layer 1: SystemTheme struct and Windows implementation
- 🚧 Layer 2: DesignTokens struct (basic version)
- 🚧 Simple style! macro (background, padding, border-radius only)

### Phase 2: Core Features (Next Sprint)
- Full style! macro with pseudo-states (:hover, :focus)
- Media queries (@media, @platform)
- macOS SystemTheme implementation
- Shadow rendering in render-engine

### Phase 3: Polish (Future)
- Linux SystemTheme (GTK/Qt theme detection)
- Android Material You integration
- Animation support (transition: all 0.2s ease)
- Theme hot-reloading for development

## Performance Characteristics

- **Static Styles**: Zero runtime cost (resolved at compile/init time)
- **Dynamic Styles**: Token lookup overhead (~100ns per property)
- **Material Application**: OS API call overhead (< 1ms per window)
- **Memory**: ~200 bytes per DesignTokens instance

## Platform Support Matrix

| Platform | Layer 1 Materials | Layer 2 Tokens | Layer 3 Custom | Status |
|----------|-------------------|----------------|----------------|--------|
| Windows  | Mica, Acrylic     | ✅              | ✅              | 🚧 Phase 1 |
| macOS    | Vibrancy          | ✅              | ✅              | 📅 Phase 2 |
| Linux    | GTK/Qt themes     | ✅              | ✅              | 📅 Phase 2 |
| Android  | Material You      | ✅              | ✅              | 📅 Phase 3 |
| iOS      | UIBlurEffect      | ✅              | ✅              | 📅 Phase 3 |
| Web      | backdrop-filter   | ✅              | ✅              | 📅 Phase 4 |

## Testing Strategy

**Unit Tests**:
- Token resolution logic
- Style macro parsing
- Fallback chains

**Integration Tests**:
- SystemTheme queries on each platform
- Material application (visual regression)
- Dark mode switching

**Manual Testing**:
- Verify native appearance on each OS
- Test with system theme changes (dark/light mode)
- Validate fallbacks on older OS versions (Win10 Acrylic vs Win11 Mica)

## References

- **Windows Materials**:
  - [Windows 11 Mica](https://learn.microsoft.com/en-us/windows/apps/design/style/mica)
  - [Windows Acrylic](https://learn.microsoft.com/en-us/windows/apps/design/style/acrylic)
- **macOS Vibrancy**:
  - [NSVisualEffectView](https://developer.apple.com/documentation/appkit/nsvisualeffectview)
- **Material You**:
  - [Material Design 3](https://m3.material.io/)
- **Design Tokens**:
  - [Design Tokens Community Group](https://www.designtokens.org/)
- **ADR 0005**: Platform Abstraction Strategy (plat-core)
- **ADR 0004**: wgpu Rendering Backend

## Future Considerations

- **CSS Custom Properties**: Expose design tokens as `--surface-primary` for web target
- **Theme Editor**: Visual tool for customizing tokens (inspect mode)
- **Animation Tokens**: `transition_fast`, `transition_smooth` (duration + easing)
- **Responsive Tokens**: Token values that change based on window size
- **Theme Inheritance**: Child components inherit parent theme context
- **Color Contrast**: Automatic WCAG compliance checking for custom colors
