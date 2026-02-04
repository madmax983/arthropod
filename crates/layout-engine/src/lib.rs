//! Layout Engine - Flexbox layout using taffy
//!
//! Provides flexbox layout computation with caching for Arthropod UI.
//! Targets < 1ms for 1000 nodes with incremental updates.

use taffy::prelude::*;

pub mod cache;

/// Re-export common types
pub use taffy::{geometry::Size, style::FlexDirection as TaffyFlexDirection};

/// Layout engine wrapping taffy with caching
pub struct LayoutEngine {
    taffy: TaffyTree<()>,
}

/// Flexbox style properties
///
/// Defines how a widget should be laid out relative to its parent and siblings.
#[derive(Debug, Clone, Default)]
pub struct FlexStyle {
    /// Direction of the main axis (Row or Column)
    pub direction: FlexDirection,
    /// How much this item grows relative to siblings (0.0 = none)
    pub flex_grow: f32,
    /// How much this item shrinks if space is limited (1.0 = standard)
    pub flex_shrink: f32,
    /// Fixed width in points (None = auto/flex)
    pub width: Option<f32>,
    /// Fixed height in points (None = auto/flex)
    pub height: Option<f32>,
    /// Gap between children in points
    pub gap: f32,
    /// Left padding in points
    pub padding_left: f32,
    /// Right padding in points
    pub padding_right: f32,
    /// Top padding in points
    pub padding_top: f32,
    /// Bottom padding in points
    pub padding_bottom: f32,
}

/// Flex direction
///
/// Determines the main axis of the layout.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum FlexDirection {
    /// Children are arranged horizontally (left to right)
    #[default]
    Row,
    /// Children are arranged vertically (top to bottom)
    Column,
}

/// Layout constraints for root nodes
#[derive(Debug, Clone, Default)]
pub struct LayoutConstraints {
    pub max_width: Option<f32>,
    pub max_height: Option<f32>,
    pub min_width: Option<f32>,
    pub min_height: Option<f32>,
}

/// Computed layout result
#[derive(Debug, Clone, Copy)]
pub struct ComputedLayout {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

/// Opaque node handle
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(taffy::NodeId);

impl LayoutEngine {
    /// Create a new layout engine
    pub fn new() -> Self {
        Self {
            taffy: TaffyTree::new(),
        }
    }

    /// Create a new layout node with the given style
    ///
    /// # Panics
    ///
    /// Panics if memory allocation fails (extremely rare OOM condition).
    pub fn create_node(&mut self, style: FlexStyle) -> NodeId {
        let taffy_style = convert_style(style);
        let node = self
            .taffy
            .new_leaf(taffy_style)
            .expect("layout node creation should succeed (OOM?)");
        NodeId(node)
    }

    /// Add a child to a parent node
    ///
    /// # Panics
    ///
    /// Panics if `parent` or `child` NodeId is invalid. NodeIds are only
    /// created by `create_node()` and remain valid for the engine's lifetime,
    /// so this should never panic in normal usage.
    pub fn add_child(&mut self, parent: NodeId, child: NodeId) {
        self.taffy
            .add_child(parent.0, child.0)
            .expect("add_child: both parent and child NodeIds must be valid");
    }

    /// Compute layout for a node tree
    ///
    /// This resolves the flexbox layout algorithm for the entire tree rooted at `root`.
    ///
    /// # Root Constraints
    ///
    /// If the root node has `auto` size (no explicit width/height), this method
    /// automatically applies the `constraints` (e.g., window size) to the root style
    /// before computing layout. This ensures that children with `flex_grow` have
    /// a defined container size to grow into.
    ///
    /// # Panics
    ///
    /// Panics if `root` NodeId is invalid. NodeIds are only created by
    /// `create_node()` and remain valid for the engine's lifetime, so this
    /// should never panic in normal usage.
    pub fn compute_layout(&mut self, root: NodeId, constraints: LayoutConstraints) {
        // Update root node to have constraint sizes
        // This ensures flex_grow works correctly
        let current_style = self
            .taffy
            .style(root.0)
            .expect("compute_layout: root NodeId must be valid")
            .clone();
        let mut updated_style = current_style;

        // Set root size from constraints if not already set
        if updated_style.size.width == Dimension::Auto {
            if let Some(width) = constraints.max_width {
                updated_style.size.width = Dimension::Length(width);
            }
        }

        if updated_style.size.height == Dimension::Auto {
            if let Some(height) = constraints.max_height {
                updated_style.size.height = Dimension::Length(height);
            }
        }

        self.taffy
            .set_style(root.0, updated_style)
            .expect("compute_layout: root NodeId must be valid for set_style");

        let available_space = Size {
            width: constraints
                .max_width
                .map_or(AvailableSpace::MaxContent, AvailableSpace::Definite),
            height: constraints
                .max_height
                .map_or(AvailableSpace::MaxContent, AvailableSpace::Definite),
        };

        self.taffy
            .compute_layout(root.0, available_space)
            .expect("compute_layout: layout computation should succeed for valid tree");
    }

    /// Get computed layout for a node
    pub fn get_layout(&self, node: NodeId) -> Option<ComputedLayout> {
        self.taffy.layout(node.0).ok().map(|layout| ComputedLayout {
            x: layout.location.x,
            y: layout.location.y,
            width: layout.size.width,
            height: layout.size.height,
        })
    }
}

impl Default for LayoutEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Convert FlexStyle to taffy Style
fn convert_style(style: FlexStyle) -> Style {
    Style {
        flex_direction: match style.direction {
            FlexDirection::Row => TaffyFlexDirection::Row,
            FlexDirection::Column => TaffyFlexDirection::Column,
        },
        flex_grow: style.flex_grow,
        flex_shrink: style.flex_shrink,
        size: Size {
            width: style.width.map_or(Dimension::Auto, Dimension::Length),
            height: style.height.map_or(Dimension::Auto, Dimension::Length),
        },
        gap: Size {
            width: LengthPercentage::Length(style.gap),
            height: LengthPercentage::Length(style.gap),
        },
        padding: Rect {
            left: LengthPercentage::Length(style.padding_left),
            right: LengthPercentage::Length(style.padding_right),
            top: LengthPercentage::Length(style.padding_top),
            bottom: LengthPercentage::Length(style.padding_bottom),
        },
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_node() {
        let mut engine = LayoutEngine::new();
        let node = engine.create_node(FlexStyle::default());
        assert!(engine.get_layout(node).is_some());
    }
}
