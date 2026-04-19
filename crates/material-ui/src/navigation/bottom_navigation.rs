//! MD3 BottomNavigation widget -- fixed bottom bar with icon items.

use crate::theme::MaterialTheme;
use flux_state::{Effect, Signal};
use glam::Vec4;
use layout_engine::{FlexAlign, FlexDirection, FlexJustifyContent, FlexStyle};
use render_engine::node::NodeContent;
use render_engine::{Color, NodeId, TextContent, VisualStyle};
use widget_core::Widget;
use widget_core::WidgetContext;

// ---------------------------------------------------------------------------
// MD3 fallback colors
// ---------------------------------------------------------------------------

/// MD3 primary (#6750A4) -- selected item icon and label.
const FALLBACK_PRIMARY: Vec4 = Vec4::new(0.404, 0.314, 0.643, 1.0);

/// MD3 surface (#FEF7FF) -- bar background.
const FALLBACK_SURFACE: Vec4 = Vec4::new(0.996, 0.969, 1.0, 1.0);

/// MD3 secondary_container (#E8DEF8) -- indicator pill behind selected icon.
const FALLBACK_SECONDARY_CONTAINER: Vec4 = Vec4::new(0.906, 0.831, 0.996, 1.0);

/// MD3 on_surface_variant (#49454F) -- unselected item icon and label.
const FALLBACK_ON_SURFACE_VARIANT: Vec4 = Vec4::new(0.286, 0.271, 0.310, 1.0);

// ---------------------------------------------------------------------------
// Dimension constants (dp)
// ---------------------------------------------------------------------------

/// Height of the bottom navigation bar.
const BAR_HEIGHT: f32 = 80.0;

/// Icon text font size.
const ICON_FONT_SIZE: f32 = 24.0;

/// Label text font size.
const LABEL_FONT_SIZE: f32 = 12.0;

/// Indicator pill width.
const INDICATOR_WIDTH: f32 = 64.0;

/// Indicator pill height.
const INDICATOR_HEIGHT: f32 = 32.0;

/// Indicator pill corner radius (fully rounded).
const INDICATOR_RADIUS: f32 = 16.0;

/// Gap between icon and label.
const ICON_LABEL_GAP: f32 = 4.0;

/// A single item in a [`BottomNavigation`] bar.
///
/// # Example
///
/// ```rust,no_run
/// use material_ui::navigation::BottomNavItem;
///
/// let item = BottomNavItem::new("H", "Home");
/// ```
pub struct BottomNavItem {
    /// Icon text placeholder.
    pub icon: String,
    /// Display label below the icon.
    pub label: String,
}

impl BottomNavItem {
    /// Create a new bottom navigation item.
    pub fn new(icon: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            icon: icon.into(),
            label: label.into(),
        }
    }
}

/// MD3 BottomNavigation -- fixed bottom bar with icon items.
///
/// Renders a horizontal bar at the bottom of the screen with icon+label
/// items. The selected item is highlighted with the primary color and an
/// indicator pill. Selection state is driven by a [`Signal<usize>`].
///
/// # Example
///
/// ```rust,no_run
/// use flux_state::{Runtime, Signal};
/// use material_ui::navigation::{BottomNavItem, BottomNavigation};
///
/// let runtime = Runtime::new();
/// let selected = Signal::new(runtime, 0usize);
///
/// let nav = BottomNavigation::new(
///     vec![
///         BottomNavItem::new("H", "Home"),
///         BottomNavItem::new("S", "Search"),
///         BottomNavItem::new("P", "Profile"),
///     ],
///     selected,
/// );
/// ```
pub struct BottomNavigation {
    items: Vec<BottomNavItem>,
    signal: Signal<usize>,
}

impl BottomNavigation {
    /// Create a new bottom navigation bar.
    ///
    /// # Arguments
    ///
    /// * `items` - Navigation items to display.
    /// * `signal` - A signal holding the currently selected item index.
    pub fn new(items: Vec<BottomNavItem>, signal: Signal<usize>) -> Self {
        Self { items, signal }
    }

    #[allow(clippy::too_many_arguments)]
    fn build_item(
        &self,
        ctx: &mut WidgetContext,
        root: NodeId,
        item: &BottomNavItem,
        i: usize,
        is_selected: bool,
        primary: Vec4,
        on_surface_variant: Vec4,
        secondary_container: Vec4,
        write: flux_state::WriteSignal<usize>,
        runtime: std::sync::Arc<flux_state::Runtime>,
        icon_color_writers: &mut Vec<flux_state::WriteSignal<Color>>,
        label_color_writers: &mut Vec<flux_state::WriteSignal<Color>>,
        indicator_color_writers: &mut Vec<flux_state::WriteSignal<Color>>,
    ) {
        let item_color = if is_selected {
            primary
        } else {
            on_surface_variant
        };
        let indicator_fill = if is_selected {
            secondary_container
        } else {
            Vec4::new(0.0, 0.0, 0.0, 0.0) // Transparent
        };

        // Item column: centered, flex_grow=1
        let item_col = ctx.create_node(root, NodeContent::Empty);
        ctx.set_layout_style(
            item_col,
            FlexStyle {
                direction: FlexDirection::Column,
                flex_grow: 1.0,
                align_items: FlexAlign::Center,
                justify_content: FlexJustifyContent::Center,
                gap: ICON_LABEL_GAP,
                height: Some(BAR_HEIGHT),
                ..Default::default()
            },
        );

        // Indicator pill (behind icon, acts as a background highlight)
        let indicator_style = VisualStyle::new()
            .solid_fill(indicator_fill)
            .corner_radius(INDICATOR_RADIUS);
        let indicator_node = ctx.create_node(
            item_col,
            NodeContent::Styled {
                style: Box::new(indicator_style),
            },
        );
        ctx.set_layout_style(
            indicator_node,
            FlexStyle {
                width: Some(INDICATOR_WIDTH),
                height: Some(INDICATOR_HEIGHT),
                justify_content: FlexJustifyContent::Center,
                align_items: FlexAlign::Center,
                ..Default::default()
            },
        );

        // Icon text (inside indicator pill)
        let icon_style = VisualStyle::new()
            .solid_fill(item_color)
            .text(TextContent::new(item.icon.clone(), ICON_FONT_SIZE));
        let icon_node = ctx.create_node(
            indicator_node,
            NodeContent::Styled {
                style: Box::new(icon_style),
            },
        );

        // Label text (below indicator)
        let label_style = VisualStyle::new()
            .solid_fill(item_color)
            .text(TextContent::new(item.label.clone(), LABEL_FONT_SIZE));
        let label_node = ctx.create_node(
            item_col,
            NodeContent::Styled {
                style: Box::new(label_style),
            },
        );

        // Click handler
        let w = write.clone();
        let idx = i;
        ctx.add_clickable(
            item_col,
            std::sync::Arc::new(move || {
                w.set(idx);
            }),
        );

        // Reactive color signals for icon, label, and indicator
        let icon_color_signal = Signal::new(
            runtime.clone(),
            Color::rgba(item_color.x, item_color.y, item_color.z, item_color.w),
        );
        let (icon_cr, icon_cw) = icon_color_signal.split();
        ctx.add_reactive_color_state(icon_node, icon_cr);
        icon_color_writers.push(icon_cw);

        let label_color_signal = Signal::new(
            runtime.clone(),
            Color::rgba(item_color.x, item_color.y, item_color.z, item_color.w),
        );
        let (label_cr, label_cw) = label_color_signal.split();
        ctx.add_reactive_color_state(label_node, label_cr);
        label_color_writers.push(label_cw);

        let indicator_color_signal = Signal::new(
            runtime.clone(),
            Color::rgba(
                indicator_fill.x,
                indicator_fill.y,
                indicator_fill.z,
                indicator_fill.w,
            ),
        );
        let (ind_cr, ind_cw) = indicator_color_signal.split();
        ctx.add_reactive_color_state(indicator_node, ind_cr);
        indicator_color_writers.push(ind_cw);
    }
}

impl Widget for BottomNavigation {
    fn build(&self, ctx: &mut WidgetContext) -> NodeId {
        let (read, write) = self.signal.clone().split();
        let runtime = read.runtime().clone();

        // -- Resolve theme colors --
        let theme = ctx.get_extension::<MaterialTheme>().cloned();
        let primary = theme
            .as_ref()
            .map(|t| t.color.primary)
            .unwrap_or(FALLBACK_PRIMARY);
        let surface = theme
            .as_ref()
            .map(|t| t.color.surface)
            .unwrap_or(FALLBACK_SURFACE);
        let secondary_container = theme
            .as_ref()
            .map(|t| t.color.secondary_container)
            .unwrap_or(FALLBACK_SECONDARY_CONTAINER);
        let on_surface_variant = theme
            .as_ref()
            .map(|t| t.color.on_surface_variant)
            .unwrap_or(FALLBACK_ON_SURFACE_VARIANT);

        let current_index = read.get_untracked();

        // -- Root bar: Row, full width, surface background --
        let root = ctx.create_node(
            ctx.root(),
            NodeContent::Styled {
                style: Box::new(VisualStyle::new().solid_fill(surface)),
            },
        );
        ctx.set_layout_style(
            root,
            FlexStyle {
                direction: FlexDirection::Row,
                height: Some(BAR_HEIGHT),
                align_items: FlexAlign::Center,
                ..Default::default()
            },
        );

        // Collect write signals for the reactive effect
        let mut icon_color_writers = Vec::new();
        let mut label_color_writers = Vec::new();
        let mut indicator_color_writers = Vec::new();

        for (i, item) in self.items.iter().enumerate() {
            let is_selected = i == current_index;
            self.build_item(
                ctx,
                root,
                item,
                i,
                is_selected,
                primary,
                on_surface_variant,
                secondary_container,
                write.clone(),
                runtime.clone(),
                &mut icon_color_writers,
                &mut label_color_writers,
                &mut indicator_color_writers,
            );
        }

        // -- Reactive effect: update all item colors when selection changes --
        let effect = Effect::new(runtime.clone(), move || {
            let current = read.get();

            for (i, ((icon_w, label_w), ind_w)) in icon_color_writers
                .iter()
                .zip(label_color_writers.iter())
                .zip(indicator_color_writers.iter())
                .enumerate()
            {
                let selected = i == current;
                let color = if selected {
                    primary
                } else {
                    on_surface_variant
                };
                let ind_fill = if selected {
                    secondary_container
                } else {
                    Vec4::new(0.0, 0.0, 0.0, 0.0) // Transparent
                };

                icon_w.set(Color::rgba(color.x, color.y, color.z, color.w));
                label_w.set(Color::rgba(color.x, color.y, color.z, color.w));
                ind_w.set(Color::rgba(ind_fill.x, ind_fill.y, ind_fill.z, ind_fill.w));
            }
        });
        ctx.store_effect(effect);

        root
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use flux_state::Runtime;
    use render_engine::Paint;

    #[test]
    fn test_bottom_navigation_builds_items() {
        let runtime = Runtime::new();
        let selected = Signal::new(runtime, 0usize);

        let mut ctx = WidgetContext::new_test();
        let nav = BottomNavigation::new(
            vec![
                BottomNavItem::new("H", "Home"),
                BottomNavItem::new("S", "Search"),
                BottomNavItem::new("P", "Profile"),
            ],
            selected,
        );
        let root_id = nav.build(&mut ctx);

        let root_node = ctx.scene().get_node(root_id).unwrap();
        assert_eq!(
            root_node.children.len(),
            3,
            "BottomNavigation with 3 items should have 3 children, got {}",
            root_node.children.len()
        );
    }

    #[test]
    fn test_bottom_navigation_selected_styling() {
        let runtime = Runtime::new();
        let selected = Signal::new(runtime, 1usize); // Select "Search"

        let mut ctx = WidgetContext::new_test();
        let nav = BottomNavigation::new(
            vec![
                BottomNavItem::new("H", "Home"),
                BottomNavItem::new("S", "Search"),
            ],
            selected,
        );
        let root_id = nav.build(&mut ctx);

        let scene = ctx.scene();
        let root_node = scene.get_node(root_id).unwrap();

        // Selected item (index 1) should have a visible indicator pill
        let selected_col_id = root_node.children[1];
        let selected_col = scene.get_node(selected_col_id).unwrap();
        // children[0] = indicator_node, children[1] = label_node
        let indicator_id = selected_col.children[0];
        let indicator_node = scene.get_node(indicator_id).unwrap();

        if let NodeContent::Styled { ref style } = indicator_node.content {
            if let Some(Paint::Solid(color)) = style.fills.first() {
                assert!(
                    color.w > 0.5,
                    "Selected indicator should be visible (alpha > 0.5), got {}",
                    color.w
                );
                assert!(
                    (color.x - FALLBACK_SECONDARY_CONTAINER.x).abs() < 0.01,
                    "Selected indicator should match secondary_container, got {color:?}"
                );
            } else {
                panic!("Indicator fill should be solid");
            }
        } else {
            panic!("Indicator should be Styled content");
        }

        // Unselected item (index 0) indicator should be transparent
        let unselected_col_id = root_node.children[0];
        let unselected_col = scene.get_node(unselected_col_id).unwrap();
        let unselected_ind_id = unselected_col.children[0];
        let unselected_ind = scene.get_node(unselected_ind_id).unwrap();

        if let NodeContent::Styled { ref style } = unselected_ind.content
            && let Some(Paint::Solid(color)) = style.fills.first()
        {
            assert!(
                color.w < 0.01,
                "Unselected indicator should be transparent, got alpha={}",
                color.w
            );
        }
    }

    #[test]
    fn test_bottom_navigation_click_changes_signal() {
        let runtime = Runtime::new();
        let selected = Signal::new(runtime, 0usize);
        let observer = selected.clone();
        let (obs_read, _) = observer.split();

        let mut ctx = WidgetContext::new_test();
        let nav = BottomNavigation::new(
            vec![
                BottomNavItem::new("H", "Home"),
                BottomNavItem::new("S", "Search"),
            ],
            selected,
        );
        let root_id = nav.build(&mut ctx);

        assert_eq!(obs_read.get_untracked(), 0, "Should start at item 0");

        // Click the second item column
        let second_col_id = ctx.scene().get_node(root_id).unwrap().children[1];
        ctx.trigger_click(second_col_id);

        assert_eq!(
            obs_read.get_untracked(),
            1,
            "After clicking second item, signal should be 1"
        );
    }

    #[test]
    fn test_bottom_navigation_correct_item_count() {
        let runtime = Runtime::new();
        let selected = Signal::new(runtime, 0usize);

        let mut ctx = WidgetContext::new_test();
        let nav = BottomNavigation::new(
            vec![
                BottomNavItem::new("H", "Home"),
                BottomNavItem::new("S", "Search"),
                BottomNavItem::new("N", "Notifications"),
                BottomNavItem::new("P", "Profile"),
            ],
            selected,
        );
        let root_id = nav.build(&mut ctx);

        let root_node = ctx.scene().get_node(root_id).unwrap();
        assert_eq!(
            root_node.children.len(),
            4,
            "4 nav items should create 4 item columns, got {}",
            root_node.children.len()
        );

        // Verify each item column has indicator + label
        for (i, &child_id) in root_node.children.iter().enumerate() {
            let child = ctx.scene().get_node(child_id).unwrap();
            assert_eq!(
                child.children.len(),
                2,
                "Item {} should have 2 children (indicator + label), got {}",
                i,
                child.children.len()
            );
        }
    }
}
