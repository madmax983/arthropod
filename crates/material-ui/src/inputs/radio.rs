//! MD3 Radio button widget -- single selection indicator with circular design.

use crate::theme::MaterialTheme;
use flux_state::{Effect, ReadSignal, Signal};
use glam::Vec4;
use layout_engine::{FlexAlign, FlexDirection, FlexJustifyContent, FlexStyle};
use render_engine::node::NodeContent;
use render_engine::{Color, NodeId};
use std::sync::Arc;
use widget_core::Widget;
use widget_core::{Text, WidgetContext};

// ---------------------------------------------------------------------------
// MD3 fallback colors (used when no MaterialTheme is provided)
// ---------------------------------------------------------------------------

/// MD3 primary (#6750A4)
const FALLBACK_PRIMARY: Vec4 = Vec4::new(0.404, 0.314, 0.643, 1.0);

/// MD3 outline (#79747E)
const FALLBACK_OUTLINE: Vec4 = Vec4::new(0.475, 0.455, 0.494, 1.0);

/// Transparent (for hidden inner dot)
const TRANSPARENT: Vec4 = Vec4::new(0.0, 0.0, 0.0, 0.0);

/// MD3 Radio button.
///
/// A circular selection indicator following Material Design 3 guidelines.
/// Radio buttons are used in groups where only one option can be selected
/// at a time. Each radio has a 20dp outer circle with a 10dp inner dot
/// that becomes visible when selected.
///
/// # Example
///
/// ```rust,no_run
/// use flux_state::{Runtime, Signal};
/// use material_ui::inputs::Radio;
///
/// let runtime = Runtime::new();
/// let selected = Signal::new(runtime.clone(), None);
/// let (read, write) = selected.split();
///
/// let w = write.clone();
/// let radio = Radio::new("option_a", read, move |val| w.set(Some(val)))
///     .label("Option A");
/// ```
pub struct Radio {
    value: String,
    selected: ReadSignal<Option<String>>,
    on_select: Arc<dyn Fn(String) + Send + Sync>,
    label: Option<String>,
    disabled: bool,
}

impl Radio {
    /// Create a new radio button.
    ///
    /// # Arguments
    ///
    /// * `value` - The value this radio represents when selected.
    /// * `selected` - A read signal holding the currently selected value (or `None`).
    /// * `on_select` - Callback invoked with this radio's value when clicked.
    pub fn new(
        value: impl Into<String>,
        selected: ReadSignal<Option<String>>,
        on_select: impl Fn(String) + Send + Sync + 'static,
    ) -> Self {
        Self {
            value: value.into(),
            selected,
            on_select: Arc::new(on_select),
            label: None,
            disabled: false,
        }
    }

    /// Attach a text label displayed to the right of the radio circle.
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Disable interaction. The radio still reflects signal changes visually,
    /// but clicks are ignored.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

impl Widget for Radio {
    fn build(&self, ctx: &mut WidgetContext) -> NodeId {
        let runtime = self.selected.runtime().clone();
        let read = self.selected.clone();
        let value = self.value.clone();

        // -- Resolve theme colors --
        let theme = ctx.get_extension::<MaterialTheme>().cloned();
        let primary_color = theme
            .as_ref()
            .map(|t| t.color.primary)
            .unwrap_or(FALLBACK_PRIMARY);
        let outline_color = theme
            .as_ref()
            .map(|t| t.color.outline)
            .unwrap_or(FALLBACK_OUTLINE);
        // -- Determine initial selected state --
        let is_selected = read.get_untracked().as_ref().is_some_and(|v| *v == value);

        // -- Reactive color signals for outer border and inner dot --
        let initial_border_color = if is_selected {
            primary_color
        } else {
            outline_color
        };
        let initial_dot_color = if is_selected {
            primary_color
        } else {
            TRANSPARENT
        };

        let border_color_signal = Signal::new(
            runtime.clone(),
            Color::rgba(
                initial_border_color.x,
                initial_border_color.y,
                initial_border_color.z,
                initial_border_color.w,
            ),
        );
        let (border_color_read, border_color_write) = border_color_signal.split();

        let dot_color_signal = Signal::new(
            runtime.clone(),
            Color::rgba(
                initial_dot_color.x,
                initial_dot_color.y,
                initial_dot_color.z,
                initial_dot_color.w,
            ),
        );
        let (dot_color_read, dot_color_write) = dot_color_signal.split();

        // -- Effect: sync selected signal -> border/dot colors --
        let read_for_effect = read.clone();
        let value_for_effect = value.clone();
        let effect = Effect::new(runtime.clone(), move || {
            let current = read_for_effect.get();
            let selected = current.as_ref().is_some_and(|v| *v == value_for_effect);
            if selected {
                border_color_write.set(Color::rgba(
                    primary_color.x,
                    primary_color.y,
                    primary_color.z,
                    primary_color.w,
                ));
                dot_color_write.set(Color::rgba(
                    primary_color.x,
                    primary_color.y,
                    primary_color.z,
                    primary_color.w,
                ));
            } else {
                border_color_write.set(Color::rgba(
                    outline_color.x,
                    outline_color.y,
                    outline_color.z,
                    outline_color.w,
                ));
                dot_color_write.set(Color::rgba(
                    TRANSPARENT.x,
                    TRANSPARENT.y,
                    TRANSPARENT.z,
                    TRANSPARENT.w,
                ));
            }
        });
        ctx.store_effect(effect);

        // -- Build scene nodes --

        // 1. Root row container (radio circle + optional label)
        let root = ctx.create_node(ctx.root(), NodeContent::Empty);
        ctx.set_layout_style(
            root,
            FlexStyle {
                direction: FlexDirection::Row,
                gap: 8.0,
                align_items: FlexAlign::Center,
                ..Default::default()
            },
        );

        // 2. Outer circle (20x20, corner_radius=10 for full circle)
        let outer_circle = ctx.create_node(
            root,
            NodeContent::Styled {
                style: Box::new(
                    render_engine::VisualStyle::new()
                        .solid_fill(initial_border_color)
                        .corner_radius(10.0),
                ),
            },
        );
        ctx.set_layout_style(
            outer_circle,
            FlexStyle {
                width: Some(20.0),
                height: Some(20.0),
                justify_content: FlexJustifyContent::Center,
                align_items: FlexAlign::Center,
                ..Default::default()
            },
        );
        ctx.add_reactive_color_state(outer_circle, border_color_read);

        // 3. Inner dot (10x10, corner_radius=5 for full circle)
        let inner_dot = ctx.create_node(
            outer_circle,
            NodeContent::Styled {
                style: Box::new(
                    render_engine::VisualStyle::new()
                        .solid_fill(initial_dot_color)
                        .corner_radius(5.0),
                ),
            },
        );
        ctx.set_layout_style(
            inner_dot,
            FlexStyle {
                width: Some(10.0),
                height: Some(10.0),
                ..Default::default()
            },
        );
        ctx.add_reactive_color_state(inner_dot, dot_color_read);

        // 4. Click handler on root (calls on_select with this radio's value)
        if !self.disabled {
            let on_select = Arc::clone(&self.on_select);
            let click_value = value.clone();
            ctx.add_clickable(
                root,
                Arc::new(move || {
                    on_select(click_value.clone());
                }),
            );
        }

        // 5. Optional label
        if let Some(ref label_text) = self.label {
            let label_widget = Text::new(label_text.clone());
            let label_id = label_widget.build(ctx);
            ctx.reparent_to(label_id, root);

            // Also make label clickable if enabled
            if !self.disabled {
                let on_select = Arc::clone(&self.on_select);
                let click_value = value.clone();
                ctx.add_clickable(
                    label_id,
                    Arc::new(move || {
                        on_select(click_value.clone());
                    }),
                );
            }
        }

        root
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use flux_state::Runtime;

    #[test]
    fn test_radio_builds_with_circle() {
        let runtime = Runtime::new();
        let selected = Signal::new(runtime.clone(), None::<String>);
        let (read, write) = selected.split();

        let mut ctx = WidgetContext::new_test();
        let w = write.clone();
        let radio = Radio::new("opt_a", read, move |val| w.set(Some(val)));
        let root_id = radio.build(&mut ctx);

        // Root node should exist
        assert!(
            ctx.scene().get_node(root_id).is_some(),
            "Radio root node should exist in scene"
        );

        // Root should have the outer circle as a child
        let root_node = ctx.scene().get_node(root_id).unwrap();
        assert!(
            !root_node.children.is_empty(),
            "Radio root should have children (outer circle)"
        );

        // Outer circle should have the inner dot as a child
        let outer_id = root_node.children[0];
        let outer_node = ctx.scene().get_node(outer_id).unwrap();
        assert!(
            !outer_node.children.is_empty(),
            "Outer circle should have children (inner dot)"
        );
    }

    #[test]
    fn test_radio_selected_shows_inner_dot() {
        let runtime = Runtime::new();
        // Pre-select "opt_a" so the radio starts selected
        let selected = Signal::new(runtime.clone(), Some("opt_a".to_string()));
        let (read, write) = selected.split();

        let mut ctx = WidgetContext::new_test();
        let w = write.clone();
        let radio = Radio::new("opt_a", read, move |val| w.set(Some(val)));
        let root_id = radio.build(&mut ctx);

        // Navigate to inner dot: root -> outer_circle -> inner_dot
        let root_node = ctx.scene().get_node(root_id).unwrap();
        let outer_id = root_node.children[0];
        let outer_node = ctx.scene().get_node(outer_id).unwrap();
        let dot_id = outer_node.children[0];
        let dot_node = ctx.scene().get_node(dot_id).unwrap();

        // When selected, inner dot should have a non-transparent fill (primary color)
        if let NodeContent::Styled { ref style } = dot_node.content {
            assert!(
                !style.fills.is_empty(),
                "Selected radio inner dot should have a fill"
            );
            // The fill should not be transparent (alpha > 0)
            if let render_engine::Paint::Solid(color) = &style.fills[0] {
                assert!(
                    color.w > 0.0,
                    "Selected radio inner dot fill should be non-transparent, got alpha={}",
                    color.w
                );
            }
        } else {
            panic!("Inner dot should be Styled content");
        }
    }

    #[test]
    fn test_radio_with_label() {
        let runtime = Runtime::new();
        let selected = Signal::new(runtime.clone(), None::<String>);
        let (read, write) = selected.split();

        let mut ctx = WidgetContext::new_test();
        let w = write.clone();
        let radio = Radio::new("opt_b", read, move |val| w.set(Some(val))).label("Option B");
        let root_id = radio.build(&mut ctx);

        let scene = ctx.scene();
        let root_node = scene.get_node(root_id).unwrap();

        // Root should have outer_circle + label = at least 2 children
        assert!(
            root_node.children.len() >= 2,
            "Radio with label should have at least 2 children (circle + label), got {}",
            root_node.children.len()
        );
    }

    #[test]
    fn test_radio_click_triggers_on_select() {
        let runtime = Runtime::new();
        let selected = Signal::new(runtime.clone(), None::<String>);
        let observer = selected.clone();
        let (obs_read, _) = observer.split();
        let (read, write) = selected.split();

        let mut ctx = WidgetContext::new_test();
        let w = write.clone();
        let radio = Radio::new("clicked_value", read, move |val| w.set(Some(val)));
        let root_id = radio.build(&mut ctx);

        // Initially nothing selected
        assert_eq!(
            obs_read.get_untracked(),
            None,
            "Should start with no selection"
        );

        // Click the radio
        ctx.trigger_click(root_id);
        assert_eq!(
            obs_read.get_untracked(),
            Some("clicked_value".to_string()),
            "After click, selected should be this radio's value"
        );
    }

    #[test]
    fn test_radio_disabled_not_clickable() {
        let runtime = Runtime::new();
        let selected = Signal::new(runtime.clone(), None::<String>);
        let (read, write) = selected.split();

        let mut ctx = WidgetContext::new_test();
        let w = write.clone();
        let radio = Radio::new("opt_disabled", read, move |val| w.set(Some(val))).disabled(true);
        let root_id = radio.build(&mut ctx);

        assert!(
            !ctx.has_clickable(root_id),
            "Disabled radio should not be clickable"
        );
    }
}
