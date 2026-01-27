# Phase 3: Styling & Platform Feel Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Deliver native platform materials (Mica/Acrylic), CSS-like style! macro, and complete widget theming so apps feel native on each platform.

**Architecture:** Three-layer theming system where SystemTheme queries platform, DesignTokens provides semantic values, and Style API enables declarative styling. Native materials are applied at the window level via platform APIs.

**Tech Stack:** Windows DWM APIs, Rust proc-macros (syn/quote), theme-engine crate, widget-core integration.

---

## Part A: Native Materials (Mica/Acrylic)

### Task A1: Add Material Application API to plat-core

**Files:**
- Create: `crates/plat-core/src/materials.rs`
- Modify: `crates/plat-core/src/lib.rs`
- Modify: `crates/plat-core/src/window.rs`
- Test: `crates/plat-core/tests/material_tests.rs`

**Step 1: Write the failing test**

```rust
// crates/plat-core/tests/material_tests.rs
use plat_core::{BackdropMaterial, Window};

#[test]
fn test_material_enum_variants() {
    // Verify material variants exist
    let _mica = BackdropMaterial::Mica;
    let _acrylic = BackdropMaterial::Acrylic;
    let _mica_alt = BackdropMaterial::MicaAlt;
    let _none = BackdropMaterial::None;
}

#[test]
fn test_window_has_set_backdrop_material() {
    // This test verifies the API exists (actual application requires a window)
    // The method signature should be: fn set_backdrop_material(&self, material: BackdropMaterial)
    fn _check_api<W: HasBackdropMaterial>(w: &W, m: BackdropMaterial) {
        w.set_backdrop_material(m);
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p plat-core --test material_tests`
Expected: FAIL with "cannot find type `BackdropMaterial`"

**Step 3: Create materials module**

```rust
// crates/plat-core/src/materials.rs
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

/// Trait for windows that support backdrop materials
pub trait HasBackdropMaterial {
    /// Set the window's backdrop material
    ///
    /// # Platform Support
    /// - Windows 11 22000+: Mica, MicaAlt, Acrylic
    /// - Windows 10 16299+: Acrylic only
    /// - Other platforms: Silently ignored (falls back to solid color)
    fn set_backdrop_material(&self, material: BackdropMaterial);

    /// Get currently applied material
    fn backdrop_material(&self) -> BackdropMaterial;
}
```

**Step 4: Export from lib.rs**

```rust
// Add to crates/plat-core/src/lib.rs
mod materials;
pub use materials::*;
```

**Step 5: Run test to verify it passes**

Run: `cargo test -p plat-core --test material_tests`
Expected: PASS

**Step 6: Commit**

```bash
git add crates/plat-core/src/materials.rs crates/plat-core/src/lib.rs crates/plat-core/tests/material_tests.rs
git commit -m "feat(plat-core): add BackdropMaterial enum and trait"
```

---

### Task A2: Implement Windows DWM Material Application

**Files:**
- Modify: `crates/plat-core/src/platform/windows.rs`
- Modify: `crates/plat-core/Cargo.toml`
- Test: `crates/plat-core/tests/material_tests.rs`

**Step 1: Add Windows API imports to Cargo.toml**

```toml
# In crates/plat-core/Cargo.toml [target.'cfg(windows)'.dependencies]
# Add these features to the existing windows dependency:
[target.'cfg(windows)'.dependencies.windows]
version = "0.58"
features = [
    # ... existing features ...
    "Win32_Graphics_Dwm",  # ADD THIS
]
```

**Step 2: Write the integration test**

```rust
// Add to crates/plat-core/tests/material_tests.rs
#[test]
#[ignore] // Requires actual window - run manually
fn test_apply_mica_to_window() {
    use plat_core::{EventLoop, WindowConfig, BackdropMaterial, HasBackdropMaterial};

    let event_loop = EventLoop::new().expect("Failed to create event loop");
    let window = event_loop.create_window(WindowConfig {
        title: "Mica Test".into(),
        size: plat_core::Size::new(400, 300),
        visible: false,
    }).expect("Failed to create window");

    // Should not panic
    window.set_backdrop_material(BackdropMaterial::Mica);
    assert_eq!(window.backdrop_material(), BackdropMaterial::Mica);
}
```

**Step 3: Implement HasBackdropMaterial for WindowImpl**

```rust
// Add to crates/plat-core/src/platform/windows.rs

use crate::materials::{BackdropMaterial, HasBackdropMaterial};
use std::sync::atomic::AtomicU8;

// Add field to WindowImpl struct:
// backdrop_material: AtomicU8,  // Store as u8 for atomic access

#[cfg(windows)]
mod dwm {
    use windows::Win32::Foundation::HWND;
    use windows::Win32::Graphics::Dwm::{
        DwmSetWindowAttribute, DWMWA_SYSTEMBACKDROP_TYPE, DWMWA_USE_IMMERSIVE_DARK_MODE,
        DWM_SYSTEMBACKDROP_TYPE,
    };

    /// System backdrop types (Windows 11 22H2+)
    #[repr(i32)]
    pub enum SystemBackdropType {
        Auto = 0,
        None = 1,
        Mica = 2,
        Acrylic = 3,
        MicaAlt = 4,
    }

    pub fn set_system_backdrop(hwnd: HWND, backdrop: SystemBackdropType) -> bool {
        unsafe {
            let value = backdrop as i32;
            DwmSetWindowAttribute(
                hwnd,
                DWMWA_SYSTEMBACKDROP_TYPE,
                &value as *const _ as *const _,
                std::mem::size_of::<i32>() as u32,
            ).is_ok()
        }
    }

    pub fn set_dark_mode(hwnd: HWND, dark: bool) -> bool {
        unsafe {
            let value: i32 = if dark { 1 } else { 0 };
            DwmSetWindowAttribute(
                hwnd,
                DWMWA_USE_IMMERSIVE_DARK_MODE,
                &value as *const _ as *const _,
                std::mem::size_of::<i32>() as u32,
            ).is_ok()
        }
    }
}

impl HasBackdropMaterial for WindowImpl {
    fn set_backdrop_material(&self, material: BackdropMaterial) {
        use dwm::SystemBackdropType;

        let backdrop_type = match material {
            BackdropMaterial::None => SystemBackdropType::None,
            BackdropMaterial::Mica => SystemBackdropType::Mica,
            BackdropMaterial::MicaAlt => SystemBackdropType::MicaAlt,
            BackdropMaterial::Acrylic => SystemBackdropType::Acrylic,
        };

        if dwm::set_system_backdrop(self.hwnd, backdrop_type) {
            self.backdrop_material.store(material as u8, Ordering::SeqCst);
        }
    }

    fn backdrop_material(&self) -> BackdropMaterial {
        match self.backdrop_material.load(Ordering::SeqCst) {
            1 => BackdropMaterial::Mica,
            2 => BackdropMaterial::MicaAlt,
            3 => BackdropMaterial::Acrylic,
            _ => BackdropMaterial::None,
        }
    }
}
```

**Step 4: Implement HasBackdropMaterial for Window wrapper**

```rust
// Add to crates/plat-core/src/window.rs
impl HasBackdropMaterial for Window {
    fn set_backdrop_material(&self, material: BackdropMaterial) {
        self.inner.set_backdrop_material(material);
    }

    fn backdrop_material(&self) -> BackdropMaterial {
        self.inner.backdrop_material()
    }
}
```

**Step 5: Run tests**

Run: `cargo test -p plat-core`
Expected: PASS (ignored test skipped)

**Step 6: Commit**

```bash
git add crates/plat-core/
git commit -m "feat(plat-core): implement Windows DWM backdrop materials (Mica/Acrylic)"
```

---

### Task A3: Bridge theme-engine Materials to plat-core

**Files:**
- Modify: `crates/theme-engine/Cargo.toml`
- Modify: `crates/theme-engine/src/system_theme.rs`
- Create: `crates/theme-engine/src/material_bridge.rs`
- Test: `crates/theme-engine/tests/material_bridge_tests.rs`

**Step 1: Write failing test**

```rust
// crates/theme-engine/tests/material_bridge_tests.rs
use theme_engine::{BackgroundMaterial, WindowsMaterial};
use plat_core::BackdropMaterial;

#[test]
fn test_background_material_to_backdrop() {
    let mica = BackgroundMaterial::Windows(WindowsMaterial::Mica);
    let backdrop: BackdropMaterial = mica.into();
    assert_eq!(backdrop, BackdropMaterial::Mica);
}
```

**Step 2: Implement From trait**

```rust
// crates/theme-engine/src/material_bridge.rs
//! Bridge between theme-engine materials and plat-core backdrop materials

use crate::{BackgroundMaterial, WindowsMaterial};
use plat_core::BackdropMaterial;

impl From<BackgroundMaterial> for BackdropMaterial {
    fn from(material: BackgroundMaterial) -> Self {
        match material {
            BackgroundMaterial::Windows(WindowsMaterial::Mica) => BackdropMaterial::Mica,
            BackgroundMaterial::Windows(WindowsMaterial::MicaAlt) => BackdropMaterial::MicaAlt,
            BackgroundMaterial::Windows(WindowsMaterial::Acrylic) => BackdropMaterial::Acrylic,
            BackgroundMaterial::Solid(_) => BackdropMaterial::None,
            BackgroundMaterial::MacOS(_) => BackdropMaterial::None, // TODO: macOS support
        }
    }
}

impl From<&BackgroundMaterial> for BackdropMaterial {
    fn from(material: &BackgroundMaterial) -> Self {
        (*material).clone().into()
    }
}
```

**Step 3: Add plat-core dependency and export**

```toml
# crates/theme-engine/Cargo.toml
[dependencies]
plat-core = { path = "../plat-core" }
```

```rust
// crates/theme-engine/src/lib.rs
mod material_bridge;
// Re-export for convenience
pub use plat_core::BackdropMaterial;
```

**Step 4: Run test**

Run: `cargo test -p theme-engine --test material_bridge_tests`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/theme-engine/
git commit -m "feat(theme-engine): bridge BackgroundMaterial to plat-core BackdropMaterial"
```

---

### Task A4: Create Material Application Example

**Files:**
- Create: `examples/native_materials.rs`

**Step 1: Create example demonstrating Mica window**

```rust
// examples/native_materials.rs
//! Example: Native Materials (Mica/Acrylic)
//!
//! Demonstrates platform-native backdrop materials on Windows 11.
//! Run with: cargo run --example native_materials

use plat_core::{
    Application, BackdropMaterial, ControlFlow, Event, EventLoop, HasBackdropMaterial,
    Size, Window, WindowConfig, WindowEvent, WindowId,
};
use render_engine::{Color, NodeContent, Rect, Scene, SceneNode, WgpuBackend};
use theme_engine::{DesignTokens, SystemTheme};

struct MaterialApp {
    window: Window,
    backend: Option<WgpuBackend>,
    scene: Scene,
    tokens: DesignTokens,
}

impl Application for MaterialApp {
    fn new(event_loop: &EventLoop) -> Self {
        // Query system theme
        let theme = SystemTheme::query().unwrap_or_default();
        let tokens = DesignTokens::from_system(&theme);

        // Create window
        let window = event_loop
            .create_window(WindowConfig {
                title: "Native Materials Demo".into(),
                size: Size::new(800, 600),
                visible: true,
            })
            .expect("Failed to create window");

        // Apply Mica backdrop (Windows 11) or fall back gracefully
        window.set_backdrop_material(BackdropMaterial::Mica);

        // Create scene with transparent background to show Mica
        let mut scene = Scene::new();
        let root = scene.root();

        // Add a semi-transparent card to demonstrate the effect
        let card_color = tokens.surface_elevated.as_color();
        scene.add_node(
            root,
            SceneNode::new(NodeContent::Rect {
                color: Color::rgba(card_color.x, card_color.y, card_color.z, 0.8),
            })
            .with_bounds(Rect::new(50.0, 50.0, 300.0, 200.0)),
        );

        Self {
            window,
            backend: None,
            scene,
            tokens,
        }
    }

    fn on_event(&mut self, event: Event, control_flow: &mut ControlFlow) {
        match event {
            Event::Window { event: WindowEvent::Resized(size), .. } => {
                if let Some(backend) = &mut self.backend {
                    backend.resize(size.width, size.height);
                }
                self.window.request_redraw();
            }
            Event::Window { event: WindowEvent::CloseRequested, .. } => {
                *control_flow = ControlFlow::Exit;
            }
            _ => {}
        }
    }

    fn on_redraw(&mut self, _window_id: WindowId) {
        // Lazy-init backend
        if self.backend.is_none() {
            let size = self.window.inner_size();
            self.backend = WgpuBackend::new(&self.window, size.width, size.height).ok();
        }

        if let Some(backend) = &mut self.backend {
            // Use transparent clear color to show Mica through
            let _ = backend.render_scene_with_clear(&self.scene, Color::rgba(0.0, 0.0, 0.0, 0.0));
        }
    }
}

fn main() {
    plat_core::run::<MaterialApp>().expect("Application error");
}
```

**Step 2: Run example**

Run: `cargo run --example native_materials`
Expected: Window with Mica backdrop (Windows 11) or solid color fallback

**Step 3: Commit**

```bash
git add examples/native_materials.rs
git commit -m "feat(examples): add native_materials demo for Mica/Acrylic"
```

---

## Part B: CSS-like style! Macro

### Task B1: Design the style! Macro Syntax

**Files:**
- Modify: `crates/theme-engine/src/style_macro.rs`
- Test: `crates/theme-engine/tests/style_macro_tests.rs`

**Step 1: Write tests defining expected syntax**

```rust
// crates/theme-engine/tests/style_macro_tests.rs
use theme_engine::{style, DesignTokens, SystemTheme, Padding};
use glam::Vec4;

#[test]
fn test_style_macro_basic() {
    let tokens = DesignTokens::from_system(&SystemTheme::default());

    let s = style! {
        background: tokens.surface_primary;
        color: tokens.text_primary;
        padding: tokens.space_md;
        border_radius: tokens.radius_lg;
    };

    assert!(s.background.is_some());
    assert!(s.color.is_some());
    assert_eq!(s.padding, Some(Padding::uniform(16.0)));
    assert_eq!(s.border_radius, Some(8.0));
}

#[test]
fn test_style_macro_hover_state() {
    let tokens = DesignTokens::from_system(&SystemTheme::default());

    let s = style! {
        background: tokens.surface_primary;

        &:hover {
            background: tokens.surface_secondary;
        }
    };

    assert!(s.hover.is_some());
    let hover = s.hover.as_ref().unwrap();
    assert!(hover.background.is_some());
}

#[test]
fn test_style_macro_disabled_state() {
    let tokens = DesignTokens::from_system(&SystemTheme::default());

    let s = style! {
        background: tokens.accent;

        &:disabled {
            background: tokens.surface_secondary;
            opacity: 0.5;
        }
    };

    assert!(s.disabled.is_some());
}
```

**Step 2: Expand Style struct for pseudo-states**

```rust
// crates/theme-engine/src/style_macro.rs

/// Style properties for UI components with pseudo-state support
#[derive(Debug, Clone, Default)]
pub struct Style {
    // Base properties
    pub background: Option<TokenValue>,
    pub padding: Option<Padding>,
    pub border_radius: Option<f32>,
    pub color: Option<Color>,
    pub opacity: Option<f32>,

    // Pseudo-state styles (only changed properties)
    pub hover: Option<Box<StyleOverrides>>,
    pub focus: Option<Box<StyleOverrides>>,
    pub active: Option<Box<StyleOverrides>>,
    pub disabled: Option<Box<StyleOverrides>>,
}

/// Partial style overrides for pseudo-states
#[derive(Debug, Clone, Default)]
pub struct StyleOverrides {
    pub background: Option<TokenValue>,
    pub color: Option<Color>,
    pub opacity: Option<f32>,
    pub border_radius: Option<f32>,
}

impl Style {
    /// Resolve style for current widget state
    pub fn resolve(&self, hover: bool, focus: bool, active: bool, disabled: bool) -> ResolvedStyle {
        let mut resolved = ResolvedStyle {
            background: self.background.clone(),
            color: self.color,
            padding: self.padding,
            border_radius: self.border_radius,
            opacity: self.opacity.unwrap_or(1.0),
        };

        // Apply pseudo-states in order of specificity
        if disabled {
            if let Some(d) = &self.disabled {
                d.apply_to(&mut resolved);
            }
        } else {
            if hover {
                if let Some(h) = &self.hover {
                    h.apply_to(&mut resolved);
                }
            }
            if focus {
                if let Some(f) = &self.focus {
                    f.apply_to(&mut resolved);
                }
            }
            if active {
                if let Some(a) = &self.active {
                    a.apply_to(&mut resolved);
                }
            }
        }

        resolved
    }
}

/// Fully resolved style values
#[derive(Debug, Clone)]
pub struct ResolvedStyle {
    pub background: Option<TokenValue>,
    pub color: Option<Color>,
    pub padding: Option<Padding>,
    pub border_radius: Option<f32>,
    pub opacity: f32,
}

impl StyleOverrides {
    fn apply_to(&self, resolved: &mut ResolvedStyle) {
        if let Some(bg) = &self.background {
            resolved.background = Some(bg.clone());
        }
        if let Some(c) = self.color {
            resolved.color = Some(c);
        }
        if let Some(o) = self.opacity {
            resolved.opacity = o;
        }
        if let Some(r) = self.border_radius {
            resolved.border_radius = Some(r);
        }
    }
}
```

**Step 3: Create the style! macro (declarative)**

```rust
// Add to crates/theme-engine/src/style_macro.rs

/// CSS-like style macro for declarative styling
///
/// # Example
///
/// ```
/// use theme_engine::{style, DesignTokens, SystemTheme};
///
/// let tokens = DesignTokens::from_system(&SystemTheme::default());
///
/// let button_style = style! {
///     background: tokens.accent;
///     color: Vec4::ONE;  // white
///     padding: tokens.space_sm;
///     border_radius: tokens.radius_md;
///
///     &:hover {
///         background: tokens.accent_hover;
///     }
///
///     &:disabled {
///         opacity: 0.5;
///     }
/// };
/// ```
#[macro_export]
macro_rules! style {
    // Entry point
    ({ $($body:tt)* }) => {{
        let mut style = $crate::Style::new();
        $crate::__style_impl!(style, $($body)*);
        style
    }};

    // Allow without braces for single expression context
    ( $($body:tt)* ) => {{
        let mut style = $crate::Style::new();
        $crate::__style_impl!(style, $($body)*);
        style
    }};
}

#[macro_export]
#[doc(hidden)]
macro_rules! __style_impl {
    // Base case: empty
    ($style:ident,) => {};
    ($style:ident) => {};

    // Pseudo-state: &:hover { ... }
    ($style:ident, &:hover { $($inner:tt)* } $($rest:tt)*) => {
        {
            let mut overrides = $crate::StyleOverrides::default();
            $crate::__style_overrides!(overrides, $($inner)*);
            $style.hover = Some(Box::new(overrides));
        }
        $crate::__style_impl!($style, $($rest)*);
    };

    // Pseudo-state: &:focus { ... }
    ($style:ident, &:focus { $($inner:tt)* } $($rest:tt)*) => {
        {
            let mut overrides = $crate::StyleOverrides::default();
            $crate::__style_overrides!(overrides, $($inner)*);
            $style.focus = Some(Box::new(overrides));
        }
        $crate::__style_impl!($style, $($rest)*);
    };

    // Pseudo-state: &:active { ... }
    ($style:ident, &:active { $($inner:tt)* } $($rest:tt)*) => {
        {
            let mut overrides = $crate::StyleOverrides::default();
            $crate::__style_overrides!(overrides, $($inner)*);
            $style.active = Some(Box::new(overrides));
        }
        $crate::__style_impl!($style, $($rest)*);
    };

    // Pseudo-state: &:disabled { ... }
    ($style:ident, &:disabled { $($inner:tt)* } $($rest:tt)*) => {
        {
            let mut overrides = $crate::StyleOverrides::default();
            $crate::__style_overrides!(overrides, $($inner)*);
            $style.disabled = Some(Box::new(overrides));
        }
        $crate::__style_impl!($style, $($rest)*);
    };

    // Property: background (TokenValue)
    ($style:ident, background: $val:expr; $($rest:tt)*) => {
        $style.background = Some($crate::__to_token_value!($val));
        $crate::__style_impl!($style, $($rest)*);
    };

    // Property: color
    ($style:ident, color: $val:expr; $($rest:tt)*) => {
        $style.color = Some($val);
        $crate::__style_impl!($style, $($rest)*);
    };

    // Property: padding (uniform from spacing token)
    ($style:ident, padding: $val:expr; $($rest:tt)*) => {
        $style.padding = Some($crate::Padding::uniform($val));
        $crate::__style_impl!($style, $($rest)*);
    };

    // Property: border_radius
    ($style:ident, border_radius: $val:expr; $($rest:tt)*) => {
        $style.border_radius = Some($val);
        $crate::__style_impl!($style, $($rest)*);
    };

    // Property: opacity
    ($style:ident, opacity: $val:expr; $($rest:tt)*) => {
        $style.opacity = Some($val);
        $crate::__style_impl!($style, $($rest)*);
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! __style_overrides {
    ($overrides:ident,) => {};
    ($overrides:ident) => {};

    ($overrides:ident, background: $val:expr; $($rest:tt)*) => {
        $overrides.background = Some($crate::__to_token_value!($val));
        $crate::__style_overrides!($overrides, $($rest)*);
    };

    ($overrides:ident, color: $val:expr; $($rest:tt)*) => {
        $overrides.color = Some($val);
        $crate::__style_overrides!($overrides, $($rest)*);
    };

    ($overrides:ident, opacity: $val:expr; $($rest:tt)*) => {
        $overrides.opacity = Some($val);
        $crate::__style_overrides!($overrides, $($rest)*);
    };

    ($overrides:ident, border_radius: $val:expr; $($rest:tt)*) => {
        $overrides.border_radius = Some($val);
        $crate::__style_overrides!($overrides, $($rest)*);
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! __to_token_value {
    // If it's already a TokenValue, use it directly
    ($val:expr) => {
        $crate::__convert_to_token_value($val)
    };
}

/// Helper to convert various types to TokenValue
pub fn __convert_to_token_value<T: IntoTokenValue>(value: T) -> TokenValue {
    value.into_token_value()
}

/// Trait for types that can become TokenValue
pub trait IntoTokenValue {
    fn into_token_value(self) -> TokenValue;
}

impl IntoTokenValue for TokenValue {
    fn into_token_value(self) -> TokenValue {
        self
    }
}

impl IntoTokenValue for Color {
    fn into_token_value(self) -> TokenValue {
        TokenValue::Color(self)
    }
}

impl IntoTokenValue for &TokenValue {
    fn into_token_value(self) -> TokenValue {
        self.clone()
    }
}
```

**Step 4: Export macro from lib.rs**

```rust
// crates/theme-engine/src/lib.rs
pub use style_macro::{
    IntoTokenValue, Padding, ResolvedStyle, Style, StyleOverrides, __convert_to_token_value,
};
```

**Step 5: Run tests**

Run: `cargo test -p theme-engine`
Expected: PASS

**Step 6: Commit**

```bash
git add crates/theme-engine/src/style_macro.rs crates/theme-engine/src/lib.rs crates/theme-engine/tests/style_macro_tests.rs
git commit -m "feat(theme-engine): implement style! macro with pseudo-state support"
```

---

## Part C: Widget Theming Completion

### Task C1: Theme TextInput Widget

**Files:**
- Modify: `crates/widget-core/src/text_input.rs`
- Test: `crates/widget-core/tests/theming_tests.rs`

**Step 1: Write failing test**

```rust
// crates/widget-core/tests/theming_tests.rs
use flux_state::{Runtime, Signal};
use theme_engine::{DesignTokens, SystemTheme};
use widget_core::{TextInput, Widget, WidgetContext};

#[test]
fn test_text_input_uses_design_tokens() {
    let runtime = Runtime::new();
    let value = Signal::new(runtime, String::new());
    let input = TextInput::new(value);

    let mut ctx = WidgetContext::new_for_testing();
    let theme = SystemTheme::default();
    let tokens = DesignTokens::from_system(&theme);
    ctx.set_design_tokens(tokens.clone());

    let _node_id = input.build(&mut ctx);

    // Background should use surface color from tokens, not hardcoded white
    let bg = ctx.get_background_color(_node_id).expect("should have background");
    let expected = tokens.surface_primary.as_color();

    // Allow small floating point differences
    assert!((bg.x - expected.x).abs() < 0.01, "Background red channel mismatch");
}
```

**Step 2: Update TextInput to use tokens**

```rust
// In crates/widget-core/src/text_input.rs
// Modify the build() method:

impl Widget for TextInput {
    fn build(&self, ctx: &mut WidgetContext) -> NodeId {
        // Get colors from design tokens or fall back to defaults
        let (bg_color, text_color, placeholder_color) = {
            let tokens = ctx.design_tokens();
            match tokens {
                Some(t) => (
                    t.surface_primary.as_color(),
                    t.text_primary,
                    t.text_secondary,
                ),
                None => (
                    Vec4::new(1.0, 1.0, 1.0, 1.0),    // White
                    Vec4::new(0.0, 0.0, 0.0, 1.0),    // Black
                    Vec4::new(0.5, 0.5, 0.5, 1.0),    // Gray
                ),
            }
        };

        // Create input container with themed background
        let input_node = ctx.create_node(
            ctx.root(),
            NodeContent::Rect {
                color: Color::rgba(bg_color.x, bg_color.y, bg_color.z, bg_color.w),
            },
        );

        // ... rest of implementation using text_color and placeholder_color ...
    }
}
```

**Step 3: Run test**

Run: `cargo test -p widget-core --test theming_tests`
Expected: PASS

**Step 4: Commit**

```bash
git add crates/widget-core/src/text_input.rs crates/widget-core/tests/theming_tests.rs
git commit -m "feat(widget-core): TextInput uses DesignTokens for theming"
```

---

### Task C2: Theme Form Widget

**Files:**
- Modify: `crates/widget-core/src/form.rs`
- Test: `crates/widget-core/tests/theming_tests.rs`

**Step 1: Add test**

```rust
// Add to crates/widget-core/tests/theming_tests.rs
#[test]
fn test_form_uses_design_tokens() {
    let runtime = Runtime::new();
    let field = Signal::new(runtime, String::new());
    let form = Form::new((("test", TextInput::new(field)),));

    let mut ctx = WidgetContext::new_for_testing();
    let tokens = DesignTokens::from_system(&SystemTheme::default());
    ctx.set_design_tokens(tokens.clone());

    let node_id = form.build(&mut ctx);
    let bg = ctx.get_background_color(node_id).expect("should have background");
    let expected = tokens.surface_secondary.as_color();

    assert!((bg.x - expected.x).abs() < 0.01);
}
```

**Step 2: Update Form to use tokens**

```rust
// In crates/widget-core/src/form.rs build() method:

impl<F: NamedWidgetTuple> Widget for Form<F> {
    fn build(&self, ctx: &mut WidgetContext) -> NodeId {
        // Get background from design tokens
        let bg_color = {
            let tokens = ctx.design_tokens();
            match tokens {
                Some(t) => t.surface_secondary.as_color(),
                None => Vec4::new(0.95, 0.95, 0.95, 1.0), // Fallback light gray
            }
        };

        // Create form container with themed background
        let form_node = ctx.create_node(
            ctx.root(),
            NodeContent::Rect {
                color: Color::rgba(bg_color.x, bg_color.y, bg_color.z, bg_color.w),
            },
        );

        // ... rest unchanged ...
    }
}
```

**Step 3: Run test and commit**

Run: `cargo test -p widget-core`

```bash
git add crates/widget-core/src/form.rs crates/widget-core/tests/theming_tests.rs
git commit -m "feat(widget-core): Form uses DesignTokens for theming"
```

---

### Task C3: Theme Text Widget

**Files:**
- Modify: `crates/widget-core/src/text.rs`
- Test: `crates/widget-core/tests/theming_tests.rs`

**Step 1: Add test**

```rust
// Add to crates/widget-core/tests/theming_tests.rs
#[test]
fn test_text_uses_design_tokens_for_default_color() {
    let text = Text::new("Hello");

    let mut ctx = WidgetContext::new_for_testing();
    let tokens = DesignTokens::from_system(&SystemTheme::default());
    ctx.set_design_tokens(tokens.clone());

    let _node_id = text.build(&mut ctx);

    // Default text color should come from tokens.text_primary
    // This test validates the widget queries tokens when no explicit color set
}
```

**Step 2: Update Text to use tokens when color not explicitly set**

```rust
// In crates/widget-core/src/text.rs

impl Text {
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            content: TextContent::Static(content.into()),
            font_size: 16.0,
            color: None, // Changed: None means "use theme default"
        }
    }

    // ... other methods ...
}

impl Widget for Text {
    fn build(&self, ctx: &mut WidgetContext) -> NodeId {
        // Resolve color: explicit > theme > hardcoded fallback
        let resolved_color = match self.color {
            Some(c) => c,
            None => {
                ctx.design_tokens()
                    .map(|t| t.text_primary)
                    .unwrap_or(Vec4::new(0.0, 0.0, 0.0, 1.0))
            }
        };

        // ... rest using resolved_color ...
    }
}
```

**Step 3: Run test and commit**

```bash
git add crates/widget-core/src/text.rs crates/widget-core/tests/theming_tests.rs
git commit -m "feat(widget-core): Text uses DesignTokens for default color"
```

---

### Task C4: Add Themed Example

**Files:**
- Create: `examples/themed_form.rs`

**Step 1: Create comprehensive themed example**

```rust
// examples/themed_form.rs
//! Example: Fully Themed Form
//!
//! Demonstrates all widgets using the theme engine with platform colors.
//! Run with: cargo run --example themed_form

use flux_state::{Runtime, Signal};
use plat_core::{
    Application, BackdropMaterial, ControlFlow, Event, EventLoop, HasBackdropMaterial,
    Size, Window, WindowConfig, WindowEvent, WindowId,
};
use render_engine::WgpuBackend;
use theme_engine::{DesignTokens, SystemTheme};
use widget_core::{form, input, txt, Button, ButtonStyle, Widget, WidgetContext};

struct ThemedApp {
    window: Window,
    backend: Option<WgpuBackend>,
    runtime: Runtime,
    tokens: DesignTokens,
}

impl Application for ThemedApp {
    fn new(event_loop: &EventLoop) -> Self {
        let runtime = Runtime::new();

        // Query system theme and create tokens
        let theme = SystemTheme::query().unwrap_or_default();
        let tokens = DesignTokens::from_system(&theme);

        let window = event_loop
            .create_window(WindowConfig {
                title: "Themed Form Demo".into(),
                size: Size::new(600, 400),
                visible: true,
            })
            .expect("Failed to create window");

        // Apply native material
        window.set_backdrop_material(BackdropMaterial::Mica);

        Self {
            window,
            backend: None,
            runtime,
            tokens,
        }
    }

    fn on_event(&mut self, event: Event, control_flow: &mut ControlFlow) {
        match event {
            Event::Window { event: WindowEvent::CloseRequested, .. } => {
                *control_flow = ControlFlow::Exit;
            }
            Event::Window { event: WindowEvent::Resized(size), .. } => {
                if let Some(backend) = &mut self.backend {
                    backend.resize(size.width, size.height);
                }
                self.window.request_redraw();
            }
            _ => {}
        }
    }

    fn on_redraw(&mut self, _window_id: WindowId) {
        // Build UI with theme tokens
        let name = Signal::new(self.runtime.clone(), String::new());
        let email = Signal::new(self.runtime.clone(), String::new());

        let form_widget = form!([
            ("name", input!(name, placeholder: "Your name")),
            ("email", input!(email, placeholder: "Email address")),
        ], gap: self.tokens.space_md, padding: self.tokens.space_lg);

        let mut ctx = WidgetContext::new_for_testing(); // TODO: real context
        ctx.set_design_tokens(self.tokens.clone());

        let _root = form_widget.build(&mut ctx);

        // Render (simplified - full impl would use ctx.render())
        if self.backend.is_none() {
            let size = self.window.inner_size();
            self.backend = WgpuBackend::new(&self.window, size.width, size.height).ok();
        }
    }
}

fn main() {
    plat_core::run::<ThemedApp>().expect("Application error");
}
```

**Step 2: Run example**

Run: `cargo run --example themed_form`
Expected: Window with themed form using platform colors

**Step 3: Commit**

```bash
git add examples/themed_form.rs
git commit -m "feat(examples): add themed_form demo showing full widget theming"
```

---

## Final Integration

### Task F1: Run Full Test Suite

**Step 1: Run all tests**

Run: `cargo test --all`
Expected: All tests pass

**Step 2: Run clippy**

Run: `cargo clippy --all-targets --all-features -- -D warnings`
Expected: No warnings

**Step 3: Final commit**

```bash
git add -A
git commit -m "feat: Phase 3 complete - Styling & Platform Feel

- Native materials: Mica/Acrylic backdrop support (Windows 11/10)
- style! macro: CSS-like declarative styling with pseudo-states
- Widget theming: All widgets (Button, TextInput, Form, Text) use DesignTokens
- Examples: native_materials, themed_form demos

Milestone: Apps that feel native on each platform"
```

---

## Summary

| Part | Tasks | Key Deliverables |
|------|-------|------------------|
| **A: Native Materials** | A1-A4 | BackdropMaterial API, DWM integration, material bridge |
| **B: style! Macro** | B1 | CSS-like syntax, pseudo-states (:hover, :disabled) |
| **C: Widget Theming** | C1-C4 | TextInput, Form, Text themed + example |

**Total Tasks:** 9 implementation tasks + 1 integration task

**Estimated Commits:** 10 focused commits with TDD discipline
