//! Layout Engine - Flexbox layout using taffy
//!
//! Provides flexbox layout computation with caching for Arthropod UI.
//! Targets < 1ms for 1000 nodes with incremental updates.

use taffy::{
    prelude::{AvailableSpace, Dimension, LengthPercentage, Rect, Style, TaffyTree},
    style::{
        AlignItems as TaffyAlignItems, AlignSelf as TaffyAlignSelf,
        FlexDirection as TaffyFlexDirection, FlexWrap as TaffyFlexWrap,
        JustifyContent as TaffyJustifyContent,
    },
};

pub mod cache;

/// Re-export common types
pub use taffy::geometry::Size;

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
    /// Main-axis distribution of children
    pub justify_content: FlexJustifyContent,
    /// Cross-axis alignment for children
    pub align_items: FlexAlign,
    /// Per-item cross-axis override (None = auto/inherit parent)
    pub align_self: Option<ItemAlignSelf>,
    /// Whether children wrap to additional lines
    pub wrap: FlexWrap,
    /// Preferred initial size along main axis
    pub flex_basis: Option<f32>,
    /// Max width in points
    pub max_width: Option<f32>,
    /// Max height in points
    pub max_height: Option<f32>,
    /// Min width in points
    pub min_width: Option<f32>,
    /// Min height in points
    pub min_height: Option<f32>,
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

/// Main-axis child distribution.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum FlexJustifyContent {
    #[default]
    Start,
    Center,
    End,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
}

/// Cross-axis alignment.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum FlexAlign {
    Start,
    Center,
    End,
    #[default]
    Stretch,
}

/// Child wrapping behavior.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum FlexWrap {
    #[default]
    NoWrap,
    Wrap,
}

/// Per-item cross-axis override.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ItemAlignSelf {
    #[default]
    Auto,
    Start,
    Center,
    End,
    Stretch,
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
        // Taffy's `add_child` returns a Result, but it also might panic internally or return an error
        // if the nodes are invalid. However, Taffy 0.3+ often panics on invalid SlotMap keys
        // directly if we don't check existence first, or returns an error we unwrap.
        // Let's rely on Taffy's return value and expect it with our custom message.
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
        justify_content: Some(match style.justify_content {
            FlexJustifyContent::Start => TaffyJustifyContent::Start,
            FlexJustifyContent::Center => TaffyJustifyContent::Center,
            FlexJustifyContent::End => TaffyJustifyContent::End,
            FlexJustifyContent::SpaceBetween => TaffyJustifyContent::SpaceBetween,
            FlexJustifyContent::SpaceAround => TaffyJustifyContent::SpaceAround,
            FlexJustifyContent::SpaceEvenly => TaffyJustifyContent::SpaceEvenly,
        }),
        align_items: Some(match style.align_items {
            FlexAlign::Start => TaffyAlignItems::Start,
            FlexAlign::Center => TaffyAlignItems::Center,
            FlexAlign::End => TaffyAlignItems::End,
            FlexAlign::Stretch => TaffyAlignItems::Stretch,
        }),
        align_self: style.align_self.and_then(|align| match align {
            ItemAlignSelf::Auto => None,
            ItemAlignSelf::Start => Some(TaffyAlignSelf::Start),
            ItemAlignSelf::Center => Some(TaffyAlignSelf::Center),
            ItemAlignSelf::End => Some(TaffyAlignSelf::End),
            ItemAlignSelf::Stretch => Some(TaffyAlignSelf::Stretch),
        }),
        flex_wrap: match style.wrap {
            FlexWrap::NoWrap => TaffyFlexWrap::NoWrap,
            FlexWrap::Wrap => TaffyFlexWrap::Wrap,
        },
        flex_grow: style.flex_grow,
        flex_shrink: style.flex_shrink,
        flex_basis: style.flex_basis.map_or(Dimension::Auto, Dimension::Length),
        size: Size {
            width: style.width.map_or(Dimension::Auto, Dimension::Length),
            height: style.height.map_or(Dimension::Auto, Dimension::Length),
        },
        min_size: Size {
            width: style.min_width.map_or(Dimension::Auto, Dimension::Length),
            height: style.min_height.map_or(Dimension::Auto, Dimension::Length),
        },
        max_size: Size {
            width: style.max_width.map_or(Dimension::Auto, Dimension::Length),
            height: style.max_height.map_or(Dimension::Auto, Dimension::Length),
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

    #[test]
    fn test_root_auto_size_promotion() {
        let mut engine = LayoutEngine::new();
        // Create a root node with Auto dimensions
        let root = engine.create_node(FlexStyle {
            width: None,
            height: None,
            ..Default::default()
        });

        // Compute layout with explicit constraints
        let constraints = LayoutConstraints {
            max_width: Some(800.0),
            max_height: Some(600.0),
            ..Default::default()
        };
        engine.compute_layout(root, constraints);

        let layout = engine.get_layout(root).unwrap();
        // Root should take the constraint size because it was Auto
        assert_eq!(layout.width, 800.0);
        assert_eq!(layout.height, 600.0);
    }

    #[test]
    fn test_root_fixed_size_preservation() {
        let mut engine = LayoutEngine::new();
        // Create a root node with Fixed dimensions
        let root = engine.create_node(FlexStyle {
            width: Some(100.0),
            height: Some(100.0),
            ..Default::default()
        });

        // Compute layout with DIFFERENT constraints
        let constraints = LayoutConstraints {
            max_width: Some(800.0),
            max_height: Some(600.0),
            ..Default::default()
        };
        engine.compute_layout(root, constraints);

        let layout = engine.get_layout(root).unwrap();
        // Root should KEEP its fixed size, ignoring constraints
        assert_eq!(layout.width, 100.0);
        assert_eq!(layout.height, 100.0);
    }

    #[test]
    fn test_flex_style_properties() {
        let mut engine = LayoutEngine::new();

        // Parent row with gap and padding
        let parent = engine.create_node(FlexStyle {
            direction: FlexDirection::Row,
            width: Some(300.0), // 300px width
            height: Some(100.0),
            padding_left: 10.0,
            padding_right: 10.0,
            gap: 10.0,
            ..Default::default()
        });

        // Two children sharing remaining space (300 - 20 padding - 10 gap = 270 available)
        // Child 1: flex-grow 1
        let child1 = engine.create_node(FlexStyle {
            flex_grow: 1.0,
            height: Some(50.0),
            ..Default::default()
        });

        // Child 2: flex-grow 2 (should be twice as wide as child 1)
        let child2 = engine.create_node(FlexStyle {
            flex_grow: 2.0,
            height: Some(50.0),
            ..Default::default()
        });

        engine.add_child(parent, child1);
        engine.add_child(parent, child2);

        engine.compute_layout(parent, LayoutConstraints::default());

        let l1 = engine.get_layout(child1).unwrap();
        let l2 = engine.get_layout(child2).unwrap();

        // Check widths: 270 total space. 1/3 to child1 (90), 2/3 to child2 (180).
        assert_eq!(l1.width, 90.0);
        assert_eq!(l2.width, 180.0);

        // Check positions
        // Child 1 x = padding_left = 10.0
        assert_eq!(l1.x, 10.0);
        // Child 2 x = padding_left + child1 width + gap = 10 + 90 + 10 = 110.0
        assert_eq!(l2.x, 110.0);
    }

    #[test]
    #[should_panic(expected = "invalid SlotMap key used")]
    fn test_add_child_invalid_parent() {
        let mut engine = LayoutEngine::new();
        let child = engine.create_node(FlexStyle::default());
        // Fabricate an invalid NodeId (assuming 999999 is invalid for a fresh tree)
        let invalid_parent = NodeId(taffy::NodeId::from(999999usize));
        engine.add_child(invalid_parent, child);
    }

    #[test]
    #[should_panic(expected = "invalid SlotMap key used")]
    fn test_compute_layout_invalid_root() {
        let mut engine = LayoutEngine::new();
        let invalid_root = NodeId(taffy::NodeId::from(999999usize));
        engine.compute_layout(invalid_root, LayoutConstraints::default());
    }

    #[test]
    fn test_deep_tree() {
        // Ensure recursion doesn't blow up for a reasonably deep UI tree
        let mut engine = LayoutEngine::new();
        let mut parent = engine.create_node(FlexStyle {
            width: Some(1000.0),
            height: Some(1000.0),
            ..Default::default()
        });
        let root = parent;

        for _ in 0..100 {
            let child = engine.create_node(FlexStyle {
                width: Some(10.0),
                height: Some(10.0),
                ..Default::default()
            });
            engine.add_child(parent, child);
            parent = child; // Nest deeply
        }

        engine.compute_layout(root, LayoutConstraints::default());
        // If we get here without stack overflow, success.
        assert!(engine.get_layout(parent).is_some());
    }
}
