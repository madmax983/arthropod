//! MD3 AppBar widget -- top bar with title, navigation icon, and action items.

use crate::theme::MaterialTheme;
use glam::Vec4;
use layout_engine::{FlexAlign, FlexDirection, FlexJustifyContent, FlexStyle};
use render_engine::node::NodeContent;
use render_engine::{NodeId, TextContent, VisualStyle};
use widget_core::Widget;
use widget_core::WidgetContext;

// ---------------------------------------------------------------------------
// MD3 fallback colors
// ---------------------------------------------------------------------------

/// MD3 surface (#FEF7FF) -- app bar background.
const FALLBACK_SURFACE: Vec4 = Vec4::new(0.996, 0.969, 1.0, 1.0);

/// MD3 on_surface (#1D1B20) -- title text.
const FALLBACK_ON_SURFACE: Vec4 = Vec4::new(0.114, 0.106, 0.125, 1.0);

// ---------------------------------------------------------------------------
// Dimension constants (dp)
// ---------------------------------------------------------------------------

/// Height for Small and CenterAligned variants.
const SMALL_HEIGHT: f32 = 64.0;

/// Height for Medium variant.
const MEDIUM_HEIGHT: f32 = 112.0;

/// Height for Large variant.
const LARGE_HEIGHT: f32 = 152.0;

/// Touch target size for navigation icon and action buttons.
const TOUCH_TARGET: f32 = 48.0;

/// Title font size for Small/CenterAligned (title_large).
const TITLE_LARGE_SIZE: f32 = 22.0;

/// Title font size for Medium (headline_small).
const HEADLINE_SMALL_SIZE: f32 = 24.0;

/// Title font size for Large (headline_medium).
const HEADLINE_MEDIUM_SIZE: f32 = 28.0;

/// MD3 top app bar variant.
///
/// Controls the layout, height, and title typography of an [`AppBar`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AppBarVariant {
    /// Title centered horizontally, height 64dp.
    CenterAligned,
    /// Title left-aligned, height 64dp (default).
    #[default]
    Small,
    /// Two-line layout with larger title on the bottom row, height 112dp.
    Medium,
    /// Two-line layout with the largest title on the bottom row, height 152dp.
    Large,
}

/// MD3 AppBar -- top application bar with title, navigation icon, and actions.
///
/// Renders a horizontal bar at the top of the screen following Material
/// Design 3 guidelines. Supports four variants that control the layout
/// and typography of the title.
///
/// # Example
///
/// ```rust,no_run
/// use material_ui::navigation::AppBar;
///
/// let bar = AppBar::new("Inbox")
///     .center_aligned();
/// ```
pub struct AppBar {
    title: String,
    variant: AppBarVariant,
    navigation_icon: Option<Box<dyn Widget>>,
    actions: Vec<Box<dyn Widget>>,
}

impl AppBar {
    /// Create a new app bar with the given title (default: [`AppBarVariant::Small`]).
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            variant: AppBarVariant::Small,
            navigation_icon: None,
            actions: Vec::new(),
        }
    }

    /// Set the variant to [`AppBarVariant::CenterAligned`].
    pub fn center_aligned(mut self) -> Self {
        self.variant = AppBarVariant::CenterAligned;
        self
    }

    /// Set the variant to [`AppBarVariant::Medium`].
    pub fn medium(mut self) -> Self {
        self.variant = AppBarVariant::Medium;
        self
    }

    /// Set the variant to [`AppBarVariant::Large`].
    pub fn large(mut self) -> Self {
        self.variant = AppBarVariant::Large;
        self
    }

    /// Set a navigation icon (e.g. back or menu button) displayed on the left.
    pub fn navigation_icon(mut self, widget: impl Widget + 'static) -> Self {
        self.navigation_icon = Some(Box::new(widget));
        self
    }

    /// Add an action widget displayed on the right side of the bar.
    pub fn action(mut self, widget: impl Widget + 'static) -> Self {
        self.actions.push(Box::new(widget));
        self
    }

    // -- private helpers --

    /// Build a single-row layout (Small / CenterAligned).
    fn build_single_row(&self, ctx: &mut WidgetContext, on_surface: Vec4) -> NodeId {
        let justify = if self.variant == AppBarVariant::CenterAligned {
            FlexJustifyContent::Center
        } else {
            FlexJustifyContent::Start
        };

        let row = ctx.create_node(ctx.root(), NodeContent::Empty);
        ctx.set_layout_style(
            row,
            FlexStyle {
                direction: FlexDirection::Row,
                align_items: FlexAlign::Center,
                justify_content: justify,
                flex_grow: 1.0,
                height: Some(SMALL_HEIGHT),
                ..Default::default()
            },
        );

        // Navigation icon (if any)
        if let Some(ref nav) = self.navigation_icon {
            let nav_id = self.build_touch_target(ctx, nav.as_ref());
            ctx.reparent_to(nav_id, row);
        }

        // Title
        let title_style = VisualStyle::new()
            .solid_fill(on_surface)
            .text(TextContent::new(self.title.clone(), TITLE_LARGE_SIZE));

        let title_node = ctx.create_node(
            row,
            NodeContent::Styled {
                style: Box::new(title_style),
            },
        );
        ctx.set_layout_style(
            title_node,
            FlexStyle {
                flex_grow: 1.0,
                ..Default::default()
            },
        );

        // Actions row on the right
        self.build_actions(ctx, row);

        row
    }

    /// Build a two-row layout (Medium / Large).
    fn build_two_row(
        &self,
        ctx: &mut WidgetContext,
        on_surface: Vec4,
        total_height: f32,
        title_size: f32,
    ) -> NodeId {
        // Outer column spanning full height
        let col = ctx.create_node(ctx.root(), NodeContent::Empty);
        ctx.set_layout_style(
            col,
            FlexStyle {
                direction: FlexDirection::Column,
                height: Some(total_height),
                flex_grow: 1.0,
                ..Default::default()
            },
        );

        // Top row: nav icon + spacer + actions
        let top_row = ctx.create_node(col, NodeContent::Empty);
        ctx.set_layout_style(
            top_row,
            FlexStyle {
                direction: FlexDirection::Row,
                height: Some(SMALL_HEIGHT),
                align_items: FlexAlign::Center,
                ..Default::default()
            },
        );

        if let Some(ref nav) = self.navigation_icon {
            let nav_id = self.build_touch_target(ctx, nav.as_ref());
            ctx.reparent_to(nav_id, top_row);
        }

        // Spacer in top row to push actions right
        let spacer = ctx.create_node(top_row, NodeContent::Empty);
        ctx.set_layout_style(
            spacer,
            FlexStyle {
                flex_grow: 1.0,
                ..Default::default()
            },
        );

        // Actions in top row
        self.build_actions(ctx, top_row);

        // Bottom title row
        let title_row = ctx.create_node(col, NodeContent::Empty);
        ctx.set_layout_style(
            title_row,
            FlexStyle {
                direction: FlexDirection::Row,
                flex_grow: 1.0,
                align_items: FlexAlign::End,
                padding_left: 16.0,
                padding_bottom: 16.0,
                ..Default::default()
            },
        );

        let title_style = VisualStyle::new()
            .solid_fill(on_surface)
            .text(TextContent::new(self.title.clone(), title_size));

        ctx.create_node(
            title_row,
            NodeContent::Styled {
                style: Box::new(title_style),
            },
        );

        col
    }

    /// Wrap a widget in a touch-target-sized container.
    fn build_touch_target(&self, ctx: &mut WidgetContext, widget: &dyn Widget) -> NodeId {
        let container = ctx.create_node(ctx.root(), NodeContent::Empty);
        ctx.set_layout_style(
            container,
            FlexStyle {
                width: Some(TOUCH_TARGET),
                height: Some(TOUCH_TARGET),
                justify_content: FlexJustifyContent::Center,
                align_items: FlexAlign::Center,
                ..Default::default()
            },
        );

        let child_id = widget.build(ctx);
        ctx.reparent_to(child_id, container);

        container
    }

    /// Build action widgets in a right-aligned row and reparent into `parent`.
    fn build_actions(&self, ctx: &mut WidgetContext, parent: NodeId) {
        for action in &self.actions {
            let action_id = self.build_touch_target(ctx, action.as_ref());
            ctx.reparent_to(action_id, parent);
        }
    }
}

impl Widget for AppBar {
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

        // -- Root container with surface background --
        let root_style = VisualStyle::new().solid_fill(surface);
        let root = ctx.create_node(
            ctx.root(),
            NodeContent::Styled {
                style: Box::new(root_style),
            },
        );

        let height = match self.variant {
            AppBarVariant::Small | AppBarVariant::CenterAligned => SMALL_HEIGHT,
            AppBarVariant::Medium => MEDIUM_HEIGHT,
            AppBarVariant::Large => LARGE_HEIGHT,
        };

        ctx.set_layout_style(
            root,
            FlexStyle {
                direction: FlexDirection::Row,
                height: Some(height),
                ..Default::default()
            },
        );

        // -- Build inner layout --
        let inner = match self.variant {
            AppBarVariant::Small | AppBarVariant::CenterAligned => {
                self.build_single_row(ctx, on_surface)
            }
            AppBarVariant::Medium => {
                self.build_two_row(ctx, on_surface, MEDIUM_HEIGHT, HEADLINE_SMALL_SIZE)
            }
            AppBarVariant::Large => {
                self.build_two_row(ctx, on_surface, LARGE_HEIGHT, HEADLINE_MEDIUM_SIZE)
            }
        };
        ctx.reparent_to(inner, root);

        root
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use render_engine::Paint;
    use widget_core::Text;

    #[test]
    fn test_app_bar_small_default() {
        let mut ctx = WidgetContext::new_test();
        let bar = AppBar::new("Home");
        let root_id = bar.build(&mut ctx);

        assert!(
            ctx.scene().get_node(root_id).is_some(),
            "AppBar root node should exist in scene"
        );

        // Verify surface background
        let node = ctx.scene().get_node(root_id).unwrap();
        if let NodeContent::Styled { ref style } = node.content {
            assert!(!style.fills.is_empty(), "AppBar should have a fill");
            if let Paint::Solid(color) = &style.fills[0] {
                assert!(
                    (color.x - FALLBACK_SURFACE.x).abs() < 0.01
                        && (color.y - FALLBACK_SURFACE.y).abs() < 0.01,
                    "AppBar background should match surface color, got {color:?}"
                );
            } else {
                panic!("AppBar fill should be solid");
            }
        } else {
            panic!("AppBar root should be Styled content");
        }
    }

    #[test]
    fn test_app_bar_center_aligned() {
        let mut ctx = WidgetContext::new_test();
        let bar = AppBar::new("Center Title").center_aligned();

        assert_eq!(bar.variant, AppBarVariant::CenterAligned);

        let root_id = bar.build(&mut ctx);
        assert!(
            ctx.scene().get_node(root_id).is_some(),
            "CenterAligned AppBar should build successfully"
        );
    }

    #[test]
    fn test_app_bar_medium_variant() {
        let mut ctx = WidgetContext::new_test();
        let bar = AppBar::new("Medium Title").medium();

        assert_eq!(bar.variant, AppBarVariant::Medium);

        let root_id = bar.build(&mut ctx);
        let root_node = ctx.scene().get_node(root_id).unwrap();

        // Medium uses a two-row layout: root contains one inner column
        assert!(
            !root_node.children.is_empty(),
            "Medium AppBar should have children"
        );
    }

    #[test]
    fn test_app_bar_with_navigation_icon() {
        let mut ctx = WidgetContext::new_test();
        let bar = AppBar::new("With Nav").navigation_icon(Text::new("B"));
        let root_id = bar.build(&mut ctx);

        let root_node = ctx.scene().get_node(root_id).unwrap();
        assert!(
            !root_node.children.is_empty(),
            "AppBar with nav icon should have children"
        );

        // The inner row is the first child of root
        let inner_row_id = root_node.children[0];
        let inner_row = ctx.scene().get_node(inner_row_id).unwrap();

        // Inner row should contain: nav_icon_container + title + (no actions)
        // The nav icon container is a 48dp touch target wrapping the widget
        assert!(
            inner_row.children.len() >= 2,
            "Inner row should have at least nav icon + title, got {}",
            inner_row.children.len()
        );
    }

    #[test]
    fn test_app_bar_with_actions() {
        let mut ctx = WidgetContext::new_test();
        let bar = AppBar::new("Actions")
            .action(Text::new("S"))
            .action(Text::new("M"));
        let root_id = bar.build(&mut ctx);

        let root_node = ctx.scene().get_node(root_id).unwrap();
        let inner_row_id = root_node.children[0];
        let inner_row = ctx.scene().get_node(inner_row_id).unwrap();

        // Inner row: title(1) + 2 action containers = 3
        assert_eq!(
            inner_row.children.len(),
            3,
            "Inner row with 2 actions should have 3 children (title + 2 actions), got {}",
            inner_row.children.len()
        );
    }
}
