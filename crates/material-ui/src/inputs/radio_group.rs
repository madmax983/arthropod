//! MD3 RadioGroup widget -- container managing exclusive selection across Radio children.

use crate::inputs::radio::Radio;
use flux_state::Signal;
use layout_engine::{FlexDirection, FlexStyle};
use render_engine::NodeId;
use render_engine::node::NodeContent;
use std::sync::Arc;
use widget_core::WidgetContext;
use widget_core::widget_trait::Widget;

/// A single option within a [`RadioGroup`].
///
/// Each option has a string value (used as the selection key), a human-readable
/// label, and an optional disabled flag.
pub struct RadioOption {
    /// The value this option represents when selected.
    pub value: String,
    /// The label displayed next to the radio circle.
    pub label: String,
    /// Whether this option is disabled (not clickable).
    pub disabled: bool,
}

impl RadioOption {
    /// Create a new enabled radio option.
    pub fn new(value: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
            disabled: false,
        }
    }

    /// Create a new disabled radio option.
    pub fn disabled(value: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
            disabled: true,
        }
    }
}

/// MD3 RadioGroup -- manages exclusive selection across multiple Radio children.
///
/// Wraps a set of [`RadioOption`]s in a Column (or Row) layout. All radios
/// share a single `Signal<Option<String>>`, so selecting one automatically
/// deselects the previous choice.
///
/// # Example
///
/// ```rust,no_run
/// use flux_state::{Runtime, Signal};
/// use material_ui::inputs::{RadioGroup, RadioOption};
///
/// let runtime = Runtime::new();
/// let selected = Signal::new(runtime, None::<String>);
///
/// let group = RadioGroup::new(
///     vec![
///         RadioOption::new("a", "Option A"),
///         RadioOption::new("b", "Option B"),
///         RadioOption::disabled("c", "Option C (disabled)"),
///     ],
///     selected,
/// )
/// .horizontal()
/// .gap(12.0);
/// ```
pub struct RadioGroup {
    options: Vec<RadioOption>,
    signal: Signal<Option<String>>,
    direction: FlexDirection,
    gap: f32,
}

impl RadioGroup {
    /// Create a new radio group.
    ///
    /// # Arguments
    ///
    /// * `options` - The set of radio options to display.
    /// * `signal` - A shared signal holding the currently selected value (or `None`).
    pub fn new(options: Vec<RadioOption>, signal: Signal<Option<String>>) -> Self {
        Self {
            options,
            signal,
            direction: FlexDirection::Column,
            gap: 8.0,
        }
    }

    /// Arrange radio options horizontally (Row) instead of the default Column.
    pub fn horizontal(mut self) -> Self {
        self.direction = FlexDirection::Row;
        self
    }

    /// Set the gap between radio options (default: 8.0).
    pub fn gap(mut self, gap: f32) -> Self {
        self.gap = gap;
        self
    }
}

impl Widget for RadioGroup {
    fn build(&self, ctx: &mut WidgetContext) -> NodeId {
        let (read, write) = self.signal.clone().split();

        // Root container (Column or Row depending on direction)
        let root = ctx.create_node(ctx.root(), NodeContent::Empty);
        ctx.set_layout_style(
            root,
            FlexStyle {
                direction: self.direction,
                gap: self.gap,
                ..Default::default()
            },
        );

        // Create a Radio child for each option
        for opt in &self.options {
            let w = write.clone();
            let callback = Arc::new(move |val: String| w.set(Some(val)));

            let radio = Radio::new(opt.value.clone(), read.clone(), {
                let cb = Arc::clone(&callback);
                move |val| cb(val)
            })
            .label(opt.label.clone())
            .disabled(opt.disabled);

            let radio_id = radio.build(ctx);
            ctx.reparent_to(radio_id, root);
        }

        root
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use flux_state::Runtime;

    #[test]
    fn test_radio_group_creates_radios_for_options() {
        let runtime = Runtime::new();
        let selected = Signal::new(runtime, None::<String>);

        let mut ctx = WidgetContext::new_test();
        let group = RadioGroup::new(
            vec![
                RadioOption::new("a", "Option A"),
                RadioOption::new("b", "Option B"),
                RadioOption::new("c", "Option C"),
            ],
            selected,
        );
        let root_id = group.build(&mut ctx);

        let scene = ctx.scene();
        let root_node = scene.get_node(root_id).unwrap();

        assert_eq!(
            root_node.children.len(),
            3,
            "RadioGroup with 3 options should have 3 children, got {}",
            root_node.children.len()
        );
    }

    #[test]
    fn test_radio_group_horizontal_layout() {
        let runtime = Runtime::new();
        let selected = Signal::new(runtime, None::<String>);

        let mut ctx = WidgetContext::new_test();
        let group = RadioGroup::new(
            vec![RadioOption::new("x", "X"), RadioOption::new("y", "Y")],
            selected,
        )
        .horizontal();
        let root_id = group.build(&mut ctx);

        // Verify it builds successfully with children
        let scene = ctx.scene();
        let root_node = scene.get_node(root_id).unwrap();
        assert_eq!(
            root_node.children.len(),
            2,
            "Horizontal group should still have 2 children"
        );
    }

    #[test]
    fn test_radio_group_exclusive_selection() {
        let runtime = Runtime::new();
        let selected = Signal::new(runtime, None::<String>);
        let observer = selected.clone();
        let (obs_read, _) = observer.split();

        let mut ctx = WidgetContext::new_test();
        let group = RadioGroup::new(
            vec![
                RadioOption::new("first", "First"),
                RadioOption::new("second", "Second"),
            ],
            selected,
        );
        let root_id = group.build(&mut ctx);

        // Initially nothing selected
        assert_eq!(
            obs_read.get_untracked(),
            None,
            "Should start with no selection"
        );

        // Click the first radio (first child of root)
        let first_radio_id = ctx.scene().get_node(root_id).unwrap().children[0];
        ctx.trigger_click(first_radio_id);

        assert_eq!(
            obs_read.get_untracked(),
            Some("first".to_string()),
            "After clicking first radio, signal should hold 'first'"
        );

        // Click the second radio (second child of root)
        let second_radio_id = ctx.scene().get_node(root_id).unwrap().children[1];
        ctx.trigger_click(second_radio_id);

        assert_eq!(
            obs_read.get_untracked(),
            Some("second".to_string()),
            "After clicking second radio, signal should hold 'second'"
        );
    }

    #[test]
    fn test_radio_group_custom_gap() {
        let runtime = Runtime::new();
        let selected = Signal::new(runtime, None::<String>);

        let mut ctx = WidgetContext::new_test();
        let group = RadioGroup::new(
            vec![RadioOption::new("a", "A"), RadioOption::new("b", "B")],
            selected,
        )
        .gap(16.0);
        let root_id = group.build(&mut ctx);

        // Verify it builds successfully
        assert!(
            ctx.scene().get_node(root_id).is_some(),
            "RadioGroup with custom gap should build successfully"
        );
    }

    #[test]
    fn test_radio_group_with_disabled_option() {
        let runtime = Runtime::new();
        let selected = Signal::new(runtime, None::<String>);
        let observer = selected.clone();
        let (obs_read, _) = observer.split();

        let mut ctx = WidgetContext::new_test();
        let group = RadioGroup::new(
            vec![
                RadioOption::new("enabled", "Enabled"),
                RadioOption::disabled("disabled_val", "Disabled"),
            ],
            selected,
        );
        let root_id = group.build(&mut ctx);

        // The disabled radio (second child) should not be clickable
        let disabled_radio_id = ctx.scene().get_node(root_id).unwrap().children[1];

        assert!(
            !ctx.has_clickable(disabled_radio_id),
            "Disabled radio option should not be clickable"
        );

        // Clicking the disabled radio should not change selection
        ctx.trigger_click(disabled_radio_id);
        assert_eq!(
            obs_read.get_untracked(),
            None,
            "Signal should remain None after clicking disabled radio"
        );
    }
}
