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
/// use render_engine::{SceneNode, NodeContent, Color, Transform2D, Rect};
/// use style_engine::VisualStyle;
///
/// let mut node = SceneNode::new(NodeContent::Styled {
///     style: Box::new(VisualStyle::new().solid_fill(Color::RED.as_vec4())),
/// });
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
    /// Creates a new foundational anchor for a scene hierarchy.
    ///
    /// The root node is uniquely positioned as the origin of all layout and rendering
    /// traversals. It carries no visual content (`NodeContent::Empty`) and serves
    /// strictly as the structural container for your application's top-level components.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use render_engine::{Scene, SceneNode};
    ///
    /// let mut scene = Scene::new();
    /// // The Scene already creates a root for us internally,
    /// // but if we were building our own tree:
    /// let custom_root = SceneNode::new_root();
    /// ```
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

    /// Forges a new visual element ready to be injected into the scene graph.
    ///
    /// This is the primary constructor for anything that appears on screen,
    /// from text blocks to solid color backgrounds. The node starts at the origin
    /// with an identity transform until explicitly positioned or layout is applied.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use render_engine::{SceneNode, NodeContent, Color};
    ///
    /// // Create a node that renders as pure red
    /// let node = SceneNode::new(NodeContent::SolidColor { color: Color::RED });
    /// ```
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

    /// Create a new solid color node (optimization).
    ///
    /// This avoids allocating a `Box<VisualStyle>` on the heap, which is faster
    /// and uses less memory than `NodeContent::Styled`.
    pub fn solid(color: Color) -> Self {
        Self::new(NodeContent::SolidColor { color })
    }
}

/// The visual content a node can have.
///
/// # Example
///
/// ```
/// use render_engine::{NodeContent, Color};
/// use style_engine::VisualStyle;
///
/// // Solid fill rectangle
/// let rect = NodeContent::Styled {
///     style: Box::new(VisualStyle::new()
///         .solid_fill(Color::BLUE.as_vec4())),
/// };
///
/// // Rounded rectangle
/// let rounded = NodeContent::Styled {
///     style: Box::new(VisualStyle::new()
///         .solid_fill(Color::RED.as_vec4())
///         .corner_radius(12.0)),
/// };
///
/// // Text with style
/// let text = NodeContent::Styled {
///     style: Box::new(VisualStyle::new()
///         .solid_fill(Color::WHITE.as_vec4())
///         .text(style_engine::TextContent::new("Hello", 16.0))),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeContent {
    /// Empty container (for grouping).
    Empty,
    /// Styled primitive (rectangles, text, gradients, strokes, effects).
    ///
    /// This unified variant replaces the old Rect, RoundedRect, and Text variants.
    /// It uses VisualStyle from style-engine which maps 1:1 to Figma's visual properties.
    /// Boxed to reduce enum size (VisualStyle is 304 bytes).
    Styled {
        /// The visual style (fills, stroke, effects, text, etc.)
        style: Box<style_engine::VisualStyle>,
    },
    /// Optimization for solid color rectangles (e.g. backgrounds).
    ///
    /// # Performance
    ///
    /// This variant avoids heap allocation of `Box<VisualStyle>` (~300 bytes),
    /// making it significantly faster to create and lighter in memory.
    /// Prefer this over `NodeContent::Styled` whenever you only need a
    /// simple solid color fill.
    ///
    /// This variant is equivalent to a `Styled` node with a single solid fill,
    /// no stroke, no effects, and zero corner radii.
    SolidColor {
        /// The fill color.
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

impl From<glam::Vec4> for Color {
    fn from(v: glam::Vec4) -> Self {
        Self(v)
    }
}

impl Color {
    /// Mixes a new custom color from raw component values.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use render_engine::Color;
    ///
    /// let hot_pink = Color::rgba(1.0, 0.41, 0.71, 1.0);
    /// assert_eq!(hot_pink.r(), 1.0);
    /// ```
    #[inline]
    pub const fn rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self(glam::Vec4::from_array([r, g, b, a]))
    }

    #[allow(missing_docs)]
    #[inline]
    pub fn r(&self) -> f32 {
        self.0.x
    }

    #[allow(missing_docs)]
    #[inline]
    pub fn g(&self) -> f32 {
        self.0.y
    }

    #[allow(missing_docs)]
    #[inline]
    pub fn b(&self) -> f32 {
        self.0.z
    }

    #[allow(missing_docs)]
    #[inline]
    pub fn a(&self) -> f32 {
        self.0.w
    }

    /// Exposes the underlying hardware-accelerated SIMD vector.
    ///
    /// This allows for advanced bulk mathematical operations natively via `glam`.
    #[inline]
    pub fn as_vec4(&self) -> glam::Vec4 {
        self.0
    }

    /// Converts a hardware-accelerated SIMD vector back into a framework color.
    #[inline]
    pub fn from_vec4(v: glam::Vec4) -> Self {
        Self(v)
    }

    /// Extracts the raw contiguous floats, useful for directly uploading to a shader buffer.
    #[inline]
    pub fn to_array(&self) -> [f32; 4] {
        self.0.to_array()
    }

    #[allow(missing_docs)]
    pub const RED: Self = Self::rgba(1.0, 0.0, 0.0, 1.0);
    #[allow(missing_docs)]
    pub const GREEN: Self = Self::rgba(0.0, 1.0, 0.0, 1.0);
    #[allow(missing_docs)]
    pub const BLUE: Self = Self::rgba(0.0, 0.0, 1.0, 1.0);
    #[allow(missing_docs)]
    pub const WHITE: Self = Self::rgba(1.0, 1.0, 1.0, 1.0);
    #[allow(missing_docs)]
    pub const BLACK: Self = Self::rgba(0.0, 0.0, 0.0, 1.0);
}

/// 2D affine transform backed by glam::Affine2 for SIMD performance.
#[derive(Debug, Clone, Copy, PartialEq)]
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
    /// A neutral matrix that leaves coordinates completely unchanged when applied.
    pub const IDENTITY: Self = Self(glam::Affine2::IDENTITY);

    /// Constructs a neutral matrix that does not affect positioning.
    #[inline]
    pub fn identity() -> Self {
        Self::IDENTITY
    }

    /// Creates a spatial shift operator that moves the entire coordinate system.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use render_engine::Transform2D;
    ///
    /// let shift_down_right = Transform2D::translate(10.0, 20.0);
    /// ```
    #[inline]
    pub fn translate(x: f32, y: f32) -> Self {
        Self(glam::Affine2::from_translation(glam::Vec2::new(x, y)))
    }

    /// Creates a stretching operator to expand or compress coordinates on the X or Y axes.
    #[inline]
    pub fn scale(sx: f32, sy: f32) -> Self {
        Self(glam::Affine2::from_scale(glam::Vec2::new(sx, sy)))
    }

    /// Creates a pivoting operator to spin coordinates around the mathematical origin (0, 0).
    #[inline]
    pub fn rotate_radians(angle: f32) -> Self {
        Self(glam::Affine2::from_angle(angle))
    }

    /// Peels back the abstraction to expose the raw SIMD 2D affine matrix for advanced math.
    #[inline]
    pub fn as_affine2(&self) -> glam::Affine2 {
        self.0
    }

    /// Encapsulates a raw SIMD 2D affine matrix into our safer abstraction.
    #[inline]
    pub fn from_affine2(affine: glam::Affine2) -> Self {
        Self(affine)
    }

    /// Computes the new absolute coordinates of a relative position under this matrix.
    #[inline]
    pub fn transform_point(&self, point: glam::Vec2) -> glam::Vec2 {
        self.0.transform_point2(point)
    }

    /// Get the translation component.
    #[inline]
    pub fn translation(&self) -> glam::Vec2 {
        self.0.translation
    }

    /// Extract the approximate in-plane rotation in radians.
    ///
    /// This is robust for the common UI transform case (rotation +/- scale).
    #[inline]
    pub fn rotation_radians(&self) -> f32 {
        self.0.matrix2.x_axis.y.atan2(self.0.matrix2.x_axis.x)
    }

    /// Compose transforms (self * other).
    #[inline]
    pub fn compose(&self, other: &Transform2D) -> Transform2D {
        Self(self.0 * other.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_content_size() {
        // NodeContent should be relatively small.
        // Empty: 1 byte + alignment
        // Styled: 8 bytes (Box) + tag
        // SolidColor: 16 bytes (Color) + tag
        // Enum size is driven by the largest variant + alignment.
        // Color is 16 bytes (Vec4), likely 16-byte aligned on some archs, or 4-byte aligned.
        // sizeof(NodeContent) should be around 24-32 bytes.
        // If it's > 64 bytes, something is wrong.
        assert!(std::mem::size_of::<NodeContent>() <= 32);
    }

    #[test]
    fn test_transform_rotation_radians_roundtrip() {
        let angle = 15.0_f32.to_radians();
        let transform = Transform2D::rotate_radians(angle);
        let extracted = transform.rotation_radians();
        assert!((extracted - angle).abs() < 1e-5);
    }
}
