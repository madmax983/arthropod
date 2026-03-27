//! MD3 Drawer widget -- side navigation panel.
//!
//! Standard variant renders in the [`Layer::Content`] layer as a persistent
//! sidebar. Modal variant renders in the [`Layer::Dialog`] layer with a
//! semi-transparent scrim backdrop, following the same overlay pattern as
//! [`crate::feedback::Dialog`].

use crate::theme::MaterialTheme;
use glam::Vec4;
use layout_engine::{FlexDirection, FlexStyle};
use render_engine::node::NodeContent;
use render_engine::{NodeId, TextContent, VisualStyle};
use std::sync::Arc;
use widget_core::WidgetContext;
use widget_core::layer::Layer;
use widget_core::widget_trait::Widget;

// ---------------------------------------------------------------------------
// MD3 fallback colors
// ---------------------------------------------------------------------------

/// MD3 surface (#FEF7FF) -- drawer background.
const FALLBACK_SURFACE: Vec4 = Vec4::new(0.996, 0.969, 1.0, 1.0);

/// MD3 secondary-container -- selected item background.
const FALLBACK_SECONDARY_CONTAINER: Vec4 = Vec4::new(0.906, 0.831, 0.996, 1.0);

/// MD3 on-secondary-container -- selected item text.
const FALLBACK_ON_SECONDARY_CONTAINER: Vec4 = Vec4::new(0.114, 0.012, 0.329, 1.0);

/// MD3 on-surface-variant -- unselected item text.
const FALLBACK_ON_SURFACE_VARIANT: Vec4 = Vec4::new(0.286, 0.271, 0.310, 1.0);

/// MD3 on-surface (#1D1B20) -- header text.
const FALLBACK_ON_SURFACE: Vec4 = Vec4::new(0.114, 0.106, 0.125, 1.0);

// ---------------------------------------------------------------------------
// Dimension constants (dp)
// ---------------------------------------------------------------------------

/// Default drawer width in dp (MD3 standard).
const DEFAULT_WIDTH: f32 = 360.0;

/// Item height in dp.
const ITEM_HEIGHT: f32 = 56.0;

/// Horizontal padding for items in dp.
const ITEM_PADDING_HORIZONTAL: f32 = 24.0;

/// MD3 active indicator corner radius in dp (pill-shaped).
const ITEM_CORNER_RADIUS: f32 = 28.0;

/// Header height in dp.
const HEADER_HEIGHT: f32 = 56.0;

/// Header horizontal padding in dp.
const HEADER_PADDING_HORIZONTAL: f32 = 28.0;

/// Header font size (headline_small) in dp.
const HEADER_FONT_SIZE: f32 = 24.0;

/// Label font size in dp.
const LABEL_FONT_SIZE: f32 = 14.0;

/// Disabled alpha (MD3 standard).
const DISABLED_ALPHA: f32 = 0.38;

// ---------------------------------------------------------------------------
// DrawerVariant
// ---------------------------------------------------------------------------

/// Controls how the drawer is presented in the layer tree.
///
/// - [`Standard`](DrawerVariant::Standard) -- always visible, placed in the
///   [`Layer::Content`] layer alongside normal widgets.
/// - [`Modal`](DrawerVariant::Modal) -- overlays the screen in the
///   [`Layer::Dialog`] layer with a semi-transparent scrim.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DrawerVariant {
    /// Always visible, rendered in the Content layer.
    #[default]
    Standard,
    /// Overlay with scrim, rendered in the Dialog layer.
    Modal,
}

// ---------------------------------------------------------------------------
// DrawerItem
// ---------------------------------------------------------------------------

/// A single navigation entry inside a [`Drawer`].
///
/// Each item has a label and optional click callback. Items can be marked as
/// `selected` (active indicator) or `disabled` (reduced opacity, no click).
///
/// # Example
///
/// ```rust,no_run
/// use material_ui::navigation::DrawerItem;
///
/// let item = DrawerItem::new("Inbox")
///     .selected()
///     .on_click(|| println!("Inbox clicked"));
/// ```
pub struct DrawerItem {
    /// Display text for this navigation entry.
    pub label: String,
    /// Click handler, invoked when the item is activated.
    pub on_click: Option<Arc<dyn Fn() + Send + Sync>>,
    /// Whether this item is currently selected (shows active indicator).
    pub selected: bool,
    /// Whether this item is disabled (reduced opacity, no interaction).
    pub disabled: bool,
}

impl DrawerItem {
    /// Create a new drawer item with the given label.
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            on_click: None,
            selected: false,
            disabled: false,
        }
    }

    /// Set the click callback for this item.
    pub fn on_click(mut self, f: impl Fn() + Send + Sync + 'static) -> Self {
        self.on_click = Some(Arc::new(f));
        self
    }

    /// Mark this item as selected (active indicator).
    pub fn selected(mut self) -> Self {
        self.selected = true;
        self
    }

    /// Mark this item as disabled.
    pub fn disabled(mut self) -> Self {
        self.disabled = true;
        self
    }
}

// ---------------------------------------------------------------------------
// Drawer
// ---------------------------------------------------------------------------

/// MD3 Drawer -- side navigation panel with selectable items.
///
/// The drawer can operate in two modes:
///
/// - **Standard** (default): Always visible in the [`Layer::Content`] layer as
///   a persistent sidebar. Typically placed on the left side of the layout.
/// - **Modal**: Overlays the UI in the [`Layer::Dialog`] layer with a
///   semi-transparent scrim. Clicking the scrim invokes the [`on_close`]
///   callback to dismiss the drawer.
///
/// # Structure
///
/// 1. (Modal only) **Scrim** -- black overlay at 0.32 alpha
/// 2. **Drawer container** -- column, surface background, fixed width
///    - Optional header (headline_small, 24dp font)
///    - Navigation items (56dp tall, pill-shaped active indicator)
///
/// # Example
///
/// ```rust,no_run
/// use material_ui::navigation::{Drawer, DrawerItem};
///
/// let drawer = Drawer::new(vec![
///     DrawerItem::new("Inbox").selected(),
///     DrawerItem::new("Outbox"),
///     DrawerItem::new("Trash"),
/// ])
/// .header("Mail")
/// .modal()
/// .on_close(|| println!("drawer dismissed"));
/// ```
///
/// [`on_close`]: Drawer::on_close
pub struct Drawer {
    items: Vec<DrawerItem>,
    variant: DrawerVariant,
    width: f32,
    header: Option<String>,
    on_close: Option<Arc<dyn Fn() + Send + Sync>>,
}

impl Drawer {
    /// Create a new drawer with the given items (default: [`DrawerVariant::Standard`]).
    pub fn new(items: Vec<DrawerItem>) -> Self {
        Self {
            items,
            variant: DrawerVariant::Standard,
            width: DEFAULT_WIDTH,
            header: None,
            on_close: None,
        }
    }

    /// Switch to the modal variant (Dialog layer with scrim).
    pub fn modal(mut self) -> Self {
        self.variant = DrawerVariant::Modal;
        self
    }

    /// Override the drawer width in dp (default: 360.0).
    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }

    /// Set an optional header title displayed at the top of the drawer.
    pub fn header(mut self, title: impl Into<String>) -> Self {
        self.header = Some(title.into());
        self
    }

    /// Set the on-close callback (modal variant: invoked on scrim click).
    pub fn on_close(mut self, f: impl Fn() + Send + Sync + 'static) -> Self {
        self.on_close = Some(Arc::new(f));
        self
    }

    // -- private helpers --

    /// Build the drawer container and its children, returning the container
    /// `NodeId`. The container is created under `layer`.
    fn build_container(
        &self,
        ctx: &mut WidgetContext,
        layer: Layer,
        colors: &DrawerColors,
    ) -> NodeId {
        let container_style = VisualStyle::new().solid_fill(colors.surface);

        let container = ctx.add_to_layer(
            layer,
            NodeContent::Styled {
                style: Box::new(container_style),
            },
        );

        ctx.set_layout_style(
            container,
            FlexStyle {
                direction: FlexDirection::Column,
                width: Some(self.width),
                ..Default::default()
            },
        );

        // -- Header (optional) --
        if let Some(ref header_text) = self.header {
            let header_style = VisualStyle::new()
                .solid_fill(colors.on_surface)
                .text(TextContent::new(header_text.clone(), HEADER_FONT_SIZE));

            let header_node = ctx.create_node(
                container,
                NodeContent::Styled {
                    style: Box::new(header_style),
                },
            );

            ctx.set_layout_style(
                header_node,
                FlexStyle {
                    height: Some(HEADER_HEIGHT),
                    padding_left: HEADER_PADDING_HORIZONTAL,
                    padding_right: HEADER_PADDING_HORIZONTAL,
                    ..Default::default()
                },
            );
        }

        // -- Items --
        for item in &self.items {
            let (bg_color, text_color) = if item.selected {
                (colors.secondary_container, colors.on_secondary_container)
            } else {
                (Vec4::new(0.0, 0.0, 0.0, 0.0), colors.on_surface_variant)
            };

            // Apply disabled alpha to the text color
            let text_color = if item.disabled {
                Vec4::new(text_color.x, text_color.y, text_color.z, DISABLED_ALPHA)
            } else {
                text_color
            };

            // Item container with background + corner radius
            let item_bg_style = VisualStyle::new()
                .solid_fill(bg_color)
                .corner_radius(ITEM_CORNER_RADIUS);

            let item_node = ctx.create_node(
                container,
                NodeContent::Styled {
                    style: Box::new(item_bg_style),
                },
            );

            ctx.set_layout_style(
                item_node,
                FlexStyle {
                    height: Some(ITEM_HEIGHT),
                    padding_left: ITEM_PADDING_HORIZONTAL,
                    padding_right: ITEM_PADDING_HORIZONTAL,
                    ..Default::default()
                },
            );

            // Label text inside the item container
            let label_style = VisualStyle::new()
                .solid_fill(text_color)
                .text(TextContent::new(item.label.clone(), LABEL_FONT_SIZE));

            ctx.create_node(
                item_node,
                NodeContent::Styled {
                    style: Box::new(label_style),
                },
            );

            // Click handler (skip if disabled)
            if !item.disabled
                && let Some(ref callback) = item.on_click
            {
                ctx.add_clickable(item_node, callback.clone());
            }
        }

        container
    }
}

/// Resolved color values for the drawer, extracted from the theme or fallbacks.
struct DrawerColors {
    surface: Vec4,
    on_surface: Vec4,
    secondary_container: Vec4,
    on_secondary_container: Vec4,
    on_surface_variant: Vec4,
}

impl Widget for Drawer {
    fn build(&self, ctx: &mut WidgetContext) -> NodeId {
        // -- Resolve theme colors --
        let theme = ctx.get_extension::<MaterialTheme>().cloned();
        let surface = theme
            .as_ref()
            .map(|t| t.color.surface)
            .unwrap_or(FALLBACK_SURFACE);
        let colors = DrawerColors {
            surface,
            on_surface: theme
                .as_ref()
                .map(|t| t.color.on_surface)
                .unwrap_or(FALLBACK_ON_SURFACE),
            secondary_container: theme
                .as_ref()
                .map(|t| t.color.secondary_container)
                .unwrap_or(FALLBACK_SECONDARY_CONTAINER),
            on_secondary_container: theme
                .as_ref()
                .map(|t| t.color.on_secondary_container)
                .unwrap_or(FALLBACK_ON_SECONDARY_CONTAINER),
            on_surface_variant: theme
                .as_ref()
                .map(|t| t.color.on_surface_variant)
                .unwrap_or(FALLBACK_ON_SURFACE_VARIANT),
        };

        match self.variant {
            DrawerVariant::Standard => self.build_container(ctx, Layer::Content, &colors),
            DrawerVariant::Modal => {
                // -- 1. Scrim in Dialog layer --
                let scrim_id = ctx.add_to_layer(
                    Layer::Dialog,
                    NodeContent::SolidColor {
                        color: render_engine::Color::rgba(0.0, 0.0, 0.0, 0.32),
                    },
                );
                if let Some(ref callback) = self.on_close {
                    ctx.add_clickable(scrim_id, callback.clone());
                }

                // -- 2. Drawer container in Dialog layer --
                self.build_container(ctx, Layer::Dialog, &colors)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use render_engine::Paint;
    use std::sync::atomic::{AtomicBool, Ordering};

    #[test]
    fn test_drawer_standard_builds() {
        let mut ctx = WidgetContext::new_test();
        let drawer = Drawer::new(vec![DrawerItem::new("Inbox")]);
        let drawer_id = drawer.build(&mut ctx);

        // Standard drawer should be in the Content layer
        let content_root = ctx.layer_root(Layer::Content);
        let scene = ctx.scene();
        let layer_node = scene.get_node(content_root).unwrap();
        assert!(
            layer_node.children.contains(&drawer_id),
            "Standard drawer should be a child of the Content layer root"
        );
    }

    #[test]
    fn test_drawer_modal_in_dialog_layer() {
        let mut ctx = WidgetContext::new_test();
        let drawer = Drawer::new(vec![DrawerItem::new("Inbox")]).modal();
        let _drawer_id = drawer.build(&mut ctx);

        let dialog_root = ctx.layer_root(Layer::Dialog);
        let scene = ctx.scene();
        let dialog_node = scene.get_node(dialog_root).unwrap();
        assert!(
            !dialog_node.children.is_empty(),
            "Dialog layer should have children after building a modal Drawer"
        );
    }

    #[test]
    fn test_drawer_items_count() {
        let mut ctx = WidgetContext::new_test();
        let drawer = Drawer::new(vec![
            DrawerItem::new("Inbox"),
            DrawerItem::new("Outbox"),
            DrawerItem::new("Trash"),
        ]);
        let drawer_id = drawer.build(&mut ctx);

        let scene = ctx.scene();
        let container = scene.get_node(drawer_id).unwrap();

        // 3 items, no header => 3 children
        assert_eq!(
            container.children.len(),
            3,
            "Drawer with 3 items (no header) should have 3 children, got {}",
            container.children.len()
        );
    }

    #[test]
    fn test_drawer_selected_item_styling() {
        let mut ctx = WidgetContext::new_test();
        let drawer = Drawer::new(vec![
            DrawerItem::new("Selected").selected(),
            DrawerItem::new("Unselected"),
        ]);
        let drawer_id = drawer.build(&mut ctx);

        let scene = ctx.scene();
        let container = scene.get_node(drawer_id).unwrap();

        // First item should be selected -> secondary_container background
        let selected_item_id = container.children[0];
        let selected_item = scene.get_node(selected_item_id).unwrap();

        if let NodeContent::Styled { ref style } = selected_item.content {
            assert!(!style.fills.is_empty(), "Selected item should have a fill");
            if let Paint::Solid(color) = &style.fills[0] {
                assert!(
                    (color.x - FALLBACK_SECONDARY_CONTAINER.x).abs() < 0.01
                        && (color.y - FALLBACK_SECONDARY_CONTAINER.y).abs() < 0.01
                        && (color.z - FALLBACK_SECONDARY_CONTAINER.z).abs() < 0.01,
                    "Selected item bg should match secondary_container, got {color:?}"
                );
            } else {
                panic!("Selected item fill should be solid");
            }
        } else {
            panic!("Selected item should be Styled content");
        }
    }

    #[test]
    fn test_drawer_item_click() {
        let clicked = Arc::new(AtomicBool::new(false));
        let clicked_clone = clicked.clone();

        let mut ctx = WidgetContext::new_test();
        let drawer = Drawer::new(vec![DrawerItem::new("Clickable").on_click(move || {
            clicked_clone.store(true, Ordering::SeqCst);
        })]);
        let drawer_id = drawer.build(&mut ctx);

        let scene = ctx.scene();
        let container = scene.get_node(drawer_id).unwrap();
        let item_id = container.children[0];

        assert!(
            ctx.has_clickable(item_id),
            "Item with on_click should have a click handler"
        );

        ctx.trigger_click(item_id);

        assert!(
            clicked.load(Ordering::SeqCst),
            "Clicking the item should trigger the on_click callback"
        );
    }

    #[test]
    fn test_drawer_modal_scrim_close() {
        let closed = Arc::new(AtomicBool::new(false));
        let closed_clone = closed.clone();

        let mut ctx = WidgetContext::new_test();
        let drawer = Drawer::new(vec![DrawerItem::new("Item")])
            .modal()
            .on_close(move || {
                closed_clone.store(true, Ordering::SeqCst);
            });
        let _drawer_id = drawer.build(&mut ctx);

        // Scrim is the first child of the Dialog layer root
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
    fn test_drawer_with_header() {
        let mut ctx = WidgetContext::new_test();
        let drawer = Drawer::new(vec![DrawerItem::new("Inbox")]).header("Mail");
        let drawer_id = drawer.build(&mut ctx);

        let scene = ctx.scene();
        let container = scene.get_node(drawer_id).unwrap();

        // Should have header + 1 item = 2 children
        assert_eq!(
            container.children.len(),
            2,
            "Drawer with header and 1 item should have 2 children, got {}",
            container.children.len()
        );

        // First child should be the header with text
        let header_id = container.children[0];
        let header_node = scene.get_node(header_id).unwrap();
        if let NodeContent::Styled { ref style } = header_node.content {
            let text = style
                .text
                .as_ref()
                .expect("Header node should have TextContent");
            assert_eq!(text.text, "Mail", "Header text should be 'Mail'");
            assert!(
                (text.font_size - HEADER_FONT_SIZE).abs() < 0.01,
                "Header font size should be {HEADER_FONT_SIZE}, got {}",
                text.font_size
            );
        } else {
            panic!("Header node should be Styled content");
        }
    }

    #[test]
    fn test_drawer_default_width() {
        let drawer = Drawer::new(vec![]);
        assert!(
            (drawer.width - DEFAULT_WIDTH).abs() < f32::EPSILON,
            "Default drawer width should be {DEFAULT_WIDTH}, got {}",
            drawer.width
        );
    }

    #[test]
    fn test_drawer_custom_width() {
        let drawer = Drawer::new(vec![]).width(280.0);
        assert!(
            (drawer.width - 280.0).abs() < f32::EPSILON,
            "Custom drawer width should be 280.0, got {}",
            drawer.width
        );
    }

    #[test]
    fn test_drawer_has_surface_background() {
        let mut ctx = WidgetContext::new_test();
        let drawer = Drawer::new(vec![DrawerItem::new("Item")]);
        let drawer_id = drawer.build(&mut ctx);

        let scene = ctx.scene();
        let node = scene.get_node(drawer_id).unwrap();

        if let NodeContent::Styled { ref style } = node.content {
            assert!(!style.fills.is_empty(), "Drawer should have a fill");
            if let Paint::Solid(color) = &style.fills[0] {
                assert!(
                    (color.x - FALLBACK_SURFACE.x).abs() < 0.01
                        && (color.y - FALLBACK_SURFACE.y).abs() < 0.01
                        && (color.z - FALLBACK_SURFACE.z).abs() < 0.01
                        && (color.w - FALLBACK_SURFACE.w).abs() < 0.01,
                    "Drawer background should match surface color, got {color:?}"
                );
            } else {
                panic!("Drawer fill should be a solid paint");
            }
        } else {
            panic!("Drawer node should be Styled content");
        }
    }

    #[test]
    fn test_drawer_modal_has_scrim() {
        let mut ctx = WidgetContext::new_test();
        let drawer = Drawer::new(vec![DrawerItem::new("Item")]).modal();
        let _drawer_id = drawer.build(&mut ctx);

        let dialog_root = ctx.layer_root(Layer::Dialog);
        let scene = ctx.scene();
        let layer_node = scene.get_node(dialog_root).unwrap();

        // Dialog layer should have at least 2 children: scrim + container
        assert!(
            layer_node.children.len() >= 2,
            "Modal drawer should have at least 2 children (scrim + container), got {}",
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
    fn test_drawer_disabled_item_no_click() {
        let clicked = Arc::new(AtomicBool::new(false));
        let clicked_clone = clicked.clone();

        let mut ctx = WidgetContext::new_test();
        let drawer = Drawer::new(vec![DrawerItem::new("Disabled").disabled().on_click(
            move || {
                clicked_clone.store(true, Ordering::SeqCst);
            },
        )]);
        let drawer_id = drawer.build(&mut ctx);

        let scene = ctx.scene();
        let container = scene.get_node(drawer_id).unwrap();
        let item_id = container.children[0];

        assert!(
            !ctx.has_clickable(item_id),
            "Disabled item should not have a click handler"
        );
    }

    #[test]
    fn test_drawer_standard_not_in_dialog_layer() {
        let mut ctx = WidgetContext::new_test();
        let drawer = Drawer::new(vec![DrawerItem::new("Item")]);
        let _drawer_id = drawer.build(&mut ctx);

        let dialog_root = ctx.layer_root(Layer::Dialog);
        let scene = ctx.scene();
        let dialog_node = scene.get_node(dialog_root).unwrap();

        assert!(
            dialog_node.children.is_empty(),
            "Standard drawer should NOT place anything in the Dialog layer"
        );
    }

    #[test]
    fn test_drawer_item_builder_pattern() {
        let item = DrawerItem::new("Test").selected().disabled();

        assert_eq!(item.label, "Test");
        assert!(item.selected);
        assert!(item.disabled);
        assert!(item.on_click.is_none());
    }

    #[test]
    fn test_drawer_variant_default_is_standard() {
        let drawer = Drawer::new(vec![]);
        assert_eq!(drawer.variant, DrawerVariant::Standard);
    }
}
