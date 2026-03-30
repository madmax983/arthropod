//! MD3 Tooltip widget -- informational overlay rendered in the Tooltip layer.

use glam::Vec4;
use render_engine::node::NodeContent;
use render_engine::{NodeId, TextContent, VisualStyle};
use widget_core::Layer;
use widget_core::Widget;
use widget_core::WidgetContext;

// ---------------------------------------------------------------------------
// MD3 tooltip colors
// ---------------------------------------------------------------------------

/// MD3 inverse-surface / tooltip background (#313033).
const TOOLTIP_BG: Vec4 = Vec4::new(0.192, 0.188, 0.200, 1.0);

/// White text for tooltip labels (#FFFFFF).
const TOOLTIP_TEXT: Vec4 = Vec4::new(1.0, 1.0, 1.0, 1.0);

/// MD3 tooltip corner radius (dp).
const TOOLTIP_CORNER_RADIUS: f32 = 4.0;

/// Default tooltip font size (dp).
const DEFAULT_FONT_SIZE: f32 = 12.0;

/// MD3 Tooltip -- informational text overlay.
///
/// Wraps a target widget and renders supplementary text in the
/// [`Layer::Tooltip`] layer (z=4), which renders above all normal UI content.
///
/// Currently the tooltip node is always created in the scene graph.
/// Hover-based visibility toggling is planned for a future release.
///
/// # Example
///
/// ```rust,no_run
/// use material_ui::data_display::Tooltip;
/// use widget_core::Text;
///
/// let widget = Tooltip::new("Save your changes", Text::new("Save"))
///     .font_size(14.0);
/// ```
pub struct Tooltip {
    text: String,
    target: Box<dyn Widget>,
    font_size: f32,
}

impl Tooltip {
    /// Create a new tooltip that displays `text` above the given `target` widget.
    pub fn new(text: impl Into<String>, target: impl Widget + 'static) -> Self {
        Self {
            text: text.into(),
            target: Box::new(target),
            font_size: DEFAULT_FONT_SIZE,
        }
    }

    /// Override the tooltip text font size in dp (default: 12.0).
    pub fn font_size(mut self, size: f32) -> Self {
        self.font_size = size;
        self
    }
}

impl Widget for Tooltip {
    fn build(&self, ctx: &mut WidgetContext) -> NodeId {
        // -- Build the target widget in the normal content layer --
        let target_id = self.target.build(ctx);

        // -- Create tooltip container in the Tooltip layer --
        let container_style = VisualStyle::new()
            .solid_fill(TOOLTIP_BG)
            .corner_radius(TOOLTIP_CORNER_RADIUS);

        let tooltip_container = ctx.add_to_layer(
            Layer::Tooltip,
            NodeContent::Styled {
                style: Box::new(container_style),
            },
        );

        // -- Add text child inside tooltip container --
        let text_style = VisualStyle::new()
            .solid_fill(TOOLTIP_TEXT)
            .text(TextContent::new(self.text.clone(), self.font_size));

        ctx.create_node(
            tooltip_container,
            NodeContent::Styled {
                style: Box::new(text_style),
            },
        );

        // Return the target widget's NodeId -- the tooltip is a side effect.
        target_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use render_engine::Paint;

    #[test]
    fn test_tooltip_builds_target() {
        let mut ctx = WidgetContext::new_test();
        let target = widget_core::Text::new("Save");
        let tooltip = Tooltip::new("Save your changes", target);
        let root_id = tooltip.build(&mut ctx);

        assert!(
            ctx.scene().get_node(root_id).is_some(),
            "Target widget node should exist in scene"
        );
    }

    #[test]
    fn test_tooltip_creates_node_in_tooltip_layer() {
        let mut ctx = WidgetContext::new_test();
        let tooltip = Tooltip::new("Help text", widget_core::Text::new("?"));
        let _root_id = tooltip.build(&mut ctx);

        let tooltip_root = ctx.layer_root(Layer::Tooltip);
        let scene = ctx.scene();
        let layer_node = scene.get_node(tooltip_root).unwrap();
        assert!(
            !layer_node.children.is_empty(),
            "Tooltip layer should have children after building a Tooltip widget"
        );
    }

    #[test]
    fn test_tooltip_has_dark_background() {
        let mut ctx = WidgetContext::new_test();
        let tooltip = Tooltip::new("Info", widget_core::Text::new("i"));
        let _root_id = tooltip.build(&mut ctx);

        let tooltip_root = ctx.layer_root(Layer::Tooltip);
        let scene = ctx.scene();
        let layer_node = scene.get_node(tooltip_root).unwrap();

        // The first child of the tooltip layer root is our container
        let container_id = layer_node.children[0];
        let container = scene.get_node(container_id).unwrap();

        if let NodeContent::Styled { ref style } = container.content {
            assert!(
                !style.fills.is_empty(),
                "Tooltip container should have a fill"
            );
            if let Paint::Solid(color) = &style.fills[0] {
                assert!(
                    (color.x - TOOLTIP_BG.x).abs() < 0.01
                        && (color.y - TOOLTIP_BG.y).abs() < 0.01
                        && (color.z - TOOLTIP_BG.z).abs() < 0.01
                        && (color.w - TOOLTIP_BG.w).abs() < 0.01,
                    "Tooltip background should be dark (#313033), got {color:?}"
                );
            } else {
                panic!("Tooltip fill should be a solid paint");
            }
        } else {
            panic!("Tooltip container should be Styled content");
        }
    }

    #[test]
    fn test_tooltip_returns_target_id() {
        let mut ctx = WidgetContext::new_test();
        let target = widget_core::Text::new("Target");

        // Build target directly to get its expected node structure
        let mut ctx2 = WidgetContext::new_test();
        let direct_id = target.build(&mut ctx2);
        let direct_node = ctx2.scene().get_node(direct_id).unwrap();

        // Now build via Tooltip
        let tooltip = Tooltip::new("Tip", widget_core::Text::new("Target"));
        let tooltip_result = tooltip.build(&mut ctx);

        // The returned ID should be a content node, not a tooltip-layer node
        let content_root = ctx.layer_root(Layer::Content);
        let scene = ctx.scene();
        let content_node = scene.get_node(content_root).unwrap();
        assert!(
            content_node.children.contains(&tooltip_result),
            "Tooltip.build() should return the target's NodeId (content layer child)"
        );

        // Verify the node matches the target's content type
        let result_node = scene.get_node(tooltip_result).unwrap();
        assert!(
            matches!(result_node.content, NodeContent::Styled { .. }),
            "Returned node should be the Styled target, got {:?}",
            std::mem::discriminant(&result_node.content)
        );

        // The tooltip layer should also have content (side effect)
        let tooltip_root = ctx.layer_root(Layer::Tooltip);
        let tooltip_layer = scene.get_node(tooltip_root).unwrap();
        assert!(
            !tooltip_layer.children.is_empty(),
            "Tooltip layer should have children (the tooltip is a side effect)"
        );

        // Verify the direct target node has same content kind
        assert!(
            matches!(direct_node.content, NodeContent::Styled { .. }),
            "Direct build should also be Styled"
        );
    }

    #[test]
    fn test_tooltip_custom_font_size() {
        let mut ctx = WidgetContext::new_test();
        let tooltip = Tooltip::new("Custom size", widget_core::Text::new("X")).font_size(16.0);
        let _root_id = tooltip.build(&mut ctx);

        let tooltip_root = ctx.layer_root(Layer::Tooltip);
        let scene = ctx.scene();
        let layer_node = scene.get_node(tooltip_root).unwrap();
        let container_id = layer_node.children[0];
        let container = scene.get_node(container_id).unwrap();

        // Text is the child of the container
        let text_id = container.children[0];
        let text_node = scene.get_node(text_id).unwrap();

        if let NodeContent::Styled { ref style } = text_node.content {
            let text = style
                .text
                .as_ref()
                .expect("Tooltip text node should have TextContent");
            assert!(
                (text.font_size - 16.0).abs() < 0.01,
                "Font size should be 16.0, got {}",
                text.font_size
            );
        } else {
            panic!("Tooltip text node should be Styled content");
        }
    }

    #[test]
    fn test_tooltip_text_is_white() {
        let mut ctx = WidgetContext::new_test();
        let tooltip = Tooltip::new("White text", widget_core::Text::new("W"));
        let _root_id = tooltip.build(&mut ctx);

        let tooltip_root = ctx.layer_root(Layer::Tooltip);
        let scene = ctx.scene();
        let layer_node = scene.get_node(tooltip_root).unwrap();
        let container_id = layer_node.children[0];
        let container = scene.get_node(container_id).unwrap();
        let text_id = container.children[0];
        let text_node = scene.get_node(text_id).unwrap();

        if let NodeContent::Styled { ref style } = text_node.content {
            assert!(!style.fills.is_empty(), "Text should have a fill");
            if let Paint::Solid(color) = &style.fills[0] {
                assert!(
                    (color.x - 1.0).abs() < 0.01
                        && (color.y - 1.0).abs() < 0.01
                        && (color.z - 1.0).abs() < 0.01,
                    "Tooltip text should be white, got {color:?}"
                );
            } else {
                panic!("Text fill should be a solid paint");
            }
        } else {
            panic!("Text node should be Styled content");
        }
    }
}
