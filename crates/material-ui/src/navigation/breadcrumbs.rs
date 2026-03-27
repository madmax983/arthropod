//! MD3 Breadcrumbs widget -- path navigation with separator.

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

/// MD3 primary (#6750A4) -- clickable breadcrumb items.
const FALLBACK_PRIMARY: Vec4 = Vec4::new(0.404, 0.314, 0.643, 1.0);

/// MD3 on_surface (#1D1B20) -- current (last) breadcrumb item.
const FALLBACK_ON_SURFACE: Vec4 = Vec4::new(0.114, 0.106, 0.125, 1.0);

/// MD3 on_surface_variant (#49454F) -- separator text.
const FALLBACK_ON_SURFACE_VARIANT: Vec4 = Vec4::new(0.286, 0.271, 0.310, 1.0);

// ---------------------------------------------------------------------------
// Dimension constants (dp)
// ---------------------------------------------------------------------------

/// Default label font size.
const LABEL_FONT_SIZE: f32 = 14.0;

/// Default separator string.
const DEFAULT_SEPARATOR: &str = "/";

/// Gap between breadcrumb items and separators.
const ITEM_GAP: f32 = 4.0;

/// MD3 Breadcrumbs -- path navigation with customizable separator.
///
/// Renders a horizontal row of text items interspersed with separator strings.
/// All items except the last are styled with the primary color and are
/// clickable. The last item is rendered with `on_surface` to indicate the
/// current location.
///
/// # Example
///
/// ```rust,no_run
/// use material_ui::navigation::Breadcrumbs;
///
/// let crumbs = Breadcrumbs::new(vec!["Home", "Products", "Detail"])
///     .on_click(|index| println!("Navigate to crumb {index}"))
///     .separator(">");
/// ```
pub struct Breadcrumbs {
    items: Vec<String>,
    on_click: Option<Arc<dyn Fn(usize) + Send + Sync>>,
    separator: String,
}

impl Breadcrumbs {
    /// Create a new breadcrumbs widget with the given path items.
    pub fn new(items: Vec<impl Into<String>>) -> Self {
        Self {
            items: items.into_iter().map(Into::into).collect(),
            on_click: None,
            separator: DEFAULT_SEPARATOR.to_owned(),
        }
    }

    /// Set the callback invoked when a breadcrumb item is clicked.
    ///
    /// The callback receives the zero-based index of the clicked item.
    /// The last (current) item is not clickable.
    pub fn on_click(mut self, f: impl Fn(usize) + Send + Sync + 'static) -> Self {
        self.on_click = Some(Arc::new(f));
        self
    }

    /// Set a custom separator string (default: `"/"`).
    pub fn separator(mut self, sep: impl Into<String>) -> Self {
        self.separator = sep.into();
        self
    }
}

impl Widget for Breadcrumbs {
    fn build(&self, ctx: &mut WidgetContext) -> NodeId {
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

        let last_index = self.items.len().saturating_sub(1);

        for (i, item_text) in self.items.iter().enumerate() {
            let is_last = i == last_index;

            // Item text color: last item = on_surface (current), others = primary (clickable)
            let text_color = if is_last { on_surface } else { primary };

            let item_style = VisualStyle::new()
                .solid_fill(text_color)
                .text(TextContent::new(item_text.clone(), LABEL_FONT_SIZE));

            let item_node = ctx.create_node(
                root,
                NodeContent::Styled {
                    style: Box::new(item_style),
                },
            );

            // Click handler for non-last items
            if !is_last && let Some(ref on_click) = self.on_click {
                let callback = Arc::clone(on_click);
                let index = i;
                ctx.add_clickable(
                    item_node,
                    Arc::new(move || {
                        callback(index);
                    }),
                );
            }

            // Add separator between items (not after the last one)
            if !is_last {
                let sep_style = VisualStyle::new()
                    .solid_fill(on_surface_variant)
                    .text(TextContent::new(self.separator.clone(), LABEL_FONT_SIZE));

                ctx.create_node(
                    root,
                    NodeContent::Styled {
                        style: Box::new(sep_style),
                    },
                );
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
    fn test_breadcrumbs_builds_items_and_separators() {
        let mut ctx = WidgetContext::new_test();
        let crumbs = Breadcrumbs::new(vec!["Home", "Products", "Detail"]);
        let root_id = crumbs.build(&mut ctx);

        let root_node = ctx.scene().get_node(root_id).unwrap();
        // 3 items + 2 separators = 5 children
        assert_eq!(
            root_node.children.len(),
            5,
            "3 items + 2 separators = 5 children, got {}",
            root_node.children.len()
        );
    }

    #[test]
    fn test_breadcrumbs_click_callback_with_index() {
        let clicked_index = Arc::new(AtomicUsize::new(usize::MAX));
        let clicked_clone = Arc::clone(&clicked_index);

        let mut ctx = WidgetContext::new_test();
        let crumbs = Breadcrumbs::new(vec!["Home", "Products", "Detail"]).on_click(move |idx| {
            clicked_clone.store(idx, Ordering::SeqCst);
        });
        let root_id = crumbs.build(&mut ctx);

        // Collect all child IDs up front to avoid borrow conflicts
        let children: Vec<NodeId> = ctx.scene().get_node(root_id).unwrap().children.clone();

        // Click first item (index 0) -- children[0] is "Home"
        let first_item_id = children[0];
        assert!(
            ctx.has_clickable(first_item_id),
            "First breadcrumb item should be clickable"
        );
        ctx.trigger_click(first_item_id);
        assert_eq!(
            clicked_index.load(Ordering::SeqCst),
            0,
            "Clicking first item should pass index 0"
        );

        // Click second item (index 1) -- children[2] is "Products" (children[1] is separator)
        let second_item_id = children[2];
        ctx.trigger_click(second_item_id);
        assert_eq!(
            clicked_index.load(Ordering::SeqCst),
            1,
            "Clicking second item should pass index 1"
        );

        // Last item should NOT be clickable
        let last_item_id = children[4];
        assert!(
            !ctx.has_clickable(last_item_id),
            "Last breadcrumb item (current) should not be clickable"
        );
    }

    #[test]
    fn test_breadcrumbs_custom_separator() {
        let mut ctx = WidgetContext::new_test();
        let crumbs = Breadcrumbs::new(vec!["A", "B"]).separator(">");
        let root_id = crumbs.build(&mut ctx);

        let root_node = ctx.scene().get_node(root_id).unwrap();
        // children[1] is the separator node
        let sep_id = root_node.children[1];
        let sep_node = ctx.scene().get_node(sep_id).unwrap();

        if let NodeContent::Styled { ref style } = sep_node.content {
            let text = style
                .text
                .as_ref()
                .expect("Separator should have text content");
            assert_eq!(
                text.text, ">",
                "Custom separator should be '>', got '{}'",
                text.text
            );
            // Separator should use on_surface_variant color
            if let Some(Paint::Solid(color)) = style.fills.first() {
                assert!(
                    (color.x - FALLBACK_ON_SURFACE_VARIANT.x).abs() < 0.01,
                    "Separator should use on_surface_variant color, got {color:?}"
                );
            }
        } else {
            panic!("Separator node should be Styled content");
        }
    }
}
