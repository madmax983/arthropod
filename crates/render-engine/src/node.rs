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
    RoundedRect { color: Color, corner_radius: f32 },
}

/// RGBA color - will be migrated to glam::Vec4 internally.
/// For now, keeping public fields for backwards compatibility.
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

    /// Convert to glam::Vec4 for SIMD operations.
    #[inline]
    pub fn as_vec4(&self) -> glam::Vec4 {
        glam::Vec4::new(self.r, self.g, self.b, self.a)
    }

    /// Create from glam::Vec4.
    #[inline]
    pub fn from_vec4(v: glam::Vec4) -> Self {
        Self {
            r: v.x,
            g: v.y,
            b: v.z,
            a: v.w,
        }
    }

    /// Convert to array [r, g, b, a].
    #[inline]
    pub fn to_array(&self) -> [f32; 4] {
        [self.r, self.g, self.b, self.a]
    }

    pub const RED: Self = Self::rgba(1.0, 0.0, 0.0, 1.0);
    pub const GREEN: Self = Self::rgba(0.0, 1.0, 0.0, 1.0);
    pub const BLUE: Self = Self::rgba(0.0, 0.0, 1.0, 1.0);
    pub const WHITE: Self = Self::rgba(1.0, 1.0, 1.0, 1.0);
    pub const BLACK: Self = Self::rgba(0.0, 0.0, 0.0, 1.0);
}

/// 2D affine transform - will be migrated to glam::Affine2 internally.
/// For now, keeping current representation for backwards compatibility.
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

    /// Convert to glam::Affine2 for SIMD operations.
    #[inline]
    pub fn as_affine2(&self) -> glam::Affine2 {
        // Our matrix is [[m11, m12, m13], [m21, m22, m23]]
        // glam::Affine2::from_cols expects (x_axis, y_axis, translation)
        glam::Affine2::from_cols(
            glam::Vec2::new(self.matrix[0][0], self.matrix[1][0]), // x_axis (m11, m21)
            glam::Vec2::new(self.matrix[0][1], self.matrix[1][1]), // y_axis (m12, m22)
            glam::Vec2::new(self.matrix[0][2], self.matrix[1][2]), // translation (m13, m23)
        )
    }

    /// Create from glam::Affine2.
    #[inline]
    pub fn from_affine2(affine: glam::Affine2) -> Self {
        let cols = affine.to_cols_array();
        // glam stores as [m11, m21, m12, m22, m13, m23] (column-major)
        Self {
            matrix: [
                [cols[0], cols[2], cols[4]], // Row 0: m11, m12, m13
                [cols[1], cols[3], cols[5]], // Row 1: m21, m22, m23
            ],
        }
    }
}
