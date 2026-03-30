//! MD3 Snackbar widget -- notification toast rendered in the Notification layer.

use crate::theme::MaterialTheme;
use glam::Vec4;
use layout_engine::{FlexAlign, FlexDirection, FlexStyle};
use render_engine::node::NodeContent;
use render_engine::{NodeId, TextContent, VisualStyle};
use std::sync::Arc;
use widget_core::Layer;
use widget_core::Widget;
use widget_core::WidgetContext;

// ---------------------------------------------------------------------------
// MD3 fallback colors (inverse palette for snackbar)
// ---------------------------------------------------------------------------

/// MD3 inverse-surface (#313033) -- snackbar background.
const FALLBACK_INVERSE_SURFACE: Vec4 = Vec4::new(0.192, 0.188, 0.200, 1.0);

/// MD3 inverse-on-surface (#F5EFF7) -- snackbar message text.
const FALLBACK_INVERSE_ON_SURFACE: Vec4 = Vec4::new(0.961, 0.937, 0.969, 1.0);

/// MD3 inverse-primary (#D0BCFF) -- snackbar action button text.
const FALLBACK_INVERSE_PRIMARY: Vec4 = Vec4::new(0.816, 0.737, 1.0, 1.0);

/// MD3 snackbar corner radius in dp.
const SNACKBAR_CORNER_RADIUS: f32 = 4.0;

/// MD3 snackbar minimum height in dp.
const SNACKBAR_MIN_HEIGHT: f32 = 48.0;

/// MD3 snackbar horizontal padding in dp.
const SNACKBAR_PADDING_H: f32 = 16.0;

/// MD3 snackbar vertical padding in dp.
const SNACKBAR_PADDING_V: f32 = 14.0;

/// MD3 snackbar row gap in dp.
const SNACKBAR_GAP: f32 = 8.0;

/// MD3 snackbar body text font size in dp.
const MESSAGE_FONT_SIZE: f32 = 14.0;

/// MD3 snackbar action label font size in dp.
const ACTION_FONT_SIZE: f32 = 14.0;

/// Dismiss button font size in dp.
const DISMISS_FONT_SIZE: f32 = 14.0;

/// MD3 Snackbar -- notification toast overlay.
///
/// Renders in the [`Layer::Notification`] layer (z=3) with an inverse-surface
/// background, providing brief feedback about an operation. Supports an
/// optional action button and a dismiss ("X") control.
///
/// # Structure
///
/// 1. **Container** -- inverse-surface background, rounded rectangle (4dp radius)
///    - Message text (inverse-on-surface color)
///    - Optional action label (inverse-primary color, uppercase, clickable)
///    - Optional dismiss button ("✕", clickable)
///
/// # Example
///
/// ```rust,no_run
/// use material_ui::feedback::Snackbar;
///
/// let snackbar = Snackbar::new("Item deleted")
///     .action("Undo", || println!("undo!"))
///     .on_dismiss(|| println!("dismissed"));
/// ```
pub struct Snackbar {
    message: String,
    action_label: Option<String>,
    on_action: Option<Arc<dyn Fn() + Send + Sync>>,
    on_dismiss: Option<Arc<dyn Fn() + Send + Sync>>,
}

impl Snackbar {
    /// Create a new snackbar with the given message text.
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            action_label: None,
            on_action: None,
            on_dismiss: None,
        }
    }

    /// Add an action button with a label and callback.
    ///
    /// The label is rendered in inverse-primary color and uppercased per MD3 spec.
    pub fn action(
        mut self,
        label: impl Into<String>,
        f: impl Fn() + Send + Sync + 'static,
    ) -> Self {
        self.action_label = Some(label.into());
        self.on_action = Some(Arc::new(f));
        self
    }

    /// Set a dismiss callback. When set, a "✕" close button appears at the
    /// right edge of the snackbar.
    pub fn on_dismiss(mut self, f: impl Fn() + Send + Sync + 'static) -> Self {
        self.on_dismiss = Some(Arc::new(f));
        self
    }
}

impl Widget for Snackbar {
    fn build(&self, ctx: &mut WidgetContext) -> NodeId {
        // -- Resolve theme colors --
        let theme = ctx.get_extension::<MaterialTheme>().cloned();
        let bg_color = theme
            .as_ref()
            .map(|t| t.color.inverse_surface)
            .unwrap_or(FALLBACK_INVERSE_SURFACE);
        let text_color = theme
            .as_ref()
            .map(|t| t.color.inverse_on_surface)
            .unwrap_or(FALLBACK_INVERSE_ON_SURFACE);
        let action_color = theme
            .as_ref()
            .map(|t| t.color.inverse_primary)
            .unwrap_or(FALLBACK_INVERSE_PRIMARY);

        // -- 1. Snackbar container in Notification layer --
        let container_style = VisualStyle::new()
            .solid_fill(bg_color)
            .corner_radius(SNACKBAR_CORNER_RADIUS);

        let container = ctx.add_to_layer(
            Layer::Notification,
            NodeContent::Styled {
                style: Box::new(container_style),
            },
        );

        ctx.set_layout_style(
            container,
            FlexStyle {
                direction: FlexDirection::Row,
                align_items: FlexAlign::Center,
                gap: SNACKBAR_GAP,
                min_height: Some(SNACKBAR_MIN_HEIGHT),
                padding_left: SNACKBAR_PADDING_H,
                padding_right: SNACKBAR_PADDING_H,
                padding_top: SNACKBAR_PADDING_V,
                padding_bottom: SNACKBAR_PADDING_V,
                ..Default::default()
            },
        );

        // -- 2. Message text --
        let message_style = VisualStyle::new()
            .solid_fill(text_color)
            .text(TextContent::new(self.message.clone(), MESSAGE_FONT_SIZE));

        ctx.create_node(
            container,
            NodeContent::Styled {
                style: Box::new(message_style),
            },
        );

        // -- 3. Action button (optional) --
        if let (Some(label), Some(callback)) = (&self.action_label, &self.on_action) {
            let action_style = VisualStyle::new()
                .solid_fill(action_color)
                .text(TextContent::new(label.to_uppercase(), ACTION_FONT_SIZE));

            let action_id = ctx.create_node(
                container,
                NodeContent::Styled {
                    style: Box::new(action_style),
                },
            );

            ctx.add_clickable(action_id, callback.clone());
        }

        // -- 4. Dismiss button (optional) --
        if let Some(ref callback) = self.on_dismiss {
            let dismiss_style = VisualStyle::new()
                .solid_fill(text_color)
                .text(TextContent::new("\u{2715}".to_string(), DISMISS_FONT_SIZE));

            let dismiss_id = ctx.create_node(
                container,
                NodeContent::Styled {
                    style: Box::new(dismiss_style),
                },
            );

            ctx.add_clickable(dismiss_id, callback.clone());
        }

        container
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use render_engine::Paint;
    use std::sync::atomic::{AtomicBool, Ordering};

    #[test]
    fn test_snackbar_builds_in_notification_layer() {
        let mut ctx = WidgetContext::new_test();
        let snackbar = Snackbar::new("Test message");
        let _snackbar_id = snackbar.build(&mut ctx);

        let notif_root = ctx.layer_root(Layer::Notification);
        let scene = ctx.scene();
        let notif_node = scene.get_node(notif_root).unwrap();
        assert!(
            !notif_node.children.is_empty(),
            "Notification layer should have children after building a Snackbar"
        );
    }

    #[test]
    fn test_snackbar_has_message() {
        let mut ctx = WidgetContext::new_test();
        let snackbar = Snackbar::new("Item deleted");
        let snackbar_id = snackbar.build(&mut ctx);

        let scene = ctx.scene();
        let container = scene.get_node(snackbar_id).unwrap();

        // Container should have at least 1 child (message text)
        assert!(
            !container.children.is_empty(),
            "Snackbar container should have at least a message child"
        );

        // First child is the message text node
        let msg_id = container.children[0];
        let msg_node = scene.get_node(msg_id).unwrap();
        if let NodeContent::Styled { ref style } = msg_node.content {
            let text = style
                .text
                .as_ref()
                .expect("Message node should have TextContent");
            assert_eq!(text.text, "Item deleted", "Message text should match");
            assert!(
                (text.font_size - MESSAGE_FONT_SIZE).abs() < 0.01,
                "Message font size should be {MESSAGE_FONT_SIZE}, got {}",
                text.font_size
            );
        } else {
            panic!("Message node should be Styled content");
        }
    }

    #[test]
    fn test_snackbar_with_action() {
        let mut ctx = WidgetContext::new_test();
        let snackbar = Snackbar::new("Archived").action("Undo", || {});
        let snackbar_id = snackbar.build(&mut ctx);

        let scene = ctx.scene();
        let container = scene.get_node(snackbar_id).unwrap();

        // Should have 2 children: message + action
        assert_eq!(
            container.children.len(),
            2,
            "Snackbar with action should have 2 children (message + action), got {}",
            container.children.len()
        );

        // Second child is the action text node (uppercase)
        let action_id = container.children[1];
        let action_node = scene.get_node(action_id).unwrap();
        if let NodeContent::Styled { ref style } = action_node.content {
            let text = style
                .text
                .as_ref()
                .expect("Action node should have TextContent");
            assert_eq!(text.text, "UNDO", "Action label should be uppercased");
        } else {
            panic!("Action node should be Styled content");
        }
    }

    #[test]
    fn test_snackbar_action_clickable() {
        let clicked = Arc::new(AtomicBool::new(false));
        let clicked_clone = clicked.clone();

        let mut ctx = WidgetContext::new_test();
        let snackbar = Snackbar::new("Deleted").action("Undo", move || {
            clicked_clone.store(true, Ordering::SeqCst);
        });
        let snackbar_id = snackbar.build(&mut ctx);

        let scene = ctx.scene();
        let container = scene.get_node(snackbar_id).unwrap();
        let action_id = container.children[1];

        assert!(
            ctx.has_clickable(action_id),
            "Action button should have a click handler"
        );

        ctx.trigger_click(action_id);
        assert!(
            clicked.load(Ordering::SeqCst),
            "Clicking the action should fire the callback"
        );
    }

    #[test]
    fn test_snackbar_with_dismiss() {
        let mut ctx = WidgetContext::new_test();
        let snackbar = Snackbar::new("Saved").on_dismiss(|| {});
        let snackbar_id = snackbar.build(&mut ctx);

        let scene = ctx.scene();
        let container = scene.get_node(snackbar_id).unwrap();

        // Should have 2 children: message + dismiss
        assert_eq!(
            container.children.len(),
            2,
            "Snackbar with dismiss should have 2 children (message + dismiss), got {}",
            container.children.len()
        );

        // Last child is the dismiss "✕" node
        let dismiss_id = *container.children.last().unwrap();
        let dismiss_node = scene.get_node(dismiss_id).unwrap();
        if let NodeContent::Styled { ref style } = dismiss_node.content {
            let text = style
                .text
                .as_ref()
                .expect("Dismiss node should have TextContent");
            assert_eq!(
                text.text, "\u{2715}",
                "Dismiss text should be the X character"
            );
        } else {
            panic!("Dismiss node should be Styled content");
        }

        assert!(
            ctx.has_clickable(dismiss_id),
            "Dismiss button should have a click handler"
        );
    }

    #[test]
    fn test_snackbar_dismiss_fires_callback() {
        let dismissed = Arc::new(AtomicBool::new(false));
        let dismissed_clone = dismissed.clone();

        let mut ctx = WidgetContext::new_test();
        let snackbar = Snackbar::new("Saved").on_dismiss(move || {
            dismissed_clone.store(true, Ordering::SeqCst);
        });
        let snackbar_id = snackbar.build(&mut ctx);

        let scene = ctx.scene();
        let container = scene.get_node(snackbar_id).unwrap();
        let dismiss_id = *container.children.last().unwrap();

        ctx.trigger_click(dismiss_id);
        assert!(
            dismissed.load(Ordering::SeqCst),
            "Clicking dismiss should fire the on_dismiss callback"
        );
    }

    #[test]
    fn test_snackbar_dark_background() {
        let mut ctx = WidgetContext::new_test();
        let snackbar = Snackbar::new("Test");
        let snackbar_id = snackbar.build(&mut ctx);

        let scene = ctx.scene();
        let node = scene.get_node(snackbar_id).unwrap();

        if let NodeContent::Styled { ref style } = node.content {
            assert!(
                !style.fills.is_empty(),
                "Snackbar container should have a fill"
            );
            if let Paint::Solid(color) = &style.fills[0] {
                assert!(
                    (color.x - FALLBACK_INVERSE_SURFACE.x).abs() < 0.01
                        && (color.y - FALLBACK_INVERSE_SURFACE.y).abs() < 0.01
                        && (color.z - FALLBACK_INVERSE_SURFACE.z).abs() < 0.01
                        && (color.w - FALLBACK_INVERSE_SURFACE.w).abs() < 0.01,
                    "Snackbar background should match inverse-surface color, got {color:?}"
                );
            } else {
                panic!("Snackbar fill should be a solid paint");
            }
        } else {
            panic!("Snackbar container should be Styled content");
        }
    }

    #[test]
    fn test_snackbar_corner_radius() {
        let mut ctx = WidgetContext::new_test();
        let snackbar = Snackbar::new("Test");
        let snackbar_id = snackbar.build(&mut ctx);

        let scene = ctx.scene();
        let node = scene.get_node(snackbar_id).unwrap();

        if let NodeContent::Styled { ref style } = node.content {
            let radii = &style.corner_radii;
            assert!(
                (radii.top_left - SNACKBAR_CORNER_RADIUS).abs() < 0.01
                    && (radii.top_right - SNACKBAR_CORNER_RADIUS).abs() < 0.01
                    && (radii.bottom_right - SNACKBAR_CORNER_RADIUS).abs() < 0.01
                    && (radii.bottom_left - SNACKBAR_CORNER_RADIUS).abs() < 0.01,
                "Snackbar corner radii should be {SNACKBAR_CORNER_RADIUS}, got {radii:?}"
            );
        } else {
            panic!("Snackbar container should be Styled content");
        }
    }

    #[test]
    fn test_snackbar_message_text_color() {
        let mut ctx = WidgetContext::new_test();
        let snackbar = Snackbar::new("Hello");
        let snackbar_id = snackbar.build(&mut ctx);

        let scene = ctx.scene();
        let container = scene.get_node(snackbar_id).unwrap();
        let msg_id = container.children[0];
        let msg_node = scene.get_node(msg_id).unwrap();

        if let NodeContent::Styled { ref style } = msg_node.content {
            assert!(!style.fills.is_empty(), "Message should have a fill");
            if let Paint::Solid(color) = &style.fills[0] {
                assert!(
                    (color.x - FALLBACK_INVERSE_ON_SURFACE.x).abs() < 0.01
                        && (color.y - FALLBACK_INVERSE_ON_SURFACE.y).abs() < 0.01
                        && (color.z - FALLBACK_INVERSE_ON_SURFACE.z).abs() < 0.01,
                    "Message text color should match inverse-on-surface, got {color:?}"
                );
            } else {
                panic!("Message fill should be solid");
            }
        } else {
            panic!("Message node should be Styled content");
        }
    }

    #[test]
    fn test_snackbar_action_color() {
        let mut ctx = WidgetContext::new_test();
        let snackbar = Snackbar::new("Deleted").action("Undo", || {});
        let snackbar_id = snackbar.build(&mut ctx);

        let scene = ctx.scene();
        let container = scene.get_node(snackbar_id).unwrap();
        let action_id = container.children[1];
        let action_node = scene.get_node(action_id).unwrap();

        if let NodeContent::Styled { ref style } = action_node.content {
            assert!(!style.fills.is_empty(), "Action should have a fill");
            if let Paint::Solid(color) = &style.fills[0] {
                assert!(
                    (color.x - FALLBACK_INVERSE_PRIMARY.x).abs() < 0.01
                        && (color.y - FALLBACK_INVERSE_PRIMARY.y).abs() < 0.01
                        && (color.z - FALLBACK_INVERSE_PRIMARY.z).abs() < 0.01,
                    "Action text color should match inverse-primary, got {color:?}"
                );
            } else {
                panic!("Action fill should be solid");
            }
        } else {
            panic!("Action node should be Styled content");
        }
    }

    #[test]
    fn test_snackbar_container_is_child_of_notification_root() {
        let mut ctx = WidgetContext::new_test();
        let snackbar = Snackbar::new("Test");
        let snackbar_id = snackbar.build(&mut ctx);

        let notif_root = ctx.layer_root(Layer::Notification);
        let scene = ctx.scene();
        let layer_node = scene.get_node(notif_root).unwrap();

        assert!(
            layer_node.children.contains(&snackbar_id),
            "Snackbar container should be a direct child of the Notification layer root"
        );
    }

    #[test]
    fn test_snackbar_with_action_and_dismiss() {
        let mut ctx = WidgetContext::new_test();
        let snackbar = Snackbar::new("Item moved")
            .action("Undo", || {})
            .on_dismiss(|| {});
        let snackbar_id = snackbar.build(&mut ctx);

        let scene = ctx.scene();
        let container = scene.get_node(snackbar_id).unwrap();

        // Should have 3 children: message + action + dismiss
        assert_eq!(
            container.children.len(),
            3,
            "Snackbar with action and dismiss should have 3 children, got {}",
            container.children.len()
        );
    }

    #[test]
    fn test_snackbar_with_theme() {
        let mut ctx = WidgetContext::new_test();
        let theme = MaterialTheme::from_seed(Vec4::new(0.4, 0.2, 0.8, 1.0));
        ctx.set_extension(theme.clone());

        let snackbar = Snackbar::new("Themed message");
        let snackbar_id = snackbar.build(&mut ctx);

        let scene = ctx.scene();
        let node = scene.get_node(snackbar_id).unwrap();

        if let NodeContent::Styled { ref style } = node.content {
            if let Paint::Solid(color) = &style.fills[0] {
                // Should use theme inverse_surface color, not fallback
                assert!(
                    (color.x - theme.color.inverse_surface.x).abs() < 0.01
                        && (color.y - theme.color.inverse_surface.y).abs() < 0.01
                        && (color.z - theme.color.inverse_surface.z).abs() < 0.01,
                    "Snackbar with theme should use theme inverse_surface color, got {color:?}"
                );
            } else {
                panic!("Snackbar fill should be solid");
            }
        } else {
            panic!("Snackbar container should be Styled");
        }
    }

    #[test]
    fn test_snackbar_message_only_has_one_child() {
        let mut ctx = WidgetContext::new_test();
        let snackbar = Snackbar::new("Simple message");
        let snackbar_id = snackbar.build(&mut ctx);

        let scene = ctx.scene();
        let container = scene.get_node(snackbar_id).unwrap();

        assert_eq!(
            container.children.len(),
            1,
            "Snackbar with only a message should have 1 child, got {}",
            container.children.len()
        );
    }
}
