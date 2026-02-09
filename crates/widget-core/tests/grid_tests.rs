//! Grid widget tests - Written FIRST following TDD

use widget_core::{Grid, Text, Widget, WidgetContext};

#[test]
fn test_grid_empty() {
    let mut ctx = WidgetContext::new_test();

    let grid = Grid::new((), 2);
    let node_id = grid.build(&mut ctx);

    // Verify scene node was created
    let scene_node = ctx.scene().get_node(node_id).unwrap();
    assert_eq!(
        scene_node.children.len(),
        0,
        "Empty grid should have no rows"
    );
}

#[test]
fn test_grid_single_row() {
    let mut ctx = WidgetContext::new_test();

    // 3 items, 3 columns = 1 row
    let grid = Grid::new((Text::new("A"), Text::new("B"), Text::new("C")), 3);

    let node_id = grid.build(&mut ctx);

    let scene_node = ctx.scene().get_node(node_id).unwrap();
    assert_eq!(scene_node.children.len(), 1, "Should have 1 row");

    // First row should have 3 children
    let row_id = scene_node.children[0];
    let row_node = ctx.scene().get_node(row_id).unwrap();
    assert_eq!(row_node.children.len(), 3, "Row should have 3 children");
}

#[test]
fn test_grid_multiple_rows() {
    let mut ctx = WidgetContext::new_test();

    // 6 items, 2 columns = 3 rows
    let grid = Grid::new(
        (
            Text::new("A"),
            Text::new("B"),
            Text::new("C"),
            Text::new("D"),
            Text::new("E"),
            Text::new("F"),
        ),
        2,
    );

    let node_id = grid.build(&mut ctx);

    let scene_node = ctx.scene().get_node(node_id).unwrap();
    assert_eq!(scene_node.children.len(), 3, "Should have 3 rows");

    // Each row should have 2 children
    for row_idx in 0..3 {
        let row_id = scene_node.children[row_idx];
        let row_node = ctx.scene().get_node(row_id).unwrap();
        assert_eq!(
            row_node.children.len(),
            2,
            "Each row should have 2 children"
        );
    }
}

#[test]
fn test_grid_partial_last_row() {
    let mut ctx = WidgetContext::new_test();

    // 5 items, 2 columns = 3 rows (last row has 1 item)
    let grid = Grid::new(
        (
            Text::new("A"),
            Text::new("B"),
            Text::new("C"),
            Text::new("D"),
            Text::new("E"),
        ),
        2,
    );

    let node_id = grid.build(&mut ctx);

    let scene_node = ctx.scene().get_node(node_id).unwrap();
    assert_eq!(scene_node.children.len(), 3, "Should have 3 rows");

    // First two rows have 2 children
    let row_0 = ctx.scene().get_node(scene_node.children[0]).unwrap();
    assert_eq!(row_0.children.len(), 2);

    let row_1 = ctx.scene().get_node(scene_node.children[1]).unwrap();
    assert_eq!(row_1.children.len(), 2);

    // Last row has 1 child
    let row_2 = ctx.scene().get_node(scene_node.children[2]).unwrap();
    assert_eq!(row_2.children.len(), 1);
}

#[test]
fn test_grid_with_gap() {
    let mut ctx = WidgetContext::new_test();

    let grid = Grid::new((Text::new("A"), Text::new("B")), 2).gap(10.0);

    let node_id = grid.build(&mut ctx);

    // Grid container should have gap
    let layout = ctx.get_layout_style(node_id).unwrap();
    assert_eq!(layout.gap, 10.0, "Grid should have gap of 10.0");
}

#[test]
fn test_grid_with_column_gap() {
    let mut ctx = WidgetContext::new_test();

    let grid = Grid::new((Text::new("A"), Text::new("B")), 2).column_gap(8.0);

    let node_id = grid.build(&mut ctx);
    let scene_node = ctx.scene().get_node(node_id).unwrap();

    // Each row should have column_gap
    let row_id = scene_node.children[0];
    let row_layout = ctx.get_layout_style(row_id).unwrap();
    assert_eq!(row_layout.gap, 8.0, "Row should have gap of 8.0");
}

#[test]
fn test_grid_with_row_gap() {
    let mut ctx = WidgetContext::new_test();

    let grid = Grid::new(
        (
            Text::new("A"),
            Text::new("B"),
            Text::new("C"),
            Text::new("D"),
        ),
        2,
    )
    .row_gap(12.0);

    let node_id = grid.build(&mut ctx);

    // Grid container (column direction) should have row_gap
    let layout = ctx.get_layout_style(node_id).unwrap();
    assert_eq!(layout.gap, 12.0, "Grid should have row gap of 12.0");
}

#[test]
fn test_grid_with_padding() {
    let mut ctx = WidgetContext::new_test();

    let grid = Grid::new((Text::new("A"), Text::new("B")), 2).padding(16.0);

    let node_id = grid.build(&mut ctx);

    let layout = ctx.get_layout_style(node_id).unwrap();
    assert_eq!(layout.padding_top, 16.0);
    assert_eq!(layout.padding_bottom, 16.0);
    assert_eq!(layout.padding_left, 16.0);
    assert_eq!(layout.padding_right, 16.0);
}

#[test]
fn test_grid_single_column() {
    let mut ctx = WidgetContext::new_test();

    // 3 items, 1 column = 3 rows
    let grid = Grid::new((Text::new("A"), Text::new("B"), Text::new("C")), 1);

    let node_id = grid.build(&mut ctx);

    let scene_node = ctx.scene().get_node(node_id).unwrap();
    assert_eq!(scene_node.children.len(), 3, "Should have 3 rows");

    // Each row should have 1 child
    for row_idx in 0..3 {
        let row_id = scene_node.children[row_idx];
        let row_node = ctx.scene().get_node(row_id).unwrap();
        assert_eq!(row_node.children.len(), 1, "Each row should have 1 child");
    }
}
