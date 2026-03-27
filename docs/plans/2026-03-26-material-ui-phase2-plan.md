# Material UI Phase 2: Leaf Inputs Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build 11 leaf input widgets that don't require the overlay/layer system.

**Architecture:** All widgets live in `crates/material-ui/src/inputs/`. Each implements the `Widget` trait from `widget-core`. Material theming via `ctx.get_extension::<MaterialTheme>()` with fallback defaults.

**Tech Stack:** Rust 2024, widget-core, flux-state (Signal/Effect), render-engine, glam

**Worktree:** `.worktrees/material-ui` on branch `feature/material-ui`

---

## Common Pattern for All Widgets

Every material-ui input widget follows this pattern:

```rust
use widget_core::{Widget, WidgetContext};
use render_engine::node::NodeContent;
use render_engine::scene::NodeId;
use crate::theme::MaterialTheme;

pub struct MyWidget { /* fields */ }

impl MyWidget {
    pub fn new(/* required args */) -> Self { /* ... */ }
    // Builder methods...
}

impl Widget for MyWidget {
    fn build(&self, ctx: &mut WidgetContext) -> NodeId {
        // 1. Get theme (optional, use defaults if not set)
        let theme = ctx.get_extension::<MaterialTheme>().cloned();
        let (colors, shape) = match &theme {
            Some(t) => (/* MD3 colors */, /* MD3 shape */),
            None => (/* fallback colors */, /* fallback shape */),
        };

        // 2. Create root node
        let root = ctx.create_node(ctx.root(), NodeContent::Empty);
        // 3. Set layout
        ctx.set_layout_style(root, FlexStyle { ... });
        // 4. Create children, wire up reactivity
        // 5. Return root
        root
    }
}
```

### Theme Access Pattern

```rust
fn get_theme(ctx: &WidgetContext) -> Option<MaterialTheme> {
    ctx.get_extension::<MaterialTheme>().cloned()
}
```

Widgets should work WITHOUT a theme (use reasonable defaults). Theme is optional enhancement.

---

## Task 1: Create inputs module + Switch widget

**Files:**
- Create: `crates/material-ui/src/inputs/mod.rs`
- Create: `crates/material-ui/src/inputs/switch.rs`
- Modify: `crates/material-ui/src/lib.rs` (add `pub mod inputs;`)

### Switch

MD3 toggle switch. 52×32dp track with 24dp thumb (28dp pressed).

```rust
pub struct Switch {
    read_signal: ReadSignal<bool>,
    write_signal: WriteSignal<bool>,
    disabled: bool,
    label: Option<String>,
}

impl Switch {
    pub fn new(signal: Signal<bool>) -> Self;
    pub fn disabled(mut self, disabled: bool) -> Self;
    pub fn label(mut self, label: impl Into<String>) -> Self;
}
```

**Build behavior:**
- Root: Row container (horizontal, gap=12)
- Track: 52×32 node, corner_radius=full(16), background color based on state (primary when on, surface_variant when off)
- Thumb: 24×24 node inside track, corner_radius=full(12), positioned left/right based on state
- Label: Optional Text child after track
- Click on track toggles the signal
- Reactive Effect watches signal, updates track color and thumb position

**Tests:**
- test_switch_builds_with_track_and_thumb
- test_switch_toggle_changes_state
- test_switch_disabled_not_clickable
- test_switch_with_label
- test_switch_default_off_state

Commit: `feat(material-ui): add Switch input widget`

---

## Task 2: Radio + RadioGroup

**Files:**
- Create: `crates/material-ui/src/inputs/radio.rs`
- Create: `crates/material-ui/src/inputs/radio_group.rs`
- Modify: `crates/material-ui/src/inputs/mod.rs`

### Radio

Single radio button. 20dp outer circle, 10dp inner dot when selected.

```rust
pub struct Radio {
    value: String,
    selected: ReadSignal<Option<String>>,
    on_select: Arc<dyn Fn(String) + Send + Sync>,
    label: Option<String>,
    disabled: bool,
}

impl Radio {
    pub fn new(
        value: impl Into<String>,
        selected: ReadSignal<Option<String>>,
        on_select: impl Fn(String) + Send + Sync + 'static,
    ) -> Self;
    pub fn label(mut self, label: impl Into<String>) -> Self;
    pub fn disabled(mut self, disabled: bool) -> Self;
}
```

### RadioGroup

Container managing exclusive selection.

```rust
pub struct RadioGroup<C> {
    signal: Signal<Option<String>>,
    children_fn: Box<dyn Fn(ReadSignal<Option<String>>, Arc<dyn Fn(String) + Send + Sync>) -> C>,
    direction: FlexDirection,
    gap: f32,
}

// Simpler API:
pub struct RadioGroup {
    options: Vec<RadioOption>,
    signal: Signal<Option<String>>,
    direction: FlexDirection,
    gap: f32,
}

pub struct RadioOption {
    pub value: String,
    pub label: String,
    pub disabled: bool,
}

impl RadioGroup {
    pub fn new(options: Vec<RadioOption>, signal: Signal<Option<String>>) -> Self;
    pub fn horizontal(mut self) -> Self;
    pub fn gap(mut self, gap: f32) -> Self;
}
```

**Build behavior (RadioGroup):**
- Creates a Column (or Row if horizontal) container
- For each option, creates a Radio child with shared signal
- Only one can be selected at a time

**Tests:**
- test_radio_builds_circle
- test_radio_selected_shows_inner_dot
- test_radio_click_triggers_on_select
- test_radio_group_creates_radios_for_options
- test_radio_group_horizontal_layout
- test_radio_group_exclusive_selection

Commit: `feat(material-ui): add Radio and RadioGroup widgets`

---

## Task 3: Slider

**Files:**
- Create: `crates/material-ui/src/inputs/slider.rs`
- Modify: `crates/material-ui/src/inputs/mod.rs`

### Slider

Continuous or discrete value slider. Track 4dp height, thumb 20dp.

```rust
pub struct Slider {
    signal: Signal<f32>,
    min: f32,
    max: f32,
    step: Option<f32>,      // None = continuous, Some(n) = discrete
    disabled: bool,
    width: f32,             // Track width (default 200.0)
}

impl Slider {
    pub fn new(signal: Signal<f32>) -> Self;
    pub fn range(mut self, min: f32, max: f32) -> Self;
    pub fn step(mut self, step: f32) -> Self;
    pub fn disabled(mut self, disabled: bool) -> Self;
    pub fn width(mut self, width: f32) -> Self;
}
```

**Build behavior:**
- Root: Container with fixed width
- Track: Full-width, 4dp height, rounded, surface_variant color
- Active track: Partial width based on value ratio, primary color
- Thumb: 20dp circle at value position, primary color
- Note: Full drag interaction requires pointer events (not yet available). For now, the slider renders correctly and updates via signal, but drag interaction is deferred.

**Tests:**
- test_slider_builds_track_and_thumb
- test_slider_value_positions_thumb
- test_slider_range_clamp
- test_slider_discrete_steps
- test_slider_disabled_styling

Commit: `feat(material-ui): add Slider input widget`

---

## Task 4: Rating

**Files:**
- Create: `crates/material-ui/src/inputs/rating.rs`
- Modify: `crates/material-ui/src/inputs/mod.rs`

### Rating

Star-based rating (1-5). Icons change between filled/outlined.

```rust
pub struct Rating {
    signal: Signal<f32>,
    max: u32,               // Default 5
    disabled: bool,
    size: f32,              // Icon size, default 24.0
}

impl Rating {
    pub fn new(signal: Signal<f32>) -> Self;
    pub fn max(mut self, max: u32) -> Self;
    pub fn disabled(mut self, disabled: bool) -> Self;
    pub fn size(mut self, size: f32) -> Self;
}
```

**Build behavior:**
- Root: Row with gap=4
- For each star (1..=max): Create a star node
  - Filled (primary color) if index <= value
  - Outlined (outline color) if index > value
  - Half-filled support: if value is between index-1 and index
- Each star is clickable, sets signal to that index value
- Reactive: Effect watches signal, updates star fill states

**Tests:**
- test_rating_builds_correct_star_count
- test_rating_default_5_stars
- test_rating_custom_max
- test_rating_click_sets_value
- test_rating_disabled_not_clickable

Commit: `feat(material-ui): add Rating input widget`

---

## Task 5: ToggleButton + ToggleButtonGroup

**Files:**
- Create: `crates/material-ui/src/inputs/toggle_button.rs`
- Create: `crates/material-ui/src/inputs/toggle_button_group.rs`
- Modify: `crates/material-ui/src/inputs/mod.rs`

### ToggleButton

Single pressable toggle with text/icon.

```rust
pub struct ToggleButton {
    text: String,
    selected: bool,
    on_toggle: Option<Arc<dyn Fn(bool) + Send + Sync>>,
    disabled: bool,
}
```

### ToggleButtonGroup

Segmented button group. Single-select or multi-select.

```rust
pub enum SelectionMode {
    Single,
    Multi,
}

pub struct ToggleButtonGroup {
    options: Vec<ToggleOption>,
    signal: Signal<Vec<String>>,  // Selected values
    mode: SelectionMode,
    gap: f32,
}

pub struct ToggleOption {
    pub value: String,
    pub label: String,
    pub disabled: bool,
}
```

**Build behavior (Group):**
- Row container with shared border radius (first gets left, last gets right)
- Each button styled as outlined MD3 segment
- Single mode: clicking one deselects others
- Multi mode: clicking toggles independently
- 1dp dividers between buttons

**Tests:**
- test_toggle_button_renders
- test_toggle_button_group_single_select
- test_toggle_button_group_multi_select
- test_toggle_button_group_shared_borders
- test_toggle_button_disabled

Commit: `feat(material-ui): add ToggleButton and ToggleButtonGroup widgets`

---

## Task 6: FAB (Floating Action Button)

**Files:**
- Create: `crates/material-ui/src/inputs/fab.rs`
- Modify: `crates/material-ui/src/inputs/mod.rs`

### FAB

```rust
pub enum FabSize {
    Small,    // 40dp
    Regular,  // 56dp
    Large,    // 96dp
}

pub struct Fab {
    icon: String,           // Icon name
    label: Option<String>,  // Extended FAB text
    size: FabSize,
    on_click: Option<Arc<dyn Fn() + Send + Sync>>,
    disabled: bool,
}

impl Fab {
    pub fn new(icon: impl Into<String>) -> Self;
    pub fn label(mut self, label: impl Into<String>) -> Self;  // Makes it extended
    pub fn small(mut self) -> Self;
    pub fn large(mut self) -> Self;
    pub fn on_click(mut self, f: impl Fn() + Send + Sync + 'static) -> Self;
    pub fn disabled(mut self, disabled: bool) -> Self;
}
```

**Build behavior:**
- Primary container color, large corner radius (16dp regular, 12dp small, 28dp large)
- Icon centered (or left of label for extended)
- Elevation level 3
- Extended: Row with icon + label text

**Tests:**
- test_fab_default_regular_size
- test_fab_small_size
- test_fab_large_size
- test_fab_extended_with_label
- test_fab_on_click
- test_fab_disabled

Commit: `feat(material-ui): add FAB (Floating Action Button) widget`

---

## Task 7: ButtonGroup + Link

**Files:**
- Create: `crates/material-ui/src/inputs/button_group.rs`
- Create: `crates/material-ui/src/inputs/link.rs`
- Modify: `crates/material-ui/src/inputs/mod.rs`

### ButtonGroup

Row of Buttons sharing border radius.

```rust
pub struct ButtonGroup {
    labels: Vec<String>,
    on_click: Option<Arc<dyn Fn(usize) + Send + Sync>>,
    disabled: bool,
}

impl ButtonGroup {
    pub fn new(labels: Vec<String>) -> Self;
    pub fn on_click(mut self, f: impl Fn(usize) + Send + Sync + 'static) -> Self;
    pub fn disabled(mut self, disabled: bool) -> Self;
}
```

### Link

Styled clickable text.

```rust
pub struct Link {
    text: String,
    on_click: Option<Arc<dyn Fn() + Send + Sync>>,
    disabled: bool,
    font_size: f32,
}

impl Link {
    pub fn new(text: impl Into<String>) -> Self;
    pub fn on_click(mut self, f: impl Fn() + Send + Sync + 'static) -> Self;
    pub fn disabled(mut self, disabled: bool) -> Self;
    pub fn font_size(mut self, size: f32) -> Self;
}
```

**Build behavior:**
- ButtonGroup: Row with shared border radius (first left, last right, middle none), outlined style, 1dp dividers
- Link: Text node in primary color, underline on hover

**Tests:**
- test_button_group_creates_buttons
- test_button_group_shared_borders
- test_button_group_click_index
- test_link_primary_color
- test_link_on_click
- test_link_disabled

Commit: `feat(material-ui): add ButtonGroup and Link widgets`

---

## Task 8: Final polish + module exports

**Steps:**
1. Ensure all widgets are re-exported from `crates/material-ui/src/inputs/mod.rs`
2. Re-export from `crates/material-ui/src/lib.rs`
3. `cargo fmt --all`
4. `cargo clippy -p material-ui --all-targets -- -D warnings`
5. `cargo test -p material-ui`
6. Commit any fixes

---

## Summary

| Task | Widget(s) | Key State |
|------|-----------|-----------|
| 1 | Switch | Signal<bool> |
| 2 | Radio + RadioGroup | Signal<Option<String>> |
| 3 | Slider | Signal<f32> |
| 4 | Rating | Signal<f32> |
| 5 | ToggleButton + ToggleButtonGroup | Signal<Vec<String>> |
| 6 | FAB | Click handler |
| 7 | ButtonGroup + Link | Click handlers |
| 8 | Polish + exports | — |
