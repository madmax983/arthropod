//! MD3 Pagination widget -- page navigation with prev/next buttons.

use crate::theme::MaterialTheme;
use flux_state::{Effect, Signal};
use glam::Vec4;
use layout_engine::{FlexAlign, FlexDirection, FlexStyle};
use render_engine::node::NodeContent;
use render_engine::{Color, NodeId, TextContent, VisualStyle};
use std::sync::Arc;
use widget_core::Widget;
use widget_core::WidgetContext;

// ---------------------------------------------------------------------------
// MD3 fallback colors
// ---------------------------------------------------------------------------

/// MD3 primary (#6750A4) -- selected page number.
const FALLBACK_PRIMARY: Vec4 = Vec4::new(0.404, 0.314, 0.643, 1.0);

/// MD3 on_surface (#1D1B20) -- unselected page numbers and button text.
const FALLBACK_ON_SURFACE: Vec4 = Vec4::new(0.114, 0.106, 0.125, 1.0);

/// MD3 on_surface_variant (#49454F) -- disabled button text.
const FALLBACK_ON_SURFACE_VARIANT: Vec4 = Vec4::new(0.286, 0.271, 0.310, 1.0);

// ---------------------------------------------------------------------------
// Dimension constants (dp)
// ---------------------------------------------------------------------------

/// Font size for page numbers and nav buttons.
const LABEL_FONT_SIZE: f32 = 14.0;

/// Gap between pagination elements.
const ITEM_GAP: f32 = 4.0;

/// Touch target size for page number buttons.
const PAGE_BUTTON_SIZE: f32 = 36.0;

/// MD3 Pagination -- page navigation.
///
/// Renders a horizontal row with "< Prev" and "Next >" buttons flanking a
/// series of page number buttons. The selected page is styled with the
/// primary color; unselected pages use `on_surface`. Clicking a page number
/// sets the signal. Prev/Next buttons adjust the signal by -1/+1 (clamped
/// to 1..=total_pages).
///
/// # Example
///
/// ```rust,no_run
/// use flux_state::{Runtime, Signal};
/// use material_ui::navigation::Pagination;
///
/// let runtime = Runtime::new();
/// let page = Signal::new(runtime, 1usize);
///
/// let pagination = Pagination::new(5, page);
/// ```
pub struct Pagination {
    total_pages: usize,
    signal: Signal<usize>,
}

impl Pagination {
    /// Create a new pagination widget.
    ///
    /// # Arguments
    ///
    /// * `total_pages` - Total number of pages (1-based).
    /// * `signal` - A signal holding the currently selected page (1-based).
    pub fn new(total_pages: usize, signal: Signal<usize>) -> Self {
        Self {
            total_pages,
            signal,
        }
    }
}

impl Widget for Pagination {
    fn build(&self, ctx: &mut WidgetContext) -> NodeId {
        let (read, write) = self.signal.clone().split();
        let runtime = read.runtime().clone();

        // -- Resolve theme colors --
        let theme = ctx.get_extension::<MaterialTheme>().cloned();
        let primary = theme
            .as_ref()
            .map(|t| t.color.primary)
            .unwrap_or(FALLBACK_PRIMARY);
        let on_surface = theme
            .as_ref()
            .map(|t| t.color.on_surface)
            .unwrap_or(FALLBACK_ON_SURFACE);
        let on_surface_variant = theme
            .as_ref()
            .map(|t| t.color.on_surface_variant)
            .unwrap_or(FALLBACK_ON_SURFACE_VARIANT);

        let current_page = read.get_untracked();

        // -- Root row container --
        let root = ctx.create_node(ctx.root(), NodeContent::Empty);
        ctx.set_layout_style(
            root,
            FlexStyle {
                direction: FlexDirection::Row,
                align_items: FlexAlign::Center,
                gap: ITEM_GAP,
                ..Default::default()
            },
        );

        // -- "< Prev" button --
        let prev_color = if current_page <= 1 {
            on_surface_variant
        } else {
            on_surface
        };
        let prev_style = VisualStyle::new()
            .solid_fill(prev_color)
            .text(TextContent::new("\u{2039} Prev", LABEL_FONT_SIZE));

        let prev_node = ctx.create_node(
            root,
            NodeContent::Styled {
                style: Box::new(prev_style),
            },
        );
        ctx.set_layout_style(
            prev_node,
            FlexStyle {
                height: Some(PAGE_BUTTON_SIZE),
                align_items: FlexAlign::Center,
                ..Default::default()
            },
        );

        // Prev click handler
        {
            let w = write.clone();
            let r = read.clone();
            ctx.add_clickable(
                prev_node,
                Arc::new(move || {
                    let current = r.get_untracked();
                    if current > 1 {
                        w.set(current - 1);
                    }
                }),
            );
        }

        // Reactive color on prev button
        let prev_color_signal = Signal::new(
            runtime.clone(),
            Color::rgba(prev_color.x, prev_color.y, prev_color.z, prev_color.w),
        );
        let (prev_cr, prev_cw) = prev_color_signal.split();
        ctx.add_reactive_color_state(prev_node, prev_cr);

        // -- Page number buttons --
        let mut page_color_writers = Vec::new();

        for page_num in 1..=self.total_pages {
            let is_selected = page_num == current_page;
            let text_color = if is_selected { primary } else { on_surface };

            let page_style = VisualStyle::new()
                .solid_fill(text_color)
                .text(TextContent::new(page_num.to_string(), LABEL_FONT_SIZE));

            let page_node = ctx.create_node(
                root,
                NodeContent::Styled {
                    style: Box::new(page_style),
                },
            );
            ctx.set_layout_style(
                page_node,
                FlexStyle {
                    width: Some(PAGE_BUTTON_SIZE),
                    height: Some(PAGE_BUTTON_SIZE),
                    align_items: FlexAlign::Center,
                    justify_content: layout_engine::FlexJustifyContent::Center,
                    ..Default::default()
                },
            );

            // Click handler
            let w = write.clone();
            ctx.add_clickable(
                page_node,
                Arc::new(move || {
                    w.set(page_num);
                }),
            );

            // Reactive color
            let page_color_signal = Signal::new(
                runtime.clone(),
                Color::rgba(text_color.x, text_color.y, text_color.z, text_color.w),
            );
            let (page_cr, page_cw) = page_color_signal.split();
            ctx.add_reactive_color_state(page_node, page_cr);
            page_color_writers.push(page_cw);
        }

        // -- "Next >" button --
        let next_color = if current_page >= self.total_pages {
            on_surface_variant
        } else {
            on_surface
        };
        let next_style = VisualStyle::new()
            .solid_fill(next_color)
            .text(TextContent::new("Next \u{203A}", LABEL_FONT_SIZE));

        let next_node = ctx.create_node(
            root,
            NodeContent::Styled {
                style: Box::new(next_style),
            },
        );
        ctx.set_layout_style(
            next_node,
            FlexStyle {
                height: Some(PAGE_BUTTON_SIZE),
                align_items: FlexAlign::Center,
                ..Default::default()
            },
        );

        // Next click handler
        {
            let w = write.clone();
            let r = read.clone();
            let total = self.total_pages;
            ctx.add_clickable(
                next_node,
                Arc::new(move || {
                    let current = r.get_untracked();
                    if current < total {
                        w.set(current + 1);
                    }
                }),
            );
        }

        // Reactive color on next button
        let next_color_signal = Signal::new(
            runtime.clone(),
            Color::rgba(next_color.x, next_color.y, next_color.z, next_color.w),
        );
        let (next_cr, next_cw) = next_color_signal.split();
        ctx.add_reactive_color_state(next_node, next_cr);

        // -- Reactive effect: update colors when page changes --
        let total_pages = self.total_pages;
        let effect = Effect::new(runtime.clone(), move || {
            let current = read.get();

            // Update prev button color
            let pc = if current <= 1 {
                on_surface_variant
            } else {
                on_surface
            };
            prev_cw.set(Color::rgba(pc.x, pc.y, pc.z, pc.w));

            // Update next button color
            let nc = if current >= total_pages {
                on_surface_variant
            } else {
                on_surface
            };
            next_cw.set(Color::rgba(nc.x, nc.y, nc.z, nc.w));

            // Update page number colors
            for (i, writer) in page_color_writers.iter().enumerate() {
                let page = i + 1;
                let color = if page == current { primary } else { on_surface };
                writer.set(Color::rgba(color.x, color.y, color.z, color.w));
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

    #[test]
    fn test_pagination_builds_page_numbers() {
        let runtime = Runtime::new();
        let page = Signal::new(runtime, 1usize);

        let mut ctx = WidgetContext::new_test();
        let pagination = Pagination::new(5, page);
        let root_id = pagination.build(&mut ctx);

        let root_node = ctx.scene().get_node(root_id).unwrap();
        // "< Prev" + 5 page numbers + "Next >" = 7 children
        assert_eq!(
            root_node.children.len(),
            7,
            "Prev + 5 pages + Next = 7 children, got {}",
            root_node.children.len()
        );
    }

    #[test]
    fn test_pagination_click_page_sets_signal() {
        let runtime = Runtime::new();
        let page = Signal::new(runtime, 1usize);
        let observer = page.clone();
        let (obs_read, _) = observer.split();

        let mut ctx = WidgetContext::new_test();
        let pagination = Pagination::new(5, page);
        let root_id = pagination.build(&mut ctx);

        assert_eq!(obs_read.get_untracked(), 1, "Should start at page 1");

        let root_node = ctx.scene().get_node(root_id).unwrap();
        // children[0] = Prev, children[1..=5] = pages 1-5, children[6] = Next
        // Click page 3 (children[3])
        let page3_id = root_node.children[3];
        ctx.trigger_click(page3_id);

        assert_eq!(
            obs_read.get_untracked(),
            3,
            "After clicking page 3, signal should be 3"
        );
    }

    #[test]
    fn test_pagination_prev_next_buttons() {
        let runtime = Runtime::new();
        let page = Signal::new(runtime, 3usize);
        let observer = page.clone();
        let (obs_read, _) = observer.split();

        let mut ctx = WidgetContext::new_test();
        let pagination = Pagination::new(5, page);
        let root_id = pagination.build(&mut ctx);

        let root_node = ctx.scene().get_node(root_id).unwrap();
        let prev_id = root_node.children[0];
        let next_id = *root_node.children.last().unwrap();

        // Click prev
        ctx.trigger_click(prev_id);
        assert_eq!(
            obs_read.get_untracked(),
            2,
            "After clicking Prev from page 3, signal should be 2"
        );

        // Click next
        ctx.trigger_click(next_id);
        assert_eq!(
            obs_read.get_untracked(),
            3,
            "After clicking Next from page 2, signal should be 3"
        );
    }

    #[test]
    fn test_pagination_clamping_at_bounds() {
        let runtime = Runtime::new();
        // Start at page 1
        let page = Signal::new(runtime, 1usize);
        let observer = page.clone();
        let (obs_read, _) = observer.split();

        let mut ctx = WidgetContext::new_test();
        let pagination = Pagination::new(3, page);
        let root_id = pagination.build(&mut ctx);

        let children: Vec<NodeId> = ctx.scene().get_node(root_id).unwrap().children.clone();
        let prev_id = children[0];
        let next_id = *children.last().unwrap();

        // Click prev at page 1 -- should stay at 1
        ctx.trigger_click(prev_id);
        assert_eq!(
            obs_read.get_untracked(),
            1,
            "Prev at page 1 should clamp to 1"
        );

        // Navigate to last page
        let page3_id = children[3]; // page 3
        ctx.trigger_click(page3_id);
        assert_eq!(obs_read.get_untracked(), 3);

        // Click next at last page -- should stay at 3
        ctx.trigger_click(next_id);
        assert_eq!(
            obs_read.get_untracked(),
            3,
            "Next at last page should clamp to total_pages"
        );
    }
}
