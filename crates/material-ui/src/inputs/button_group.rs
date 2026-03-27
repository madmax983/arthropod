//! MD3 ButtonGroup widget -- row of outlined buttons sharing border radius.

use crate::theme::MaterialTheme;
use glam::Vec4;
use layout_engine::{FlexAlign, FlexDirection, FlexJustifyContent, FlexStyle};
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

/// MD3 on_surface (#1D1B20)
const FALLBACK_ON_SURFACE: Vec4 = Vec4::new(0.114, 0.106, 0.125, 1.0);

/// Transparent fill
const TRANSPARENT: Vec4 = Vec4::new(0.0, 0.0, 0.0, 0.0);

/// Fixed button height in dp.
const BUTTON_HEIGHT: f32 = 40.0;

/// Horizontal padding in dp.
const BUTTON_PADDING_H: f32 = 12.0;

/// Default corner radius for first/last segment buttons.
const SEGMENT_CORNER_RADIUS: f32 = 12.0;

/// MD3 ButtonGroup -- row of outlined buttons sharing border radius.
///
/// Each button segment has a 1dp outline border with transparent fill and
/// on_surface text. The first button gets left corners rounded, the last
/// gets right corners rounded, and middle buttons get no rounding.
///
/// # Example
///
/// ```rust,no_run
/// use material_ui::inputs::ButtonGroup;
///
/// let group = ButtonGroup::new(vec!["Cut", "Copy", "Paste"])
///     .on_click(|index| println!("Clicked button {index}"));
/// ```
pub struct ButtonGroup {
    labels: Vec<String>,
    on_click: Option<Arc<dyn Fn(usize) + Send + Sync>>,
    disabled: bool,
}

impl ButtonGroup {
    /// Create a new button group with the given labels.
    pub fn new(labels: Vec<impl Into<String>>) -> Self {
        Self {
            labels: labels.into_iter().map(Into::into).collect(),
            on_click: None,
            disabled: false,
        }
    }

    /// Set the callback invoked when a button is clicked.
    ///
    /// The callback receives the zero-based index of the clicked button.
    pub fn on_click(mut self, f: impl Fn(usize) + Send + Sync + 'static) -> Self {
        self.on_click = Some(Arc::new(f));
        self
    }

    /// Disable interaction. Buttons render normally but ignore click events.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Compute per-corner radii for a segment at the given position.
    fn segment_radii(index: usize, total: usize) -> CornerRadii {
        if total <= 1 {
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

impl Widget for ButtonGroup {
    fn build(&self, ctx: &mut WidgetContext) -> NodeId {
        let total = self.labels.len();

        // -- Resolve theme colors --
        let theme = ctx.get_extension::<MaterialTheme>().cloned();
        let outline = theme
            .as_ref()
            .map(|t| t.color.outline)
            .unwrap_or(FALLBACK_OUTLINE);
        let on_surface = theme
            .as_ref()
            .map(|t| t.color.on_surface)
            .unwrap_or(FALLBACK_ON_SURFACE);

        // Root container (horizontal row, gap=0 so buttons touch)
        let root = ctx.create_node(ctx.root(), NodeContent::Empty);
        ctx.set_layout_style(
            root,
            FlexStyle {
                direction: FlexDirection::Row,
                gap: 0.0,
                ..Default::default()
            },
        );

        for (i, label) in self.labels.iter().enumerate() {
            let radii = Self::segment_radii(i, total);

            // Outlined style: transparent fill, 1dp outline border
            let style = VisualStyle::new()
                .solid_fill(TRANSPARENT)
                .corner_radii(radii)
                .stroke(StrokeStyle::solid(
                    Paint::solid(outline),
                    1.0,
                    StrokeAlign::Inside,
                ));

            let segment = ctx.create_node(
                root,
                NodeContent::Styled {
                    style: Box::new(style),
                },
            );
            ctx.set_layout_style(
                segment,
                FlexStyle {
                    height: Some(BUTTON_HEIGHT),
                    padding_left: BUTTON_PADDING_H,
                    padding_right: BUTTON_PADDING_H,
                    justify_content: FlexJustifyContent::Center,
                    align_items: FlexAlign::Center,
                    ..Default::default()
                },
            );

            // Text label
            let text_widget = Text::new(label.clone()).color(render_engine::Color::rgba(
                on_surface.x,
                on_surface.y,
                on_surface.z,
                on_surface.w,
            ));
            let text_id = text_widget.build(ctx);
            ctx.reparent_to(text_id, segment);

            // Click handler
            if !self.disabled
                && let Some(ref on_click) = self.on_click
            {
                let callback = Arc::clone(on_click);
                ctx.add_clickable(
                    segment,
                    Arc::new(move || {
                        callback(i);
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
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn test_button_group_creates_buttons() {
        let mut ctx = WidgetContext::new_test();
        let group = ButtonGroup::new(vec!["Cut", "Copy", "Paste"]);
        let root_id = group.build(&mut ctx);

        let scene = ctx.scene();
        let root_node = scene.get_node(root_id).unwrap();

        assert_eq!(
            root_node.children.len(),
            3,
            "ButtonGroup with 3 labels should have 3 children, got {}",
            root_node.children.len()
        );
    }

    #[test]
    fn test_button_group_click_index() {
        let clicked_index = Arc::new(AtomicUsize::new(usize::MAX));
        let clicked_clone = Arc::clone(&clicked_index);

        let mut ctx = WidgetContext::new_test();
        let group = ButtonGroup::new(vec!["A", "B", "C"]).on_click(move |i| {
            clicked_clone.store(i, Ordering::SeqCst);
        });
        let root_id = group.build(&mut ctx);

        // Click the second button (index 1)
        let second_id = ctx.scene().get_node(root_id).unwrap().children[1];
        ctx.trigger_click(second_id);

        assert_eq!(
            clicked_index.load(Ordering::SeqCst),
            1,
            "Clicking the second button should invoke on_click with index 1"
        );
    }

    #[test]
    fn test_button_group_disabled() {
        let mut ctx = WidgetContext::new_test();
        let group = ButtonGroup::new(vec!["A", "B"])
            .on_click(|_| {})
            .disabled(true);
        let root_id = group.build(&mut ctx);

        let scene = ctx.scene();
        let root_node = scene.get_node(root_id).unwrap();

        for &child_id in &root_node.children {
            assert!(
                !ctx.has_clickable(child_id),
                "Disabled ButtonGroup segments should not be clickable"
            );
        }
    }
}
