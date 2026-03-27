//! MD3 Dialog widget -- modal dialog rendered in the Dialog layer.

use crate::theme::MaterialTheme;
use glam::Vec4;
use layout_engine::{FlexDirection, FlexJustifyContent, FlexStyle};
use render_engine::node::NodeContent;
use render_engine::{NodeId, TextContent, VisualStyle};
use std::sync::Arc;
use widget_core::WidgetContext;
use widget_core::layer::Layer;
use widget_core::widget_trait::Widget;

// ---------------------------------------------------------------------------
// MD3 fallback colors
// ---------------------------------------------------------------------------

/// MD3 surface (#FEF7FF) -- dialog background.
const FALLBACK_SURFACE: Vec4 = Vec4::new(0.996, 0.969, 1.0, 1.0);

/// MD3 on-surface (#1D1B20) -- title / primary text.
const FALLBACK_ON_SURFACE: Vec4 = Vec4::new(0.114, 0.106, 0.125, 1.0);

/// MD3 dialog corner radius in dp (large).
const DIALOG_CORNER_RADIUS: f32 = 28.0;

/// Default dialog width in dp (MD3 minimum).
const DEFAULT_WIDTH: f32 = 280.0;

/// MD3 dialog padding in dp.
const DIALOG_PADDING: f32 = 24.0;

/// MD3 headline-small font size in dp.
const TITLE_FONT_SIZE: f32 = 24.0;

/// MD3 actions row gap in dp.
const ACTIONS_GAP: f32 = 8.0;

/// MD3 actions row top margin in dp.
const ACTIONS_MARGIN_TOP: f32 = 24.0;

/// MD3 Dialog -- modal overlay with scrim, title, content, and action buttons.
///
/// Renders in the [`Layer::Dialog`] layer with a semi-transparent scrim
/// (backdrop) behind it. The dialog container has a surface-colored background,
/// large corner radius (28dp per MD3), and a column layout with 24dp padding.
///
/// # Structure
///
/// 1. **Scrim** -- semi-transparent black overlay (alpha 0.32)
/// 2. **Dialog container** -- surface-colored, rounded rectangle
///    - Optional title (headline-small, 24dp)
///    - Optional content widget
///    - Action buttons row (right-aligned)
///
/// # Example
///
/// ```rust,no_run
/// use material_ui::feedback::Dialog;
/// use widget_core::{Text, Button};
///
/// let dialog = Dialog::new()
///     .title("Confirm deletion")
///     .content(Text::new("Are you sure you want to delete this item?"))
///     .action(Button::new("Cancel"))
///     .action(Button::new("Delete").primary())
///     .on_close(|| println!("dialog dismissed"));
/// ```
pub struct Dialog {
    title: Option<String>,
    content: Option<Box<dyn Widget>>,
    actions: Vec<Box<dyn Widget>>,
    on_close: Option<Arc<dyn Fn() + Send + Sync>>,
    width: f32,
}

impl Dialog {
    /// Create a new empty dialog with MD3 defaults.
    pub fn new() -> Self {
        Self {
            title: None,
            content: None,
            actions: Vec::new(),
            on_close: None,
            width: DEFAULT_WIDTH,
        }
    }

    /// Set the dialog title (headline-small typography, 24dp).
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Set the dialog body content widget.
    pub fn content(mut self, widget: impl Widget + 'static) -> Self {
        self.content = Some(Box::new(widget));
        self
    }

    /// Add an action button to the dialog (e.g., Cancel, OK).
    ///
    /// Actions are displayed in a right-aligned horizontal row at the
    /// bottom of the dialog, in the order they are added.
    pub fn action(mut self, widget: impl Widget + 'static) -> Self {
        self.actions.push(Box::new(widget));
        self
    }

    /// Set the on-close callback, invoked when the scrim is clicked.
    ///
    /// If not set, clicking the scrim does nothing (non-dismissible dialog).
    pub fn on_close(mut self, f: impl Fn() + Send + Sync + 'static) -> Self {
        self.on_close = Some(Arc::new(f));
        self
    }

    /// Override the dialog width in dp (default: 280.0, the MD3 minimum).
    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }
}

impl Default for Dialog {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for Dialog {
    fn build(&self, ctx: &mut WidgetContext) -> NodeId {
        // -- Resolve theme colors --
        let theme = ctx.get_extension::<MaterialTheme>().cloned();
        let surface = theme
            .as_ref()
            .map(|t| t.color.surface)
            .unwrap_or(FALLBACK_SURFACE);
        let on_surface = theme
            .as_ref()
            .map(|t| t.color.on_surface)
            .unwrap_or(FALLBACK_ON_SURFACE);

        // -- 1. Scrim/Backdrop in Dialog layer --
        let scrim_id = ctx.add_to_layer(
            Layer::Dialog,
            NodeContent::SolidColor {
                color: render_engine::Color::rgba(0.0, 0.0, 0.0, 0.32),
            },
        );
        if let Some(ref callback) = self.on_close {
            ctx.add_clickable(scrim_id, callback.clone());
        }

        // -- 2. Dialog container in Dialog layer --
        let container_style = VisualStyle::new()
            .solid_fill(surface)
            .corner_radius(DIALOG_CORNER_RADIUS);

        let dialog_container = ctx.add_to_layer(
            Layer::Dialog,
            NodeContent::Styled {
                style: Box::new(container_style),
            },
        );

        ctx.set_layout_style(
            dialog_container,
            FlexStyle {
                direction: FlexDirection::Column,
                width: Some(self.width),
                padding_left: DIALOG_PADDING,
                padding_right: DIALOG_PADDING,
                padding_top: DIALOG_PADDING,
                padding_bottom: DIALOG_PADDING,
                ..Default::default()
            },
        );

        // -- 2a. Title (optional) --
        if let Some(ref title_text) = self.title {
            let title_style = VisualStyle::new()
                .solid_fill(on_surface)
                .text(TextContent::new(title_text.clone(), TITLE_FONT_SIZE));

            ctx.create_node(
                dialog_container,
                NodeContent::Styled {
                    style: Box::new(title_style),
                },
            );
        }

        // -- 2b. Content area (optional) --
        if let Some(ref content_widget) = self.content {
            let content_id = content_widget.build(ctx);
            ctx.reparent_to(content_id, dialog_container);
        }

        // -- 2c. Actions row (if any) --
        if !self.actions.is_empty() {
            // Create a row container for actions, right-aligned with gap
            let actions_row_style = VisualStyle::new();
            let actions_row = ctx.create_node(
                dialog_container,
                NodeContent::Styled {
                    style: Box::new(actions_row_style),
                },
            );

            ctx.set_layout_style(
                actions_row,
                FlexStyle {
                    direction: FlexDirection::Row,
                    justify_content: FlexJustifyContent::End,
                    gap: ACTIONS_GAP,
                    padding_top: ACTIONS_MARGIN_TOP,
                    ..Default::default()
                },
            );

            for action in &self.actions {
                let action_id = action.build(ctx);
                ctx.reparent_to(action_id, actions_row);
            }
        }

        // Return the dialog container NodeId (not the scrim)
        dialog_container
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use render_engine::Paint;
    use std::sync::atomic::{AtomicBool, Ordering};

    #[test]
    fn test_dialog_builds_in_dialog_layer() {
        let mut ctx = WidgetContext::new_test();
        let dialog = Dialog::new();
        let _dialog_id = dialog.build(&mut ctx);

        let dialog_root = ctx.layer_root(Layer::Dialog);
        let scene = ctx.scene();
        let dialog_node = scene.get_node(dialog_root).unwrap();
        assert!(
            !dialog_node.children.is_empty(),
            "Dialog layer should have children after building a Dialog"
        );
    }

    #[test]
    fn test_dialog_has_scrim() {
        let mut ctx = WidgetContext::new_test();
        let dialog = Dialog::new();
        let _dialog_id = dialog.build(&mut ctx);

        let dialog_root = ctx.layer_root(Layer::Dialog);
        let scene = ctx.scene();
        let layer_node = scene.get_node(dialog_root).unwrap();

        // Dialog layer should have at least 2 children: scrim + container
        assert!(
            layer_node.children.len() >= 2,
            "Dialog layer should have at least 2 children (scrim + container), got {}",
            layer_node.children.len()
        );

        // First child should be the scrim (SolidColor)
        let scrim_id = layer_node.children[0];
        let scrim_node = scene.get_node(scrim_id).unwrap();
        assert!(
            matches!(scrim_node.content, NodeContent::SolidColor { .. }),
            "First child of Dialog layer should be SolidColor scrim"
        );
    }

    #[test]
    fn test_dialog_scrim_opacity() {
        let mut ctx = WidgetContext::new_test();
        let dialog = Dialog::new();
        let _dialog_id = dialog.build(&mut ctx);

        let dialog_root = ctx.layer_root(Layer::Dialog);
        let scene = ctx.scene();
        let layer_node = scene.get_node(dialog_root).unwrap();
        let scrim_id = layer_node.children[0];
        let scrim_node = scene.get_node(scrim_id).unwrap();

        if let NodeContent::SolidColor { color } = &scrim_node.content {
            assert!(
                (color.a() - 0.32).abs() < 0.01,
                "Scrim alpha should be ~0.32, got {}",
                color.a()
            );
            assert!(
                color.r() < 0.01,
                "Scrim red should be ~0, got {}",
                color.r()
            );
            assert!(
                color.g() < 0.01,
                "Scrim green should be ~0, got {}",
                color.g()
            );
            assert!(
                color.b() < 0.01,
                "Scrim blue should be ~0, got {}",
                color.b()
            );
        } else {
            panic!("Scrim should be SolidColor content");
        }
    }

    #[test]
    fn test_dialog_with_title() {
        let mut ctx = WidgetContext::new_test();
        let dialog = Dialog::new().title("Confirm");
        let dialog_id = dialog.build(&mut ctx);

        let scene = ctx.scene();
        let container = scene.get_node(dialog_id).unwrap();

        // Container should have a title child
        assert!(
            !container.children.is_empty(),
            "Dialog with title should have children"
        );

        // First child should be the title node with text content
        let title_id = container.children[0];
        let title_node = scene.get_node(title_id).unwrap();
        if let NodeContent::Styled { ref style } = title_node.content {
            let text = style
                .text
                .as_ref()
                .expect("Title node should have TextContent");
            assert_eq!(text.text, "Confirm", "Title text should be 'Confirm'");
            assert!(
                (text.font_size - TITLE_FONT_SIZE).abs() < 0.01,
                "Title font size should be {TITLE_FONT_SIZE}, got {}",
                text.font_size
            );
        } else {
            panic!("Title node should be Styled content");
        }
    }

    #[test]
    fn test_dialog_with_content() {
        let mut ctx = WidgetContext::new_test();
        let dialog = Dialog::new()
            .title("Alert")
            .content(widget_core::Text::new("Are you sure?"));
        let dialog_id = dialog.build(&mut ctx);

        let scene = ctx.scene();
        let container = scene.get_node(dialog_id).unwrap();

        // Should have title + content = 2 children
        assert!(
            container.children.len() >= 2,
            "Dialog with title and content should have at least 2 children, got {}",
            container.children.len()
        );
    }

    #[test]
    fn test_dialog_with_actions() {
        let mut ctx = WidgetContext::new_test();
        let dialog = Dialog::new()
            .action(widget_core::Button::new("Cancel"))
            .action(widget_core::Button::new("OK"));
        let dialog_id = dialog.build(&mut ctx);

        let scene = ctx.scene();
        let container = scene.get_node(dialog_id).unwrap();

        // Container should have 1 child: the actions row
        assert!(
            !container.children.is_empty(),
            "Dialog with actions should have children"
        );

        // The actions row is the last child
        let actions_row_id = *container.children.last().unwrap();
        let actions_row = scene.get_node(actions_row_id).unwrap();

        // Actions row should have 2 children (Cancel + OK)
        assert_eq!(
            actions_row.children.len(),
            2,
            "Actions row should have 2 children, got {}",
            actions_row.children.len()
        );
    }

    #[test]
    fn test_dialog_scrim_click_triggers_on_close() {
        let closed = Arc::new(AtomicBool::new(false));
        let closed_clone = closed.clone();

        let mut ctx = WidgetContext::new_test();
        let dialog = Dialog::new().on_close(move || {
            closed_clone.store(true, Ordering::SeqCst);
        });
        let _dialog_id = dialog.build(&mut ctx);

        // The scrim is the first child of the Dialog layer root
        let dialog_root = ctx.layer_root(Layer::Dialog);
        let scene = ctx.scene();
        let layer_node = scene.get_node(dialog_root).unwrap();
        let scrim_id = layer_node.children[0];

        assert!(
            ctx.has_clickable(scrim_id),
            "Scrim should have a click handler when on_close is set"
        );

        ctx.trigger_click(scrim_id);

        assert!(
            closed.load(Ordering::SeqCst),
            "Clicking the scrim should trigger the on_close callback"
        );
    }

    #[test]
    fn test_dialog_scrim_no_handler_without_on_close() {
        let mut ctx = WidgetContext::new_test();
        let dialog = Dialog::new();
        let _dialog_id = dialog.build(&mut ctx);

        let dialog_root = ctx.layer_root(Layer::Dialog);
        let scene = ctx.scene();
        let layer_node = scene.get_node(dialog_root).unwrap();
        let scrim_id = layer_node.children[0];

        assert!(
            !ctx.has_clickable(scrim_id),
            "Scrim without on_close should not have a click handler"
        );
    }

    #[test]
    fn test_dialog_returns_container_not_scrim() {
        let mut ctx = WidgetContext::new_test();
        let dialog = Dialog::new();
        let dialog_id = dialog.build(&mut ctx);

        let scene = ctx.scene();
        let node = scene.get_node(dialog_id).unwrap();

        // The returned node should be the styled container, not the scrim
        assert!(
            matches!(node.content, NodeContent::Styled { .. }),
            "Dialog.build() should return the Styled container, not the SolidColor scrim"
        );
    }

    #[test]
    fn test_dialog_container_has_surface_background() {
        let mut ctx = WidgetContext::new_test();
        let dialog = Dialog::new();
        let dialog_id = dialog.build(&mut ctx);

        let scene = ctx.scene();
        let node = scene.get_node(dialog_id).unwrap();

        if let NodeContent::Styled { ref style } = node.content {
            assert!(
                !style.fills.is_empty(),
                "Dialog container should have a fill"
            );
            if let Paint::Solid(color) = &style.fills[0] {
                assert!(
                    (color.x - FALLBACK_SURFACE.x).abs() < 0.01
                        && (color.y - FALLBACK_SURFACE.y).abs() < 0.01
                        && (color.z - FALLBACK_SURFACE.z).abs() < 0.01
                        && (color.w - FALLBACK_SURFACE.w).abs() < 0.01,
                    "Dialog background should match surface color, got {color:?}"
                );
            } else {
                panic!("Dialog fill should be a solid paint");
            }
        } else {
            panic!("Dialog container should be Styled content");
        }
    }

    #[test]
    fn test_dialog_container_has_corner_radius() {
        let mut ctx = WidgetContext::new_test();
        let dialog = Dialog::new();
        let dialog_id = dialog.build(&mut ctx);

        let scene = ctx.scene();
        let node = scene.get_node(dialog_id).unwrap();

        if let NodeContent::Styled { ref style } = node.content {
            let radii = &style.corner_radii;
            assert!(
                (radii.top_left - DIALOG_CORNER_RADIUS).abs() < 0.01
                    && (radii.top_right - DIALOG_CORNER_RADIUS).abs() < 0.01
                    && (radii.bottom_right - DIALOG_CORNER_RADIUS).abs() < 0.01
                    && (radii.bottom_left - DIALOG_CORNER_RADIUS).abs() < 0.01,
                "Dialog corner radii should be {DIALOG_CORNER_RADIUS}, got {radii:?}"
            );
        } else {
            panic!("Dialog container should be Styled content");
        }
    }

    #[test]
    fn test_dialog_default_width() {
        let dialog = Dialog::new();
        assert!(
            (dialog.width - DEFAULT_WIDTH).abs() < f32::EPSILON,
            "Default dialog width should be {DEFAULT_WIDTH}, got {}",
            dialog.width
        );
    }

    #[test]
    fn test_dialog_custom_width() {
        let mut ctx = WidgetContext::new_test();
        let dialog = Dialog::new().width(400.0);
        let dialog_id = dialog.build(&mut ctx);

        // Should build successfully
        assert!(
            ctx.scene().get_node(dialog_id).is_some(),
            "Dialog with custom width should build successfully"
        );
        assert!(
            (dialog.width - 400.0).abs() < f32::EPSILON,
            "Custom width should be 400.0, got {}",
            dialog.width
        );
    }

    #[test]
    fn test_dialog_default_is_same_as_new() {
        let a = Dialog::new();
        let b = Dialog::default();
        assert!((a.width - b.width).abs() < f32::EPSILON);
        assert!(a.title.is_none());
        assert!(b.title.is_none());
        assert!(a.content.is_none());
        assert!(b.content.is_none());
        assert!(a.actions.is_empty());
        assert!(b.actions.is_empty());
        assert!(a.on_close.is_none());
        assert!(b.on_close.is_none());
    }

    #[test]
    fn test_dialog_with_theme() {
        let mut ctx = WidgetContext::new_test();
        let theme = MaterialTheme::from_seed(Vec4::new(0.4, 0.2, 0.8, 1.0));
        ctx.set_extension(theme.clone());

        let dialog = Dialog::new().title("Themed");
        let dialog_id = dialog.build(&mut ctx);

        let scene = ctx.scene();
        let node = scene.get_node(dialog_id).unwrap();

        if let NodeContent::Styled { ref style } = node.content {
            if let Paint::Solid(color) = &style.fills[0] {
                // Should use theme surface color, not fallback
                assert!(
                    (color.x - theme.color.surface.x).abs() < 0.01
                        && (color.y - theme.color.surface.y).abs() < 0.01
                        && (color.z - theme.color.surface.z).abs() < 0.01,
                    "Dialog with theme should use theme surface color, got {color:?}"
                );
            } else {
                panic!("Dialog fill should be solid");
            }
        } else {
            panic!("Dialog container should be Styled");
        }
    }

    #[test]
    fn test_dialog_full_composition() {
        let mut ctx = WidgetContext::new_test();
        let dialog = Dialog::new()
            .title("Delete Item")
            .content(widget_core::Text::new("This action cannot be undone."))
            .action(widget_core::Button::new("Cancel"))
            .action(widget_core::Button::new("Delete"));
        let dialog_id = dialog.build(&mut ctx);

        let scene = ctx.scene();
        let container = scene.get_node(dialog_id).unwrap();

        // Should have 3 children: title + content + actions row
        assert_eq!(
            container.children.len(),
            3,
            "Full dialog should have 3 children (title + content + actions row), got {}",
            container.children.len()
        );

        // Last child is actions row with 2 buttons
        let actions_row_id = container.children[2];
        let actions_row = scene.get_node(actions_row_id).unwrap();
        assert_eq!(
            actions_row.children.len(),
            2,
            "Actions row should have 2 buttons, got {}",
            actions_row.children.len()
        );
    }

    #[test]
    fn test_dialog_container_is_child_of_dialog_root() {
        let mut ctx = WidgetContext::new_test();
        let dialog = Dialog::new();
        let dialog_id = dialog.build(&mut ctx);

        let dialog_root = ctx.layer_root(Layer::Dialog);
        let scene = ctx.scene();
        let layer_node = scene.get_node(dialog_root).unwrap();

        assert!(
            layer_node.children.contains(&dialog_id),
            "Dialog container should be a direct child of the Dialog layer root"
        );
    }
}
