//! Grid widget - 2D grid layout

use crate::{Widget, WidgetContext, WidgetTuple};
use layout_engine::{FlexDirection, FlexStyle};
use render_engine::{NodeContent, NodeId};

/// Grid widget for 2D grid layouts
///
/// Arranges children in a grid with a fixed number of columns.
/// Children are laid out left-to-right, top-to-bottom.
/// Implemented using nested row containers within a column container.
///
/// # Example
///
/// ```no_run
/// use widget_core::{Grid, Text};
///
/// // 2-column grid with 6 items (3 rows)
/// let grid = Grid::new(
///     (
///         Text::new("A"), Text::new("B"),
///         Text::new("C"), Text::new("D"),
///         Text::new("E"), Text::new("F"),
///     ),
///     2, // columns
/// )
/// .gap(10.0)         // Both row and column gap
/// .padding(20.0);
///
/// // Or use specific gaps:
/// let grid2 = Grid::new(
///     (Text::new("1"), Text::new("2")),
///     2,
/// )
/// .column_gap(8.0)  // Horizontal spacing
/// .row_gap(12.0);   // Vertical spacing
/// ```
pub struct Grid<C: WidgetTuple> {
    children: C,
    columns: usize,
    column_gap: f32,
    row_gap: f32,
    padding: f32,
}

impl<C: WidgetTuple> Grid<C> {
    /// Create a new grid with children and column count
    ///
    /// # Arguments
    ///
    /// * `children` - Tuple of widgets to arrange in the grid
    /// * `columns` - Number of columns in the grid (must be > 0)
    ///
    /// # Panics
    ///
    /// Panics if `columns` is 0.
    pub fn new(children: C, columns: usize) -> Self {
        assert!(columns > 0, "Grid must have at least 1 column");
        Self {
            children,
            columns,
            column_gap: 0.0,
            row_gap: 0.0,
            padding: 0.0,
        }
    }

    /// Set both row and column gap
    pub fn gap(mut self, gap: f32) -> Self {
        self.column_gap = gap;
        self.row_gap = gap;
        self
    }

    /// Set gap between columns (horizontal spacing)
    pub fn column_gap(mut self, gap: f32) -> Self {
        self.column_gap = gap;
        self
    }

    /// Set gap between rows (vertical spacing)
    pub fn row_gap(mut self, gap: f32) -> Self {
        self.row_gap = gap;
        self
    }

    /// Set uniform padding around the grid
    pub fn padding(mut self, padding: f32) -> Self {
        self.padding = padding;
        self
    }
}

impl<C: WidgetTuple> Widget for Grid<C> {
    fn build(&self, ctx: &mut WidgetContext) -> NodeId {
        // Create grid container node (column direction for rows)
        let root_id = ctx.root();
        let grid_id = ctx.create_node(root_id, NodeContent::Empty);

        // Build all children and collect their IDs
        let child_ids = self.children.build_all_to_vec(ctx);

        // Organize children into rows
        let mut row_start = 0;
        while row_start < child_ids.len() {
            let row_end = (row_start + self.columns).min(child_ids.len());
            let row_children = &child_ids[row_start..row_end];

            // Create row container
            let row_id = ctx.create_node(grid_id, NodeContent::Empty);

            // Reparent children to this row
            for &child_id in row_children {
                ctx.reparent_to(child_id, row_id);
            }

            // Configure row layout (horizontal)
            let row_style = FlexStyle {
                direction: FlexDirection::Row,
                gap: self.column_gap,
                ..Default::default()
            };
            ctx.set_layout_style(row_id, row_style);

            row_start = row_end;
        }

        // Configure grid container layout (vertical)
        let grid_style = FlexStyle {
            direction: FlexDirection::Column,
            gap: self.row_gap,
            padding_left: self.padding,
            padding_right: self.padding,
            padding_top: self.padding,
            padding_bottom: self.padding,
            ..Default::default()
        };
        ctx.set_layout_style(grid_id, grid_style);

        grid_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestWidget {
        _label: &'static str,
    }

    impl Widget for TestWidget {
        fn build(&self, ctx: &mut WidgetContext) -> NodeId {
            ctx.create_node(
                ctx.root(),
                NodeContent::Rect {
                    color: render_engine::Color::rgba(1.0, 0.0, 0.0, 1.0),
                },
            )
        }
    }

    #[test]
    fn test_grid_creates_container() {
        let mut ctx = WidgetContext::new_test();
        let grid = Grid::new((TestWidget { _label: "A" },), 1);

        let node_id = grid.build(&mut ctx);

        // Verify node exists
        assert!(ctx.scene().get_node(node_id).is_some());
    }

    #[test]
    fn test_grid_row_count() {
        let mut ctx = WidgetContext::new_test();

        // 4 items, 2 columns = 2 rows
        let grid = Grid::new(
            (
                TestWidget { _label: "A" },
                TestWidget { _label: "B" },
                TestWidget { _label: "C" },
                TestWidget { _label: "D" },
            ),
            2,
        );

        let node_id = grid.build(&mut ctx);
        let scene_node = ctx.scene().get_node(node_id).unwrap();

        assert_eq!(scene_node.children.len(), 2, "Should have 2 rows");
    }

    #[test]
    fn test_grid_column_layout() {
        let mut ctx = WidgetContext::new_test();
        let grid = Grid::new((TestWidget { _label: "A" },), 1);

        let node_id = grid.build(&mut ctx);
        let layout = ctx.get_layout_style(node_id).unwrap();

        // Grid container should be column direction (vertical)
        assert_eq!(
            layout.direction,
            FlexDirection::Column,
            "Grid should use column layout"
        );
    }

    #[test]
    fn test_grid_row_direction() {
        let mut ctx = WidgetContext::new_test();
        let grid = Grid::new(
            (TestWidget { _label: "A" }, TestWidget { _label: "B" }),
            2,
        );

        let node_id = grid.build(&mut ctx);
        let scene_node = ctx.scene().get_node(node_id).unwrap();

        // First row should be row direction (horizontal)
        let row_id = scene_node.children[0];
        let row_layout = ctx.get_layout_style(row_id).unwrap();

        assert!(
            crate::context::is_row_layout(&row_layout),
            "Row should use row layout"
        );
    }

    #[test]
    #[should_panic(expected = "Grid must have at least 1 column")]
    fn test_grid_zero_columns_panics() {
        Grid::new((TestWidget { _label: "A" },), 0);
    }
}
