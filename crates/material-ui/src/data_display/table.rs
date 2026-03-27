//! MD3 Table widget -- data table with header row and data rows.

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

/// MD3 surface (#FEF7FF)
const FALLBACK_SURFACE: Vec4 = Vec4::new(0.996, 0.969, 1.0, 1.0);

/// MD3 surface_variant (#E7E0EC)
const FALLBACK_SURFACE_VARIANT: Vec4 = Vec4::new(0.906, 0.878, 0.925, 1.0);

/// MD3 on_surface (#1D1B20)
const FALLBACK_ON_SURFACE: Vec4 = Vec4::new(0.114, 0.106, 0.125, 1.0);

/// MD3 outline (#79747E)
const FALLBACK_OUTLINE: Vec4 = Vec4::new(0.475, 0.455, 0.494, 1.0);

/// Transparent fill.
const TRANSPARENT: Vec4 = Vec4::new(0.0, 0.0, 0.0, 0.0);

/// Header row height in dp.
const HEADER_HEIGHT: f32 = 56.0;

/// Data row height in dp.
const ROW_HEIGHT: f32 = 52.0;

/// Header font size in dp (MD3 label_large).
const HEADER_FONT_SIZE: f32 = 14.0;

/// Body font size in dp (MD3 body_medium).
const BODY_FONT_SIZE: f32 = 14.0;

/// Divider thickness in dp.
const DIVIDER_THICKNESS: f32 = 1.0;

/// Cell horizontal padding in dp.
const CELL_PADDING_H: f32 = 16.0;

/// MD3 Table -- data table with header row and data rows.
///
/// Renders a structured table with a styled header row and alternating data
/// rows. Each row is a horizontal container of text cells. Optionally supports
/// row-click callbacks and custom column widths.
///
/// # Example
///
/// ```rust,no_run
/// use material_ui::data_display::Table;
///
/// let table = Table::new(
///     vec!["Name", "Age", "City"],
///     vec![
///         vec!["Alice", "30", "Portland"],
///         vec!["Bob", "25", "Seattle"],
///     ],
/// )
/// .on_row_click(|idx| println!("Clicked row {idx}"));
/// ```
pub struct Table {
    headers: Vec<String>,
    rows: Vec<Vec<String>>,
    column_widths: Option<Vec<f32>>,
    on_row_click: Option<Arc<dyn Fn(usize) + Send + Sync>>,
}

impl Table {
    /// Create a new table with the given headers and rows.
    pub fn new(headers: Vec<impl Into<String>>, rows: Vec<Vec<impl Into<String>>>) -> Self {
        Self {
            headers: headers.into_iter().map(Into::into).collect(),
            rows: rows
                .into_iter()
                .map(|row| row.into_iter().map(Into::into).collect())
                .collect(),
            column_widths: None,
            on_row_click: None,
        }
    }

    /// Set explicit column widths in dp. If not set, columns share equal width.
    pub fn column_widths(mut self, widths: Vec<f32>) -> Self {
        self.column_widths = Some(widths);
        self
    }

    /// Set a callback invoked when a data row is clicked. The callback receives
    /// the zero-based row index.
    pub fn on_row_click(mut self, f: impl Fn(usize) + Send + Sync + 'static) -> Self {
        self.on_row_click = Some(Arc::new(f));
        self
    }
}

impl Widget for Table {
    fn build(&self, ctx: &mut WidgetContext) -> NodeId {
        // -- Resolve theme colors --
        let theme = ctx.get_extension::<MaterialTheme>().cloned();
        let surface = theme
            .as_ref()
            .map(|t| t.color.surface)
            .unwrap_or(FALLBACK_SURFACE);
        let surface_variant = theme
            .as_ref()
            .map(|t| t.color.surface_variant)
            .unwrap_or(FALLBACK_SURFACE_VARIANT);
        let on_surface = theme
            .as_ref()
            .map(|t| t.color.on_surface)
            .unwrap_or(FALLBACK_ON_SURFACE);
        let outline = theme
            .as_ref()
            .map(|t| t.color.outline)
            .unwrap_or(FALLBACK_OUTLINE);

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
                ..Default::default()
            },
        );

        // -- Header row --
        let header_style = VisualStyle::new().solid_fill(surface_variant);
        let header_row = ctx.create_node(
            root,
            NodeContent::Styled {
                style: Box::new(header_style),
            },
        );
        ctx.set_layout_style(
            header_row,
            FlexStyle {
                direction: FlexDirection::Row,
                height: Some(HEADER_HEIGHT),
                align_items: FlexAlign::Center,
                ..Default::default()
            },
        );

        for (col_idx, header_text) in self.headers.iter().enumerate() {
            let cell_style = VisualStyle::new()
                .solid_fill(on_surface)
                .text(TextContent::new(header_text.clone(), HEADER_FONT_SIZE));
            let cell = ctx.create_node(
                header_row,
                NodeContent::Styled {
                    style: Box::new(cell_style),
                },
            );
            let mut cell_layout = FlexStyle {
                padding_left: CELL_PADDING_H,
                padding_right: CELL_PADDING_H,
                ..Default::default()
            };
            if let Some(ref widths) = self.column_widths {
                if let Some(&w) = widths.get(col_idx) {
                    cell_layout.width = Some(w);
                }
            } else {
                cell_layout.flex_grow = 1.0;
            }
            ctx.set_layout_style(cell, cell_layout);
        }

        // -- Data rows --
        for (row_idx, row_data) in self.rows.iter().enumerate() {
            // Divider between header/rows and between rows
            let divider_style = VisualStyle::new().solid_fill(outline);
            let divider = ctx.create_node(
                root,
                NodeContent::Styled {
                    style: Box::new(divider_style),
                },
            );
            ctx.set_layout_style(
                divider,
                FlexStyle {
                    height: Some(DIVIDER_THICKNESS),
                    ..Default::default()
                },
            );

            // Alternating row background
            let row_bg = if row_idx % 2 == 0 {
                TRANSPARENT
            } else {
                surface
            };
            let row_style = VisualStyle::new().solid_fill(row_bg);
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
                    height: Some(ROW_HEIGHT),
                    align_items: FlexAlign::Center,
                    ..Default::default()
                },
            );

            for (col_idx, cell_text) in row_data.iter().enumerate() {
                let cell_style = VisualStyle::new()
                    .solid_fill(on_surface)
                    .text(TextContent::new(cell_text.clone(), BODY_FONT_SIZE));
                let cell = ctx.create_node(
                    row_node,
                    NodeContent::Styled {
                        style: Box::new(cell_style),
                    },
                );
                let mut cell_layout = FlexStyle {
                    padding_left: CELL_PADDING_H,
                    padding_right: CELL_PADDING_H,
                    ..Default::default()
                };
                if let Some(ref widths) = self.column_widths {
                    if let Some(&w) = widths.get(col_idx) {
                        cell_layout.width = Some(w);
                    }
                } else {
                    cell_layout.flex_grow = 1.0;
                }
                ctx.set_layout_style(cell, cell_layout);
            }

            // -- Row click handler --
            if let Some(ref on_click) = self.on_row_click {
                let callback = Arc::clone(on_click);
                let idx = row_idx;
                ctx.add_clickable(row_node, Arc::new(move || callback(idx)));
            }
        }

        root
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use render_engine::Paint;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn test_table_builds_with_headers_and_rows() {
        let mut ctx = WidgetContext::new_test();
        let table = Table::new(
            vec!["Name", "Age"],
            vec![vec!["Alice", "30"], vec!["Bob", "25"]],
        );
        let root_id = table.build(&mut ctx);

        assert!(
            ctx.scene().get_node(root_id).is_some(),
            "Table root node should exist in scene"
        );
    }

    #[test]
    fn test_table_correct_row_count() {
        let mut ctx = WidgetContext::new_test();
        let table = Table::new(
            vec!["A", "B"],
            vec![vec!["1", "2"], vec!["3", "4"], vec!["5", "6"]],
        );
        let root_id = table.build(&mut ctx);

        let root_node = ctx.scene().get_node(root_id).unwrap();
        // Children: 1 header + 3*(divider + row) = 1 + 6 = 7
        assert_eq!(
            root_node.children.len(),
            7,
            "Table should have header + 3*(divider+row) = 7 children, got {}",
            root_node.children.len()
        );
    }

    #[test]
    fn test_table_header_styling() {
        let mut ctx = WidgetContext::new_test();
        let table = Table::new(vec!["Name"], vec![vec!["Alice"]]);
        let root_id = table.build(&mut ctx);

        let root_node = ctx.scene().get_node(root_id).unwrap();
        let header_id = root_node.children[0];
        let header_node = ctx.scene().get_node(header_id).unwrap();

        // Header should have surface_variant background
        if let NodeContent::Styled { ref style } = header_node.content {
            assert!(!style.fills.is_empty(), "Header should have a fill");
            if let Paint::Solid(color) = &style.fills[0] {
                assert!(
                    (color.x - FALLBACK_SURFACE_VARIANT.x).abs() < 0.01
                        && (color.y - FALLBACK_SURFACE_VARIANT.y).abs() < 0.01
                        && (color.z - FALLBACK_SURFACE_VARIANT.z).abs() < 0.01,
                    "Header fill should match surface_variant, got {color:?}"
                );
            } else {
                panic!("Header fill should be a solid paint");
            }
        } else {
            panic!("Header node should be Styled content");
        }

        // Header should have cell children
        assert!(
            !header_node.children.is_empty(),
            "Header row should have cell children"
        );
    }

    #[test]
    fn test_table_row_click_callback() {
        let clicked_row = Arc::new(AtomicUsize::new(usize::MAX));
        let clicked_clone = Arc::clone(&clicked_row);

        let mut ctx = WidgetContext::new_test();
        let table =
            Table::new(vec!["Col"], vec![vec!["Row 0"], vec!["Row 1"]]).on_row_click(move |idx| {
                clicked_clone.store(idx, Ordering::SeqCst);
            });
        let root_id = table.build(&mut ctx);

        let root_node = ctx.scene().get_node(root_id).unwrap();
        // Children: header, divider, row0, divider, row1 => indices 0,1,2,3,4
        // Row 1 is at index 4
        let row1_id = root_node.children[4];
        assert!(
            ctx.has_clickable(row1_id),
            "Data row with on_row_click should be clickable"
        );

        ctx.trigger_click(row1_id);
        assert_eq!(
            clicked_row.load(Ordering::SeqCst),
            1,
            "Clicking second row should pass index 1"
        );
    }

    #[test]
    fn test_table_empty_rows() {
        let mut ctx = WidgetContext::new_test();
        let table = Table::new(vec!["Name", "Age"], Vec::<Vec<String>>::new());
        let root_id = table.build(&mut ctx);

        let root_node = ctx.scene().get_node(root_id).unwrap();
        // Only the header row, no data rows or dividers
        assert_eq!(
            root_node.children.len(),
            1,
            "Table with no data rows should have only the header, got {}",
            root_node.children.len()
        );
    }
}
