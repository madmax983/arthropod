# Material UI Parity — Full Design Document

**Date:** 2026-03-26
**Status:** Approved
**Scope:** 42 new widgets + 8 restyled existing widgets + layer system + MD3 theme

## 1. Architecture Overview

A new `material-ui` crate wraps `widget-core` with Material Design 3 tokens and component variants.

```
material-ui          (MD3 components + theme)
├── widget-core      (base widgets, Widget trait, WidgetContext)
├── theme-engine     (DesignTokens, token resolution)
├── style-engine     (VisualStyle, Paint, CornerRadii)
└── flux-state       (reactive primitives)
```

Three major additions:

1. **Layer System** — Built into `widget-core`'s scene tree. Named layers with guaranteed z-order for overlays, dialogs, tooltips, and notifications. Inspired by Flash/Flex's `SystemManager` display list architecture.

2. **MD3 Token System** — `MaterialTheme` struct generating `DesignTokens` from an MD3 tonal palette. Covers color roles, typography scale, elevation levels, and shape scale.

3. **42 New Widgets + 8 Material Restylings** — Each widget in `material-ui` wraps or extends `widget-core` base widgets, applying MD3 tokens. Some are pure styling wrappers, others are entirely new components.

### Crate Structure

```
crates/material-ui/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── theme/
│   │   ├── mod.rs
│   │   ├── color.rs          (ColorScheme, tonal palette generation)
│   │   ├── typography.rs     (TypographyScale, TextStyle, FontWeight)
│   │   ├── shape.rs          (ShapeScale)
│   │   └── elevation.rs      (ElevationScale, ElevationLevel)
│   ├── inputs/
│   │   ├── mod.rs
│   │   ├── switch.rs
│   │   ├── radio.rs
│   │   ├── radio_group.rs
│   │   ├── select.rs
│   │   ├── slider.rs
│   │   ├── toggle_button.rs
│   │   ├── toggle_button_group.rs
│   │   ├── rating.rs
│   │   ├── autocomplete.rs
│   │   ├── fab.rs
│   │   └── button_group.rs
│   ├── data_display/
│   │   ├── mod.rs
│   │   ├── avatar.rs
│   │   ├── badge.rs
│   │   ├── chip.rs
│   │   ├── table.rs
│   │   ├── tooltip.rs
│   │   ├── link.rs
│   │   └── image_list.rs
│   ├── feedback/
│   │   ├── mod.rs
│   │   ├── alert.rs
│   │   ├── dialog.rs
│   │   ├── snackbar.rs
│   │   ├── skeleton.rs
│   │   └── backdrop.rs
│   ├── navigation/
│   │   ├── mod.rs
│   │   ├── app_bar.rs
│   │   ├── tabs.rs
│   │   ├── drawer.rs
│   │   ├── menu.rs
│   │   ├── menu_item.rs
│   │   ├── breadcrumbs.rs
│   │   ├── pagination.rs
│   │   ├── stepper.rs
│   │   └── bottom_navigation.rs
│   ├── surfaces/
│   │   ├── mod.rs
│   │   ├── accordion.rs
│   │   └── paper.rs
│   └── material/
│       ├── mod.rs
│       ├── button.rs         (MD3 Button wrapping widget-core Button)
│       ├── text_input.rs     (MD3 TextField wrapping widget-core TextInput)
│       ├── checkbox.rs       (MD3 Checkbox wrapping widget-core Checkbox)
│       ├── card.rs           (MD3 Card wrapping widget-core Card)
│       ├── progress_bar.rs   (MD3 LinearProgress wrapping widget-core ProgressBar)
│       ├── icon.rs           (MD3 Icon wrapping widget-core Icon)
│       ├── divider.rs        (MD3 Divider wrapping widget-core Divider)
│       └── form.rs           (MD3 Form wrapping widget-core Form)
├── tests/
├── benches/
└── examples/
    ├── material_gallery.rs
    ├── material_dashboard.rs
    └── material_form.rs
```

## 2. Layer System

Added to `widget-core`. Fundamental scene capability used by any theme.

### Layer Enum

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Layer {
    Content,       // z=0 — Normal widget tree (default)
    Dropdown,      // z=1 — Select, Menu, Autocomplete popups
    Dialog,        // z=2 — Modal, Dialog, Drawer overlay
    Notification,  // z=3 — Snackbar, Toast
    Tooltip,       // z=4 — Tooltips (always on top of UI)
    Cursor,        // z=5 — Drag previews, custom cursors
}
```

### Scene Initialization

Layer nodes are created as direct children of the scene root, in z-order:

```
Scene Root
├── Content [NodeId]
├── Dropdown [NodeId]
├── Dialog [NodeId]
├── Notification [NodeId]
├── Tooltip [NodeId]
└── Cursor [NodeId]
```

### LayerManager

```rust
pub struct LayerManager {
    layers: HashMap<Layer, NodeId>,
}

impl LayerManager {
    pub fn new(scene: &mut Scene) -> Self;
    pub fn get(&self, layer: Layer) -> NodeId;
}
```

### WidgetContext API

```rust
impl WidgetContext {
    /// Create a node in a specific layer
    pub fn add_to_layer(&mut self, layer: Layer, content: NodeContent) -> NodeId;

    /// Reparent an existing node to a layer
    pub fn move_to_layer(&mut self, node_id: NodeId, layer: Layer);

    /// Return node to content layer
    pub fn move_to_content(&mut self, node_id: NodeId);

    /// Get the root NodeId of a layer
    pub fn layer_root(&self, layer: Layer) -> NodeId;
}
```

### Rendering

No rendering changes needed. The renderer already traverses children in order. Layer nodes are ordered children of root, so they naturally z-stack correctly. Child ordering IS the z-order — the Flash/Flex display list model.

### Modal Backdrop

When a Dialog opens in modal mode, it inserts a semi-transparent backdrop node as the first child of the Dialog layer, then the dialog content after it. The backdrop catches clicks to close (or blocks interaction for modal dialogs).

## 3. MD3 Token System

Lives in `material-ui/src/theme/`.

### MaterialTheme

```rust
pub struct MaterialTheme {
    pub color: ColorScheme,
    pub typography: TypographyScale,
    pub shape: ShapeScale,
    pub elevation: ElevationScale,
}

impl MaterialTheme {
    /// Generate full MD3 theme from a single seed color
    pub fn from_seed(seed: Vec4) -> Self;
    pub fn from_seed_light(seed: Vec4) -> Self;
    pub fn from_seed_dark(seed: Vec4) -> Self;

    /// Convert to DesignTokens for widget-core consumption
    pub fn to_design_tokens(&self) -> DesignTokens;
}
```

### ColorScheme

Full MD3 color roles:

```rust
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
```

### Tonal Palette Generation

MD3 generates colors from a seed using HCT (Hue-Chroma-Tone) color space:

1. Extract hue from seed color
2. Generate 5 tonal palettes (primary, secondary, tertiary, neutral, neutral-variant)
3. Pick specific tones for each color role (e.g., primary = tone 40 light / tone 80 dark)

We implement a simplified HCT algorithm or port Google's `material-color-utilities`.

### TypographyScale

MD3's 15-slot type system:

```rust
pub struct TypographyScale {
    pub display_large: TextStyle,   // 57/64
    pub display_medium: TextStyle,  // 45/52
    pub display_small: TextStyle,   // 36/44
    pub headline_large: TextStyle,  // 32/40
    pub headline_medium: TextStyle, // 28/36
    pub headline_small: TextStyle,  // 24/32
    pub title_large: TextStyle,     // 22/28
    pub title_medium: TextStyle,    // 16/24 medium weight
    pub title_small: TextStyle,     // 14/20 medium weight
    pub body_large: TextStyle,      // 16/24
    pub body_medium: TextStyle,     // 14/20
    pub body_small: TextStyle,      // 12/16
    pub label_large: TextStyle,     // 14/20 medium weight
    pub label_medium: TextStyle,    // 12/16 medium weight
    pub label_small: TextStyle,     // 11/16 medium weight
}

pub struct TextStyle {
    pub font_size: f32,
    pub line_height: f32,
    pub font_weight: FontWeight,
    pub letter_spacing: f32,
}

pub enum FontWeight {
    Regular,   // 400
    Medium,    // 500
    Bold,      // 700
}
```

### ShapeScale

Corner radius presets:

```rust
pub struct ShapeScale {
    pub none: f32,         // 0
    pub extra_small: f32,  // 4
    pub small: f32,        // 8
    pub medium: f32,       // 12
    pub large: f32,        // 16
    pub extra_large: f32,  // 28
    pub full: f32,         // 9999 (pill shape)
}
```

### ElevationScale

MD3 tonal elevation (surface tint, not just shadows):

```rust
pub struct ElevationScale {
    pub level0: ElevationLevel, // 0dp
    pub level1: ElevationLevel, // 1dp
    pub level2: ElevationLevel, // 3dp
    pub level3: ElevationLevel, // 6dp
    pub level4: ElevationLevel, // 8dp
    pub level5: ElevationLevel, // 12dp
}

pub struct ElevationLevel {
    pub tint_opacity: f32,
    pub shadow_offset: f32,
}
```

## 4. Widget Inventory

### Batch 1: Inputs (11 widgets)

#### Switch
- **Variants:** Default, with icon
- **State:** Signal<bool> for on/off
- **Behavior:** Thumb slides between positions with animation. Track color changes with state.
- **MD3 spec:** 52x32dp, thumb 24dp (28dp when pressed), track with outline when off

#### Radio + RadioGroup
- **Radio:** Single option with label
- **RadioGroup:** Container managing exclusive selection via Signal<Option<String>>
- **Variants:** Vertical, horizontal layout
- **MD3 spec:** 20dp outer circle, 10dp inner dot when selected

#### Select
- **Variants:** Filled, outlined (matching TextField variants)
- **State:** Signal<Option<String>> for selected value
- **Behavior:** Opens dropdown Menu in Dropdown layer. Clicking item updates signal and closes.
- **MD3 spec:** Matches TextField dimensions, dropdown uses Menu styling

#### Slider
- **Variants:** Continuous, discrete (with steps)
- **State:** Signal<f32> for value (or Signal<(f32, f32)> for range)
- **Behavior:** Thumb drag updates value. Discrete shows tick marks. Optional value label tooltip.
- **MD3 spec:** Track 4dp height, thumb 20dp (28dp active), active/inactive track colors

#### ToggleButton + ToggleButtonGroup
- **ToggleButton:** Single pressable toggle with icon/text
- **ToggleButtonGroup:** Row managing single-select or multi-select via Signal
- **MD3 spec:** Segmented button style, shared border radius between items

#### Rating
- **State:** Signal<f32> for value (0.0-5.0, supports half-stars)
- **Behavior:** Click/hover to set rating. Icons change between filled/outlined.
- **MD3 spec:** Uses filled/outlined star icons, 24dp each with 4dp gap

#### Autocomplete
- **Behavior:** TextInput + filtered dropdown list in Dropdown layer. Types to filter, arrow keys to navigate, enter to select.
- **State:** Signal<String> for input, Vec<String> for options, Signal<Option<String>> for selected
- **Depends on:** TextInput, Menu, MenuItem

#### FAB (Floating Action Button)
- **Variants:** Small (40dp), regular (56dp), large (96dp), extended (with text)
- **Behavior:** Fixed position button, typically bottom-right. Primary action.
- **MD3 spec:** Primary container color, large corner radius, elevation level 3

#### ButtonGroup
- **Behavior:** Row of Buttons sharing border radius (first gets left radius, last gets right, middle gets none)
- **MD3 spec:** Outlined style, 1dp dividers between buttons

### Batch 2: Data Display (7 widgets)

#### Avatar
- **Variants:** Image, letter, icon
- **Behavior:** Circular (or rounded) clip. Letter fallback shows initials on colored background. Icon fallback shows person icon.
- **Sizes:** Small (24dp), medium (40dp), large (56dp)

#### Badge
- **Variants:** Dot, count
- **Behavior:** Positioned overlay (top-right of child widget). Count truncates to "99+" for 3+ digits.
- **State:** Signal<u32> for count (reactive)

#### Chip
- **Variants:** Assist, filter, input, suggestion
- **Behavior:** Compact element with optional leading icon and trailing delete. Filter chips show checkmark when selected.
- **State:** Signal<bool> for filter selection
- **MD3 spec:** 32dp height, small corner radius

#### Table
- **Structure:** Header row + data rows. Column definitions with width/alignment/sort.
- **Behavior:** Click column header to sort. Optional row selection (checkbox column). Striped/hover row styling.
- **State:** Signal<Vec<SortDirection>> for sort, Signal<HashSet<usize>> for selection

#### Tooltip
- **Variants:** Plain (text only), rich (with title/action)
- **Behavior:** Hover-triggered (delay configurable). Renders in Tooltip layer. Positioned relative to anchor.
- **MD3 spec:** Plain: surface-inverse color, body-small text. Rich: surface-container color.

#### Link
- **Behavior:** Styled text with click handler. Underline on hover. Primary color.
- **MD3 spec:** Primary color, body text style, underline on hover/focus

#### ImageList
- **Variants:** Standard (uniform grid), masonry (varying heights), quilted (spanning cells)
- **Behavior:** Grid of Image widgets with optional title bar overlay.

### Batch 3: Feedback (5 widgets)

#### Alert
- **Variants:** Standard, filled, outlined
- **Severity:** Error, warning, info, success (each with icon and color)
- **Behavior:** Icon + message text + optional action button + optional close button
- **MD3 spec:** Uses error/warning color containers, medium corner radius

#### Dialog
- **Variants:** Basic, full-screen
- **Behavior:** Opens in Dialog layer. Modal backdrop blocks interaction. Title + content + action buttons. Esc/backdrop-click to close (configurable).
- **State:** Signal<bool> for open/closed
- **MD3 spec:** Surface-container-highest, extra-large corner radius, scrim backdrop at 0.32 opacity

#### Snackbar
- **Variants:** Default, with action button
- **Behavior:** Timed notification (default 4s) in Notification layer. Queue system — only one visible at a time, next shows after current dismisses.
- **Position:** Bottom-center (configurable)
- **MD3 spec:** Inverse-surface color, body-medium text, medium corner radius

#### Skeleton
- **Variants:** Text (single line), circular, rectangular, rounded
- **Behavior:** Pulsing opacity animation (0.4 → 1.0 → 0.4) on surface color. Dimensions match the content it replaces.

#### Backdrop
- **Behavior:** Full-screen scrim overlay. Configurable opacity. Click handler for dismissal.
- **MD3 spec:** Scrim color at 0.32 opacity

### Batch 4: Navigation (9 widgets)

#### AppBar
- **Variants:** Center-aligned, small, medium, large
- **Behavior:** Top bar with navigation icon (left), title, and action icons (right). Scrolling behavior: fixed or scroll-off.
- **MD3 spec:** Surface color, elevation on scroll, title uses headline/title typography

#### Tabs
- **Variants:** Primary (underline), secondary (underline)
- **Behavior:** Horizontal tab selection. Indicator slides between tabs with animation. Scrollable when many tabs.
- **State:** Signal<usize> for active tab index
- **MD3 spec:** Primary indicator 3dp thick, full-width (primary) or content-width (secondary)

#### Drawer
- **Variants:** Standard (permanent), modal (overlay), bottom (sheet)
- **Behavior:** Side navigation panel. Modal opens in Dialog layer with backdrop. Standard is always visible. Contains navigation items.
- **MD3 spec:** Surface-container-low, 360dp max width, large corner radius (end corners only)

#### Menu + MenuItem
- **Menu:** Container opening in Dropdown layer, positioned relative to anchor
- **MenuItem:** Row with optional leading icon, text, trailing text/icon. Hover highlight.
- **Behavior:** Keyboard navigation (arrow keys, enter, esc). Click outside to close.
- **MD3 spec:** Surface-container, extra-small corner radius, elevation level 2

#### Breadcrumbs
- **Behavior:** Path segments with separator (default "/"). Last item is non-clickable (current page). Optional collapse for long paths (shows "..." with expand).
- **MD3 spec:** Body-small text, on-surface-variant color, primary color on hover

#### Pagination
- **Behavior:** Page number buttons with prev/next arrows. Optional first/last. Ellipsis for large ranges.
- **State:** Signal<usize> for current page

#### Stepper
- **Variants:** Horizontal, vertical
- **Behavior:** Multi-step progress indicator. Steps have states: active, completed, error, disabled. Optional step content.
- **State:** Signal<usize> for active step

#### BottomNavigation
- **Behavior:** Fixed bottom bar with 3-5 items. Icon + optional label. Active item highlighted.
- **State:** Signal<usize> for active item
- **MD3 spec:** Surface-container, 80dp height, active uses indicator pill

### Batch 5: Surfaces (2 widgets)

#### Accordion
- **Behavior:** Header row (always visible) + collapsible content. Click header to expand/collapse. Optional expand icon rotates.
- **State:** Signal<bool> for expanded
- **MD3 spec:** Surface color, divider between items

#### Paper
- **Variants:** Outlined, elevated
- **Behavior:** Basic surface container. Outlined has 1dp border. Elevated uses tonal elevation.
- **MD3 spec:** Surface color, medium corner radius, configurable elevation level

### Batch 6: Material Restyling (8 existing widgets)

#### Material Button
- **Variants:** Filled, tonal, outlined, text, elevated
- **Changes:** Pill shape (full radius), label-large typography, MD3 color roles, state layers (hover/press/focus opacity overlays)
- **MD3 spec:** 40dp height, 24dp horizontal padding, 8dp icon-to-text gap

#### Material TextField (TextInput)
- **Variants:** Filled, outlined
- **Changes:** Floating label animation, supporting/error text below, leading/trailing icons, character counter
- **MD3 spec:** 56dp height, small corner radius (top only for filled), 1dp outline

#### Material Checkbox
- **Changes:** Rounded check container, indeterminate state (dash icon), MD3 colors, error state
- **MD3 spec:** 18dp container, 2dp corner radius, checkmark animation

#### Material Card
- **Variants:** Elevated, filled, outlined
- **Changes:** MD3 elevation system, surface-container colors, medium corner radius
- **MD3 spec:** Medium corner radius, elevation level 1 (elevated variant)

#### Material ProgressBar (LinearProgress)
- **Changes:** Track + active indicator, indeterminate animation (sliding bar), MD3 colors, rounded ends
- **MD3 spec:** 4dp height, full corner radius on indicator

#### Material Icon
- **Changes:** 24dp default size, filled/outlined style toggle, MD3 color roles
- **MD3 spec:** 24dp optical size, on-surface-variant default color

#### Material Divider
- **Variants:** Full-width, inset (16dp left margin), middle-inset (16dp both margins)
- **Changes:** Outline-variant color, 1dp thickness

#### Material Form
- **Changes:** 16dp vertical gap between fields, MD3 error styling, helper text, section headers with headline-small

## 5. Build Sequence & Dependencies

### Dependency Graph

```
Layer System ──────────────────┐
                               ├──→ Dialog, Snackbar, Tooltip, Menu, Select, Drawer
MD3 Theme ─────────────────────┤
                               ├──→ ALL material widgets
Backdrop ──────────────────────┤
                               ├──→ Dialog (modal), Drawer (modal)
MenuItem ──────────────────────┘
                               └──→ Menu, Select, Autocomplete
```

### Phase 1 — Foundation (serial)
1. Layer System in `widget-core`
2. `material-ui` crate scaffold + MD3 Theme (color, typography, shape, elevation)
3. Backdrop

### Phase 2 — Leaf Inputs (parallelizable)
4. Switch
5. Radio + RadioGroup
6. Slider
7. Rating
8. ToggleButton + ToggleButtonGroup
9. FAB
10. ButtonGroup
11. Link

### Phase 3 — Overlay-Dependent Widgets (needs Layer System)
12. Tooltip
13. Menu + MenuItem
14. Select (depends on Menu)
15. Dialog
16. Snackbar
17. Autocomplete (depends on Menu + TextInput)

### Phase 4 — Data Display
18. Avatar
19. Badge
20. Chip
21. Table
22. Skeleton
23. ImageList

### Phase 5 — Navigation
24. AppBar
25. Tabs
26. Drawer
27. Breadcrumbs
28. Pagination
29. Stepper
30. BottomNavigation

### Phase 6 — Surfaces + Restyling
31. Accordion
32. Paper
33. Material Button
34. Material TextField
35. Material Checkbox
36. Material Card
37. Material ProgressBar
38. Material Icon
39. Material Divider
40. Material Form

### Phase 7 — Examples + Polish
41. Material widget gallery example
42. Material dashboard example
43. Material form example
44. Benchmark suite (1,000 instances per widget type)

## 6. Testing Strategy

Every widget gets:

1. **Unit tests** — State logic, variant behavior, signal reactivity (TDD: test first)
2. **Integration test** — `build()` produces correct scene nodes with expected styling
3. **Benchmark** — Instantiation + build of 1,000 instances via criterion
4. **Gallery entry** — Visual example in the material gallery

### Performance Targets

- Widget instantiation: < 1 μs per widget
- 1,000 widget build: < 500 μs
- Layer system overhead: < 100 ns per layer lookup
- Theme token resolution: < 50 ns per lookup
- No regression > 10% on existing widget benchmarks

### Test Organization

```
crates/material-ui/
├── tests/
│   ├── theme_tests.rs          (color generation, token conversion)
│   ├── input_tests.rs          (Switch, Radio, Select, Slider, etc.)
│   ├── data_display_tests.rs   (Avatar, Badge, Chip, Table, etc.)
│   ├── feedback_tests.rs       (Alert, Dialog, Snackbar, etc.)
│   ├── navigation_tests.rs     (AppBar, Tabs, Drawer, etc.)
│   └── integration_tests.rs    (full app scenarios)
├── benches/
│   ├── widget_benchmarks.rs    (per-widget instantiation)
│   └── theme_benchmarks.rs     (token generation/resolution)

crates/widget-core/
├── tests/
│   └── layer_tests.rs          (layer system unit + integration)
├── benches/
│   └── layer_benchmarks.rs     (layer lookup/reparent performance)
```

## 7. Open Questions

1. **HCT color space** — Implement from scratch or find a Rust crate? Google's reference impl is ~2k lines of Dart. A Rust port (`material-colors` crate) may exist.
2. **Ripple/ink effect** — MD3 uses state layers (opacity overlays on hover/press/focus) rather than ink ripples. Simpler to implement but less visually distinctive. Start with state layers, add ripple later?
3. **Text rendering** — Current text rendering has known artifacts (see MEMORY.md). Typography-heavy widgets (Table, Breadcrumbs, Tabs) may be limited until text rendering is fixed.
4. **Animation** — Switch thumb slide, tab indicator movement, dialog open/close, snackbar slide-in all need animation. Timeline system exists in widget-core but needs validation for these use cases.
