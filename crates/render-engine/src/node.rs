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

/// RGBA color backed by glam::Vec4 for SIMD performance.
#[derive(Debug, Clone, Copy)]
pub struct Color(pub glam::Vec4);

impl Color {
    #[inline]
    pub const fn rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self(glam::Vec4::from_array([r, g, b, a]))
    }

    #[inline]
    pub fn r(&self) -> f32 {
        self.0.x
    }

    #[inline]
    pub fn g(&self) -> f32 {
        self.0.y
    }

    #[inline]
    pub fn b(&self) -> f32 {
        self.0.z
    }

    #[inline]
    pub fn a(&self) -> f32 {
        self.0.w
    }

    #[inline]
    pub fn as_vec4(&self) -> glam::Vec4 {
        self.0
    }

    #[inline]
    pub fn from_vec4(v: glam::Vec4) -> Self {
        Self(v)
    }

    #[inline]
    pub fn to_array(&self) -> [f32; 4] {
        self.0.to_array()
    }

    pub const RED: Self = Self::rgba(1.0, 0.0, 0.0, 1.0);
    pub const GREEN: Self = Self::rgba(0.0, 1.0, 0.0, 1.0);
    pub const BLUE: Self = Self::rgba(0.0, 0.0, 1.0, 1.0);
    pub const WHITE: Self = Self::rgba(1.0, 1.0, 1.0, 1.0);
    pub const BLACK: Self = Self::rgba(0.0, 0.0, 0.0, 1.0);
}

/// 2D affine transform backed by glam::Affine2 for SIMD performance.
#[derive(Debug, Clone, Copy)]
pub struct Transform2D(pub glam::Affine2);

impl Transform2D {
    pub const IDENTITY: Self = Self(glam::Affine2::IDENTITY);

    #[inline]
    pub fn identity() -> Self {
        Self::IDENTITY
    }

    #[inline]
    pub fn translate(x: f32, y: f32) -> Self {
        Self(glam::Affine2::from_translation(glam::Vec2::new(x, y)))
    }

    #[inline]
    pub fn scale(sx: f32, sy: f32) -> Self {
        Self(glam::Affine2::from_scale(glam::Vec2::new(sx, sy)))
    }

    #[inline]
    pub fn as_affine2(&self) -> glam::Affine2 {
        self.0
    }

    #[inline]
    pub fn from_affine2(affine: glam::Affine2) -> Self {
        Self(affine)
    }

    /// Transform a point.
    #[inline]
    pub fn transform_point(&self, point: glam::Vec2) -> glam::Vec2 {
        self.0.transform_point2(point)
    }

    /// Get the translation component.
    #[inline]
    pub fn translation(&self) -> glam::Vec2 {
        self.0.translation
    }

    /// Compose transforms (self * other).
    #[inline]
    pub fn compose(&self, other: &Transform2D) -> Transform2D {
        Self(self.0 * other.0)
    }
}
