//! MD3 MenuItem widget -- single selectable row within a menu.

use crate::theme::MaterialTheme;
use glam::Vec4;
use layout_engine::{FlexAlign, FlexDirection, FlexStyle};
use render_engine::node::NodeContent;
use render_engine::{NodeId, TextContent, VisualStyle};
use std::sync::Arc;
use widget_core::WidgetContext;
use widget_core::widget_trait::Widget;

// ---------------------------------------------------------------------------
// MD3 fallback colors
// ---------------------------------------------------------------------------

/// MD3 on_surface (#1D1B20) -- label text.
const FALLBACK_ON_SURFACE: Vec4 = Vec4::new(0.114, 0.106, 0.125, 1.0);

/// MD3 on_surface_variant (#49454F) -- leading/icon text.
const FALLBACK_ON_SURFACE_VARIANT: Vec4 = Vec4::new(0.286, 0.271, 0.310, 1.0);

/// Default label font size in dp.
const LABEL_FONT_SIZE: f32 = 14.0;

/// Default leading text font size in dp.
const LEADING_FONT_SIZE: f32 = 18.0;

/// Menu item height in dp.
const ITEM_HEIGHT: f32 = 48.0;

/// Horizontal padding in dp.
const HORIZONTAL_PADDING: f32 = 12.0;

/// Gap between leading and label in dp.
const ITEM_GAP: f32 = 12.0;

/// Leading text slot width in dp.
const LEADING_WIDTH: f32 = 24.0;

/// MD3 MenuItem -- a single selectable row in a dropdown menu.
///
/// Renders as a horizontal row containing an optional leading text slot
/// and a label. Supports click handlers and a disabled state.
///
/// # Example
///
/// ```rust,no_run
/// use material_ui::navigation::MenuItem;
///
/// let item = MenuItem::new("Edit")
///     .leading("E")
///     .on_click(|| println!("Edit clicked"));
/// ```
pub struct MenuItem {
    label: String,
    on_click: Option<Arc<dyn Fn() + Send + Sync>>,
    disabled: bool,
    leading_text: Option<String>,
}

impl MenuItem {
    /// Create a new menu item with the given label.
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            on_click: None,
            disabled: false,
            leading_text: None,
        }
    }

    /// Set the click handler invoked when the item is selected.
    pub fn on_click(mut self, f: impl Fn() + Send + Sync + 'static) -> Self {
        self.on_click = Some(Arc::new(f));
        self
    }

    /// Disable interaction. The item renders normally but ignores clicks.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Set leading text (icon placeholder) displayed before the label.
    pub fn leading(mut self, text: impl Into<String>) -> Self {
        self.leading_text = Some(text.into());
        self
    }
}

impl Widget for MenuItem {
    fn build(&self, ctx: &mut WidgetContext) -> NodeId {
        // -- Resolve theme colors --
        let theme = ctx.get_extension::<MaterialTheme>().cloned();
        let on_surface = theme
            .as_ref()
            .map(|t| t.color.on_surface)
            .unwrap_or(FALLBACK_ON_SURFACE);
        let on_surface_variant = theme
            .as_ref()
            .map(|t| t.color.on_surface_variant)
            .unwrap_or(FALLBACK_ON_SURFACE_VARIANT);

        // -- Root row container (transparent background) --
        let root = ctx.create_node(ctx.root(), NodeContent::Empty);
        ctx.set_layout_style(
            root,
            FlexStyle {
                direction: FlexDirection::Row,
                height: Some(ITEM_HEIGHT),
                padding_left: HORIZONTAL_PADDING,
                padding_right: HORIZONTAL_PADDING,
                gap: ITEM_GAP,
                align_items: FlexAlign::Center,
                ..Default::default()
            },
        );

        // -- Hover state --
        ctx.add_hover_state(root);

        // -- Optional leading text --
        if let Some(ref leading) = self.leading_text {
            let leading_style = VisualStyle::new()
                .solid_fill(on_surface_variant)
                .text(TextContent::new(leading.clone(), LEADING_FONT_SIZE));

            let leading_node = ctx.create_node(
                root,
                NodeContent::Styled {
                    style: Box::new(leading_style),
                },
            );
            ctx.set_layout_style(
                leading_node,
                FlexStyle {
                    width: Some(LEADING_WIDTH),
                    ..Default::default()
                },
            );
        }

        // -- Label text (flex_grow = 1) --
        let label_style = VisualStyle::new()
            .solid_fill(on_surface)
            .text(TextContent::new(self.label.clone(), LABEL_FONT_SIZE));

        let label_node = ctx.create_node(
            root,
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

        // -- Click handler (unless disabled) --
        if !self.disabled
            && let Some(ref callback) = self.on_click
        {
            ctx.add_clickable(root, Arc::clone(callback));
        }

        root
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, Ordering};

    #[test]
    fn test_menu_item_builds() {
        let mut ctx = WidgetContext::new_test();
        let item = MenuItem::new("Cut");
        let root_id = item.build(&mut ctx);

        assert!(
            ctx.scene().get_node(root_id).is_some(),
            "MenuItem root node should exist in scene"
        );
    }

    #[test]
    fn test_menu_item_with_leading() {
        let mut ctx = WidgetContext::new_test();
        let item = MenuItem::new("Copy").leading("C");
        let root_id = item.build(&mut ctx);

        let scene = ctx.scene();
        let root_node = scene.get_node(root_id).unwrap();

        // Should have leading + label = 2 children
        assert_eq!(
            root_node.children.len(),
            2,
            "MenuItem with leading text should have 2 children (leading + label), got {}",
            root_node.children.len()
        );
    }

    #[test]
    fn test_menu_item_without_leading() {
        let mut ctx = WidgetContext::new_test();
        let item = MenuItem::new("Paste");
        let root_id = item.build(&mut ctx);

        let scene = ctx.scene();
        let root_node = scene.get_node(root_id).unwrap();

        // Should have label only = 1 child
        assert_eq!(
            root_node.children.len(),
            1,
            "MenuItem without leading text should have 1 child (label), got {}",
            root_node.children.len()
        );
    }

    #[test]
    fn test_menu_item_click() {
        let clicked = Arc::new(AtomicBool::new(false));
        let clicked_clone = Arc::clone(&clicked);

        let mut ctx = WidgetContext::new_test();
        let item = MenuItem::new("Delete").on_click(move || {
            clicked_clone.store(true, Ordering::SeqCst);
        });
        let root_id = item.build(&mut ctx);

        assert!(
            ctx.has_clickable(root_id),
            "MenuItem with on_click should be clickable"
        );

        ctx.trigger_click(root_id);
        assert!(
            clicked.load(Ordering::SeqCst),
            "MenuItem click callback should have fired"
        );
    }

    #[test]
    fn test_menu_item_disabled() {
        let mut ctx = WidgetContext::new_test();
        let item = MenuItem::new("Disabled Item")
            .on_click(|| {})
            .disabled(true);
        let root_id = item.build(&mut ctx);

        assert!(
            !ctx.has_clickable(root_id),
            "Disabled MenuItem should not be clickable"
        );
    }

    #[test]
    fn test_menu_item_has_hover_state() {
        let mut ctx = WidgetContext::new_test();
        let item = MenuItem::new("Hover me");
        let root_id = item.build(&mut ctx);

        assert!(
            ctx.has_hover_state(root_id),
            "MenuItem should register hover state"
        );
    }

    #[test]
    fn test_menu_item_label_uses_on_surface_color() {
        use render_engine::Paint;

        let mut ctx = WidgetContext::new_test();
        let item = MenuItem::new("Styled");
        let root_id = item.build(&mut ctx);

        let scene = ctx.scene();
        let root_node = scene.get_node(root_id).unwrap();

        // Last child is the label
        let label_id = *root_node.children.last().unwrap();
        let label_node = scene.get_node(label_id).unwrap();

        if let NodeContent::Styled { ref style } = label_node.content {
            assert!(!style.fills.is_empty(), "Label should have a fill");
            if let Paint::Solid(color) = &style.fills[0] {
                assert!(
                    (color.x - FALLBACK_ON_SURFACE.x).abs() < 0.01
                        && (color.y - FALLBACK_ON_SURFACE.y).abs() < 0.01
                        && (color.z - FALLBACK_ON_SURFACE.z).abs() < 0.01,
                    "Label fill should match on_surface color, got {color:?}"
                );
            } else {
                panic!("Label fill should be a solid paint");
            }
        } else {
            panic!("Label node should be Styled content");
        }
    }
}
