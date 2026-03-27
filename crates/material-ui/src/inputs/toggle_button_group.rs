//! MD3 ToggleButtonGroup widget -- segmented button group with single or multi-select.

use crate::inputs::toggle_button::ToggleButton;
use flux_state::Signal;
use layout_engine::{FlexDirection, FlexStyle};
use render_engine::node::NodeContent;
use render_engine::{CornerRadii, NodeId};
use widget_core::WidgetContext;
use widget_core::widget_trait::Widget;

/// Selection mode for a [`ToggleButtonGroup`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectionMode {
    /// Only one option can be selected at a time.
    Single,
    /// Multiple options can be selected simultaneously.
    Multi,
}

/// A single option within a [`ToggleButtonGroup`].
///
/// Each option has a string value (used as the selection key), a human-readable
/// label, and an optional disabled flag.
pub struct ToggleOption {
    /// The value this option represents when selected.
    pub value: String,
    /// The label displayed on the toggle button segment.
    pub label: String,
    /// Whether this option is disabled (not clickable).
    pub disabled: bool,
}

impl ToggleOption {
    /// Create a new enabled toggle option.
    pub fn new(value: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
            disabled: false,
        }
    }

    /// Create a new disabled toggle option.
    pub fn disabled(value: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
            disabled: true,
        }
    }
}

/// MD3 ToggleButtonGroup -- segmented button group.
///
/// Wraps a set of [`ToggleOption`]s in a horizontal Row layout. All segments
/// share a single `Signal<Vec<String>>` that holds the currently selected
/// values. Supports both single-select and multi-select modes.
///
/// # Example
///
/// ```rust,no_run
/// use flux_state::{Runtime, Signal};
/// use material_ui::inputs::{ToggleButtonGroup, ToggleOption};
///
/// let runtime = Runtime::new();
/// let selected = Signal::new(runtime, vec![]);
///
/// let group = ToggleButtonGroup::new(
///     vec![
///         ToggleOption::new("bold", "Bold"),
///         ToggleOption::new("italic", "Italic"),
///         ToggleOption::new("underline", "Underline"),
///     ],
///     selected,
/// )
/// .multi()
/// .gap(0.0);
/// ```
pub struct ToggleButtonGroup {
    options: Vec<ToggleOption>,
    signal: Signal<Vec<String>>,
    mode: SelectionMode,
    gap: f32,
}

/// Default corner radius for first/last segment buttons.
const SEGMENT_CORNER_RADIUS: f32 = 12.0;

impl ToggleButtonGroup {
    /// Create a new toggle button group.
    ///
    /// # Arguments
    ///
    /// * `options` - The set of toggle options to display.
    /// * `signal` - A shared signal holding the currently selected values.
    pub fn new(options: Vec<ToggleOption>, signal: Signal<Vec<String>>) -> Self {
        Self {
            options,
            signal,
            mode: SelectionMode::Single,
            gap: 0.0,
        }
    }

    /// Set single-select mode (default). Clicking an option replaces the
    /// selection with just that value.
    pub fn single(mut self) -> Self {
        self.mode = SelectionMode::Single;
        self
    }

    /// Set multi-select mode. Clicking an option toggles it in/out of the
    /// selection set.
    pub fn multi(mut self) -> Self {
        self.mode = SelectionMode::Multi;
        self
    }

    /// Set the gap between segments (default: 0.0 for continuous segmented look).
    pub fn gap(mut self, gap: f32) -> Self {
        self.gap = gap;
        self
    }

    /// Compute per-corner radii for a segment at the given position.
    fn segment_radii(index: usize, total: usize) -> CornerRadii {
        if total <= 1 {
            // Single segment gets full radii
            CornerRadii::uniform(SEGMENT_CORNER_RADIUS)
        } else if index == 0 {
            // First segment: left corners rounded
            CornerRadii::new(SEGMENT_CORNER_RADIUS, 0.0, 0.0, SEGMENT_CORNER_RADIUS)
        } else if index == total - 1 {
            // Last segment: right corners rounded
            CornerRadii::new(0.0, SEGMENT_CORNER_RADIUS, SEGMENT_CORNER_RADIUS, 0.0)
        } else {
            // Middle segments: no rounding
            CornerRadii::uniform(0.0)
        }
    }
}

impl Widget for ToggleButtonGroup {
    fn build(&self, ctx: &mut WidgetContext) -> NodeId {
        let (read, write) = self.signal.clone().split();
        let total = self.options.len();

        // Root container (horizontal row)
        let root = ctx.create_node(ctx.root(), NodeContent::Empty);
        ctx.set_layout_style(
            root,
            FlexStyle {
                direction: FlexDirection::Row,
                gap: self.gap,
                ..Default::default()
            },
        );

        // Create a toggle button segment for each option
        for (i, opt) in self.options.iter().enumerate() {
            let current_values = read.get_untracked();
            let is_selected = current_values.contains(&opt.value);

            let mode = self.mode;
            let value = opt.value.clone();
            let w = write.clone();
            let r = read.clone();

            let mut toggle = ToggleButton::new(opt.label.clone())
                .selected(is_selected)
                .disabled(opt.disabled)
                .corner_radii(Self::segment_radii(i, total));

            if !opt.disabled {
                toggle = toggle.on_toggle(move |_new_state| match mode {
                    SelectionMode::Single => {
                        w.set(vec![value.clone()]);
                    }
                    SelectionMode::Multi => {
                        let mut current = r.get_untracked();
                        if let Some(pos) = current.iter().position(|v| *v == value) {
                            current.remove(pos);
                        } else {
                            current.push(value.clone());
                        }
                        w.set(current);
                    }
                });
            }

            let segment_id = toggle.build(ctx);
            ctx.reparent_to(segment_id, root);
        }

        root
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use flux_state::Runtime;

    #[test]
    fn test_toggle_button_group_creates_segments() {
        let runtime = Runtime::new();
        let selected = Signal::new(runtime, vec![]);

        let mut ctx = WidgetContext::new_test();
        let group = ToggleButtonGroup::new(
            vec![
                ToggleOption::new("a", "A"),
                ToggleOption::new("b", "B"),
                ToggleOption::new("c", "C"),
            ],
            selected,
        );
        let root_id = group.build(&mut ctx);

        let scene = ctx.scene();
        let root_node = scene.get_node(root_id).unwrap();

        assert_eq!(
            root_node.children.len(),
            3,
            "ToggleButtonGroup with 3 options should have 3 children, got {}",
            root_node.children.len()
        );
    }

    #[test]
    fn test_toggle_button_group_single_select() {
        let runtime = Runtime::new();
        let selected = Signal::new(runtime, vec![]);
        let observer = selected.clone();
        let (obs_read, _) = observer.split();

        let mut ctx = WidgetContext::new_test();
        let group = ToggleButtonGroup::new(
            vec![
                ToggleOption::new("first", "First"),
                ToggleOption::new("second", "Second"),
            ],
            selected,
        )
        .single();
        let root_id = group.build(&mut ctx);

        // Initially empty
        assert!(
            obs_read.get_untracked().is_empty(),
            "Should start with no selection"
        );

        // Click the first toggle segment
        let first_id = ctx.scene().get_node(root_id).unwrap().children[0];
        ctx.trigger_click(first_id);

        assert_eq!(
            obs_read.get_untracked(),
            vec!["first".to_string()],
            "After clicking first, signal should hold ['first']"
        );

        // Click the second toggle segment
        let second_id = ctx.scene().get_node(root_id).unwrap().children[1];
        ctx.trigger_click(second_id);

        assert_eq!(
            obs_read.get_untracked(),
            vec!["second".to_string()],
            "After clicking second, signal should hold only ['second'] (single-select)"
        );
    }

    #[test]
    fn test_toggle_button_group_multi_select() {
        let runtime = Runtime::new();
        let selected = Signal::new(runtime, vec![]);
        let observer = selected.clone();
        let (obs_read, _) = observer.split();

        let mut ctx = WidgetContext::new_test();
        let group = ToggleButtonGroup::new(
            vec![
                ToggleOption::new("bold", "Bold"),
                ToggleOption::new("italic", "Italic"),
                ToggleOption::new("underline", "Underline"),
            ],
            selected,
        )
        .multi();
        let root_id = group.build(&mut ctx);

        // Click the first (Bold)
        let bold_id = ctx.scene().get_node(root_id).unwrap().children[0];
        ctx.trigger_click(bold_id);

        let values = obs_read.get_untracked();
        assert!(
            values.contains(&"bold".to_string()),
            "Bold should be selected"
        );

        // Click the second (Italic)
        let italic_id = ctx.scene().get_node(root_id).unwrap().children[1];
        ctx.trigger_click(italic_id);

        let values = obs_read.get_untracked();
        assert!(
            values.contains(&"bold".to_string()) && values.contains(&"italic".to_string()),
            "Both bold and italic should be selected, got {values:?}"
        );
    }

    #[test]
    fn test_toggle_button_group_multi_deselect() {
        let runtime = Runtime::new();
        let selected = Signal::new(runtime, vec!["bold".to_string()]);
        let observer = selected.clone();
        let (obs_read, _) = observer.split();

        let mut ctx = WidgetContext::new_test();
        let group = ToggleButtonGroup::new(
            vec![
                ToggleOption::new("bold", "Bold"),
                ToggleOption::new("italic", "Italic"),
            ],
            selected,
        )
        .multi();
        let root_id = group.build(&mut ctx);

        // Bold is pre-selected. Click it to deselect.
        let bold_id = ctx.scene().get_node(root_id).unwrap().children[0];
        ctx.trigger_click(bold_id);

        let values = obs_read.get_untracked();
        assert!(
            !values.contains(&"bold".to_string()),
            "Bold should be deselected after clicking again, got {values:?}"
        );
    }

    #[test]
    fn test_toggle_button_group_disabled_option() {
        let runtime = Runtime::new();
        let selected = Signal::new(runtime, vec![]);
        let observer = selected.clone();
        let (obs_read, _) = observer.split();

        let mut ctx = WidgetContext::new_test();
        let group = ToggleButtonGroup::new(
            vec![
                ToggleOption::new("enabled", "Enabled"),
                ToggleOption::disabled("disabled_val", "Disabled"),
            ],
            selected,
        );
        let root_id = group.build(&mut ctx);

        // The disabled segment (second child) should not be clickable
        let disabled_id = ctx.scene().get_node(root_id).unwrap().children[1];
        assert!(
            !ctx.has_clickable(disabled_id),
            "Disabled toggle option should not be clickable"
        );

        // Clicking disabled segment should not change selection
        ctx.trigger_click(disabled_id);
        assert!(
            obs_read.get_untracked().is_empty(),
            "Signal should remain empty after clicking disabled option"
        );
    }

    #[test]
    fn test_toggle_button_group_segment_corner_radii() {
        // First segment: left corners, Last: right corners, Middle: none
        let first = ToggleButtonGroup::segment_radii(0, 3);
        assert!(
            first.top_left > 0.0 && first.bottom_left > 0.0,
            "First segment should have left corner radii"
        );
        assert!(
            first.top_right < 0.01 && first.bottom_right < 0.01,
            "First segment should have no right corner radii"
        );

        let middle = ToggleButtonGroup::segment_radii(1, 3);
        assert!(
            middle.top_left < 0.01
                && middle.top_right < 0.01
                && middle.bottom_left < 0.01
                && middle.bottom_right < 0.01,
            "Middle segment should have no corner radii"
        );

        let last = ToggleButtonGroup::segment_radii(2, 3);
        assert!(
            last.top_right > 0.0 && last.bottom_right > 0.0,
            "Last segment should have right corner radii"
        );
        assert!(
            last.top_left < 0.01 && last.bottom_left < 0.01,
            "Last segment should have no left corner radii"
        );
    }
}
