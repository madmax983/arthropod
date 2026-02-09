use glam::{Vec2, Vec4};
use serde::{Deserialize, Serialize};

/// RGBA color (same as glam::Vec4)
pub type Color = Vec4;

/// Unique identifier for image assets
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ImageId(pub u64);

/// Image scaling mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ImageScaleMode {
    /// Fill the area
    Fill,
    /// Fit within the area
    Fit,
    /// Crop to fill
    Crop,
    /// Tile the image
    Tile,
}

/// Color stop for gradients
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ColorStop {
    /// Position along gradient (0.0 to 1.0)
    pub position: f32,
    /// Color at this stop
    pub color: Color,
}

impl ColorStop {
    /// Create a new color stop
    pub fn new(position: f32, color: Color) -> Self {
        Self { position, color }
    }
}

/// Linear gradient parameters
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LinearGradient {
    /// Gradient start point
    pub start: Vec2,
    /// Gradient end point
    pub end: Vec2,
    /// Color stops
    pub stops: Vec<ColorStop>,
}

/// Radial gradient parameters
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RadialGradient {
    /// Gradient center point
    pub center: Vec2,
    /// Gradient radius
    pub radius: f32,
    /// Color stops
    pub stops: Vec<ColorStop>,
}

/// Angular gradient parameters (sweep around a center point)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AngularGradient {
    /// Gradient center point
    pub center: Vec2,
    /// Rotation angle in radians
    pub angle: f32,
    /// Color stops
    pub stops: Vec<ColorStop>,
}

/// Diamond gradient parameters (Figma-specific)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DiamondGradient {
    /// Gradient center point
    pub center: Vec2,
    /// Gradient scale
    pub scale: f32,
    /// Color stops
    pub stops: Vec<ColorStop>,
}

/// Image fill parameters
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ImageFill {
    /// Image asset ID
    pub image_id: ImageId,
    /// Scaling mode
    pub scale_mode: ImageScaleMode,
    /// Transform matrix (optional)
    pub transform: Option<[f32; 9]>,
}

/// Paint type (fills and strokes)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Paint {
    /// Solid color fill
    Solid(Color),
    /// Linear gradient
    Linear(LinearGradient),
    /// Radial gradient
    Radial(RadialGradient),
    /// Angular gradient
    Angular(AngularGradient),
    /// Diamond gradient (Figma-specific)
    Diamond(DiamondGradient),
    /// Image fill
    Image(ImageFill),
}

impl Paint {
    /// Create a solid color paint
    pub fn solid(color: Color) -> Self {
        Self::Solid(color)
    }

    /// Interpolate between color stops at a given position
    pub fn interpolate_stops(position: f32, stops: &[ColorStop]) -> Color {
        if stops.is_empty() {
            return Vec4::ZERO;
        }
        if stops.len() == 1 {
            return stops[0].color;
        }

        // Clamp position to [0, 1]
        let t = position.clamp(0.0, 1.0);

        // Find surrounding stops
        let mut before = &stops[0];
        let mut after = &stops[stops.len() - 1];

        for i in 0..stops.len() - 1 {
            if stops[i].position <= t && t <= stops[i + 1].position {
                before = &stops[i];
                after = &stops[i + 1];
                break;
            }
        }

        // Handle same position
        if (after.position - before.position).abs() < 1e-6 {
            return before.color;
        }

        // Linear interpolation
        let local_t = (t - before.position) / (after.position - before.position);
        before.color.lerp(after.color, local_t)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_solid_paint() {
        let red = Vec4::new(1.0, 0.0, 0.0, 1.0);
        let paint = Paint::solid(red);
        match paint {
            Paint::Solid(color) => assert_eq!(color, red),
            _ => panic!("Expected solid paint"),
        }
    }

    #[test]
    fn test_linear_gradient_with_stops() {
        let stops = vec![
            ColorStop::new(0.0, Vec4::new(1.0, 0.0, 0.0, 1.0)),
            ColorStop::new(1.0, Vec4::new(0.0, 0.0, 1.0, 1.0)),
        ];

        let gradient = LinearGradient {
            start: Vec2::ZERO,
            end: Vec2::new(100.0, 0.0),
            stops: stops.clone(),
        };

        assert_eq!(gradient.stops.len(), 2);
        assert_eq!(gradient.stops[0].position, 0.0);
        assert_eq!(gradient.stops[1].position, 1.0);
    }

    #[test]
    fn test_interpolate_stops() {
        let black = Vec4::new(0.0, 0.0, 0.0, 1.0);
        let white = Vec4::new(1.0, 1.0, 1.0, 1.0);

        let stops = vec![ColorStop::new(0.0, black), ColorStop::new(1.0, white)];

        let gray = Paint::interpolate_stops(0.5, &stops);

        // Should be approximately gray (0.5, 0.5, 0.5, 1.0)
        assert!((gray.x - 0.5).abs() < 0.01);
        assert!((gray.y - 0.5).abs() < 0.01);
        assert!((gray.z - 0.5).abs() < 0.01);
        assert!((gray.w - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_all_gradient_types_compile() {
        // Just ensure all types compile
        let _linear = Paint::Linear(LinearGradient {
            start: Vec2::ZERO,
            end: Vec2::ONE,
            stops: vec![],
        });

        let _radial = Paint::Radial(RadialGradient {
            center: Vec2::ZERO,
            radius: 10.0,
            stops: vec![],
        });

        let _angular = Paint::Angular(AngularGradient {
            center: Vec2::ZERO,
            angle: 0.0,
            stops: vec![],
        });

        let _diamond = Paint::Diamond(DiamondGradient {
            center: Vec2::ZERO,
            scale: 1.0,
            stops: vec![],
        });

        let _image = Paint::Image(ImageFill {
            image_id: ImageId(123),
            scale_mode: ImageScaleMode::Fill,
            transform: None,
        });
    }

    #[test]
    fn test_serde_roundtrip_solid() {
        let original = Paint::solid(Vec4::new(1.0, 0.0, 0.0, 1.0));
        let json = serde_json::to_string(&original).expect("serialize failed");
        let deserialized: Paint = serde_json::from_str(&json).expect("deserialize failed");
        assert_eq!(original, deserialized);
    }

    #[test]
    fn test_serde_roundtrip_gradient() {
        let gradient = Paint::Linear(LinearGradient {
            start: Vec2::ZERO,
            end: Vec2::new(100.0, 0.0),
            stops: vec![
                ColorStop::new(0.0, Vec4::new(1.0, 0.0, 0.0, 1.0)),
                ColorStop::new(1.0, Vec4::new(0.0, 0.0, 1.0, 1.0)),
            ],
        });

        let json = serde_json::to_string(&gradient).expect("serialize failed");
        let deserialized: Paint = serde_json::from_str(&json).expect("deserialize failed");
        assert_eq!(gradient, deserialized);
    }
}
