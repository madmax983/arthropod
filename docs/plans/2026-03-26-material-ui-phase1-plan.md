# Material UI Phase 1: Foundation Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build the layer system, MD3 token system, and backdrop widget — the foundation everything else depends on.

**Architecture:** Layer system goes into `widget-core` (scene-tree z-ordering via named layer nodes). MD3 theme system lives in a new `material-ui` crate. WidgetContext gets a generic extension mechanism so material widgets can access MaterialTheme without circular dependencies.

**Tech Stack:** Rust 2024, glam 0.29, thiserror 2.0, widget-core, theme-engine, style-engine, flux-state

**Worktree:** `.worktrees/material-ui` on branch `feature/material-ui`

---

## Task 1: Add Layer enum and LayerManager to widget-core

**Files:**
- Create: `crates/widget-core/src/layer.rs`
- Modify: `crates/widget-core/src/lib.rs` (add module + export)

**Step 1: Create `crates/widget-core/src/layer.rs` with Layer enum and LayerManager**

```rust
use render_engine::scene::{NodeId, Scene};
use render_engine::node::{NodeContent, SceneNode};
use std::collections::HashMap;

/// Named layers with guaranteed z-ordering in the scene tree.
/// Inspired by Flash/Flex's SystemManager display list architecture.
///
/// Layers are created as direct children of the scene root, in order.
/// The renderer traverses children in order, so layers naturally z-stack.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Layer {
    /// z=0 — Normal widget tree (default)
    Content,
    /// z=1 — Select, Menu, Autocomplete popups
    Dropdown,
    /// z=2 — Modal, Dialog, Drawer overlay
    Dialog,
    /// z=3 — Snackbar, Toast
    Notification,
    /// z=4 — Tooltips (always on top of UI content)
    Tooltip,
    /// z=5 — Drag previews, custom cursors
    Cursor,
}

impl Layer {
    /// All layers in z-order (lowest to highest).
    pub const ALL: [Layer; 6] = [
        Layer::Content,
        Layer::Dropdown,
        Layer::Dialog,
        Layer::Notification,
        Layer::Tooltip,
        Layer::Cursor,
    ];
}

/// Manages the mapping from Layer enum to scene NodeIds.
/// Created once during WidgetContext initialization.
pub struct LayerManager {
    layers: HashMap<Layer, NodeId>,
}

impl LayerManager {
    /// Initialize all layer nodes as children of the scene root.
    /// Must be called once during WidgetContext construction.
    pub fn new(scene: &mut Scene) -> Self {
        let root = scene.root();
        let mut layers = HashMap::new();

        for layer in Layer::ALL {
            let node = SceneNode::new(NodeContent::Empty);
            let node_id = scene.add_node(root, node);
            layers.insert(layer, node_id);
        }

        Self { layers }
    }

    /// Get the root NodeId for a given layer.
    /// Panics if the layer doesn't exist (should never happen after init).
    pub fn get(&self, layer: Layer) -> NodeId {
        self.layers[&layer]
    }
}
```

**Step 2: Add module and exports to `crates/widget-core/src/lib.rs`**

Add `mod layer;` with the other module declarations, and add to the public exports:
```rust
pub use layer::{Layer, LayerManager};
```

**Step 3: Run `cargo check -p widget-core`**

Expected: Compiles with no errors.

**Step 4: Commit**

```bash
git add crates/widget-core/src/layer.rs crates/widget-core/src/lib.rs
git commit -m "feat(widget-core): add Layer enum and LayerManager

Introduces a named layer system for z-ordered overlays in the scene
tree. Six layers (Content, Dropdown, Dialog, Notification, Tooltip,
Cursor) are created as ordered children of the scene root.

Inspired by Flash/Flex's SystemManager display list architecture."
```

---

## Task 2: Integrate LayerManager into WidgetContext

**Files:**
- Modify: `crates/widget-core/src/context/mod.rs`
- Modify: `crates/widget-core/src/lib.rs` (export new methods)

**Step 1: Add LayerManager field to WidgetContext**

In `crates/widget-core/src/context/mod.rs`, add to the struct fields (after `effects`):

```rust
/// Layer manager for z-ordered overlay system
layer_manager: LayerManager,
```

Add import at the top:
```rust
use crate::layer::{Layer, LayerManager};
```

**Step 2: Initialize LayerManager in `WidgetContext::new()`**

In the `new()` method, after `let mut scene = Scene::new();`, add:
```rust
let layer_manager = LayerManager::new(&mut scene);
```

And add `layer_manager` to the struct initialization.

**Step 3: Add layer API methods to WidgetContext**

Add these methods to the `impl WidgetContext` block:

```rust
/// Create a node in a specific layer (returns NodeId).
pub fn add_to_layer(&mut self, layer: Layer, content: NodeContent) -> NodeId {
    let parent = self.layer_manager.get(layer);
    self.scene.add_node(parent, SceneNode::new(content))
}

/// Reparent an existing node to a layer's root.
pub fn move_to_layer(&mut self, node_id: NodeId, layer: Layer) {
    let layer_root = self.layer_manager.get(layer);
    if let Some(current_parent) = self.scene.parent(node_id) {
        self.scene.reparent_node(node_id, current_parent, layer_root);
    }
}

/// Return a node to the content layer.
pub fn move_to_content(&mut self, node_id: NodeId) {
    self.move_to_layer(node_id, Layer::Content);
}

/// Get the root NodeId of a layer (for building children under it).
pub fn layer_root(&self, layer: Layer) -> NodeId {
    self.layer_manager.get(layer)
}

/// Get the content layer root (convenience — most widgets use this).
pub fn content_root(&self) -> NodeId {
    self.layer_manager.get(Layer::Content)
}
```

**Step 4: Update `root()` to return content layer root**

The existing `root()` method returns `self.scene.root()`. Widgets currently build under the scene root. After layers, the scene root holds layer nodes — widgets should build under the Content layer instead.

Change the existing `root()` method:
```rust
/// Returns the content layer root — where normal widgets are built.
/// For the actual scene root, use `scene_root()`.
pub fn root(&self) -> NodeId {
    self.layer_manager.get(Layer::Content)
}

/// Returns the actual scene root (parent of all layers).
pub fn scene_root(&self) -> NodeId {
    self.scene.root()
}
```

**Step 5: Run `cargo check -p widget-core`**

Expected: Compiles. Some existing tests/examples may need adjustment if they use `root()` for node counting.

**Step 6: Commit**

```bash
git add crates/widget-core/src/context/mod.rs crates/widget-core/src/lib.rs
git commit -m "feat(widget-core): integrate LayerManager into WidgetContext

WidgetContext now owns a LayerManager initialized during construction.
New API: add_to_layer(), move_to_layer(), move_to_content(), layer_root(),
content_root(). root() now returns the Content layer root instead of the
scene root, so widgets naturally build into the correct layer."
```

---

## Task 3: Layer system tests

**Files:**
- Create: `crates/widget-core/tests/layer_tests.rs`

**Step 1: Write tests for Layer enum**

```rust
use widget_core::{Layer, LayerManager, WidgetContext};
use render_engine::node::NodeContent;

#[test]
fn test_layer_ordering() {
    // Verify Layer::ALL is in z-order
    let layers = Layer::ALL;
    assert_eq!(layers[0], Layer::Content);
    assert_eq!(layers[1], Layer::Dropdown);
    assert_eq!(layers[2], Layer::Dialog);
    assert_eq!(layers[3], Layer::Notification);
    assert_eq!(layers[4], Layer::Tooltip);
    assert_eq!(layers[5], Layer::Cursor);
}

#[test]
fn test_layer_manager_creates_layer_nodes() {
    let mut ctx = WidgetContext::new();

    // Each layer should have a distinct NodeId
    let content = ctx.layer_root(Layer::Content);
    let dropdown = ctx.layer_root(Layer::Dropdown);
    let dialog = ctx.layer_root(Layer::Dialog);
    let notification = ctx.layer_root(Layer::Notification);
    let tooltip = ctx.layer_root(Layer::Tooltip);
    let cursor = ctx.layer_root(Layer::Cursor);

    // All should be different
    let ids = [content, dropdown, dialog, notification, tooltip, cursor];
    for i in 0..ids.len() {
        for j in (i + 1)..ids.len() {
            assert_ne!(ids[i], ids[j], "Layer {:?} and {:?} share NodeId", Layer::ALL[i], Layer::ALL[j]);
        }
    }
}

#[test]
fn test_root_returns_content_layer() {
    let ctx = WidgetContext::new();
    assert_eq!(ctx.root(), ctx.layer_root(Layer::Content));
    assert_ne!(ctx.root(), ctx.scene_root());
}

#[test]
fn test_add_to_layer() {
    let mut ctx = WidgetContext::new();

    // Add a node to the Dialog layer
    let node_id = ctx.add_to_layer(Layer::Dialog, NodeContent::Empty);

    // The node should exist and its parent should be the Dialog layer root
    let dialog_root = ctx.layer_root(Layer::Dialog);
    let scene = ctx.scene();
    let dialog_node = scene.get_node(dialog_root).unwrap();
    assert!(dialog_node.children.contains(&node_id));
}

#[test]
fn test_move_to_layer() {
    let mut ctx = WidgetContext::new();

    // Create a node in content layer
    let content_root = ctx.root();
    let node_id = ctx.create_node(content_root, NodeContent::Empty);

    // Move it to tooltip layer
    ctx.move_to_layer(node_id, Layer::Tooltip);

    let tooltip_root = ctx.layer_root(Layer::Tooltip);
    let scene = ctx.scene();

    // Should be under tooltip now
    let tooltip_node = scene.get_node(tooltip_root).unwrap();
    assert!(tooltip_node.children.contains(&node_id));

    // Should NOT be under content anymore
    let content_node = scene.get_node(content_root).unwrap();
    assert!(!content_node.children.contains(&node_id));
}

#[test]
fn test_move_to_content() {
    let mut ctx = WidgetContext::new();

    // Create in dialog layer, then move back to content
    let node_id = ctx.add_to_layer(Layer::Dialog, NodeContent::Empty);
    ctx.move_to_content(node_id);

    let content_root = ctx.root();
    let scene = ctx.scene();
    let content_node = scene.get_node(content_root).unwrap();
    assert!(content_node.children.contains(&node_id));
}

#[test]
fn test_layers_are_children_of_scene_root() {
    let ctx = WidgetContext::new();
    let scene = ctx.scene();
    let root = scene.get_node(ctx.scene_root()).unwrap();

    // Scene root should have exactly 6 children (one per layer)
    assert_eq!(root.children.len(), 6);

    // And they should be the layer roots in order
    for (i, layer) in Layer::ALL.iter().enumerate() {
        assert_eq!(root.children[i], ctx.layer_root(*layer));
    }
}
```

**Step 2: Run tests**

```bash
cargo test -p widget-core --test layer_tests
```

Expected: All 6 tests pass.

**Step 3: Commit**

```bash
git add crates/widget-core/tests/layer_tests.rs
git commit -m "test(widget-core): add layer system tests

Tests for Layer enum ordering, LayerManager node creation,
add_to_layer, move_to_layer, move_to_content, and scene
root structure verification."
```

---

## Task 4: Add WidgetContext extension mechanism

**Files:**
- Modify: `crates/widget-core/src/context/mod.rs`

Material widgets need access to `MaterialTheme` during build, but `widget-core` can't depend on `material-ui`. Solution: a generic type-keyed extension map on WidgetContext.

**Step 1: Add extension storage to WidgetContext**

Add to imports:
```rust
use std::any::{Any, TypeId};
```

Add to WidgetContext struct fields:
```rust
/// Generic extension storage for theme systems and plugins.
/// Keyed by TypeId so any crate can store/retrieve typed data.
extensions: HashMap<TypeId, Box<dyn Any>>,
```

Initialize in `new()`:
```rust
extensions: HashMap::new(),
```

**Step 2: Add extension API methods**

```rust
/// Store a typed extension value (e.g., MaterialTheme).
/// Overwrites any previous value of the same type.
pub fn set_extension<T: 'static>(&mut self, value: T) {
    self.extensions.insert(TypeId::of::<T>(), Box::new(value));
}

/// Retrieve a typed extension value by type.
pub fn get_extension<T: 'static>(&self) -> Option<&T> {
    self.extensions
        .get(&TypeId::of::<T>())
        .and_then(|v| v.downcast_ref::<T>())
}
```

**Step 3: Write test in layer_tests.rs (or a new extensions_tests.rs)**

```rust
#[test]
fn test_widget_context_extensions() {
    let mut ctx = WidgetContext::new();

    // No extension set yet
    assert!(ctx.get_extension::<String>().is_none());

    // Set a string extension
    ctx.set_extension("hello".to_string());
    assert_eq!(ctx.get_extension::<String>().unwrap(), "hello");

    // Overwrite
    ctx.set_extension("world".to_string());
    assert_eq!(ctx.get_extension::<String>().unwrap(), "world");

    // Different type doesn't interfere
    ctx.set_extension(42u32);
    assert_eq!(ctx.get_extension::<u32>().unwrap(), &42);
    assert_eq!(ctx.get_extension::<String>().unwrap(), "world");
}
```

**Step 4: Run tests**

```bash
cargo test -p widget-core --test layer_tests
```

Expected: All tests pass.

**Step 5: Commit**

```bash
git add crates/widget-core/src/context/mod.rs crates/widget-core/tests/layer_tests.rs
git commit -m "feat(widget-core): add generic extension mechanism to WidgetContext

Type-keyed HashMap<TypeId, Box<dyn Any>> allows external crates
(e.g., material-ui) to store and retrieve typed data without
circular dependencies. Used by MaterialTheme to make MD3 tokens
available to material widgets during build."
```

---

## Task 5: Scaffold material-ui crate

**Files:**
- Create: `crates/material-ui/Cargo.toml`
- Create: `crates/material-ui/src/lib.rs`
- Create: `crates/material-ui/src/theme/mod.rs`
- Create: `crates/material-ui/src/theme/color.rs`
- Create: `crates/material-ui/src/theme/typography.rs`
- Create: `crates/material-ui/src/theme/shape.rs`
- Create: `crates/material-ui/src/theme/elevation.rs`
- Modify: `Cargo.toml` (workspace root — add member)

**Step 1: Add to workspace**

In root `Cargo.toml`, add `"crates/material-ui"` to the `members` array.

**Step 2: Create `crates/material-ui/Cargo.toml`**

```toml
[package]
name = "material-ui"
version = "0.1.0"
edition = "2024"

[dependencies]
widget-core = { path = "../widget-core" }
theme-engine = { path = "../theme-engine" }
style-engine = { path = "../style-engine" }
flux-state = { path = "../flux-state" }
render-engine = { path = "../render-engine" }
glam = { workspace = true }
thiserror = { workspace = true }

[dev-dependencies]
criterion = "0.5"

[[bench]]
name = "material_benchmarks"
harness = false
```

**Step 3: Create `crates/material-ui/src/lib.rs`**

```rust
//! Material UI — Material Design 3 component library for Arthropod.
//!
//! Provides MD3-themed widgets built on top of `widget-core`.
//! Use [`MaterialTheme::from_seed`] to generate a complete MD3 theme
//! from a single seed color, then store it in [`WidgetContext`] via
//! the extension mechanism.
//!
//! # Quick Start
//!
//! ```ignore
//! use material_ui::theme::MaterialTheme;
//! use widget_core::WidgetContext;
//! use glam::Vec4;
//!
//! let theme = MaterialTheme::from_seed(Vec4::new(0.4, 0.3, 0.9, 1.0));
//! let mut ctx = WidgetContext::new();
//! ctx.set_extension(theme);
//! ```

pub mod theme;
```

**Step 4: Create `crates/material-ui/src/theme/mod.rs`**

```rust
//! Material Design 3 theme system.
//!
//! Generates a complete MD3 token set (color, typography, shape, elevation)
//! from a single seed color using tonal palette generation.

mod color;
mod elevation;
mod shape;
mod typography;

pub use color::ColorScheme;
pub use elevation::{ElevationLevel, ElevationScale};
pub use shape::ShapeScale;
pub use typography::{FontWeight, TextStyle, TypographyScale};

use glam::Vec4;
use theme_engine::DesignTokens;

/// Top-level Material Design 3 theme.
///
/// Generate from a seed color, then store in WidgetContext:
/// ```ignore
/// let theme = MaterialTheme::from_seed(Vec4::new(0.4, 0.3, 0.9, 1.0));
/// ctx.set_extension(theme);
/// ```
#[derive(Debug, Clone)]
pub struct MaterialTheme {
    pub color: ColorScheme,
    pub typography: TypographyScale,
    pub shape: ShapeScale,
    pub elevation: ElevationScale,
}

impl MaterialTheme {
    /// Generate a full MD3 light theme from a single seed color.
    pub fn from_seed(seed: Vec4) -> Self {
        Self::from_seed_light(seed)
    }

    /// Generate an MD3 light theme from a seed color.
    pub fn from_seed_light(seed: Vec4) -> Self {
        Self {
            color: ColorScheme::from_seed_light(seed),
            typography: TypographyScale::default(),
            shape: ShapeScale::default(),
            elevation: ElevationScale::default(),
        }
    }

    /// Generate an MD3 dark theme from a seed color.
    pub fn from_seed_dark(seed: Vec4) -> Self {
        Self {
            color: ColorScheme::from_seed_dark(seed),
            typography: TypographyScale::default(),
            shape: ShapeScale::default(),
            elevation: ElevationScale::default(),
        }
    }

    /// Convert to widget-core DesignTokens for compatibility with
    /// base widgets that read DesignTokens.
    pub fn to_design_tokens(&self) -> DesignTokens {
        let c = &self.color;
        DesignTokens {
            surface_primary: theme_engine::TokenValue::Color(c.surface),
            surface_secondary: theme_engine::TokenValue::Color(c.surface_container),
            surface_elevated: theme_engine::TokenValue::Color(c.surface_container_high),
            text_primary: c.on_surface,
            text_secondary: c.on_surface_variant,
            text_tertiary: c.outline,
            accent: c.primary,
            accent_hover: c.primary_container,
            accent_pressed: c.on_primary_container,
            space_xs: 4.0,
            space_sm: 8.0,
            space_md: 16.0,
            space_lg: 24.0,
            space_xl: 32.0,
            space_2xl: 48.0,
            radius_sm: self.shape.extra_small,
            radius_md: self.shape.small,
            radius_lg: self.shape.medium,
            radius_xl: self.shape.large,
        }
    }
}
```

**Step 5: Create placeholder files for the theme modules**

`crates/material-ui/src/theme/color.rs`:
```rust
use glam::Vec4;

/// MD3 color scheme with all color roles.
#[derive(Debug, Clone)]
pub struct ColorScheme {
    // Primary
    pub primary: Vec4,
    pub on_primary: Vec4,
    pub primary_container: Vec4,
    pub on_primary_container: Vec4,

    // Secondary
    pub secondary: Vec4,
    pub on_secondary: Vec4,
    pub secondary_container: Vec4,
    pub on_secondary_container: Vec4,

    // Tertiary
    pub tertiary: Vec4,
    pub on_tertiary: Vec4,
    pub tertiary_container: Vec4,
    pub on_tertiary_container: Vec4,

    // Error
    pub error: Vec4,
    pub on_error: Vec4,
    pub error_container: Vec4,
    pub on_error_container: Vec4,

    // Surface
    pub surface: Vec4,
    pub on_surface: Vec4,
    pub surface_variant: Vec4,
    pub on_surface_variant: Vec4,
    pub surface_container_lowest: Vec4,
    pub surface_container_low: Vec4,
    pub surface_container: Vec4,
    pub surface_container_high: Vec4,
    pub surface_container_highest: Vec4,

    // Outline
    pub outline: Vec4,
    pub outline_variant: Vec4,

    // Inverse
    pub inverse_surface: Vec4,
    pub inverse_on_surface: Vec4,
    pub inverse_primary: Vec4,

    // Misc
    pub scrim: Vec4,
    pub shadow: Vec4,
}

impl ColorScheme {
    /// Generate light color scheme from seed.
    /// Uses simplified HSL-based tonal generation.
    pub fn from_seed_light(_seed: Vec4) -> Self {
        todo!("Implement in Task 6")
    }

    /// Generate dark color scheme from seed.
    pub fn from_seed_dark(_seed: Vec4) -> Self {
        todo!("Implement in Task 6")
    }
}
```

`crates/material-ui/src/theme/typography.rs`:
```rust
/// Font weight for MD3 type scale.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FontWeight {
    Regular, // 400
    Medium,  // 500
    Bold,    // 700
}

/// A single text style in the MD3 type scale.
#[derive(Debug, Clone, Copy)]
pub struct TextStyle {
    pub font_size: f32,
    pub line_height: f32,
    pub font_weight: FontWeight,
    pub letter_spacing: f32,
}

/// MD3 typography scale with 15 slots.
#[derive(Debug, Clone)]
pub struct TypographyScale {
    pub display_large: TextStyle,
    pub display_medium: TextStyle,
    pub display_small: TextStyle,
    pub headline_large: TextStyle,
    pub headline_medium: TextStyle,
    pub headline_small: TextStyle,
    pub title_large: TextStyle,
    pub title_medium: TextStyle,
    pub title_small: TextStyle,
    pub body_large: TextStyle,
    pub body_medium: TextStyle,
    pub body_small: TextStyle,
    pub label_large: TextStyle,
    pub label_medium: TextStyle,
    pub label_small: TextStyle,
}

impl Default for TypographyScale {
    fn default() -> Self {
        Self {
            display_large: TextStyle { font_size: 57.0, line_height: 64.0, font_weight: FontWeight::Regular, letter_spacing: -0.25 },
            display_medium: TextStyle { font_size: 45.0, line_height: 52.0, font_weight: FontWeight::Regular, letter_spacing: 0.0 },
            display_small: TextStyle { font_size: 36.0, line_height: 44.0, font_weight: FontWeight::Regular, letter_spacing: 0.0 },
            headline_large: TextStyle { font_size: 32.0, line_height: 40.0, font_weight: FontWeight::Regular, letter_spacing: 0.0 },
            headline_medium: TextStyle { font_size: 28.0, line_height: 36.0, font_weight: FontWeight::Regular, letter_spacing: 0.0 },
            headline_small: TextStyle { font_size: 24.0, line_height: 32.0, font_weight: FontWeight::Regular, letter_spacing: 0.0 },
            title_large: TextStyle { font_size: 22.0, line_height: 28.0, font_weight: FontWeight::Regular, letter_spacing: 0.0 },
            title_medium: TextStyle { font_size: 16.0, line_height: 24.0, font_weight: FontWeight::Medium, letter_spacing: 0.15 },
            title_small: TextStyle { font_size: 14.0, line_height: 20.0, font_weight: FontWeight::Medium, letter_spacing: 0.1 },
            body_large: TextStyle { font_size: 16.0, line_height: 24.0, font_weight: FontWeight::Regular, letter_spacing: 0.5 },
            body_medium: TextStyle { font_size: 14.0, line_height: 20.0, font_weight: FontWeight::Regular, letter_spacing: 0.25 },
            body_small: TextStyle { font_size: 12.0, line_height: 16.0, font_weight: FontWeight::Regular, letter_spacing: 0.4 },
            label_large: TextStyle { font_size: 14.0, line_height: 20.0, font_weight: FontWeight::Medium, letter_spacing: 0.1 },
            label_medium: TextStyle { font_size: 12.0, line_height: 16.0, font_weight: FontWeight::Medium, letter_spacing: 0.5 },
            label_small: TextStyle { font_size: 11.0, line_height: 16.0, font_weight: FontWeight::Medium, letter_spacing: 0.5 },
        }
    }
}
```

`crates/material-ui/src/theme/shape.rs`:
```rust
/// MD3 shape scale — corner radius presets.
#[derive(Debug, Clone, Copy)]
pub struct ShapeScale {
    pub none: f32,
    pub extra_small: f32,
    pub small: f32,
    pub medium: f32,
    pub large: f32,
    pub extra_large: f32,
    pub full: f32,
}

impl Default for ShapeScale {
    fn default() -> Self {
        Self {
            none: 0.0,
            extra_small: 4.0,
            small: 8.0,
            medium: 12.0,
            large: 16.0,
            extra_large: 28.0,
            full: 9999.0,
        }
    }
}
```

`crates/material-ui/src/theme/elevation.rs`:
```rust
/// A single elevation level in the MD3 scale.
#[derive(Debug, Clone, Copy)]
pub struct ElevationLevel {
    /// Surface tint blending opacity (0.0 - 1.0).
    pub tint_opacity: f32,
    /// Shadow offset in dp (for optional drop shadow).
    pub shadow_offset: f32,
}

/// MD3 elevation scale — tonal elevation levels.
#[derive(Debug, Clone, Copy)]
pub struct ElevationScale {
    pub level0: ElevationLevel,
    pub level1: ElevationLevel,
    pub level2: ElevationLevel,
    pub level3: ElevationLevel,
    pub level4: ElevationLevel,
    pub level5: ElevationLevel,
}

impl Default for ElevationScale {
    fn default() -> Self {
        Self {
            level0: ElevationLevel { tint_opacity: 0.0, shadow_offset: 0.0 },
            level1: ElevationLevel { tint_opacity: 0.05, shadow_offset: 1.0 },
            level2: ElevationLevel { tint_opacity: 0.08, shadow_offset: 3.0 },
            level3: ElevationLevel { tint_opacity: 0.11, shadow_offset: 6.0 },
            level4: ElevationLevel { tint_opacity: 0.12, shadow_offset: 8.0 },
            level5: ElevationLevel { tint_opacity: 0.14, shadow_offset: 12.0 },
        }
    }
}
```

**Step 6: Run `cargo check -p material-ui`**

Expected: Compiles (color.rs has `todo!()` but that's fine for check).

**Step 7: Commit**

```bash
git add crates/material-ui/ Cargo.toml
git commit -m "feat: scaffold material-ui crate with MD3 theme structure

New crate with MaterialTheme, ColorScheme (stub), TypographyScale,
ShapeScale, and ElevationScale. Typography defaults match MD3 spec.
to_design_tokens() bridges to widget-core's existing token system."
```

---

## Task 6: Implement MD3 color generation

**Files:**
- Modify: `crates/material-ui/src/theme/color.rs`

This is the most complex part. MD3 uses HCT (Hue-Chroma-Tone) color space. We implement a simplified version using HSL for the initial pass, which produces visually reasonable results. Can upgrade to full HCT later.

**Step 1: Write the failing test first**

Add to `crates/material-ui/src/theme/color.rs` at the bottom:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_seed_light_produces_valid_colors() {
        let seed = Vec4::new(0.4, 0.3, 0.9, 1.0); // Purple-ish
        let scheme = ColorScheme::from_seed_light(seed);

        // All colors should have alpha = 1.0
        assert_eq!(scheme.primary.w, 1.0);
        assert_eq!(scheme.on_primary.w, 1.0);
        assert_eq!(scheme.surface.w, 1.0);
        assert_eq!(scheme.on_surface.w, 1.0);

        // Primary should be a saturated color (not gray)
        let primary = scheme.primary;
        let max_channel = primary.x.max(primary.y).max(primary.z);
        let min_channel = primary.x.min(primary.y).min(primary.z);
        assert!(max_channel - min_channel > 0.1, "Primary should be saturated");

        // Surface should be near-white for light theme
        assert!(scheme.surface.x > 0.9, "Light surface should be near-white");
        assert!(scheme.surface.y > 0.9, "Light surface should be near-white");
        assert!(scheme.surface.z > 0.9, "Light surface should be near-white");

        // on_surface should be near-black for light theme
        assert!(scheme.on_surface.x < 0.15, "Light on_surface should be near-black");
    }

    #[test]
    fn test_from_seed_dark_produces_valid_colors() {
        let seed = Vec4::new(0.4, 0.3, 0.9, 1.0);
        let scheme = ColorScheme::from_seed_dark(seed);

        // Surface should be near-black for dark theme
        assert!(scheme.surface.x < 0.15, "Dark surface should be near-black");
        assert!(scheme.surface.y < 0.15, "Dark surface should be near-black");

        // on_surface should be near-white for dark theme
        assert!(scheme.on_surface.x > 0.85, "Dark on_surface should be near-white");
    }

    #[test]
    fn test_different_seeds_produce_different_primaries() {
        let red = ColorScheme::from_seed_light(Vec4::new(0.9, 0.2, 0.2, 1.0));
        let blue = ColorScheme::from_seed_light(Vec4::new(0.2, 0.2, 0.9, 1.0));

        // Different seeds should produce different primary colors
        let diff = (red.primary - blue.primary).length();
        assert!(diff > 0.1, "Different seeds should produce different primaries, diff={}", diff);
    }

    #[test]
    fn test_error_colors_are_red_toned() {
        let scheme = ColorScheme::from_seed_light(Vec4::new(0.2, 0.8, 0.2, 1.0)); // Green seed
        // Error should still be reddish regardless of seed
        assert!(scheme.error.x > scheme.error.y, "Error should be red-dominant");
        assert!(scheme.error.x > scheme.error.z, "Error should be red-dominant");
    }

    #[test]
    fn test_scrim_is_black_with_alpha() {
        let scheme = ColorScheme::from_seed_light(Vec4::new(0.5, 0.5, 0.5, 1.0));
        assert!(scheme.scrim.x < 0.05);
        assert!(scheme.scrim.y < 0.05);
        assert!(scheme.scrim.z < 0.05);
        assert_eq!(scheme.scrim.w, 1.0);
    }
}
```

**Step 2: Run tests to verify they fail**

```bash
cargo test -p material-ui
```

Expected: FAIL (todo! panics).

**Step 3: Implement color generation**

Replace the `todo!()` stubs in `ColorScheme` with the full implementation. Here's the approach:

```rust
use glam::Vec4;

// --- HSL helpers ---

fn rgb_to_hsl(r: f32, g: f32, b: f32) -> (f32, f32, f32) {
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let l = (max + min) / 2.0;

    if (max - min).abs() < 1e-6 {
        return (0.0, 0.0, l);
    }

    let d = max - min;
    let s = if l > 0.5 {
        d / (2.0 - max - min)
    } else {
        d / (max + min)
    };

    let h = if (max - r).abs() < 1e-6 {
        ((g - b) / d + if g < b { 6.0 } else { 0.0 }) / 6.0
    } else if (max - g).abs() < 1e-6 {
        ((b - r) / d + 2.0) / 6.0
    } else {
        ((r - g) / d + 4.0) / 6.0
    };

    (h, s, l)
}

fn hsl_to_rgb(h: f32, s: f32, l: f32) -> (f32, f32, f32) {
    if s.abs() < 1e-6 {
        return (l, l, l);
    }

    let q = if l < 0.5 { l * (1.0 + s) } else { l + s - l * s };
    let p = 2.0 * l - q;

    let r = hue_to_rgb(p, q, h + 1.0 / 3.0);
    let g = hue_to_rgb(p, q, h);
    let b = hue_to_rgb(p, q, h - 1.0 / 3.0);

    (r, g, b)
}

fn hue_to_rgb(p: f32, q: f32, mut t: f32) -> f32 {
    if t < 0.0 { t += 1.0; }
    if t > 1.0 { t -= 1.0; }
    if t < 1.0 / 6.0 { return p + (q - p) * 6.0 * t; }
    if t < 1.0 / 2.0 { return q; }
    if t < 2.0 / 3.0 { return p + (q - p) * (2.0 / 3.0 - t) * 6.0; }
    p
}

/// Generate a color at a specific tone (lightness) preserving hue and
/// adjusting saturation to stay in gamut.
fn tone(h: f32, s: f32, lightness: f32) -> Vec4 {
    // Reduce saturation at extreme lightness to stay in gamut
    let s_adjusted = s * (1.0 - (2.0 * lightness - 1.0).abs() * 0.5);
    let (r, g, b) = hsl_to_rgb(h, s_adjusted, lightness);
    Vec4::new(r.clamp(0.0, 1.0), g.clamp(0.0, 1.0), b.clamp(0.0, 1.0), 1.0)
}

impl ColorScheme {
    pub fn from_seed_light(seed: Vec4) -> Self {
        let (h, s, _) = rgb_to_hsl(seed.x, seed.y, seed.z);
        // MD3: Secondary is desaturated, Tertiary is hue-shifted +60°
        let s_primary = s.max(0.4); // Ensure minimum saturation
        let s_secondary = s_primary * 0.5;
        let h_tertiary = (h + 60.0 / 360.0) % 1.0;
        let s_tertiary = s_primary * 0.7;

        // Neutral palette: very desaturated version of primary
        let s_neutral = s_primary * 0.08;
        let s_neutral_variant = s_primary * 0.15;

        // Error palette: fixed red hue
        let h_error = 0.0; // Red
        let s_error: f32 = 0.8;

        Self {
            // Primary tones: 40 (light theme primary)
            primary: tone(h, s_primary, 0.40),
            on_primary: tone(h, s_primary, 1.0),
            primary_container: tone(h, s_primary, 0.90),
            on_primary_container: tone(h, s_primary, 0.10),

            // Secondary
            secondary: tone(h, s_secondary, 0.40),
            on_secondary: tone(h, s_secondary, 1.0),
            secondary_container: tone(h, s_secondary, 0.90),
            on_secondary_container: tone(h, s_secondary, 0.10),

            // Tertiary
            tertiary: tone(h_tertiary, s_tertiary, 0.40),
            on_tertiary: tone(h_tertiary, s_tertiary, 1.0),
            tertiary_container: tone(h_tertiary, s_tertiary, 0.90),
            on_tertiary_container: tone(h_tertiary, s_tertiary, 0.10),

            // Error (always red, independent of seed)
            error: tone(h_error, s_error, 0.40),
            on_error: tone(h_error, s_error, 1.0),
            error_container: tone(h_error, s_error, 0.90),
            on_error_container: tone(h_error, s_error, 0.10),

            // Surface (neutral)
            surface: tone(h, s_neutral, 0.98),
            on_surface: tone(h, s_neutral, 0.10),
            surface_variant: tone(h, s_neutral_variant, 0.90),
            on_surface_variant: tone(h, s_neutral_variant, 0.30),
            surface_container_lowest: tone(h, s_neutral, 1.0),
            surface_container_low: tone(h, s_neutral, 0.96),
            surface_container: tone(h, s_neutral, 0.94),
            surface_container_high: tone(h, s_neutral, 0.92),
            surface_container_highest: tone(h, s_neutral, 0.90),

            // Outline
            outline: tone(h, s_neutral_variant, 0.50),
            outline_variant: tone(h, s_neutral_variant, 0.80),

            // Inverse
            inverse_surface: tone(h, s_neutral, 0.20),
            inverse_on_surface: tone(h, s_neutral, 0.95),
            inverse_primary: tone(h, s_primary, 0.80),

            // Misc
            scrim: Vec4::new(0.0, 0.0, 0.0, 1.0),
            shadow: Vec4::new(0.0, 0.0, 0.0, 1.0),
        }
    }

    pub fn from_seed_dark(seed: Vec4) -> Self {
        let (h, s, _) = rgb_to_hsl(seed.x, seed.y, seed.z);
        let s_primary = s.max(0.4);
        let s_secondary = s_primary * 0.5;
        let h_tertiary = (h + 60.0 / 360.0) % 1.0;
        let s_tertiary = s_primary * 0.7;
        let s_neutral = s_primary * 0.08;
        let s_neutral_variant = s_primary * 0.15;
        let h_error = 0.0;
        let s_error: f32 = 0.8;

        Self {
            // Dark theme: primary at tone 80, on_primary at tone 20
            primary: tone(h, s_primary, 0.80),
            on_primary: tone(h, s_primary, 0.20),
            primary_container: tone(h, s_primary, 0.30),
            on_primary_container: tone(h, s_primary, 0.90),

            secondary: tone(h, s_secondary, 0.80),
            on_secondary: tone(h, s_secondary, 0.20),
            secondary_container: tone(h, s_secondary, 0.30),
            on_secondary_container: tone(h, s_secondary, 0.90),

            tertiary: tone(h_tertiary, s_tertiary, 0.80),
            on_tertiary: tone(h_tertiary, s_tertiary, 0.20),
            tertiary_container: tone(h_tertiary, s_tertiary, 0.30),
            on_tertiary_container: tone(h_tertiary, s_tertiary, 0.90),

            error: tone(h_error, s_error, 0.80),
            on_error: tone(h_error, s_error, 0.20),
            error_container: tone(h_error, s_error, 0.30),
            on_error_container: tone(h_error, s_error, 0.90),

            surface: tone(h, s_neutral, 0.06),
            on_surface: tone(h, s_neutral, 0.90),
            surface_variant: tone(h, s_neutral_variant, 0.30),
            on_surface_variant: tone(h, s_neutral_variant, 0.80),
            surface_container_lowest: tone(h, s_neutral, 0.04),
            surface_container_low: tone(h, s_neutral, 0.10),
            surface_container: tone(h, s_neutral, 0.12),
            surface_container_high: tone(h, s_neutral, 0.17),
            surface_container_highest: tone(h, s_neutral, 0.22),

            outline: tone(h, s_neutral_variant, 0.60),
            outline_variant: tone(h, s_neutral_variant, 0.30),

            inverse_surface: tone(h, s_neutral, 0.90),
            inverse_on_surface: tone(h, s_neutral, 0.20),
            inverse_primary: tone(h, s_primary, 0.40),

            scrim: Vec4::new(0.0, 0.0, 0.0, 1.0),
            shadow: Vec4::new(0.0, 0.0, 0.0, 1.0),
        }
    }
}
```

**Step 4: Run tests**

```bash
cargo test -p material-ui
```

Expected: All 5 color tests pass.

**Step 5: Commit**

```bash
git add crates/material-ui/src/theme/color.rs
git commit -m "feat(material-ui): implement MD3 tonal color generation

HSL-based tonal palette generation from a seed color. Produces all
MD3 color roles for both light and dark schemes: primary, secondary
(desaturated), tertiary (hue-shifted +60°), error (fixed red),
surface (neutral), outline, inverse, scrim, and shadow.

Can be upgraded to HCT color space later for more accurate results."
```

---

## Task 7: MaterialTheme integration tests

**Files:**
- Create: `crates/material-ui/tests/theme_tests.rs`

**Step 1: Write integration tests**

```rust
use material_ui::theme::{MaterialTheme, ColorScheme, TypographyScale, ShapeScale, ElevationScale, FontWeight};
use glam::Vec4;

#[test]
fn test_material_theme_from_seed() {
    let theme = MaterialTheme::from_seed(Vec4::new(0.4, 0.3, 0.9, 1.0));

    // Should produce a valid theme with all components
    let _ = &theme.color;
    let _ = &theme.typography;
    let _ = &theme.shape;
    let _ = &theme.elevation;
}

#[test]
fn test_to_design_tokens() {
    let theme = MaterialTheme::from_seed(Vec4::new(0.4, 0.3, 0.9, 1.0));
    let tokens = theme.to_design_tokens();

    // Surface should map from MD3 surface
    let surface_color = tokens.surface_primary.as_color();
    assert!(surface_color.is_some());

    // Accent should be primary
    assert_eq!(tokens.accent, theme.color.primary);

    // Spacing should be standard 8px scale
    assert_eq!(tokens.space_sm, 8.0);
    assert_eq!(tokens.space_md, 16.0);

    // Radius should map from shape scale
    assert_eq!(tokens.radius_sm, theme.shape.extra_small);
    assert_eq!(tokens.radius_md, theme.shape.small);
}

#[test]
fn test_typography_defaults_match_md3_spec() {
    let typo = TypographyScale::default();

    // Display large: 57/64
    assert_eq!(typo.display_large.font_size, 57.0);
    assert_eq!(typo.display_large.line_height, 64.0);

    // Body medium: 14/20
    assert_eq!(typo.body_medium.font_size, 14.0);
    assert_eq!(typo.body_medium.line_height, 20.0);

    // Label large should be medium weight
    assert_eq!(typo.label_large.font_weight, FontWeight::Medium);

    // Title medium should be medium weight
    assert_eq!(typo.title_medium.font_weight, FontWeight::Medium);
}

#[test]
fn test_shape_defaults_match_md3_spec() {
    let shape = ShapeScale::default();

    assert_eq!(shape.none, 0.0);
    assert_eq!(shape.extra_small, 4.0);
    assert_eq!(shape.small, 8.0);
    assert_eq!(shape.medium, 12.0);
    assert_eq!(shape.large, 16.0);
    assert_eq!(shape.extra_large, 28.0);
    assert_eq!(shape.full, 9999.0);
}

#[test]
fn test_elevation_defaults() {
    let elevation = ElevationScale::default();

    // Level 0 should have no tint or shadow
    assert_eq!(elevation.level0.tint_opacity, 0.0);
    assert_eq!(elevation.level0.shadow_offset, 0.0);

    // Higher levels should increase
    assert!(elevation.level3.tint_opacity > elevation.level1.tint_opacity);
    assert!(elevation.level5.shadow_offset > elevation.level3.shadow_offset);
}

#[test]
fn test_light_and_dark_themes_differ() {
    let seed = Vec4::new(0.4, 0.3, 0.9, 1.0);
    let light = MaterialTheme::from_seed_light(seed);
    let dark = MaterialTheme::from_seed_dark(seed);

    // Surfaces should be very different
    let surface_diff = (light.color.surface - dark.color.surface).length();
    assert!(surface_diff > 0.5, "Light/dark surfaces should differ significantly");

    // Primary tone should differ (40 vs 80)
    let primary_diff = (light.color.primary - dark.color.primary).length();
    assert!(primary_diff > 0.2, "Light/dark primaries should differ");
}

#[test]
fn test_theme_stored_in_widget_context() {
    let theme = MaterialTheme::from_seed(Vec4::new(0.4, 0.3, 0.9, 1.0));
    let mut ctx = widget_core::WidgetContext::new();

    // Store theme as extension
    ctx.set_extension(theme.clone());

    // Retrieve it
    let retrieved = ctx.get_extension::<MaterialTheme>().unwrap();
    assert_eq!(retrieved.color.primary, theme.color.primary);
    assert_eq!(retrieved.shape.medium, 12.0);
}
```

**Step 2: Run tests**

```bash
cargo test -p material-ui
```

Expected: All tests pass.

**Step 3: Commit**

```bash
git add crates/material-ui/tests/theme_tests.rs
git commit -m "test(material-ui): add comprehensive theme integration tests

Tests MaterialTheme generation, DesignTokens conversion, MD3 spec
compliance for typography/shape/elevation defaults, light vs dark
theme differentiation, and WidgetContext extension storage."
```

---

## Task 8: Backdrop widget

**Files:**
- Create: `crates/material-ui/src/feedback/mod.rs`
- Create: `crates/material-ui/src/feedback/backdrop.rs`
- Modify: `crates/material-ui/src/lib.rs` (add module)

**Step 1: Write the failing test first**

In `crates/material-ui/src/feedback/backdrop.rs`:

```rust
use widget_core::{Layer, NodeContent, Widget, WidgetContext};
use render_engine::node::SceneNode;
use glam::Vec4;
use std::sync::Arc;

/// MD3 Backdrop — full-screen scrim overlay.
///
/// Renders in the Dialog layer as a semi-transparent overlay.
/// Used by Dialog and Drawer for modal behavior.
///
/// # Example
/// ```ignore
/// use material_ui::feedback::Backdrop;
///
/// Backdrop::new()
///     .opacity(0.32)
///     .on_click(|| { /* dismiss */ })
/// ```
pub struct Backdrop {
    opacity: f32,
    on_click: Option<Arc<dyn Fn() + Send + Sync>>,
}

impl Backdrop {
    pub fn new() -> Self {
        Self {
            opacity: 0.32, // MD3 default scrim opacity
            on_click: None,
        }
    }

    /// Set scrim opacity (0.0 - 1.0). MD3 default is 0.32.
    pub fn opacity(mut self, opacity: f32) -> Self {
        self.opacity = opacity;
        self
    }

    /// Called when the backdrop is clicked (typically to dismiss a modal).
    pub fn on_click(mut self, callback: impl Fn() + Send + Sync + 'static) -> Self {
        self.on_click = Some(Arc::new(callback));
        self
    }
}

impl Default for Backdrop {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for Backdrop {
    fn build(&self, ctx: &mut WidgetContext) -> render_engine::scene::NodeId {
        // Backdrop renders in the Dialog layer
        let color = render_engine::Color::rgba(0.0, 0.0, 0.0, self.opacity);
        let node_id = ctx.add_to_layer(
            Layer::Dialog,
            NodeContent::SolidColor { color },
        );

        // Make it clickable if handler provided
        if let Some(ref callback) = self.on_click {
            ctx.add_clickable(node_id, callback.clone());
        }

        node_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, Ordering};

    #[test]
    fn test_backdrop_builds_in_dialog_layer() {
        let mut ctx = WidgetContext::new();
        let backdrop = Backdrop::new();
        let node_id = backdrop.build(&mut ctx);

        // Node should be a child of the Dialog layer
        let dialog_root = ctx.layer_root(Layer::Dialog);
        let scene = ctx.scene();
        let dialog_node = scene.get_node(dialog_root).unwrap();
        assert!(dialog_node.children.contains(&node_id));
    }

    #[test]
    fn test_backdrop_default_opacity() {
        let mut ctx = WidgetContext::new();
        let backdrop = Backdrop::new();
        let node_id = backdrop.build(&mut ctx);

        let scene = ctx.scene();
        let node = scene.get_node(node_id).unwrap();

        // Should be a SolidColor with black at 0.32 alpha
        match &node.content {
            NodeContent::SolidColor { color } => {
                assert!(color.r() < 0.01);
                assert!(color.g() < 0.01);
                assert!(color.b() < 0.01);
                assert!((color.a() - 0.32).abs() < 0.01);
            }
            _ => panic!("Backdrop should use SolidColor content"),
        }
    }

    #[test]
    fn test_backdrop_custom_opacity() {
        let mut ctx = WidgetContext::new();
        let backdrop = Backdrop::new().opacity(0.5);
        let node_id = backdrop.build(&mut ctx);

        let scene = ctx.scene();
        let node = scene.get_node(node_id).unwrap();

        match &node.content {
            NodeContent::SolidColor { color } => {
                assert!((color.a() - 0.5).abs() < 0.01);
            }
            _ => panic!("Backdrop should use SolidColor content"),
        }
    }

    #[test]
    fn test_backdrop_on_click() {
        let clicked = Arc::new(AtomicBool::new(false));
        let clicked_clone = clicked.clone();

        let mut ctx = WidgetContext::new();
        let backdrop = Backdrop::new().on_click(move || {
            clicked_clone.store(true, Ordering::SeqCst);
        });
        let node_id = backdrop.build(&mut ctx);

        // Should be registered as clickable
        assert!(ctx.has_clickable(node_id));
    }
}
```

**Step 2: Create module files**

`crates/material-ui/src/feedback/mod.rs`:
```rust
mod backdrop;

pub use backdrop::Backdrop;
```

Update `crates/material-ui/src/lib.rs`:
```rust
pub mod feedback;
pub mod theme;
```

**Step 3: Run tests**

```bash
cargo test -p material-ui
```

Expected: All Backdrop tests pass.

**Step 4: Commit**

```bash
git add crates/material-ui/src/feedback/ crates/material-ui/src/lib.rs
git commit -m "feat(material-ui): add Backdrop widget

Full-screen scrim overlay rendering in the Dialog layer.
MD3 default 0.32 opacity. Configurable opacity and on_click
handler for modal dismissal. First widget using the layer system."
```

---

## Task 9: Fix existing tests and examples

**Files:**
- Potentially modify any test/example that relies on `ctx.root()` being the scene root

**Step 1: Run full test suite**

```bash
cargo test --workspace --exclude plat-core 2>&1
```

**Step 2: Fix any failures**

The `root()` change (now returns Content layer root instead of scene root) may break tests that check `scene.get_node(ctx.root()).unwrap().children.len()` or similar. Fix by either:
- Using `ctx.scene_root()` where the test explicitly needs the scene root
- Adjusting expected child counts to account for the layer indirection

**Step 3: Run tests again to confirm all pass**

```bash
cargo test --workspace --exclude plat-core
```

Expected: All tests pass.

**Step 4: Commit fixes**

```bash
git add -A
git commit -m "fix: update tests for layer system root() change

root() now returns Content layer root. Updated tests that relied
on root() being the scene root to use scene_root() or adjusted
expected values."
```

---

## Task 10: Benchmarks

**Files:**
- Create: `crates/material-ui/benches/material_benchmarks.rs`
- Create: `crates/widget-core/benches/layer_benchmarks.rs`

**Step 1: Create material-ui benchmarks**

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use material_ui::theme::MaterialTheme;
use glam::Vec4;

fn bench_theme_from_seed(c: &mut Criterion) {
    let seed = Vec4::new(0.4, 0.3, 0.9, 1.0);
    c.bench_function("MaterialTheme::from_seed", |b| {
        b.iter(|| MaterialTheme::from_seed(black_box(seed)));
    });
}

fn bench_to_design_tokens(c: &mut Criterion) {
    let theme = MaterialTheme::from_seed(Vec4::new(0.4, 0.3, 0.9, 1.0));
    c.bench_function("MaterialTheme::to_design_tokens", |b| {
        b.iter(|| theme.to_design_tokens());
    });
}

fn bench_color_scheme_light(c: &mut Criterion) {
    use material_ui::theme::ColorScheme;
    let seed = Vec4::new(0.4, 0.3, 0.9, 1.0);
    c.bench_function("ColorScheme::from_seed_light", |b| {
        b.iter(|| ColorScheme::from_seed_light(black_box(seed)));
    });
}

criterion_group!(benches, bench_theme_from_seed, bench_to_design_tokens, bench_color_scheme_light);
criterion_main!(benches);
```

**Step 2: Create layer benchmarks**

Add to `crates/widget-core/Cargo.toml` a new benchmark entry:
```toml
[[bench]]
name = "layer_benchmarks"
harness = false
```

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use widget_core::{Layer, WidgetContext};
use render_engine::node::NodeContent;

fn bench_layer_root_lookup(c: &mut Criterion) {
    let ctx = WidgetContext::new();
    c.bench_function("layer_root lookup", |b| {
        b.iter(|| ctx.layer_root(black_box(Layer::Dialog)));
    });
}

fn bench_add_to_layer(c: &mut Criterion) {
    c.bench_function("add_to_layer (1 node)", |b| {
        b.iter_with_setup(
            || WidgetContext::new(),
            |mut ctx| {
                ctx.add_to_layer(black_box(Layer::Dialog), NodeContent::Empty);
            },
        );
    });
}

fn bench_move_to_layer(c: &mut Criterion) {
    c.bench_function("move_to_layer", |b| {
        b.iter_with_setup(
            || {
                let mut ctx = WidgetContext::new();
                let root = ctx.root();
                let node = ctx.create_node(root, NodeContent::Empty);
                (ctx, node)
            },
            |(mut ctx, node)| {
                ctx.move_to_layer(black_box(node), Layer::Tooltip);
            },
        );
    });
}

criterion_group!(benches, bench_layer_root_lookup, bench_add_to_layer, bench_move_to_layer);
criterion_main!(benches);
```

**Step 3: Run benchmarks**

```bash
cargo bench -p material-ui
cargo bench -p widget-core --bench layer_benchmarks
```

Expected: Layer root lookup < 50ns, add_to_layer < 500ns, theme_from_seed < 10μs.

**Step 4: Commit**

```bash
git add crates/material-ui/benches/ crates/widget-core/benches/layer_benchmarks.rs crates/widget-core/Cargo.toml
git commit -m "bench: add layer system and material theme benchmarks

Layer root lookup, add_to_layer, move_to_layer benchmarks in
widget-core. Theme generation and token conversion benchmarks
in material-ui."
```

---

## Task 11: Run full workspace check and format

**Step 1: Format**

```bash
cargo fmt --all
```

**Step 2: Clippy**

```bash
cargo clippy -p widget-core -p material-ui --all-targets -- -D warnings
```

Fix any warnings.

**Step 3: Full test suite**

```bash
cargo test --workspace --exclude plat-core
```

Expected: All tests pass.

**Step 4: Commit any fixes**

```bash
git add -A
git commit -m "chore: fmt + clippy fixes for Phase 1"
```

---

## Summary

After completing all 11 tasks, Phase 1 delivers:

| Component | Location | Status |
|-----------|----------|--------|
| Layer System (Layer, LayerManager) | widget-core/src/layer.rs | New |
| WidgetContext layer API | widget-core/src/context/mod.rs | Modified |
| WidgetContext extensions | widget-core/src/context/mod.rs | Modified |
| material-ui crate scaffold | crates/material-ui/ | New |
| MaterialTheme | material-ui/src/theme/mod.rs | New |
| ColorScheme (HSL-based) | material-ui/src/theme/color.rs | New |
| TypographyScale | material-ui/src/theme/typography.rs | New |
| ShapeScale | material-ui/src/theme/shape.rs | New |
| ElevationScale | material-ui/src/theme/elevation.rs | New |
| Backdrop widget | material-ui/src/feedback/backdrop.rs | New |
| Layer tests | widget-core/tests/layer_tests.rs | New |
| Theme tests | material-ui/tests/theme_tests.rs | New |
| Benchmarks | both crates | New |

**Phase 2 (Leaf Inputs) can begin immediately after this lands.**
