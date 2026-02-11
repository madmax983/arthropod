use crate::paint::Paint;
use serde::{Deserialize, Serialize};

/// Stroke alignment relative to path
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum StrokeAlign {
    /// Stroke centered on path
    #[default]
    Center,
    /// Stroke inside path
    Inside,
    /// Stroke outside path
    Outside,
}

impl StrokeAlign {
    /// Convert to float for shader (-1.0 = outside, 0.0 = center, 1.0 = inside)
    pub fn to_float(&self) -> f32 {
        match self {
            Self::Center => 0.0,
            Self::Inside => 1.0,
            Self::Outside => -1.0,
        }
    }
}

/// Stroke cap style
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum StrokeCap {
    /// Flat cap
    #[default]
    Butt,
    /// Rounded cap
    Round,
    /// Square cap (extends beyond endpoint)
    Square,
}

/// Stroke join style
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum StrokeJoin {
    /// Miter join
    #[default]
    Miter,
    /// Rounded join
    Round,
    /// Bevel join
    Bevel,
}

/// Individual stroke weights for each side
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SideWeights {
    /// Top stroke weight
    pub top: f32,
    /// Right stroke weight
    pub right: f32,
    /// Bottom stroke weight
    pub bottom: f32,
    /// Left stroke weight
    pub left: f32,
}

impl SideWeights {
    /// Create uniform side weights
    pub fn uniform(weight: f32) -> Self {
        Self {
            top: weight,
            right: weight,
            bottom: weight,
            left: weight,
        }
    }
}

/// Stroke style
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StrokeStyle {
    /// Stroke paints (applied bottom-to-top)
    pub paints: Vec<Paint>,
    /// Stroke weight (width)
    pub weight: f32,
    /// Stroke alignment
    pub align: StrokeAlign,
    /// Stroke cap style
    pub cap: StrokeCap,
    /// Stroke join style
    pub join: StrokeJoin,
    /// Miter limit (for miter joins)
    pub miter_limit: f32,
    /// Dash pattern (empty = solid stroke)
    pub dash_pattern: Vec<f32>,
    /// Dash offset
    pub dash_offset: f32,
    /// Individual side weights (overrides weight if present)
    pub side_weights: Option<SideWeights>,
}

impl Default for StrokeStyle {
    fn default() -> Self {
        Self {
            paints: Vec::new(),
            weight: 0.0,
            align: StrokeAlign::default(),
            cap: StrokeCap::default(),
            join: StrokeJoin::default(),
            miter_limit: 4.0,
            dash_pattern: Vec::new(),
            dash_offset: 0.0,
            side_weights: None,
        }
    }
}

impl StrokeStyle {
    /// Create a solid stroke with specified color, weight, and alignment
    pub fn solid(paint: Paint, weight: f32, align: StrokeAlign) -> Self {
        Self {
            paints: vec![paint],
            weight,
            align,
            ..Default::default()
        }
    }

    /// Return the top-most stroke paint, if any.
    pub fn top_paint(&self) -> Option<&Paint> {
        self.paints.last()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::Vec4;

    #[test]
    fn test_default_stroke() {
        let stroke = StrokeStyle::default();
        assert_eq!(stroke.weight, 0.0);
        assert_eq!(stroke.align, StrokeAlign::Center);
        assert_eq!(stroke.cap, StrokeCap::Butt);
        assert_eq!(stroke.join, StrokeJoin::Miter);
    }

    #[test]
    fn test_solid_stroke_convenience() {
        let black = Paint::solid(Vec4::new(0.0, 0.0, 0.0, 1.0));
        let stroke = StrokeStyle::solid(black.clone(), 2.0, StrokeAlign::Inside);

        assert_eq!(stroke.paints, vec![black.clone()]);
        assert_eq!(stroke.top_paint(), Some(&black));
        assert_eq!(stroke.weight, 2.0);
        assert_eq!(stroke.align, StrokeAlign::Inside);
    }

    #[test]
    fn test_stroke_align_to_float() {
        assert_eq!(StrokeAlign::Center.to_float(), 0.0);
        assert_eq!(StrokeAlign::Inside.to_float(), 1.0);
        assert_eq!(StrokeAlign::Outside.to_float(), -1.0);
    }

    #[test]
    fn test_side_weights() {
        let weights = SideWeights {
            top: 1.0,
            right: 2.0,
            bottom: 3.0,
            left: 4.0,
        };

        assert_eq!(weights.top, 1.0);
        assert_eq!(weights.right, 2.0);
        assert_eq!(weights.bottom, 3.0);
        assert_eq!(weights.left, 4.0);

        let uniform = SideWeights::uniform(5.0);
        assert_eq!(uniform.top, 5.0);
        assert_eq!(uniform.right, 5.0);
        assert_eq!(uniform.bottom, 5.0);
        assert_eq!(uniform.left, 5.0);
    }

    #[test]
    fn test_serde_roundtrip() {
        let stroke = StrokeStyle::solid(
            Paint::solid(Vec4::new(1.0, 0.0, 0.0, 1.0)),
            2.0,
            StrokeAlign::Inside,
        );

        let json = serde_json::to_string(&stroke).expect("serialize failed");
        let deserialized: StrokeStyle = serde_json::from_str(&json).expect("deserialize failed");

        assert_eq!(stroke, deserialized);
    }
}
