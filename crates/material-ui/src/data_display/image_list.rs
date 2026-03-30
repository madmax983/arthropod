//! MD3 ImageList widget -- grid of image placeholders.

use crate::theme::MaterialTheme;
use glam::Vec4;
use layout_engine::{FlexDirection, FlexStyle};
use render_engine::node::NodeContent;
use render_engine::{NodeId, VisualStyle};
use widget_core::Widget;
use widget_core::WidgetContext;

// ---------------------------------------------------------------------------
// MD3 fallback colors
// ---------------------------------------------------------------------------

/// MD3 surface_variant (#E7E0EC) -- placeholder fill for image items.
const FALLBACK_SURFACE_VARIANT: Vec4 = Vec4::new(0.906, 0.878, 0.925, 1.0);

/// Default number of grid columns.
const DEFAULT_COLUMNS: u32 = 3;

/// Default item size (square) in dp.
const DEFAULT_ITEM_SIZE: f32 = 120.0;

/// Default gap between items in dp.
const DEFAULT_GAP: f32 = 4.0;

/// Item corner radius in dp.
const ITEM_CORNER_RADIUS: f32 = 4.0;

/// MD3 ImageList -- grid of image placeholders.
///
/// Renders a grid of square placeholder tiles. Each tile is a simple filled
/// rectangle with rounded corners. Real image loading is future work; for now,
/// items display as `surface_variant`-colored squares.
///
/// The grid is simulated via nested Row/Column layout since flex-wrap may not
/// be available.
///
/// # Example
///
/// ```rust,no_run
/// use material_ui::data_display::ImageList;
///
/// let gallery = ImageList::new(12)
///     .columns(4)
///     .item_size(100.0)
///     .gap(8.0);
/// ```
pub struct ImageList {
    count: usize,
    columns: u32,
    item_size: f32,
    gap: f32,
}

impl ImageList {
    /// Create a new image list with the given number of placeholder items.
    pub fn new(count: usize) -> Self {
        Self {
            count,
            columns: DEFAULT_COLUMNS,
            item_size: DEFAULT_ITEM_SIZE,
            gap: DEFAULT_GAP,
        }
    }

    /// Set the number of grid columns (default: 3).
    pub fn columns(mut self, cols: u32) -> Self {
        self.columns = cols;
        self
    }

    /// Set the item size (square side length) in dp (default: 120.0).
    pub fn item_size(mut self, size: f32) -> Self {
        self.item_size = size;
        self
    }

    /// Set the gap between items in dp (default: 4.0).
    pub fn gap(mut self, gap: f32) -> Self {
        self.gap = gap;
        self
    }
}

impl Widget for ImageList {
    fn build(&self, ctx: &mut WidgetContext) -> NodeId {
        // -- Resolve theme colors --
        let theme = ctx.get_extension::<MaterialTheme>().cloned();
        let fill = theme
            .as_ref()
            .map(|t| t.color.surface_variant)
            .unwrap_or(FALLBACK_SURFACE_VARIANT);

        // -- Root column container --
        let root_style = VisualStyle::new();
        let root = ctx.create_node(
            ctx.root(),
            NodeContent::Styled {
                style: Box::new(root_style),
            },
        );
        ctx.set_layout_style(
            root,
            FlexStyle {
                direction: FlexDirection::Column,
                gap: self.gap,
                ..Default::default()
            },
        );

        // -- Create rows of items --
        let cols = self.columns.max(1) as usize;
        let mut remaining = self.count;
        while remaining > 0 {
            let items_in_row = remaining.min(cols);

            let row_style = VisualStyle::new();
            let row_node = ctx.create_node(
                root,
                NodeContent::Styled {
                    style: Box::new(row_style),
                },
            );
            ctx.set_layout_style(
                row_node,
                FlexStyle {
                    direction: FlexDirection::Row,
                    gap: self.gap,
                    ..Default::default()
                },
            );

            for _ in 0..items_in_row {
                let item_style = VisualStyle::new()
                    .solid_fill(fill)
                    .corner_radius(ITEM_CORNER_RADIUS);
                let item = ctx.create_node(
                    row_node,
                    NodeContent::Styled {
                        style: Box::new(item_style),
                    },
                );
                ctx.set_layout_style(
                    item,
                    FlexStyle {
                        width: Some(self.item_size),
                        height: Some(self.item_size),
                        ..Default::default()
                    },
                );
            }

            remaining -= items_in_row;
        }

        root
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use render_engine::Paint;

    #[test]
    fn test_image_list_builds_grid() {
        let mut ctx = WidgetContext::new_test();
        let list = ImageList::new(6).columns(3);
        let root_id = list.build(&mut ctx);

        assert!(
            ctx.scene().get_node(root_id).is_some(),
            "ImageList root node should exist in scene"
        );

        let root_node = ctx.scene().get_node(root_id).unwrap();
        // 6 items / 3 columns = 2 rows
        assert_eq!(
            root_node.children.len(),
            2,
            "ImageList with 6 items and 3 cols should have 2 row children, got {}",
            root_node.children.len()
        );
    }

    #[test]
    fn test_image_list_correct_item_count() {
        let mut ctx = WidgetContext::new_test();
        let list = ImageList::new(7).columns(3);
        let root_id = list.build(&mut ctx);

        let root_node = ctx.scene().get_node(root_id).unwrap();
        // 7 items / 3 cols = 3 rows (3 + 3 + 1)
        assert_eq!(
            root_node.children.len(),
            3,
            "ImageList with 7 items and 3 cols should have 3 row children, got {}",
            root_node.children.len()
        );

        // Count total items across all rows
        let total_items: usize = root_node
            .children
            .iter()
            .map(|&row_id| ctx.scene().get_node(row_id).unwrap().children.len())
            .sum();
        assert_eq!(
            total_items, 7,
            "ImageList should have 7 total item nodes, got {total_items}"
        );

        // Verify each item has surface_variant fill
        let first_row_id = root_node.children[0];
        let first_row = ctx.scene().get_node(first_row_id).unwrap();
        let first_item_id = first_row.children[0];
        let first_item = ctx.scene().get_node(first_item_id).unwrap();
        if let NodeContent::Styled { ref style } = first_item.content {
            assert!(!style.fills.is_empty(), "Item should have a fill");
            if let Paint::Solid(color) = &style.fills[0] {
                assert!(
                    (color.x - FALLBACK_SURFACE_VARIANT.x).abs() < 0.01
                        && (color.y - FALLBACK_SURFACE_VARIANT.y).abs() < 0.01,
                    "Item fill should match surface_variant, got {color:?}"
                );
            }
        } else {
            panic!("Item node should be Styled content");
        }
    }

    #[test]
    fn test_image_list_custom_columns() {
        let mut ctx = WidgetContext::new_test();
        let list = ImageList::new(8).columns(4);
        let root_id = list.build(&mut ctx);

        let root_node = ctx.scene().get_node(root_id).unwrap();
        // 8 items / 4 cols = 2 rows
        assert_eq!(
            root_node.children.len(),
            2,
            "ImageList with 8 items and 4 cols should have 2 rows, got {}",
            root_node.children.len()
        );

        // Each row should have 4 items
        for &row_id in &root_node.children {
            let row = ctx.scene().get_node(row_id).unwrap();
            assert_eq!(
                row.children.len(),
                4,
                "Each row should have 4 items, got {}",
                row.children.len()
            );
        }
    }

    #[test]
    fn test_image_list_custom_gap() {
        let list = ImageList::new(4).gap(12.0);
        assert!(
            (list.gap - 12.0).abs() < 0.01,
            "Gap should be 12.0, got {}",
            list.gap
        );

        // Verify it builds successfully
        let mut ctx = WidgetContext::new_test();
        let root_id = list.build(&mut ctx);
        assert!(
            ctx.scene().get_node(root_id).is_some(),
            "ImageList with custom gap should build successfully"
        );
    }
}
