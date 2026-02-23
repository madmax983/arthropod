//! List widget - dynamic collection of widgets with type erasure

use crate::{Widget, WidgetContext};
use layout_engine::{FlexDirection, FlexStyle};
use render_engine::{NodeContent, NodeId};

/// Trait for type-erased widgets that can be stored in a Box
///
/// This enables dynamic collections like `Vec<Box<dyn WidgetBoxed>>`
/// where widgets of different types can be stored together.
///
/// # Example
///
/// ```no_run
/// use widget_core::{WidgetBoxed, Text, Button};
///
/// let mut widgets: Vec<Box<dyn WidgetBoxed>> = Vec::new();
/// widgets.push(Box::new(Text::new("Hello")));
/// widgets.push(Box::new(Button::new("Click me")));
/// ```
pub trait WidgetBoxed {
    /// Build this widget, returning the root scene node ID
    fn build_boxed(&self, ctx: &mut WidgetContext) -> NodeId;
}

/// Blanket implementation: any Widget can be WidgetBoxed
impl<W: Widget> WidgetBoxed for W {
    fn build_boxed(&self, ctx: &mut WidgetContext) -> NodeId {
        self.build(ctx)
    }
}

/// List widget for dynamic collections of widgets
///
/// Unlike Container which uses compile-time tuples, List uses runtime
/// `Vec` storage with type erasure (`Box<dyn WidgetBoxed>`). This enables
/// dynamic widget lists from runtime data.
///
/// # Example
///
/// ```
/// use widget_core::{List, Text};
///
/// let items = vec!["Apple", "Banana", "Cherry"];
///
/// // Create a list and push items
/// let mut list = List::column().gap(8.0);
/// for item in &items {
///     list.push(Text::new(*item));
/// }
///
/// // Or use extend
/// let items2 = vec!["A", "B", "C"];
/// let mut list2 = List::row();
/// list2.extend(items2.iter().map(|s| Text::new(*s)));
/// ```
pub struct List {
    direction: FlexDirection,
    children: Vec<Box<dyn WidgetBoxed>>,
    gap: f32,
    padding: f32,
}

impl List {
    /// Create a column list (vertical)
    pub fn column() -> Self {
        Self {
            direction: FlexDirection::Column,
            children: Vec::new(),
            gap: 0.0,
            padding: 0.0,
        }
    }

    /// Create a row list (horizontal)
    pub fn row() -> Self {
        Self {
            direction: FlexDirection::Row,
            children: Vec::new(),
            gap: 0.0,
            padding: 0.0,
        }
    }

    /// Set gap between children
    pub fn gap(mut self, gap: f32) -> Self {
        self.gap = gap;
        self
    }

    /// Set uniform padding
    pub fn padding(mut self, padding: f32) -> Self {
        self.padding = padding;
        self
    }

    /// Add a widget to the end of the list
    pub fn push<W: Widget + 'static>(&mut self, widget: W) {
        self.children.push(Box::new(widget));
    }

    /// Extend the list with an iterator of widgets
    pub fn extend<I, W>(&mut self, iter: I)
    where
        I: IntoIterator<Item = W>,
        W: Widget + 'static,
    {
        self.children.extend(
            iter.into_iter()
                .map(|w| Box::new(w) as Box<dyn WidgetBoxed>),
        );
    }
}

impl Widget for List {
    fn build(&self, ctx: &mut WidgetContext) -> NodeId {
        // Create empty container node
        let root_id = ctx.root();
        let node_id = ctx.create_node(root_id, NodeContent::Empty);

        // Build all children and reparent them
        for child in &self.children {
            let child_id = child.build_boxed(ctx);
            ctx.reparent_to(child_id, node_id);
        }

        // Configure layout
        let style = FlexStyle {
            direction: self.direction,
            gap: self.gap,
            padding_left: self.padding,
            padding_right: self.padding,
            padding_top: self.padding,
            padding_bottom: self.padding,
            ..Default::default()
        };

        ctx.set_layout_style(node_id, style);

        node_id
    }
}

/// Helper function to create a column list from an iterator
///
/// # Example
///
/// ```no_run
/// use widget_core::{list_from, Text};
///
/// let items = vec!["Item 1", "Item 2", "Item 3"];
/// let list = list_from(items.iter().map(|s| Text::new(*s)));
/// ```
pub fn list_from<I, W>(iter: I) -> List
where
    I: IntoIterator<Item = W>,
    W: Widget + 'static,
{
    let mut list = List::column();
    list.extend(iter);
    list
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestWidget;

    impl Widget for TestWidget {
        fn build(&self, ctx: &mut WidgetContext) -> NodeId {
            ctx.create_node(
                ctx.root(),
                NodeContent::Styled {
                    style: Box::new(
                        render_engine::VisualStyle::new()
                            .solid_fill(render_engine::Color::rgba(1.0, 0.0, 0.0, 1.0).as_vec4()),
                    ),
                },
            )
        }
    }

    #[test]
    fn test_widget_boxed_trait() {
        let mut ctx = WidgetContext::new_test();
        let widget = TestWidget;

        // Test that Widget can be used as WidgetBoxed
        let boxed: Box<dyn WidgetBoxed> = Box::new(widget);
        let node_id = boxed.build_boxed(&mut ctx);

        assert!(ctx.scene().get_node(node_id).is_some());
    }

    #[test]
    fn test_list_push() {
        let mut list = List::column();
        assert_eq!(list.children.len(), 0);

        list.push(TestWidget);
        assert_eq!(list.children.len(), 1);

        list.push(TestWidget);
        assert_eq!(list.children.len(), 2);
    }

    #[test]
    fn test_list_extend() {
        let mut list = List::column();
        let widgets = vec![TestWidget, TestWidget, TestWidget];

        list.extend(widgets);
        assert_eq!(list.children.len(), 3);
    }
}
