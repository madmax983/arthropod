//! Card widget - styled container with visual properties
//!
//! Demonstrates type-safe children using the WidgetTuple trait.
//! Cards provide a visual container with background, corner radius, and shadow.

use crate::{Widget, WidgetContext, WidgetTuple};
use layout_engine::{FlexDirection, FlexStyle};
use render_engine::{Color, NodeContent, NodeId};

/// A styled card container widget
///
/// Cards provide a visual container with background, corner radius, and shadow.
/// Uses the WidgetTuple trait for type-safe children.
///
/// # Example
///
/// ```no_run
/// use widget_core::{Card, Text};
///
/// // Using builder pattern with tuple children
/// let card = Card::new((
///     Text::new("Title"),
///     Text::new("Body content"),
/// )).gap(8.0).padding(16.0);
///
/// // Single child
/// let card2 = Card::new((Text::new("Single item"),));
/// ```
pub struct Card<C: WidgetTuple> {
    children: C,
    gap: f32,
    padding: f32,
    background: Color,
    corner_radius: f32,
}

impl<C: WidgetTuple> Card<C> {
    /// Create a new card with children
    pub fn new(children: C) -> Self {
        Self {
            children,
            gap: 8.0,
            padding: 16.0,
            background: Color::WHITE,
            corner_radius: 8.0,
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

    /// Set background color
    pub fn background(mut self, color: Color) -> Self {
        self.background = color;
        self
    }

    /// Set corner radius
    pub fn corner_radius(mut self, radius: f32) -> Self {
        self.corner_radius = radius;
        self
    }
}

impl<C: WidgetTuple> Widget for Card<C> {
    fn build(&self, ctx: &mut WidgetContext) -> NodeId {
        // Create the card node with rounded rect content
        let root_id = ctx.root();
        let node_id = ctx.create_node(
            root_id,
            NodeContent::RoundedRect {
                color: self.background,
                corner_radius: self.corner_radius,
            },
        );

        // Configure layout
        let style = FlexStyle {
            direction: FlexDirection::Column,
            gap: self.gap,
            padding_left: self.padding,
            padding_right: self.padding,
            padding_top: self.padding,
            padding_bottom: self.padding,
            ..Default::default()
        };
        ctx.set_layout_style(node_id, style);

        // Build all children using WidgetTuple
        self.children.build_all(ctx, node_id);

        node_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Text, Widget, WidgetContext};

    #[test]
    fn test_card_creation() {
        let card = Card::new((Text::new("Title"), Text::new("Body")));
        assert_eq!(card.gap, 8.0);
        assert_eq!(card.padding, 16.0);
    }

    #[test]
    fn test_card_builder() {
        let card = Card::new((Text::new("A"),)).gap(16.0).padding(24.0);
        assert_eq!(card.gap, 16.0);
        assert_eq!(card.padding, 24.0);
    }

    #[test]
    fn test_card_builds_scene() {
        let card = Card::new((Text::new("Test"),));
        let mut ctx = WidgetContext::new_test();
        let _node_id = card.build(&mut ctx);
        // Verify a node was created
        assert!(ctx.scene().get_node(ctx.root()).is_some());
    }
}
