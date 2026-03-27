//! MD3 ToggleButton widget -- single pressable toggle with text.

use crate::theme::MaterialTheme;
use glam::Vec4;
use layout_engine::{FlexAlign, FlexJustifyContent, FlexStyle};
use render_engine::node::NodeContent;
use render_engine::{CornerRadii, NodeId, Paint, StrokeStyle, VisualStyle};
use std::sync::Arc;
use style_engine::StrokeAlign;
use widget_core::widget_trait::Widget;
use widget_core::{Text, WidgetContext};

// ---------------------------------------------------------------------------
// MD3 fallback colors (used when no MaterialTheme is provided)
// ---------------------------------------------------------------------------

/// MD3 outline (#79747E)
const FALLBACK_OUTLINE: Vec4 = Vec4::new(0.475, 0.455, 0.494, 1.0);

/// MD3 secondary_container (#E8DEF8)
const FALLBACK_SECONDARY_CONTAINER: Vec4 = Vec4::new(0.906, 0.831, 0.996, 1.0);

/// MD3 on_surface (#1D1B20)
const FALLBACK_ON_SURFACE: Vec4 = Vec4::new(0.114, 0.106, 0.125, 1.0);

/// Transparent fill
const TRANSPARENT: Vec4 = Vec4::new(0.0, 0.0, 0.0, 0.0);

/// Fixed button height in dp.
const BUTTON_HEIGHT: f32 = 40.0;

/// Horizontal padding in dp.
const BUTTON_PADDING_H: f32 = 12.0;

/// Default corner radius for standalone toggle buttons.
const DEFAULT_CORNER_RADIUS: f32 = 12.0;

/// Disabled content opacity multiplier.
const DISABLED_ALPHA: f32 = 0.38;

/// MD3 ToggleButton.
///
/// A single pressable toggle with text that switches between selected and
/// unselected states. When selected, the background fills with the secondary
/// container color. The button always has a 1dp outline border.
///
/// # Example
///
/// ```rust,no_run
/// use material_ui::inputs::ToggleButton;
///
/// let toggle = ToggleButton::new("Bold")
///     .selected(true)
///     .on_toggle(|selected| println!("Bold: {selected}"));
/// ```
pub struct ToggleButton {
    text: String,
    selected: bool,
    on_toggle: Option<Arc<dyn Fn(bool) + Send + Sync>>,
    disabled: bool,
    /// Per-corner radii override (used by ToggleButtonGroup for segmented corners).
    corner_radii: Option<CornerRadii>,
}

impl ToggleButton {
    /// Create a new toggle button with the given text label.
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            selected: false,
            on_toggle: None,
            disabled: false,
            corner_radii: None,
        }
    }

    /// Set the selected state.
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    /// Set the callback invoked when the button is toggled.
    ///
    /// The callback receives the *new* selected state (`!current`).
    pub fn on_toggle(mut self, f: impl Fn(bool) + Send + Sync + 'static) -> Self {
        self.on_toggle = Some(Arc::new(f));
        self
    }

    /// Disable interaction. The button renders with reduced opacity and ignores
    /// click events.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Override corner radii (used internally by `ToggleButtonGroup` for
    /// segmented button corners).
    pub(crate) fn corner_radii(mut self, radii: CornerRadii) -> Self {
        self.corner_radii = Some(radii);
        self
    }
}

impl Widget for ToggleButton {
    fn build(&self, ctx: &mut WidgetContext) -> NodeId {
        // -- Resolve theme colors --
        let theme = ctx.get_extension::<MaterialTheme>().cloned();
        let outline = theme
            .as_ref()
            .map(|t| t.color.outline)
            .unwrap_or(FALLBACK_OUTLINE);
        let secondary_container = theme
            .as_ref()
            .map(|t| t.color.secondary_container)
            .unwrap_or(FALLBACK_SECONDARY_CONTAINER);
        let on_surface = theme
            .as_ref()
            .map(|t| t.color.on_surface)
            .unwrap_or(FALLBACK_ON_SURFACE);

        // -- Determine fill color --
        let fill_color = if self.selected {
            secondary_container
        } else {
            TRANSPARENT
        };

        // -- Determine text color (reduced alpha when disabled) --
        let text_color = if self.disabled {
            Vec4::new(on_surface.x, on_surface.y, on_surface.z, DISABLED_ALPHA)
        } else {
            on_surface
        };

        // -- Corner radii (custom or default) --
        let radii = self
            .corner_radii
            .unwrap_or(CornerRadii::uniform(DEFAULT_CORNER_RADIUS));

        // -- Build visual style with outline border --
        let style = VisualStyle::new()
            .solid_fill(fill_color)
            .corner_radii(radii)
            .stroke(StrokeStyle::solid(
                Paint::solid(outline),
                1.0,
                StrokeAlign::Inside,
            ));

        // -- Container node --
        let container = ctx.create_node(
            ctx.root(),
            NodeContent::Styled {
                style: Box::new(style),
            },
        );
        ctx.set_layout_style(
            container,
            FlexStyle {
                height: Some(BUTTON_HEIGHT),
                padding_left: BUTTON_PADDING_H,
                padding_right: BUTTON_PADDING_H,
                justify_content: FlexJustifyContent::Center,
                align_items: FlexAlign::Center,
                ..Default::default()
            },
        );

        // -- Text label --
        let text_widget = Text::new(self.text.clone()).color(render_engine::Color::rgba(
            text_color.x,
            text_color.y,
            text_color.z,
            text_color.w,
        ));
        let text_id = text_widget.build(ctx);
        ctx.reparent_to(text_id, container);

        // -- Click handler --
        if !self.disabled
            && let Some(ref on_toggle) = self.on_toggle
        {
            let callback = Arc::clone(on_toggle);
            let current = self.selected;
            ctx.add_clickable(
                container,
                Arc::new(move || {
                    callback(!current);
                }),
            );
        }

        container
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, Ordering};

    #[test]
    fn test_toggle_button_renders() {
        let mut ctx = WidgetContext::new_test();
        let toggle = ToggleButton::new("Bold");
        let root_id = toggle.build(&mut ctx);

        assert!(
            ctx.scene().get_node(root_id).is_some(),
            "ToggleButton root node should exist in scene"
        );
    }

    #[test]
    fn test_toggle_button_selected_styling() {
        let mut ctx = WidgetContext::new_test();
        let toggle = ToggleButton::new("Bold").selected(true);
        let root_id = toggle.build(&mut ctx);

        let node = ctx.scene().get_node(root_id).unwrap();
        if let NodeContent::Styled { ref style } = node.content {
            // When selected, background should be secondary_container (non-transparent)
            assert!(
                !style.fills.is_empty(),
                "Selected toggle should have a fill"
            );
            if let Paint::Solid(color) = &style.fills[0] {
                assert!(
                    color.w > 0.0,
                    "Selected toggle fill should be non-transparent, got alpha={}",
                    color.w
                );
                // Verify it's the secondary_container fallback color
                assert!(
                    (color.x - FALLBACK_SECONDARY_CONTAINER.x).abs() < 0.01,
                    "Selected fill should match secondary_container"
                );
            }
        } else {
            panic!("ToggleButton container should be Styled content");
        }
    }

    #[test]
    fn test_toggle_button_unselected_transparent() {
        let mut ctx = WidgetContext::new_test();
        let toggle = ToggleButton::new("Bold").selected(false);
        let root_id = toggle.build(&mut ctx);

        let node = ctx.scene().get_node(root_id).unwrap();
        if let NodeContent::Styled { ref style } = node.content {
            // When unselected, background should be transparent
            if let Some(Paint::Solid(color)) = style.fills.first() {
                assert!(
                    color.w < 0.01,
                    "Unselected toggle fill should be transparent, got alpha={}",
                    color.w
                );
            }
        } else {
            panic!("ToggleButton container should be Styled content");
        }
    }

    #[test]
    fn test_toggle_button_click_calls_on_toggle() {
        let toggled = Arc::new(AtomicBool::new(false));
        let toggled_clone = Arc::clone(&toggled);

        let mut ctx = WidgetContext::new_test();
        let toggle = ToggleButton::new("Bold")
            .selected(false)
            .on_toggle(move |new_state| {
                toggled_clone.store(new_state, Ordering::SeqCst);
            });
        let root_id = toggle.build(&mut ctx);

        // Should be clickable
        assert!(
            ctx.has_clickable(root_id),
            "Enabled toggle button should be clickable"
        );

        // Clicking should call on_toggle with !selected (true)
        ctx.trigger_click(root_id);
        assert!(
            toggled.load(Ordering::SeqCst),
            "on_toggle should receive true when toggling from unselected"
        );
    }

    #[test]
    fn test_toggle_button_disabled_not_clickable() {
        let mut ctx = WidgetContext::new_test();
        let toggle = ToggleButton::new("Bold").disabled(true).on_toggle(|_| {});
        let root_id = toggle.build(&mut ctx);

        assert!(
            !ctx.has_clickable(root_id),
            "Disabled toggle button should not be clickable"
        );
    }

    #[test]
    fn test_toggle_button_has_text_child() {
        let mut ctx = WidgetContext::new_test();
        let toggle = ToggleButton::new("Italic");
        let root_id = toggle.build(&mut ctx);

        let root_node = ctx.scene().get_node(root_id).unwrap();
        assert!(
            !root_node.children.is_empty(),
            "ToggleButton should have a text child"
        );
    }

    #[test]
    fn test_toggle_button_with_theme() {
        let mut ctx = WidgetContext::new_test();
        let theme = MaterialTheme::from_seed(Vec4::new(0.4, 0.2, 0.8, 1.0));
        ctx.set_extension(theme);

        let toggle = ToggleButton::new("Themed").selected(true);
        let root_id = toggle.build(&mut ctx);

        assert!(
            ctx.scene().get_node(root_id).is_some(),
            "ToggleButton with theme should build successfully"
        );
    }

    #[test]
    fn test_toggle_button_has_outline_stroke() {
        let mut ctx = WidgetContext::new_test();
        let toggle = ToggleButton::new("Outlined");
        let root_id = toggle.build(&mut ctx);

        let node = ctx.scene().get_node(root_id).unwrap();
        if let NodeContent::Styled { ref style } = node.content {
            assert!(
                style.stroke.is_some(),
                "ToggleButton should have an outline stroke"
            );
            let stroke = style.stroke.as_ref().unwrap();
            assert!(
                (stroke.weight - 1.0).abs() < 0.01,
                "Outline stroke weight should be 1dp"
            );
        } else {
            panic!("ToggleButton container should be Styled content");
        }
    }
}
