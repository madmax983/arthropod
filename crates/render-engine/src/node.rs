//! Scene node types.
//!
//! Defines the building blocks of the scene graph: `SceneNode`, `NodeContent`,
//! and fundamental types like `Color` and `Transform2D`.

use crate::NodeId;
use serde::{Deserialize, Serialize};

/// A node in the scene graph.
///
/// Contains visual content, transformation, and hierarchy information.
///
/// # Example
///
/// ```
/// use render_engine::{SceneNode, NodeContent, Color, Transform2D};
/// use plat_core::Rect;
///
/// let mut node = SceneNode::new(NodeContent::Rect { color: Color::RED });
/// node.transform = Transform2D::translate(100.0, 50.0);
/// node.bounds = Rect::new(0.0, 0.0, 200.0, 100.0);
/// ```
#[derive(Serialize, Deserialize)]
pub struct SceneNode {
    /// Visual content of this node.
    pub content: NodeContent,
    /// Local transform relative to parent.
    pub transform: Transform2D,
    /// Bounding box in local coordinates.
    pub bounds: plat_core::Rect,
    /// Child node IDs.
    pub children: Vec<NodeId>,
    /// Parent node ID (None for root node).
    ///
    /// This enables O(1) parent lookup instead of O(n) search.
    /// Maintained automatically by Scene::add_node() and Scene::reparent_node().
    pub parent: Option<NodeId>,
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
            parent: None, // Root has no parent
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
            parent: None, // Parent set by Scene::add_node()
            visible: true,
            opacity: 1.0,
        }
    }
}

/// The visual content a node can have.
///
/// # Example
///
/// ```
/// use render_engine::{NodeContent, Color};
///
/// let rect = NodeContent::Rect { color: Color::BLUE };
/// let text = NodeContent::Text {
///     text: "Hello".to_string(),
///     font_size: 16.0,
///     color: Color::WHITE,
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeContent {
    /// Empty container (for grouping).
    Empty,
    /// Solid color rectangle.
    Rect { color: Color },
    /// Rounded rectangle.
    RoundedRect { color: Color, corner_radius: f32 },
    /// Text content that will be shaped during rendering.
    ///
    /// Text is stored as a raw string and shaped on-demand using the rendering backend's
    /// TextEngine. This ensures the FontSystem used for shaping matches the one used for
    /// rasterization, avoiding CacheKey mismatches.
    Text {
        /// The text string to render
        text: String,
        /// Font size in pixels
        font_size: f32,
        /// Text color
        color: Color,
    },
}

/// RGBA color backed by glam::Vec4 for SIMD performance.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color(pub glam::Vec4);

impl Serialize for Color {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.to_array().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Color {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let arr = <[f32; 4]>::deserialize(deserializer)?;
        Ok(Color(glam::Vec4::from_array(arr)))
    }
}

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

impl Serialize for Transform2D {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        // Serialize as 6 floats: [m11, m12, m21, m22, tx, ty]
        let mat = self.0.matrix2;
        let trans = self.0.translation;
        [
            mat.x_axis.x,
            mat.x_axis.y,
            mat.y_axis.x,
            mat.y_axis.y,
            trans.x,
            trans.y,
        ]
        .serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Transform2D {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let arr = <[f32; 6]>::deserialize(deserializer)?;
        let mat = glam::Mat2::from_cols(
            glam::Vec2::new(arr[0], arr[1]),
            glam::Vec2::new(arr[2], arr[3]),
        );
        let trans = glam::Vec2::new(arr[4], arr[5]);
        Ok(Transform2D(glam::Affine2::from_mat2_translation(
            mat, trans,
        )))
    }
}

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
