//! MD3 Menu widget -- dropdown menu rendered in the Dropdown layer.

use crate::navigation::MenuItem;
use crate::theme::MaterialTheme;
use glam::Vec4;
use layout_engine::{FlexDirection, FlexStyle};
use render_engine::node::NodeContent;
use render_engine::{NodeId, VisualStyle};
use widget_core::Layer;
use widget_core::Widget;
use widget_core::WidgetContext;

// ---------------------------------------------------------------------------
// MD3 fallback colors
// ---------------------------------------------------------------------------

/// MD3 surface (#FEF7FF) -- menu background.
const FALLBACK_SURFACE: Vec4 = Vec4::new(0.996, 0.969, 1.0, 1.0);

/// MD3 menu corner radius in dp.
const MENU_CORNER_RADIUS: f32 = 4.0;

/// Default menu width in dp.
const DEFAULT_WIDTH: f32 = 200.0;

/// MD3 Menu -- a dropdown overlay containing selectable items.
///
/// Renders inside the [`Layer::Dropdown`] layer (z=1) as a styled column
/// of [`MenuItem`] widgets. The menu has a surface-colored background with
/// rounded corners per MD3 elevation level 2.
///
/// # Example
///
/// ```rust,no_run
/// use material_ui::navigation::{Menu, MenuItem};
///
/// let menu = Menu::new(vec![
///     MenuItem::new("Cut").leading("X"),
///     MenuItem::new("Copy").leading("C"),
///     MenuItem::new("Paste").leading("V"),
/// ])
/// .width(240.0);
/// ```
pub struct Menu {
    items: Vec<MenuItem>,
    width: f32,
}

impl Menu {
    /// Create a new menu with the given items.
    pub fn new(items: Vec<MenuItem>) -> Self {
        Self {
            items,
            width: DEFAULT_WIDTH,
        }
    }

    /// Override the menu width in dp (default: 200.0).
    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }
}

impl Widget for Menu {
    fn build(&self, ctx: &mut WidgetContext) -> NodeId {
        // -- Resolve theme colors --
        let theme = ctx.get_extension::<MaterialTheme>().cloned();
        let surface = theme
            .as_ref()
            .map(|t| t.color.surface)
            .unwrap_or(FALLBACK_SURFACE);

        // -- Menu container in Dropdown layer --
        let container_style = VisualStyle::new()
            .solid_fill(surface)
            .corner_radius(MENU_CORNER_RADIUS);

        let menu_container = ctx.add_to_layer(
            Layer::Dropdown,
            NodeContent::Styled {
                style: Box::new(container_style),
            },
        );

        ctx.set_layout_style(
            menu_container,
            FlexStyle {
                direction: FlexDirection::Column,
                width: Some(self.width),
                ..Default::default()
            },
        );

        // -- Build each item and reparent into menu container --
        for item in &self.items {
            let item_id = item.build(ctx);
            ctx.reparent_to(item_id, menu_container);
        }

        menu_container
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_menu_builds_in_dropdown_layer() {
        let mut ctx = WidgetContext::new_test();
        let menu = Menu::new(vec![MenuItem::new("Item 1")]);
        let _menu_id = menu.build(&mut ctx);

        let dropdown_root = ctx.layer_root(Layer::Dropdown);
        let scene = ctx.scene();
        let layer_node = scene.get_node(dropdown_root).unwrap();
        assert!(
            !layer_node.children.is_empty(),
            "Dropdown layer should have children after building a Menu"
        );
    }

    #[test]
    fn test_menu_contains_items() {
        let mut ctx = WidgetContext::new_test();
        let menu = Menu::new(vec![
            MenuItem::new("Cut"),
            MenuItem::new("Copy"),
            MenuItem::new("Paste"),
        ]);
        let menu_id = menu.build(&mut ctx);

        let scene = ctx.scene();
        let menu_node = scene.get_node(menu_id).unwrap();

        assert_eq!(
            menu_node.children.len(),
            3,
            "Menu with 3 items should have 3 children, got {}",
            menu_node.children.len()
        );
    }

    #[test]
    fn test_menu_custom_width() {
        let mut ctx = WidgetContext::new_test();
        let menu = Menu::new(vec![MenuItem::new("Wide Item")]).width(300.0);
        let menu_id = menu.build(&mut ctx);

        // Should build without error and exist in scene
        assert!(
            ctx.scene().get_node(menu_id).is_some(),
            "Menu with custom width should build successfully"
        );
    }

    #[test]
    fn test_menu_default_width() {
        let menu = Menu::new(vec![]);
        assert!(
            (menu.width - 200.0).abs() < f32::EPSILON,
            "Default menu width should be 200.0, got {}",
            menu.width
        );
    }

    #[test]
    fn test_menu_has_surface_background() {
        use render_engine::Paint;

        let mut ctx = WidgetContext::new_test();
        let menu = Menu::new(vec![MenuItem::new("Styled")]);
        let menu_id = menu.build(&mut ctx);

        let scene = ctx.scene();
        let menu_node = scene.get_node(menu_id).unwrap();

        if let NodeContent::Styled { ref style } = menu_node.content {
            assert!(!style.fills.is_empty(), "Menu should have a fill");
            if let Paint::Solid(color) = &style.fills[0] {
                assert!(
                    (color.x - FALLBACK_SURFACE.x).abs() < 0.01
                        && (color.y - FALLBACK_SURFACE.y).abs() < 0.01
                        && (color.z - FALLBACK_SURFACE.z).abs() < 0.01
                        && (color.w - FALLBACK_SURFACE.w).abs() < 0.01,
                    "Menu background should match surface color, got {color:?}"
                );
            } else {
                panic!("Menu fill should be a solid paint");
            }
        } else {
            panic!("Menu node should be Styled content");
        }
    }

    #[test]
    fn test_menu_with_theme() {
        let mut ctx = WidgetContext::new_test();
        let theme = MaterialTheme::from_seed(Vec4::new(0.4, 0.2, 0.8, 1.0));
        ctx.set_extension(theme);

        let menu = Menu::new(vec![MenuItem::new("Themed")]);
        let menu_id = menu.build(&mut ctx);

        assert!(
            ctx.scene().get_node(menu_id).is_some(),
            "Menu with theme should build successfully"
        );
    }

    #[test]
    fn test_menu_empty_items() {
        let mut ctx = WidgetContext::new_test();
        let menu = Menu::new(vec![]);
        let menu_id = menu.build(&mut ctx);

        let scene = ctx.scene();
        let menu_node = scene.get_node(menu_id).unwrap();

        assert!(
            menu_node.children.is_empty(),
            "Menu with no items should have no children"
        );
    }

    #[test]
    fn test_menu_container_is_child_of_dropdown_root() {
        let mut ctx = WidgetContext::new_test();
        let menu = Menu::new(vec![MenuItem::new("Item")]);
        let menu_id = menu.build(&mut ctx);

        let dropdown_root = ctx.layer_root(Layer::Dropdown);
        let scene = ctx.scene();
        let layer_node = scene.get_node(dropdown_root).unwrap();

        assert!(
            layer_node.children.contains(&menu_id),
            "Menu container should be a direct child of the Dropdown layer root"
        );
    }
}
