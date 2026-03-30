//! MD3 Chip widget -- compact actionable element.

use crate::theme::MaterialTheme;
use glam::Vec4;
use layout_engine::{FlexAlign, FlexDirection, FlexJustifyContent, FlexStyle};
use render_engine::node::NodeContent;
use render_engine::{NodeId, Paint, StrokeStyle, VisualStyle};
use std::sync::Arc;
use style_engine::StrokeAlign;
use widget_core::Widget;
use widget_core::{Text, WidgetContext};

// ---------------------------------------------------------------------------
// MD3 fallback colors (used when no MaterialTheme is provided)
// ---------------------------------------------------------------------------

/// MD3 outline (#79747E)
const FALLBACK_OUTLINE: Vec4 = Vec4::new(0.475, 0.455, 0.494, 1.0);

/// MD3 secondary_container (#E8DEF8) -- selected filter chip fill.
const FALLBACK_SECONDARY_CONTAINER: Vec4 = Vec4::new(0.906, 0.831, 0.996, 1.0);

/// MD3 on_surface (#1D1B20) -- label text.
const FALLBACK_ON_SURFACE: Vec4 = Vec4::new(0.114, 0.106, 0.125, 1.0);

/// Transparent fill for unselected chips.
const TRANSPARENT: Vec4 = Vec4::new(0.0, 0.0, 0.0, 0.0);

/// Chip height in dp (MD3 spec).
const CHIP_HEIGHT: f32 = 32.0;

/// Chip corner radius in dp (MD3 small shape).
const CHIP_CORNER_RADIUS: f32 = 8.0;

/// Horizontal padding in dp.
const CHIP_PADDING_H: f32 = 16.0;

/// Reduced horizontal padding when leading/trailing elements present.
const CHIP_PADDING_H_COMPACT: f32 = 8.0;

/// Label font size in dp (MD3 label_large).
const LABEL_FONT_SIZE: f32 = 14.0;

/// Gap between chip elements in dp.
const CHIP_GAP: f32 = 8.0;

/// Disabled content opacity multiplier.
const DISABLED_ALPHA: f32 = 0.38;

/// Chip variant following MD3 specifications.
///
/// Each variant has slightly different visual behavior and intended usage:
/// - **Assist**: Suggest related actions (e.g., "Add to calendar")
/// - **Filter**: Toggle a filter on/off (supports selected state)
/// - **Input**: Represent user input (e.g., tags in a text field)
/// - **Suggestion**: Dynamically generated suggestions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ChipVariant {
    /// Assist chip -- suggest related actions.
    #[default]
    Assist,
    /// Filter chip -- toggleable filter with selected state.
    Filter,
    /// Input chip -- represent user-provided input (e.g., tags).
    Input,
    /// Suggestion chip -- dynamically generated suggestion.
    Suggestion,
}

/// MD3 Chip -- compact actionable element.
///
/// Renders as a small pill-shaped container with a text label. Chips are
/// outlined by default (1dp border, transparent fill). Filter chips in the
/// selected state use the secondary container fill instead.
///
/// # Example
///
/// ```rust,no_run
/// use material_ui::data_display::Chip;
///
/// // Assist chip (default variant)
/// let chip = Chip::new("Add to calendar")
///     .on_click(|| println!("Assist!"));
///
/// // Filter chip with selection
/// let filter = Chip::filter("Vegetarian")
///     .selected(true);
///
/// // Input chip with delete
/// let input = Chip::input("Tag: Rust")
///     .on_delete(|| println!("Remove tag"));
/// ```
pub struct Chip {
    label: String,
    variant: ChipVariant,
    selected: bool,
    on_click: Option<Arc<dyn Fn() + Send + Sync>>,
    on_delete: Option<Arc<dyn Fn() + Send + Sync>>,
    disabled: bool,
}

impl Chip {
    /// Create a new assist chip with the given label.
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            variant: ChipVariant::Assist,
            selected: false,
            on_click: None,
            on_delete: None,
            disabled: false,
        }
    }

    /// Create a new filter chip with the given label.
    pub fn filter(label: impl Into<String>) -> Self {
        Self {
            variant: ChipVariant::Filter,
            ..Self::new(label)
        }
    }

    /// Create a new input chip with the given label.
    pub fn input(label: impl Into<String>) -> Self {
        Self {
            variant: ChipVariant::Input,
            ..Self::new(label)
        }
    }

    /// Create a new suggestion chip with the given label.
    pub fn suggestion(label: impl Into<String>) -> Self {
        Self {
            variant: ChipVariant::Suggestion,
            ..Self::new(label)
        }
    }

    /// Set the selected state (primarily used with filter chips).
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    /// Set the click handler invoked when the chip body is pressed.
    pub fn on_click(mut self, f: impl Fn() + Send + Sync + 'static) -> Self {
        self.on_click = Some(Arc::new(f));
        self
    }

    /// Set the delete handler. When set, a trailing "✕" button is appended.
    pub fn on_delete(mut self, f: impl Fn() + Send + Sync + 'static) -> Self {
        self.on_delete = Some(Arc::new(f));
        self
    }

    /// Disable interaction. The chip renders with reduced opacity and ignores
    /// click/delete events.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

impl Widget for Chip {
    fn build(&self, ctx: &mut WidgetContext) -> NodeId {
        // -- Resolve theme colors --
        let theme = ctx.get_extension::<MaterialTheme>().cloned();
        let outline_color = theme
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

        let has_trailing = self.on_delete.is_some();
        let padding_h = if has_trailing {
            CHIP_PADDING_H_COMPACT
        } else {
            CHIP_PADDING_H
        };

        // -- Determine fill color --
        let fill_color = if self.variant == ChipVariant::Filter && self.selected {
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

        // -- Determine outline opacity --
        let stroke_color = if self.disabled {
            Vec4::new(
                outline_color.x,
                outline_color.y,
                outline_color.z,
                DISABLED_ALPHA,
            )
        } else {
            outline_color
        };

        // -- Build container with outline --
        let style = VisualStyle::new()
            .solid_fill(fill_color)
            .corner_radius(CHIP_CORNER_RADIUS)
            .stroke(StrokeStyle::solid(
                Paint::solid(stroke_color),
                1.0,
                StrokeAlign::Inside,
            ));

        let container = ctx.create_node(
            ctx.root(),
            NodeContent::Styled {
                style: Box::new(style),
            },
        );
        ctx.set_layout_style(
            container,
            FlexStyle {
                direction: FlexDirection::Row,
                height: Some(CHIP_HEIGHT),
                padding_left: padding_h,
                padding_right: padding_h,
                gap: CHIP_GAP,
                justify_content: FlexJustifyContent::Center,
                align_items: FlexAlign::Center,
                ..Default::default()
            },
        );

        // -- Label text --
        let label_widget =
            Text::new(self.label.clone())
                .size(LABEL_FONT_SIZE)
                .color(render_engine::Color::rgba(
                    text_color.x,
                    text_color.y,
                    text_color.z,
                    text_color.w,
                ));
        let label_id = label_widget.build(ctx);
        ctx.reparent_to(label_id, container);

        // -- Trailing delete "✕" --
        if has_trailing {
            let delete_widget = Text::new("\u{2715}").color(render_engine::Color::rgba(
                text_color.x,
                text_color.y,
                text_color.z,
                text_color.w,
            ));
            let delete_id = delete_widget.build(ctx);
            ctx.reparent_to(delete_id, container);

            // Make delete button clickable
            if !self.disabled
                && let Some(ref on_delete) = self.on_delete
            {
                let callback = Arc::clone(on_delete);
                ctx.add_clickable(delete_id, callback);
            }
        }

        // -- Chip body click handler --
        if !self.disabled
            && let Some(ref on_click) = self.on_click
        {
            let callback = Arc::clone(on_click);
            ctx.add_clickable(container, callback);
        }

        container
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, Ordering};

    #[test]
    fn test_chip_assist_builds() {
        let mut ctx = WidgetContext::new_test();
        let chip = Chip::new("Add to calendar");
        let root_id = chip.build(&mut ctx);

        assert!(
            ctx.scene().get_node(root_id).is_some(),
            "Assist chip root node should exist in scene"
        );

        // Should have outline stroke
        let node = ctx.scene().get_node(root_id).unwrap();
        if let NodeContent::Styled { ref style } = node.content {
            assert!(style.stroke.is_some(), "Chip should have an outline stroke");
        } else {
            panic!("Chip container should be Styled content");
        }
    }

    #[test]
    fn test_chip_filter_selected_styling() {
        let mut ctx = WidgetContext::new_test();
        let chip = Chip::filter("Vegetarian").selected(true);
        let root_id = chip.build(&mut ctx);

        let node = ctx.scene().get_node(root_id).unwrap();
        if let NodeContent::Styled { ref style } = node.content {
            assert!(
                !style.fills.is_empty(),
                "Selected filter chip should have a fill"
            );
            if let Paint::Solid(color) = &style.fills[0] {
                // Selected filter chip should use secondary_container fill
                assert!(
                    color.w > 0.0,
                    "Selected filter chip fill should be non-transparent, got alpha={}",
                    color.w
                );
                assert!(
                    (color.x - FALLBACK_SECONDARY_CONTAINER.x).abs() < 0.01,
                    "Selected filter chip should match secondary_container color"
                );
            }
        } else {
            panic!("Chip container should be Styled content");
        }
    }

    #[test]
    fn test_chip_on_click() {
        let clicked = Arc::new(AtomicBool::new(false));
        let clicked_clone = Arc::clone(&clicked);

        let mut ctx = WidgetContext::new_test();
        let chip = Chip::new("Action").on_click(move || {
            clicked_clone.store(true, Ordering::SeqCst);
        });
        let root_id = chip.build(&mut ctx);

        assert!(
            ctx.has_clickable(root_id),
            "Chip with on_click should be clickable"
        );

        ctx.trigger_click(root_id);
        assert!(
            clicked.load(Ordering::SeqCst),
            "Chip click callback should have fired"
        );
    }

    #[test]
    fn test_chip_on_delete_adds_close_button() {
        let mut ctx = WidgetContext::new_test();
        let chip = Chip::input("Tag").on_delete(|| {});
        let root_id = chip.build(&mut ctx);

        let root_node = ctx.scene().get_node(root_id).unwrap();
        // Should have at least 2 children: label + delete "✕"
        assert!(
            root_node.children.len() >= 2,
            "Chip with on_delete should have label + delete children, got {}",
            root_node.children.len()
        );

        // The last child (delete button) should be clickable
        let delete_id = *root_node.children.last().unwrap();
        assert!(
            ctx.has_clickable(delete_id),
            "Delete button should be clickable"
        );
    }

    #[test]
    fn test_chip_disabled() {
        let mut ctx = WidgetContext::new_test();
        let chip = Chip::new("Disabled")
            .on_click(|| {})
            .on_delete(|| {})
            .disabled(true);
        let root_id = chip.build(&mut ctx);

        assert!(
            !ctx.has_clickable(root_id),
            "Disabled chip should not be clickable"
        );

        // Verify reduced alpha on fill
        let node = ctx.scene().get_node(root_id).unwrap();
        if let NodeContent::Styled { ref style } = node.content {
            // The stroke should have reduced alpha
            if let Some(ref stroke) = style.stroke
                && let Some(Paint::Solid(color)) = stroke.paints.first()
            {
                assert!(
                    (color.w - DISABLED_ALPHA).abs() < 0.01,
                    "Disabled chip stroke should have reduced alpha ({}), got {}",
                    DISABLED_ALPHA,
                    color.w
                );
            }
        }
    }

    #[test]
    fn test_chip_with_all_options() {
        let clicked = Arc::new(AtomicBool::new(false));
        let deleted = Arc::new(AtomicBool::new(false));
        let clicked_clone = Arc::clone(&clicked);
        let deleted_clone = Arc::clone(&deleted);

        let mut ctx = WidgetContext::new_test();
        let chip = Chip::filter("Complete")
            .selected(true)
            .on_click(move || {
                clicked_clone.store(true, Ordering::SeqCst);
            })
            .on_delete(move || {
                deleted_clone.store(true, Ordering::SeqCst);
            });
        let root_id = chip.build(&mut ctx);

        // Should be clickable
        assert!(
            ctx.has_clickable(root_id),
            "Chip with on_click should be clickable"
        );

        // Should have trailing delete -- capture children before mutable borrows
        let children: Vec<NodeId> = ctx.scene().get_node(root_id).unwrap().children.clone();
        assert!(
            children.len() >= 2,
            "Chip with on_delete should have multiple children"
        );

        let delete_id = *children.last().unwrap();

        // Click the chip body
        ctx.trigger_click(root_id);
        assert!(
            clicked.load(Ordering::SeqCst),
            "Chip body click should fire"
        );

        // Click the delete button
        ctx.trigger_click(delete_id);
        assert!(
            deleted.load(Ordering::SeqCst),
            "Chip delete click should fire"
        );
    }
}
