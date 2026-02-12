use glam::{Vec2, Vec3, Vec4};
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

/// Gradient color interpolation space.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum GradientInterpolationMode {
    /// Interpolate directly in gamma-encoded sRGB (legacy behavior, can look muddy).
    Srgb,
    /// Interpolate in linear-light RGB, then convert back to sRGB.
    LinearRgb,
    /// Interpolate in Oklab for perceptual smoothness.
    #[default]
    Oklab,
}

#[derive(Debug, Clone, Copy)]
struct Oklab {
    l: f32,
    a: f32,
    b: f32,
}

fn srgb_to_linear(channel: f32) -> f32 {
    if channel <= 0.04045 {
        channel / 12.92
    } else {
        ((channel + 0.055) / 1.055).powf(2.4)
    }
}

fn linear_to_srgb(channel: f32) -> f32 {
    if channel <= 0.003_130_8 {
        12.92 * channel
    } else {
        1.055 * channel.powf(1.0 / 2.4) - 0.055
    }
}

fn linear_rgb_to_oklab(rgb: Vec3) -> Oklab {
    let l = 0.412_221_46 * rgb.x + 0.536_332_55 * rgb.y + 0.051_445_995 * rgb.z;
    let m = 0.211_903_5 * rgb.x + 0.680_699_5 * rgb.y + 0.107_396_96 * rgb.z;
    let s = 0.088_302_46 * rgb.x + 0.281_718_85 * rgb.y + 0.629_978_7 * rgb.z;

    let l_ = l.cbrt();
    let m_ = m.cbrt();
    let s_ = s.cbrt();

    Oklab {
        l: 0.210_454_26 * l_ + 0.793_617_8 * m_ - 0.004_072_047 * s_,
        a: 1.977_998_5 * l_ - 2.428_592_2 * m_ + 0.450_593_7 * s_,
        b: 0.025_904_037 * l_ + 0.782_771_77 * m_ - 0.808_675_77 * s_,
    }
}

fn oklab_to_linear_rgb(lab: Oklab) -> Vec3 {
    let l_ = lab.l + 0.396_337_78 * lab.a + 0.215_803_76 * lab.b;
    let m_ = lab.l - 0.105_561_346 * lab.a - 0.063_854_17 * lab.b;
    let s_ = lab.l - 0.089_484_18 * lab.a - 1.291_485_5 * lab.b;

    let l = l_ * l_ * l_;
    let m = m_ * m_ * m_;
    let s = s_ * s_ * s_;

    Vec3::new(
        4.076_741_7 * l - 3.307_711_6 * m + 0.230_969_94 * s,
        -1.268_438 * l + 2.609_757_4 * m - 0.341_319_38 * s,
        -0.004_196_086_3 * l - 0.703_418_6 * m + 1.707_614_7 * s,
    )
}

fn lerp_color_oklab(start: Color, end: Color, t: f32) -> Color {
    let start_linear = Vec3::new(
        srgb_to_linear(start.x),
        srgb_to_linear(start.y),
        srgb_to_linear(start.z),
    );
    let end_linear = Vec3::new(
        srgb_to_linear(end.x),
        srgb_to_linear(end.y),
        srgb_to_linear(end.z),
    );

    let start_lab = linear_rgb_to_oklab(start_linear);
    let end_lab = linear_rgb_to_oklab(end_linear);

    let lab = Oklab {
        l: start_lab.l + (end_lab.l - start_lab.l) * t,
        a: start_lab.a + (end_lab.a - start_lab.a) * t,
        b: start_lab.b + (end_lab.b - start_lab.b) * t,
    };

    let rgb_linear = oklab_to_linear_rgb(lab);
    let rgb = Vec3::new(
        linear_to_srgb(rgb_linear.x.clamp(0.0, 1.0)),
        linear_to_srgb(rgb_linear.y.clamp(0.0, 1.0)),
        linear_to_srgb(rgb_linear.z.clamp(0.0, 1.0)),
    );

    let alpha = start.w + (end.w - start.w) * t;
    Vec4::new(rgb.x, rgb.y, rgb.z, alpha.clamp(0.0, 1.0))
}

fn lerp_color_srgb(start: Color, end: Color, t: f32) -> Color {
    start.lerp(end, t)
}

fn lerp_color_linear_rgb(start: Color, end: Color, t: f32) -> Color {
    let start_linear = Vec3::new(
        srgb_to_linear(start.x),
        srgb_to_linear(start.y),
        srgb_to_linear(start.z),
    );
    let end_linear = Vec3::new(
        srgb_to_linear(end.x),
        srgb_to_linear(end.y),
        srgb_to_linear(end.z),
    );

    let mixed_linear = start_linear.lerp(end_linear, t);
    let rgb = Vec3::new(
        linear_to_srgb(mixed_linear.x.clamp(0.0, 1.0)),
        linear_to_srgb(mixed_linear.y.clamp(0.0, 1.0)),
        linear_to_srgb(mixed_linear.z.clamp(0.0, 1.0)),
    );
    let alpha = start.w + (end.w - start.w) * t;
    Vec4::new(rgb.x, rgb.y, rgb.z, alpha.clamp(0.0, 1.0))
}

impl Paint {
    /// Create a solid color paint
    pub fn solid(color: Color) -> Self {
        Self::Solid(color)
    }

    /// Interpolate between color stops at a given position
    pub fn interpolate_stops(position: f32, stops: &[ColorStop]) -> Color {
        Self::interpolate_stops_with_mode(position, stops, GradientInterpolationMode::default())
    }

    /// Interpolate between color stops at a given position using an explicit color space.
    pub fn interpolate_stops_with_mode(
        position: f32,
        stops: &[ColorStop],
        mode: GradientInterpolationMode,
    ) -> Color {
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

        let local_t = (t - before.position) / (after.position - before.position);
        match mode {
            GradientInterpolationMode::Srgb => lerp_color_srgb(before.color, after.color, local_t),
            GradientInterpolationMode::LinearRgb => {
                lerp_color_linear_rgb(before.color, after.color, local_t)
            }
            GradientInterpolationMode::Oklab => {
                lerp_color_oklab(before.color, after.color, local_t)
            }
        }
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

        // In Oklab interpolation, black-white midpoint is perceptual middle gray.
        assert!((gray.x - 0.39).abs() < 0.03);
        assert!((gray.y - 0.39).abs() < 0.03);
        assert!((gray.z - 0.39).abs() < 0.03);
        assert!((gray.w - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_interpolate_stops_with_explicit_modes() {
        let black = Vec4::new(0.0, 0.0, 0.0, 1.0);
        let white = Vec4::new(1.0, 1.0, 1.0, 1.0);
        let stops = vec![ColorStop::new(0.0, black), ColorStop::new(1.0, white)];

        let srgb = Paint::interpolate_stops_with_mode(0.5, &stops, GradientInterpolationMode::Srgb);
        let linear =
            Paint::interpolate_stops_with_mode(0.5, &stops, GradientInterpolationMode::LinearRgb);
        let oklab =
            Paint::interpolate_stops_with_mode(0.5, &stops, GradientInterpolationMode::Oklab);

        // sRGB channel interpolation midpoint
        assert!((srgb.x - 0.5).abs() < 0.01);
        // linear-light midpoint is brighter than sRGB midpoint
        assert!(linear.x > srgb.x);
        // Oklab midpoint is currently the crate default behavior
        assert!((oklab.x - Paint::interpolate_stops(0.5, &stops).x).abs() < 0.001);
    }

    #[test]
    fn test_interpolate_stops_uses_perceptual_color_blending() {
        let red = Vec4::new(1.0, 0.0, 0.0, 1.0);
        let green = Vec4::new(0.0, 1.0, 0.0, 1.0);
        let stops = vec![ColorStop::new(0.0, red), ColorStop::new(1.0, green)];

        let midpoint = Paint::interpolate_stops(0.5, &stops);

        // Perceptual interpolation should produce a bright warm midpoint rather than
        // channel-wise average olive (0.5, 0.5, 0.0).
        assert!(midpoint.x > 0.7, "red channel should stay high");
        assert!(midpoint.y > 0.55, "green channel should stay high");
        assert!(midpoint.z < 0.2, "blue channel should remain low");
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
