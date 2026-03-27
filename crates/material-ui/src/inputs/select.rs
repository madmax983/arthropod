//! MD3 Select widget -- dropdown select input with overlay menu.

use crate::theme::MaterialTheme;
use flux_state::{Effect, ReadSignal, Signal, WriteSignal};
use glam::Vec4;
use layout_engine::{FlexAlign, FlexDirection, FlexStyle};
use render_engine::node::NodeContent;
use render_engine::{Color, NodeId, TextContent, VisualStyle};
use std::sync::Arc;
use widget_core::WidgetContext;
use widget_core::layer::Layer;
use widget_core::widget_trait::Widget;

// ---------------------------------------------------------------------------
// MD3 fallback colors (used when no MaterialTheme is provided)
// ---------------------------------------------------------------------------

/// MD3 outline (#79747E) -- trigger border.
const FALLBACK_OUTLINE: Vec4 = Vec4::new(0.475, 0.455, 0.494, 1.0);

/// MD3 on_surface (#1D1B20) -- selected label text.
const FALLBACK_ON_SURFACE: Vec4 = Vec4::new(0.114, 0.106, 0.125, 1.0);

/// MD3 surface (#FEF7FF) -- dropdown background.
const FALLBACK_SURFACE: Vec4 = Vec4::new(0.996, 0.969, 1.0, 1.0);

/// MD3 secondary_container (#E8DEF8) -- selected option highlight.
const FALLBACK_SECONDARY_CONTAINER: Vec4 = Vec4::new(0.906, 0.831, 0.996, 1.0);

/// MD3 on_surface_variant (#49454F) -- placeholder text.
const FALLBACK_ON_SURFACE_VARIANT: Vec4 = Vec4::new(0.286, 0.271, 0.310, 1.0);

/// Trigger height in dp.
const TRIGGER_HEIGHT: f32 = 56.0;

/// Corner radius in dp.
const CORNER_RADIUS: f32 = 4.0;

/// Horizontal padding in dp.
const HORIZONTAL_PADDING: f32 = 16.0;

/// Option row height in dp.
const OPTION_HEIGHT: f32 = 48.0;

/// Option horizontal padding in dp.
const OPTION_PADDING: f32 = 12.0;

/// Default select width in dp.
const DEFAULT_WIDTH: f32 = 200.0;

/// Label font size in dp.
const LABEL_FONT_SIZE: f32 = 14.0;

/// A single option within a [`Select`] dropdown.
///
/// # Example
///
/// ```rust,no_run
/// use material_ui::inputs::SelectOption;
///
/// let option = SelectOption::new("us", "United States");
/// let disabled = SelectOption::new("xx", "Unavailable").disabled();
/// ```
pub struct SelectOption {
    /// The value submitted when this option is selected.
    pub value: String,
    /// The display label shown in the dropdown list.
    pub label: String,
    /// Whether this option is disabled (non-selectable).
    pub disabled: bool,
}

impl SelectOption {
    /// Create a new selectable option.
    pub fn new(value: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
            disabled: false,
        }
    }

    /// Mark this option as disabled (not clickable).
    pub fn disabled(mut self) -> Self {
        self.disabled = true;
        self
    }
}

/// MD3 Select -- dropdown select input.
///
/// Renders a trigger field (outlined container) in the content layer and a
/// dropdown menu in the [`Layer::Dropdown`] overlay layer. Clicking an option
/// sets the signal to `Some(value)`.
///
/// # Example
///
/// ```rust,no_run
/// use flux_state::{Runtime, Signal};
/// use material_ui::inputs::{Select, SelectOption};
///
/// let runtime = Runtime::new();
/// let selected = Signal::new(runtime, None::<String>);
///
/// let select = Select::new(
///     vec![
///         SelectOption::new("a", "Alpha"),
///         SelectOption::new("b", "Beta"),
///         SelectOption::new("c", "Gamma"),
///     ],
///     selected,
/// )
/// .placeholder("Choose one");
/// ```
pub struct Select {
    options: Vec<SelectOption>,
    read_signal: ReadSignal<Option<String>>,
    write_signal: WriteSignal<Option<String>>,
    placeholder: String,
    disabled: bool,
    width: f32,
}

impl Select {
    /// Create a new select dropdown wired to a reactive `Option<String>` signal.
    ///
    /// The signal is split internally into read/write halves. Selecting an
    /// option writes `Some(value)` to the signal.
    pub fn new(options: Vec<SelectOption>, signal: Signal<Option<String>>) -> Self {
        let (read_signal, write_signal) = signal.split();
        Self {
            options,
            read_signal,
            write_signal,
            placeholder: String::new(),
            disabled: false,
            width: DEFAULT_WIDTH,
        }
    }

    /// Set placeholder text shown when no option is selected.
    pub fn placeholder(mut self, text: impl Into<String>) -> Self {
        self.placeholder = text.into();
        self
    }

    /// Disable interaction. The trigger is not clickable and options cannot be
    /// selected.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Override the trigger width in dp (default: 200.0).
    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }
}

impl Widget for Select {
    fn build(&self, ctx: &mut WidgetContext) -> NodeId {
        let runtime = self.read_signal.runtime().clone();
        let read = self.read_signal.clone();
        let write = self.write_signal.clone();

        // -- Resolve theme colors --
        let theme = ctx.get_extension::<MaterialTheme>().cloned();
        let outline_color = theme
            .as_ref()
            .map(|t| t.color.outline)
            .unwrap_or(FALLBACK_OUTLINE);
        let on_surface = theme
            .as_ref()
            .map(|t| t.color.on_surface)
            .unwrap_or(FALLBACK_ON_SURFACE);
        let surface = theme
            .as_ref()
            .map(|t| t.color.surface)
            .unwrap_or(FALLBACK_SURFACE);
        let secondary_container = theme
            .as_ref()
            .map(|t| t.color.secondary_container)
            .unwrap_or(FALLBACK_SECONDARY_CONTAINER);
        let on_surface_variant = theme
            .as_ref()
            .map(|t| t.color.on_surface_variant)
            .unwrap_or(FALLBACK_ON_SURFACE_VARIANT);

        // -- Determine initial display text --
        let current_value = read.get_untracked();
        let initial_text = current_value
            .as_ref()
            .and_then(|v| self.options.iter().find(|o| &o.value == v))
            .map(|o| o.label.clone())
            .unwrap_or_else(|| self.placeholder.clone());
        let is_placeholder = current_value.is_none();

        // =====================================================================
        // Trigger (Content layer)
        // =====================================================================

        // Outlined container
        let trigger = ctx.create_node(
            ctx.root(),
            NodeContent::Styled {
                style: Box::new(
                    VisualStyle::new()
                        .solid_fill(outline_color)
                        .corner_radius(CORNER_RADIUS),
                ),
            },
        );
        ctx.set_layout_style(
            trigger,
            FlexStyle {
                direction: FlexDirection::Row,
                width: Some(self.width),
                height: Some(TRIGGER_HEIGHT),
                padding_left: HORIZONTAL_PADDING,
                padding_right: HORIZONTAL_PADDING,
                align_items: FlexAlign::Center,
                ..Default::default()
            },
        );

        // Trigger label text
        let text_color = if is_placeholder {
            on_surface_variant
        } else {
            on_surface
        };

        let label_style = VisualStyle::new()
            .solid_fill(text_color)
            .text(TextContent::new(initial_text.clone(), LABEL_FONT_SIZE));

        let label_node = ctx.create_node(
            trigger,
            NodeContent::Styled {
                style: Box::new(label_style),
            },
        );
        ctx.set_layout_style(
            label_node,
            FlexStyle {
                flex_grow: 1.0,
                ..Default::default()
            },
        );

        // -- Reactive effect: update trigger label when signal changes --
        let read_for_label = read.clone();
        let options_for_label: Vec<(String, String)> = self
            .options
            .iter()
            .map(|o| (o.value.clone(), o.label.clone()))
            .collect();
        let placeholder_for_label = self.placeholder.clone();

        let label_color_signal = Signal::new(
            runtime.clone(),
            Color::rgba(text_color.x, text_color.y, text_color.z, text_color.w),
        );
        let (label_color_read, label_color_write) = label_color_signal.split();

        let effect = Effect::new(runtime.clone(), move || {
            let current = read_for_label.get();
            let _display = current
                .as_ref()
                .and_then(|v| {
                    options_for_label
                        .iter()
                        .find(|(val, _)| val == v)
                        .map(|(_, lbl)| lbl.clone())
                })
                .unwrap_or_else(|| placeholder_for_label.clone());
            let color = if current.is_none() {
                on_surface_variant
            } else {
                on_surface
            };
            label_color_write.set(Color::rgba(color.x, color.y, color.z, color.w));
        });
        ctx.store_effect(effect);
        ctx.add_reactive_color_state(label_node, label_color_read);

        // =====================================================================
        // Dropdown (Dropdown layer)
        // =====================================================================

        let dropdown_style = VisualStyle::new()
            .solid_fill(surface)
            .corner_radius(CORNER_RADIUS);

        let dropdown = ctx.add_to_layer(
            Layer::Dropdown,
            NodeContent::Styled {
                style: Box::new(dropdown_style),
            },
        );

        ctx.set_layout_style(
            dropdown,
            FlexStyle {
                direction: FlexDirection::Column,
                width: Some(self.width),
                ..Default::default()
            },
        );

        // -- Build option rows --
        for option in &self.options {
            let is_selected = read
                .get_untracked()
                .as_ref()
                .is_some_and(|v| v == &option.value);

            let bg_color = if is_selected {
                secondary_container
            } else {
                surface
            };

            let row_style = VisualStyle::new().solid_fill(bg_color).corner_radius(0.0);

            let row = ctx.create_node(
                dropdown,
                NodeContent::Styled {
                    style: Box::new(row_style),
                },
            );

            ctx.set_layout_style(
                row,
                FlexStyle {
                    direction: FlexDirection::Row,
                    height: Some(OPTION_HEIGHT),
                    padding_left: OPTION_PADDING,
                    padding_right: OPTION_PADDING,
                    align_items: FlexAlign::Center,
                    ..Default::default()
                },
            );

            // Option label text
            let option_text_style = VisualStyle::new()
                .solid_fill(on_surface)
                .text(TextContent::new(option.label.clone(), LABEL_FONT_SIZE));

            let option_label = ctx.create_node(
                row,
                NodeContent::Styled {
                    style: Box::new(option_text_style),
                },
            );
            ctx.set_layout_style(
                option_label,
                FlexStyle {
                    flex_grow: 1.0,
                    ..Default::default()
                },
            );

            // Click handler (unless disabled or select is disabled)
            if !option.disabled && !self.disabled {
                let write_clone = write.clone();
                let value = option.value.clone();
                ctx.add_clickable(
                    row,
                    Arc::new(move || {
                        write_clone.set(Some(value.clone()));
                    }),
                );
            }
        }

        // -- Trigger click handler (make trigger clickable unless disabled) --
        if !self.disabled {
            // In a full implementation this would toggle dropdown visibility.
            // For now the trigger is clickable as a signal that it is interactive.
            ctx.add_clickable(trigger, Arc::new(|| {}));
        }

        trigger
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use flux_state::Runtime;

    fn make_test_options() -> Vec<SelectOption> {
        vec![
            SelectOption::new("a", "Alpha"),
            SelectOption::new("b", "Beta"),
            SelectOption::new("c", "Gamma"),
        ]
    }

    #[test]
    fn test_select_builds_trigger() {
        let runtime = Runtime::new();
        let signal = Signal::new(runtime, None::<String>);

        let mut ctx = WidgetContext::new_test();
        let select = Select::new(make_test_options(), signal);
        let root_id = select.build(&mut ctx);

        // Root (trigger) node should exist with styled content
        let node = ctx.scene().get_node(root_id);
        assert!(node.is_some(), "Select trigger node should exist in scene");

        let node = node.unwrap();
        assert!(
            matches!(node.content, NodeContent::Styled { .. }),
            "Trigger should have Styled content"
        );
    }

    #[test]
    fn test_select_creates_dropdown_in_layer() {
        let runtime = Runtime::new();
        let signal = Signal::new(runtime, None::<String>);

        let mut ctx = WidgetContext::new_test();
        let select = Select::new(make_test_options(), signal);
        let _root_id = select.build(&mut ctx);

        let dropdown_root = ctx.layer_root(Layer::Dropdown);
        let scene = ctx.scene();
        let layer_node = scene.get_node(dropdown_root).unwrap();

        assert!(
            !layer_node.children.is_empty(),
            "Dropdown layer should have children after building a Select"
        );
    }

    #[test]
    fn test_select_options_in_dropdown() {
        let runtime = Runtime::new();
        let signal = Signal::new(runtime, None::<String>);

        let mut ctx = WidgetContext::new_test();
        let select = Select::new(make_test_options(), signal);
        let _root_id = select.build(&mut ctx);

        let dropdown_root = ctx.layer_root(Layer::Dropdown);
        let scene = ctx.scene();
        let layer_node = scene.get_node(dropdown_root).unwrap();

        // The dropdown container is a child of the layer root
        assert_eq!(
            layer_node.children.len(),
            1,
            "Should have 1 dropdown container in the Dropdown layer"
        );

        let dropdown_id = layer_node.children[0];
        let dropdown_node = scene.get_node(dropdown_id).unwrap();

        // 3 options = 3 children in the dropdown container
        assert_eq!(
            dropdown_node.children.len(),
            3,
            "Dropdown with 3 options should have 3 row children, got {}",
            dropdown_node.children.len()
        );
    }

    #[test]
    fn test_select_click_option_updates_signal() {
        let runtime = Runtime::new();
        let signal = Signal::new(runtime, None::<String>);
        let observer = signal.clone();
        let (obs_read, _) = observer.split();

        let mut ctx = WidgetContext::new_test();
        let select = Select::new(make_test_options(), signal);
        let _root_id = select.build(&mut ctx);

        // Initially no selection
        assert_eq!(
            obs_read.get_untracked(),
            None,
            "Should start with no selection"
        );

        // Find the option rows in the dropdown
        let dropdown_root = ctx.layer_root(Layer::Dropdown);
        let dropdown_container = ctx.scene().get_node(dropdown_root).unwrap().children[0];
        let option_rows: Vec<NodeId> = ctx
            .scene()
            .get_node(dropdown_container)
            .unwrap()
            .children
            .clone();

        // Click the second option ("Beta", value "b")
        ctx.trigger_click(option_rows[1]);

        assert_eq!(
            obs_read.get_untracked(),
            Some("b".to_string()),
            "After clicking Beta, signal should be Some(\"b\")"
        );
    }

    #[test]
    fn test_select_disabled_not_clickable() {
        let runtime = Runtime::new();
        let signal = Signal::new(runtime, None::<String>);

        let mut ctx = WidgetContext::new_test();
        let select = Select::new(make_test_options(), signal).disabled(true);
        let root_id = select.build(&mut ctx);

        assert!(
            !ctx.has_clickable(root_id),
            "Disabled select trigger should not be clickable"
        );

        // Options should also not be clickable
        let dropdown_root = ctx.layer_root(Layer::Dropdown);
        let dropdown_container = ctx.scene().get_node(dropdown_root).unwrap().children[0];
        let option_rows: Vec<NodeId> = ctx
            .scene()
            .get_node(dropdown_container)
            .unwrap()
            .children
            .clone();

        for row_id in &option_rows {
            assert!(
                !ctx.has_clickable(*row_id),
                "Disabled select options should not be clickable"
            );
        }
    }

    #[test]
    fn test_select_placeholder_text() {
        let runtime = Runtime::new();
        let signal = Signal::new(runtime, None::<String>);

        let mut ctx = WidgetContext::new_test();
        let select = Select::new(make_test_options(), signal).placeholder("Choose one");
        let root_id = select.build(&mut ctx);

        // The trigger should have a child label node with placeholder text
        let root_node = ctx.scene().get_node(root_id).unwrap();
        assert!(
            !root_node.children.is_empty(),
            "Trigger should have a label child"
        );

        let label_id = root_node.children[0];
        let label_node = ctx.scene().get_node(label_id).unwrap();

        if let NodeContent::Styled { ref style } = label_node.content {
            let text = style.text.as_ref().expect("Label should have text content");
            assert_eq!(
                text.text, "Choose one",
                "Placeholder text should be 'Choose one'"
            );
        } else {
            panic!("Label node should be Styled content");
        }
    }

    #[test]
    fn test_select_disabled_option_not_clickable() {
        let runtime = Runtime::new();
        let signal = Signal::new(runtime, None::<String>);

        let options = vec![
            SelectOption::new("a", "Alpha"),
            SelectOption::new("b", "Beta").disabled(),
            SelectOption::new("c", "Gamma"),
        ];

        let mut ctx = WidgetContext::new_test();
        let select = Select::new(options, signal);
        let _root_id = select.build(&mut ctx);

        let dropdown_root = ctx.layer_root(Layer::Dropdown);
        let dropdown_container = ctx.scene().get_node(dropdown_root).unwrap().children[0];
        let option_rows: Vec<NodeId> = ctx
            .scene()
            .get_node(dropdown_container)
            .unwrap()
            .children
            .clone();

        // First and third should be clickable, second (disabled) should not
        assert!(
            ctx.has_clickable(option_rows[0]),
            "Enabled option 'Alpha' should be clickable"
        );
        assert!(
            !ctx.has_clickable(option_rows[1]),
            "Disabled option 'Beta' should not be clickable"
        );
        assert!(
            ctx.has_clickable(option_rows[2]),
            "Enabled option 'Gamma' should be clickable"
        );
    }

    #[test]
    fn test_select_default_width() {
        let runtime = Runtime::new();
        let signal = Signal::new(runtime, None::<String>);

        let select = Select::new(vec![], signal);
        assert!(
            (select.width - 200.0).abs() < f32::EPSILON,
            "Default select width should be 200.0, got {}",
            select.width
        );
    }

    #[test]
    fn test_select_custom_width() {
        let runtime = Runtime::new();
        let signal = Signal::new(runtime, None::<String>);

        let mut ctx = WidgetContext::new_test();
        let select = Select::new(make_test_options(), signal).width(320.0);
        let root_id = select.build(&mut ctx);

        assert!(
            ctx.scene().get_node(root_id).is_some(),
            "Select with custom width should build successfully"
        );
    }

    #[test]
    fn test_select_with_initial_value() {
        let runtime = Runtime::new();
        let signal = Signal::new(runtime, Some("b".to_string()));

        let mut ctx = WidgetContext::new_test();
        let select = Select::new(make_test_options(), signal);
        let root_id = select.build(&mut ctx);

        // The trigger label should show "Beta" (the label for value "b")
        let root_node = ctx.scene().get_node(root_id).unwrap();
        let label_id = root_node.children[0];
        let label_node = ctx.scene().get_node(label_id).unwrap();

        if let NodeContent::Styled { ref style } = label_node.content {
            let text = style.text.as_ref().expect("Label should have text content");
            assert_eq!(
                text.text, "Beta",
                "With initial value 'b', label should show 'Beta'"
            );
        } else {
            panic!("Label node should be Styled content");
        }
    }

    #[test]
    fn test_select_with_theme() {
        let runtime = Runtime::new();
        let signal = Signal::new(runtime, None::<String>);

        let mut ctx = WidgetContext::new_test();
        let theme = MaterialTheme::from_seed(Vec4::new(0.4, 0.2, 0.8, 1.0));
        ctx.set_extension(theme);

        let select = Select::new(make_test_options(), signal).placeholder("Pick");
        let root_id = select.build(&mut ctx);

        assert!(
            ctx.scene().get_node(root_id).is_some(),
            "Select with theme should build successfully"
        );
    }

    #[test]
    fn test_select_selected_option_highlighted() {
        use render_engine::Paint;

        let runtime = Runtime::new();
        let signal = Signal::new(runtime, Some("b".to_string()));

        let mut ctx = WidgetContext::new_test();
        let select = Select::new(make_test_options(), signal);
        let _root_id = select.build(&mut ctx);

        let dropdown_root = ctx.layer_root(Layer::Dropdown);
        let dropdown_container = ctx.scene().get_node(dropdown_root).unwrap().children[0];
        let option_rows: Vec<NodeId> = ctx
            .scene()
            .get_node(dropdown_container)
            .unwrap()
            .children
            .clone();

        // Second option ("Beta") should have secondary_container background
        let beta_node = ctx.scene().get_node(option_rows[1]).unwrap();
        if let NodeContent::Styled { ref style } = beta_node.content {
            assert!(
                !style.fills.is_empty(),
                "Selected option should have a fill"
            );
            if let Paint::Solid(color) = &style.fills[0] {
                assert!(
                    (color.x - FALLBACK_SECONDARY_CONTAINER.x).abs() < 0.01
                        && (color.y - FALLBACK_SECONDARY_CONTAINER.y).abs() < 0.01
                        && (color.z - FALLBACK_SECONDARY_CONTAINER.z).abs() < 0.01,
                    "Selected option should have secondary_container color, got {color:?}"
                );
            } else {
                panic!("Option fill should be a solid paint");
            }
        } else {
            panic!("Option row should be Styled content");
        }

        // First option ("Alpha") should have surface background (not highlighted)
        let alpha_node = ctx.scene().get_node(option_rows[0]).unwrap();
        if let NodeContent::Styled { ref style } = alpha_node.content
            && let Paint::Solid(color) = &style.fills[0]
        {
            assert!(
                (color.x - FALLBACK_SURFACE.x).abs() < 0.01
                    && (color.y - FALLBACK_SURFACE.y).abs() < 0.01
                    && (color.z - FALLBACK_SURFACE.z).abs() < 0.01,
                "Unselected option should have surface color, got {color:?}"
            );
        }
    }
}
