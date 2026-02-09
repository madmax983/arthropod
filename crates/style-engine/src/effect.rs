use crate::paint::Color;
use glam::Vec2;
use serde::{Deserialize, Serialize};

/// Drop shadow effect
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct DropShadow {
    /// Shadow offset
    pub offset: Vec2,
    /// Blur radius
    pub blur: f32,
    /// Shadow color
    pub color: Color,
    /// Whether shadow is visible
    pub visible: bool,
}

impl DropShadow {
    /// Create a drop shadow with default visibility
    pub fn new(offset: Vec2, blur: f32, color: Color) -> Self {
        Self {
            offset,
            blur,
            color,
            visible: true,
        }
    }
}

/// Inner shadow effect
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct InnerShadow {
    /// Shadow offset
    pub offset: Vec2,
    /// Blur radius
    pub blur: f32,
    /// Shadow color
    pub color: Color,
    /// Whether shadow is visible
    pub visible: bool,
}

impl InnerShadow {
    /// Create an inner shadow with default visibility
    pub fn new(offset: Vec2, blur: f32, color: Color) -> Self {
        Self {
            offset,
            blur,
            color,
            visible: true,
        }
    }
}

/// Layer blur effect (entire layer)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct LayerBlur {
    /// Blur radius
    pub radius: f32,
    /// Whether blur is visible
    pub visible: bool,
}

impl LayerBlur {
    /// Create a layer blur with default visibility
    pub fn new(radius: f32) -> Self {
        Self {
            radius,
            visible: true,
        }
    }
}

/// Background blur effect (blurs content behind layer)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct BackgroundBlur {
    /// Blur radius
    pub radius: f32,
    /// Whether blur is visible
    pub visible: bool,
}

impl BackgroundBlur {
    /// Create a background blur with default visibility
    pub fn new(radius: f32) -> Self {
        Self {
            radius,
            visible: true,
        }
    }
}

/// Visual effect (shadow, blur, etc.)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Effect {
    /// Drop shadow effect
    DropShadow(DropShadow),
    /// Inner shadow effect
    InnerShadow(InnerShadow),
    /// Layer blur effect
    LayerBlur(LayerBlur),
    /// Background blur effect
    BackgroundBlur(BackgroundBlur),
}

impl Effect {
    /// Create a drop shadow effect
    pub fn drop_shadow(offset: Vec2, blur: f32, color: Color) -> Self {
        Self::DropShadow(DropShadow::new(offset, blur, color))
    }

    /// Create an inner shadow effect
    pub fn inner_shadow(offset: Vec2, blur: f32, color: Color) -> Self {
        Self::InnerShadow(InnerShadow::new(offset, blur, color))
    }

    /// Create a layer blur effect
    pub fn layer_blur(radius: f32) -> Self {
        Self::LayerBlur(LayerBlur::new(radius))
    }

    /// Create a background blur effect
    pub fn background_blur(radius: f32) -> Self {
        Self::BackgroundBlur(BackgroundBlur::new(radius))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::Vec4;

    #[test]
    fn test_drop_shadow_convenience() {
        let offset = Vec2::new(2.0, 2.0);
        let blur = 4.0;
        let color = Vec4::new(0.0, 0.0, 0.0, 0.5);

        let effect = Effect::drop_shadow(offset, blur, color);

        match effect {
            Effect::DropShadow(shadow) => {
                assert_eq!(shadow.offset, offset);
                assert_eq!(shadow.blur, blur);
                assert_eq!(shadow.color, color);
                assert!(shadow.visible, "default visibility should be true");
            }
            _ => panic!("Expected drop shadow"),
        }
    }

    #[test]
    fn test_inner_shadow_construction() {
        let offset = Vec2::new(1.0, 1.0);
        let blur = 2.0;
        let color = Vec4::new(0.0, 0.0, 0.0, 0.3);

        let effect = Effect::inner_shadow(offset, blur, color);

        match effect {
            Effect::InnerShadow(shadow) => {
                assert_eq!(shadow.offset, offset);
                assert_eq!(shadow.blur, blur);
                assert_eq!(shadow.color, color);
                assert!(shadow.visible);
            }
            _ => panic!("Expected inner shadow"),
        }
    }

    #[test]
    fn test_layer_blur_construction() {
        let effect = Effect::layer_blur(5.0);

        match effect {
            Effect::LayerBlur(blur) => {
                assert_eq!(blur.radius, 5.0);
                assert!(blur.visible);
            }
            _ => panic!("Expected layer blur"),
        }
    }

    #[test]
    fn test_background_blur_construction() {
        let effect = Effect::background_blur(10.0);

        match effect {
            Effect::BackgroundBlur(blur) => {
                assert_eq!(blur.radius, 10.0);
                assert!(blur.visible);
            }
            _ => panic!("Expected background blur"),
        }
    }

    #[test]
    fn test_serde_roundtrip() {
        let effect = Effect::drop_shadow(Vec2::new(2.0, 2.0), 4.0, Vec4::new(0.0, 0.0, 0.0, 0.5));

        let json = serde_json::to_string(&effect).expect("serialize failed");
        let deserialized: Effect = serde_json::from_str(&json).expect("deserialize failed");

        assert_eq!(effect, deserialized);
    }
}
