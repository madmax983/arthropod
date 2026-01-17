//! Scene node types.

use crate::NodeId;

/// A node in the scene graph.
pub struct SceneNode {
    /// Visual content of this node.
    pub content: NodeContent,
    /// Local transform relative to parent.
    pub transform: Transform2D,
    /// Bounding box in local coordinates.
    pub bounds: plat_core::Rect,
    /// Child node IDs.
    pub children: Vec<NodeId>,
    /// Whether this node is visible.
    pub visible: bool,
    /// Opacity (0.0 - 1.0).
    pub opacity: f32,
}

impl SceneNode {
    pub fn new_root() -> Self {
        Self {
            content: NodeContent::Empty,
            transform: Transform2D::IDENTITY,
            bounds: plat_core::Rect::new(0.0, 0.0, 0.0, 0.0),
            children: Vec::new(),
            visible: true,
            opacity: 1.0,
        }
    }

    pub fn new(content: NodeContent) -> Self {
        Self {
            content,
            transform: Transform2D::IDENTITY,
            bounds: plat_core::Rect::new(0.0, 0.0, 0.0, 0.0),
            children: Vec::new(),
            visible: true,
            opacity: 1.0,
        }
    }
}

/// The visual content a node can have.
#[derive(Debug, Clone)]
pub enum NodeContent {
    /// Empty container (for grouping).
    Empty,
    /// Solid color rectangle.
    Rect { color: Color },
    /// Rounded rectangle.
    RoundedRect {
        color: Color,
        corner_radius: f32,
    },
}

/// RGBA color.
#[derive(Debug, Clone, Copy)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub const fn rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    pub const RED: Self = Self::rgba(1.0, 0.0, 0.0, 1.0);
    pub const GREEN: Self = Self::rgba(0.0, 1.0, 0.0, 1.0);
    pub const BLUE: Self = Self::rgba(0.0, 0.0, 1.0, 1.0);
    pub const WHITE: Self = Self::rgba(1.0, 1.0, 1.0, 1.0);
    pub const BLACK: Self = Self::rgba(0.0, 0.0, 0.0, 1.0);
}

/// 2D affine transform.
#[derive(Debug, Clone, Copy)]
pub struct Transform2D {
    pub matrix: [[f32; 3]; 2],
}

impl Transform2D {
    pub const IDENTITY: Self = Self {
        matrix: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
    };

    pub fn identity() -> Self {
        Self::IDENTITY
    }

    pub fn translate(x: f32, y: f32) -> Self {
        Self {
            matrix: [[1.0, 0.0, x], [0.0, 1.0, y]],
        }
    }

    pub fn scale(sx: f32, sy: f32) -> Self {
        Self {
            matrix: [[sx, 0.0, 0.0], [0.0, sy, 0.0]],
        }
    }
}
