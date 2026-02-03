//! Core Widget traits

use crate::context::WidgetContext;
use render_engine::NodeId;

/// The Widget trait - implemented by all UI widgets
///
/// Widgets are declarative UI building blocks that:
/// 1. Build scene nodes (hierarchy)
/// 2. Spawn ECS entities (components)
/// 3. Configure layout (flexbox)
/// 4. Handle state updates (reactive)
///
/// # Implementing Custom Widgets
///
/// To create a custom widget, implement the `Widget` trait. The `build` method is responsible
/// for constructing the visual and behavioral representation of your widget.
///
/// ## Lifecycle of `build`
///
/// 1. **Create Node**: Use `ctx.create_node()` to add a node to the scene.
/// 2. **Layout**: Use `ctx.set_layout_style()` to define size, padding, etc.
/// 3. **Composition**: If your widget has children, build them and use `ctx.reparent_node()` to attach them.
/// 4. **Interaction**: Use `ctx.add_clickable()` or `ctx.add_hover_state()` for events.
///
/// ## Example: Colored Box Widget
///
/// ```
/// use widget_core::{Widget, WidgetContext};
/// use render_engine::{NodeContent, NodeId, Color};
/// use layout_engine::FlexStyle;
/// use glam::Vec4;
///
/// pub struct ColoredBox {
///     color: Color,
///     width: f32,
///     height: f32,
/// }
///
/// impl ColoredBox {
///     pub fn new(color: Color, width: f32, height: f32) -> Self {
///         Self { color, width, height }
///     }
/// }
///
/// impl Widget for ColoredBox {
///     fn build(&self, ctx: &mut WidgetContext) -> NodeId {
///         // 1. Create the scene node attached to the root (initially)
///         let node_id = ctx.create_node(
///             ctx.root(),
///             NodeContent::Rect {
///                 color: self.color,
///             },
///         );
///
///         // 2. Configure layout
///         ctx.set_layout_style(node_id, FlexStyle {
///             width: Some(self.width),
///             height: Some(self.height),
///             ..Default::default()
///         });
///
///         // 3. Return the node ID so the parent can reparent it
///         node_id
///     }
/// }
/// ```
pub trait Widget {
    /// Build this widget, returning the root scene node ID
    ///
    /// This creates:
    /// - Scene nodes for visual hierarchy
    /// - ECS entities with components
    /// - Layout nodes for flexbox
    ///
    /// Note: The returned `NodeId` is initially attached to the scene root (or whatever parent
    /// was passed to `create_node`). The caller of `build` (e.g., a container widget) is
    /// responsible for reparenting it if necessary.
    fn build(&self, ctx: &mut WidgetContext) -> NodeId;
}

/// Trait for building multiple widgets as children
///
/// This enables type-safe, generic children with no vtable overhead.
/// Implemented for:
/// - Single widgets (`impl Widget`)
/// - Empty tuple `()`
/// - Tuples of widgets `(A, B)`, `(A, B, C)`, etc.
///
/// # Example
///
/// ```no_run
/// use widget_core::{WidgetTuple, Text, Button};
///
/// pub struct Column<C: WidgetTuple> {
///     children: C,
///     gap: f32,
/// }
///
/// // Usage with tuple:
/// Column {
///     children: (Text::new("A"), Text::new("B"), Button::new("C")),
///     gap: 10.0,
/// };
/// ```
pub trait WidgetTuple {
    /// Build all widgets and attach them as children of parent
    fn build_all(&self, ctx: &mut WidgetContext, parent: NodeId);

    /// Number of children in this tuple
    fn len(&self) -> usize;

    /// Whether this tuple is empty
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

// ===== WidgetTuple Implementations =====

/// Empty tuple - no children
impl WidgetTuple for () {
    fn build_all(&self, _ctx: &mut WidgetContext, _parent: NodeId) {}

    fn len(&self) -> usize {
        0
    }
}

/// Single widget directly (no tuple wrapper needed)
/// Allows: `Container::column(my_widget)` instead of `Container::column((my_widget,))`
impl<W: Widget> WidgetTuple for W {
    fn build_all(&self, ctx: &mut WidgetContext, parent: NodeId) {
        let id = self.build(ctx);
        ctx.reparent_to(id, parent);
    }

    fn len(&self) -> usize {
        1
    }
}

/// Macro to generate WidgetTuple impls for tuples of various sizes
macro_rules! impl_widget_tuple {
    ($($idx:tt : $T:ident),+) => {
        impl<$($T: Widget),+> WidgetTuple for ($($T,)+) {
            fn build_all(&self, ctx: &mut WidgetContext, parent: NodeId) {
                $(
                    let id = self.$idx.build(ctx);
                    ctx.reparent_to(id, parent);
                )+
            }

            fn len(&self) -> usize {
                impl_widget_tuple!(@count $($T)+)
            }
        }
    };

    // Count helper
    (@count $($T:ident)+) => {
        <[()]>::len(&[$(impl_widget_tuple!(@replace $T ())),+])
    };
    (@replace $_:ident $sub:expr) => { $sub };
}

// Generate impls for tuples up to 12 elements
impl_widget_tuple!(0: A); // Single-element tuple (A,)
impl_widget_tuple!(0: A, 1: B);
impl_widget_tuple!(0: A, 1: B, 2: C);
impl_widget_tuple!(0: A, 1: B, 2: C, 3: D);
impl_widget_tuple!(0: A, 1: B, 2: C, 3: D, 4: E);
impl_widget_tuple!(0: A, 1: B, 2: C, 3: D, 4: E, 5: F);
impl_widget_tuple!(0: A, 1: B, 2: C, 3: D, 4: E, 5: F, 6: G);
impl_widget_tuple!(0: A, 1: B, 2: C, 3: D, 4: E, 5: F, 6: G, 7: H);
impl_widget_tuple!(0: A, 1: B, 2: C, 3: D, 4: E, 5: F, 6: G, 7: H, 8: I);
impl_widget_tuple!(0: A, 1: B, 2: C, 3: D, 4: E, 5: F, 6: G, 7: H, 8: I, 9: J);
impl_widget_tuple!(0: A, 1: B, 2: C, 3: D, 4: E, 5: F, 6: G, 7: H, 8: I, 9: J, 10: K);
impl_widget_tuple!(0: A, 1: B, 2: C, 3: D, 4: E, 5: F, 6: G, 7: H, 8: I, 9: J, 10: K, 11: L);

// ===== NamedWidgetTuple =====

/// Trait for building named widgets (for Form fields, slots, etc.)
///
/// Similar to `WidgetTuple` but each widget has an associated name.
/// Returns a mapping of names to node IDs for form state tracking.
///
/// # Example
///
/// ```no_run
/// use widget_core::{NamedWidgetTuple, TextInput, Widget};
/// use flux_state::{Signal, Runtime};
///
/// pub struct Form<F: NamedWidgetTuple> {
///     fields: F,
///     gap: f32,
/// }
///
/// impl<F: NamedWidgetTuple> Form<F> {
///     pub fn new(fields: F) -> Self {
///         Self { fields, gap: 10.0 }
///     }
/// }
///
/// let runtime = Runtime::new();
/// let username = Signal::new(runtime.clone(), String::new());
/// let email = Signal::new(runtime.clone(), String::new());
///
/// // Usage with tuple of (name, widget) pairs:
/// Form::new((
///     ("username", TextInput::new(username)),
///     ("email", TextInput::new(email)),
/// ));
/// ```
pub trait NamedWidgetTuple {
    /// Build all named widgets and attach as children, returning name -> node ID mapping
    fn build_all_named(
        &self,
        ctx: &mut WidgetContext,
        parent: NodeId,
    ) -> std::collections::HashMap<String, NodeId>;

    /// Number of named children
    fn len(&self) -> usize;

    /// Whether this tuple is empty
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Empty tuple - no named children
impl NamedWidgetTuple for () {
    fn build_all_named(
        &self,
        _ctx: &mut WidgetContext,
        _parent: NodeId,
    ) -> std::collections::HashMap<String, NodeId> {
        std::collections::HashMap::new()
    }

    fn len(&self) -> usize {
        0
    }
}

/// Single named widget directly (no tuple wrapper needed)
/// Allows: `Form::new(("name", widget))` instead of `Form::new((("name", widget),))`
impl<W: Widget> NamedWidgetTuple for (&str, W) {
    fn build_all_named(
        &self,
        ctx: &mut WidgetContext,
        parent: NodeId,
    ) -> std::collections::HashMap<String, NodeId> {
        let mut mapping = std::collections::HashMap::new();
        let id = self.1.build(ctx);
        ctx.reparent_to(id, parent);
        mapping.insert(self.0.to_string(), id);
        mapping
    }

    fn len(&self) -> usize {
        1
    }
}

/// Macro to generate NamedWidgetTuple impls for tuples of various sizes
macro_rules! impl_named_widget_tuple {
    ($($idx:tt : $T:ident),+) => {
        impl<$($T: Widget),+> NamedWidgetTuple for ($((&str, $T),)+) {
            fn build_all_named(&self, ctx: &mut WidgetContext, parent: NodeId) -> std::collections::HashMap<String, NodeId> {
                let mut mapping = std::collections::HashMap::new();
                $(
                    let id = self.$idx.1.build(ctx);
                    ctx.reparent_to(id, parent);
                    mapping.insert(self.$idx.0.to_string(), id);
                )+
                mapping
            }

            fn len(&self) -> usize {
                impl_named_widget_tuple!(@count $($T)+)
            }
        }
    };

    // Count helper
    (@count $($T:ident)+) => {
        <[()]>::len(&[$(impl_named_widget_tuple!(@replace $T ())),+])
    };
    (@replace $_:ident $sub:expr) => { $sub };
}

// Generate impls for named tuples up to 12 elements
impl_named_widget_tuple!(0: A); // Single-element tuple ((&str, A),)
impl_named_widget_tuple!(0: A, 1: B);
impl_named_widget_tuple!(0: A, 1: B, 2: C);
impl_named_widget_tuple!(0: A, 1: B, 2: C, 3: D);
impl_named_widget_tuple!(0: A, 1: B, 2: C, 3: D, 4: E);
impl_named_widget_tuple!(0: A, 1: B, 2: C, 3: D, 4: E, 5: F);
impl_named_widget_tuple!(0: A, 1: B, 2: C, 3: D, 4: E, 5: F, 6: G);
impl_named_widget_tuple!(0: A, 1: B, 2: C, 3: D, 4: E, 5: F, 6: G, 7: H);
impl_named_widget_tuple!(0: A, 1: B, 2: C, 3: D, 4: E, 5: F, 6: G, 7: H, 8: I);
impl_named_widget_tuple!(0: A, 1: B, 2: C, 3: D, 4: E, 5: F, 6: G, 7: H, 8: I, 9: J);
impl_named_widget_tuple!(0: A, 1: B, 2: C, 3: D, 4: E, 5: F, 6: G, 7: H, 8: I, 9: J, 10: K);
impl_named_widget_tuple!(0: A, 1: B, 2: C, 3: D, 4: E, 5: F, 6: G, 7: H, 8: I, 9: J, 10: K, 11: L);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_tuple_len() {
        let t: () = ();
        // Disambiguate between WidgetTuple and NamedWidgetTuple
        assert_eq!(WidgetTuple::len(&t), 0);
        assert!(WidgetTuple::is_empty(&t));
    }

    #[test]
    fn test_tuple_len() {
        // We can't easily test without actual Widget impls,
        // but we can verify the count macro works
        assert_eq!(<[()]>::len(&[(), (), ()]), 3);
    }
}
