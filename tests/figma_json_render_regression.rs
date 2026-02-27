#![cfg(all(not(target_arch = "wasm32"), target_os = "windows"))]

use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

use arthropod_test::visual_test::{compare_images_with_tolerance, load_image, save_image};
use plat_core::{EventLoop, Rect, Size, WindowConfig};
use render_engine::{
    BlendMode, Color, ColorStop, CornerRadii, Effect, NodeContent, Paint, Scene, SceneNode, Vec2,
    Vec4, VisualStyle,
    backend::{RenderBackend, WgpuBackend},
};
use serde::Deserialize;
use style_engine::{
    AngularGradient, BackgroundBlur, DiamondGradient, DropShadow, ImageFill, ImageId,
    ImageScaleMode, InnerShadow, LayerBlur, LinearGradient, MaskType, RadialGradient, SideWeights,
    StrokeAlign, StrokeCap, StrokeJoin, StrokeStyle, VectorPath, WindingRule,
};

const DEFAULT_CHANNEL_TOLERANCE: u8 = 2;
const DEFAULT_MAX_DIFFERENCE_RATIO: f32 = 0.02;
const FIXTURE_PATH: &str = "tests/fixtures/figma/figma_import_scene.json";
const GOLDEN_PATH: &str = "tests/visual/golden/figma/figma_import_scene.png";
const ARTIFACT_PATH: &str = "tests/visual/artifacts/figma/figma_import_scene.png";

static VISUAL_TEST_MUTEX: OnceLock<Mutex<()>> = OnceLock::new();

fn acquire_visual_test_lock() -> std::sync::MutexGuard<'static, ()> {
    VISUAL_TEST_MUTEX
        .get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct FigmaSceneFixture {
    width: u32,
    height: u32,
    clear_color: [f32; 4],
    #[serde(default)]
    images: Vec<FigmaImageAsset>,
    nodes: Vec<FigmaNodeFixture>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct FigmaImageAsset {
    id: u64,
    width: u32,
    height: u32,
    pattern: FigmaImagePattern,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum FigmaImagePattern {
    Phase4Marker,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct FigmaNodeFixture {
    #[serde(default)]
    id: Option<u64>,
    #[serde(default, alias = "parentId")]
    parent_id: Option<u64>,
    bounds: [f32; 4],
    style: FigmaStyle,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct FigmaStyle {
    #[serde(default)]
    fills: Vec<FigmaPaint>,
    #[serde(default)]
    strokes: Vec<FigmaPaint>,
    stroke_weight: Option<f32>,
    stroke_align: Option<FigmaStrokeAlign>,
    stroke_cap: Option<FigmaStrokeCap>,
    stroke_join: Option<FigmaStrokeJoin>,
    stroke_miter_angle: Option<f32>,
    stroke_miter_limit: Option<f32>,
    stroke_dashes: Option<Vec<f32>>,
    #[serde(alias = "strokeDashOffset")]
    dash_offset: Option<f32>,
    #[serde(alias = "individualStrokeWeights")]
    individual_stroke_weights: Option<FigmaSideWeights>,
    #[serde(default)]
    effects: Vec<FigmaEffect>,
    fill_geometry: Option<Vec<FigmaPathGeometry>>,
    stroke_geometry: Option<Vec<FigmaPathGeometry>>,
    corner_radius: Option<f32>,
    rectangle_corner_radii: Option<[f32; 4]>,
    corner_smoothing: Option<f32>,
    opacity: Option<f32>,
    blend_mode: Option<FigmaBlendMode>,
    clips_content: Option<bool>,
    #[serde(alias = "isMask")]
    is_mask: Option<bool>,
    #[serde(alias = "maskType")]
    mask_type: Option<FigmaMaskType>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
enum FigmaPathGeometry {
    SvgPathData(String),
    PathObject(FigmaPathGeometryObject),
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct FigmaPathGeometryObject {
    #[serde(alias = "pathData")]
    path: String,
    winding_rule: Option<FigmaWindingRule>,
}

#[derive(Debug, Clone, Copy, Deserialize)]
enum FigmaWindingRule {
    #[serde(rename = "NONZERO")]
    NonZero,
    #[serde(rename = "EVENODD", alias = "EVEN_ODD")]
    EvenOdd,
}

#[derive(Debug, Clone, Deserialize)]
struct FigmaSideWeights {
    top: f32,
    right: f32,
    bottom: f32,
    left: f32,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum FigmaStrokeAlign {
    Inside,
    Center,
    Outside,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum FigmaStrokeCap {
    None,
    Round,
    Square,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum FigmaStrokeJoin {
    Miter,
    Round,
    Bevel,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum FigmaMaskType {
    Alpha,
    Vector,
    Luminance,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum FigmaImageScaleMode {
    Fill,
    Fit,
    Crop,
    Tile,
    Stretch,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum FigmaBlendMode {
    Normal,
    Darken,
    Multiply,
    ColorBurn,
    Lighten,
    Screen,
    ColorDodge,
    Overlay,
    SoftLight,
    HardLight,
    Difference,
    Exclusion,
    Hue,
    Saturation,
    Color,
    Luminosity,
    LinearBurn,
    LinearDodge,
    PassThrough,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "SCREAMING_SNAKE_CASE")]
enum FigmaPaint {
    Solid {
        color: [f32; 4],
        opacity: Option<f32>,
    },
    GradientLinear {
        start: [f32; 2],
        end: [f32; 2],
        stops: Vec<FigmaColorStop>,
    },
    GradientRadial {
        center: [f32; 2],
        radius: f32,
        stops: Vec<FigmaColorStop>,
    },
    GradientAngular {
        center: [f32; 2],
        angle: f32,
        stops: Vec<FigmaColorStop>,
    },
    GradientDiamond {
        center: [f32; 2],
        scale: f32,
        stops: Vec<FigmaColorStop>,
    },
    Image {
        #[serde(alias = "imageId", alias = "imageRef", alias = "imageHash")]
        image_id: FigmaImageId,
        #[serde(alias = "scaleMode")]
        scale_mode: FigmaImageScaleMode,
        #[serde(alias = "scalingFactor")]
        scaling_factor: Option<f32>,
        #[serde(alias = "imageTransform")]
        transform: Option<FigmaImageTransform>,
    },
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
enum FigmaImageId {
    Numeric(u64),
    Text(String),
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
enum FigmaImageTransform {
    Matrix3x3([f32; 9]),
    Rows2x3([[f32; 3]; 2]),
    Rows3x3([[f32; 3]; 3]),
}

#[derive(Debug, Clone, Deserialize)]
struct FigmaColorStop {
    position: f32,
    color: [f32; 4],
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "SCREAMING_SNAKE_CASE")]
enum FigmaEffect {
    DropShadow {
        offset: [f32; 2],
        radius: f32,
        color: [f32; 4],
        #[serde(default = "default_visible")]
        visible: bool,
    },
    InnerShadow {
        offset: [f32; 2],
        radius: f32,
        color: [f32; 4],
        #[serde(default = "default_visible")]
        visible: bool,
    },
    LayerBlur {
        radius: f32,
        #[serde(default = "default_visible")]
        visible: bool,
    },
    BackgroundBlur {
        radius: f32,
        #[serde(default = "default_visible")]
        visible: bool,
    },
}

const fn default_visible() -> bool {
    true
}

impl FigmaPaint {
    fn into_paint(self) -> Paint {
        match self {
            Self::Solid { color, opacity } => {
                let mut c = vec4(color);
                if let Some(opacity) = opacity {
                    c.w *= opacity;
                }
                Paint::Solid(c)
            }
            Self::GradientLinear { start, end, stops } => Paint::Linear(LinearGradient {
                start: vec2(start),
                end: vec2(end),
                stops: stops
                    .into_iter()
                    .map(|stop| ColorStop::new(stop.position, vec4(stop.color)))
                    .collect(),
            }),
            Self::GradientRadial {
                center,
                radius,
                stops,
            } => Paint::Radial(RadialGradient {
                center: vec2(center),
                radius,
                stops: stops
                    .into_iter()
                    .map(|stop| ColorStop::new(stop.position, vec4(stop.color)))
                    .collect(),
            }),
            Self::GradientAngular {
                center,
                angle,
                stops,
            } => Paint::Angular(AngularGradient {
                center: vec2(center),
                angle,
                stops: stops
                    .into_iter()
                    .map(|stop| ColorStop::new(stop.position, vec4(stop.color)))
                    .collect(),
            }),
            Self::GradientDiamond {
                center,
                scale,
                stops,
            } => Paint::Diamond(DiamondGradient {
                center: vec2(center),
                scale,
                stops: stops
                    .into_iter()
                    .map(|stop| ColorStop::new(stop.position, vec4(stop.color)))
                    .collect(),
            }),
            Self::Image {
                image_id,
                scale_mode,
                scaling_factor,
                transform,
            } => Paint::Image(ImageFill {
                image_id: image_id.into_image_id(),
                scale_mode: scale_mode.into(),
                transform: figma_image_transform(scale_mode, transform, scaling_factor),
            }),
        }
    }
}

impl FigmaImageId {
    fn into_image_id(self) -> ImageId {
        match self {
            Self::Numeric(id) => ImageId(id),
            Self::Text(reference) => ImageId(figma_image_reference_to_id(&reference)),
        }
    }
}

impl FigmaImageTransform {
    fn into_matrix3x3(self) -> [f32; 9] {
        match self {
            Self::Matrix3x3(matrix) => matrix,
            Self::Rows2x3([[a, b, tx], [c, d, ty]]) => [a, b, tx, c, d, ty, 0.0, 0.0, 1.0],
            Self::Rows3x3([row0, row1, row2]) => [
                row0[0], row0[1], row0[2], row1[0], row1[1], row1[2], row2[0], row2[1], row2[2],
            ],
        }
    }
}

fn figma_image_reference_to_id(reference: &str) -> u64 {
    if let Ok(parsed) = reference.parse::<u64>() {
        return parsed;
    }
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    reference.hash(&mut hasher);
    hasher.finish() | (1_u64 << 63)
}

fn figma_image_transform(
    scale_mode: FigmaImageScaleMode,
    transform: Option<FigmaImageTransform>,
    scaling_factor: Option<f32>,
) -> Option<[f32; 9]> {
    let transform = transform.map(FigmaImageTransform::into_matrix3x3);
    if !matches!(scale_mode, FigmaImageScaleMode::Tile) {
        return transform;
    }

    let Some(scaling_factor) = scaling_factor.filter(|v| v.is_finite() && *v > 0.0) else {
        return transform;
    };
    if transform.is_some() || (scaling_factor - 1.0).abs() <= f32::EPSILON {
        return transform;
    }

    let inverse = 1.0 / scaling_factor;
    Some([inverse, 0.0, 0.0, 0.0, inverse, 0.0, 0.0, 0.0, 1.0])
}

impl FigmaEffect {
    fn into_effect(self) -> Option<Effect> {
        match self {
            Self::DropShadow {
                offset,
                radius,
                color,
                visible,
            } => visible.then_some(Effect::DropShadow(DropShadow {
                offset: vec2(offset),
                blur: radius,
                color: vec4(color),
                visible,
            })),
            Self::InnerShadow {
                offset,
                radius,
                color,
                visible,
            } => visible.then_some(Effect::InnerShadow(InnerShadow {
                offset: vec2(offset),
                blur: radius,
                color: vec4(color),
                visible,
            })),
            Self::LayerBlur { radius, visible } => {
                visible.then_some(Effect::LayerBlur(LayerBlur { radius, visible }))
            }
            Self::BackgroundBlur { radius, visible } => {
                visible.then_some(Effect::BackgroundBlur(BackgroundBlur { radius, visible }))
            }
        }
    }
}

impl From<FigmaImageScaleMode> for ImageScaleMode {
    fn from(value: FigmaImageScaleMode) -> Self {
        match value {
            FigmaImageScaleMode::Fill => Self::Fill,
            FigmaImageScaleMode::Fit => Self::Fit,
            FigmaImageScaleMode::Crop => Self::Crop,
            FigmaImageScaleMode::Tile => Self::Tile,
            FigmaImageScaleMode::Stretch => Self::Stretch,
        }
    }
}

#[test]
fn figma_image_scale_mode_stretch_maps_to_style_engine_stretch() {
    let mapped: ImageScaleMode = FigmaImageScaleMode::Stretch.into();
    assert!(matches!(mapped, ImageScaleMode::Stretch));
}

#[test]
fn figma_image_paint_image_transform_alias_maps_to_affine_matrix() {
    let figma_paint: FigmaPaint = serde_json::from_str(
        r#"{
            "type":"IMAGE",
            "imageId": 42,
            "scaleMode": "CROP",
            "imageTransform": [
                [0.5, 0.0, 0.1],
                [0.0, 0.5, 0.2]
            ]
        }"#,
    )
    .expect("failed to deserialize figma image paint");

    let paint = figma_paint.into_paint();
    let Paint::Image(fill) = paint else {
        panic!("expected image paint");
    };

    assert_eq!(
        fill.transform,
        Some([0.5, 0.0, 0.1, 0.0, 0.5, 0.2, 0.0, 0.0, 1.0]),
        "imageTransform 2x3 matrix should map to homogeneous 3x3"
    );
}

#[test]
fn figma_image_paint_image_ref_alias_maps_to_stable_image_id() {
    let from_ref_a: FigmaPaint = serde_json::from_str(
        r#"{
            "type":"IMAGE",
            "imageRef":"figma-image-ref://asset-a",
            "scaleMode":"FILL"
        }"#,
    )
    .expect("failed to deserialize figma image paint with imageRef");
    let from_ref_b: FigmaPaint = serde_json::from_str(
        r#"{
            "type":"IMAGE",
            "imageRef":"figma-image-ref://asset-a",
            "scaleMode":"FILL"
        }"#,
    )
    .expect("failed to deserialize repeated figma image paint with imageRef");
    let from_hash: FigmaPaint = serde_json::from_str(
        r#"{
            "type":"IMAGE",
            "imageHash":"figma-image-ref://asset-b",
            "scaleMode":"FILL"
        }"#,
    )
    .expect("failed to deserialize figma image paint with imageHash");

    let Paint::Image(fill_ref_a) = from_ref_a.into_paint() else {
        panic!("expected image paint from imageRef payload");
    };
    let Paint::Image(fill_ref_b) = from_ref_b.into_paint() else {
        panic!("expected image paint from repeated imageRef payload");
    };
    let Paint::Image(fill_hash) = from_hash.into_paint() else {
        panic!("expected image paint from imageHash payload");
    };

    assert_eq!(
        fill_ref_a.image_id, fill_ref_b.image_id,
        "identical imageRef strings should produce stable image ids"
    );
    assert_ne!(
        fill_ref_a.image_id, fill_hash.image_id,
        "different image references should produce distinct image ids"
    );
}

#[test]
fn figma_image_paint_tile_scaling_factor_maps_to_transform() {
    let figma_paint: FigmaPaint = serde_json::from_str(
        r#"{
            "type":"IMAGE",
            "imageId": 99,
            "scaleMode":"TILE",
            "scalingFactor": 2.0
        }"#,
    )
    .expect("failed to deserialize figma image paint with scalingFactor");

    let paint = figma_paint.into_paint();
    let Paint::Image(fill) = paint else {
        panic!("expected image paint from scalingFactor payload");
    };

    assert_eq!(
        fill.transform,
        Some([0.5, 0.0, 0.0, 0.0, 0.5, 0.0, 0.0, 0.0, 1.0]),
        "TILE scalingFactor should synthesize an inverse UV scale transform"
    );
}

#[test]
fn figma_style_corner_smoothing_maps_to_visual_style_corner_smoothing() {
    let figma_style: FigmaStyle = serde_json::from_str(
        r#"{
            "fills":[{"type":"SOLID","color":[1.0,0.0,0.0,1.0]}],
            "cornerSmoothing": 0.72
        }"#,
    )
    .expect("failed to deserialize figma style");
    let style = figma_style.into_visual_style();
    assert!(
        (style.corner_smoothing - 0.72).abs() < 1e-6,
        "corner smoothing should map from figma style"
    );
}

#[test]
fn figma_style_stroke_cap_join_and_dashes_map_to_stroke_style() {
    let figma_style: FigmaStyle = serde_json::from_str(
        r#"{
            "strokes":[{"type":"SOLID","color":[1.0,1.0,1.0,1.0]}],
            "strokeWeight": 2.0,
            "strokeCap": "ROUND",
            "strokeJoin": "BEVEL",
            "strokeDashes": [4.0, 2.0]
        }"#,
    )
    .expect("failed to deserialize figma style");
    let style = figma_style.into_visual_style();
    let stroke = style.stroke.expect("expected stroke");
    assert_eq!(stroke.cap, StrokeCap::Round);
    assert_eq!(stroke.join, StrokeJoin::Bevel);
    assert_eq!(stroke.dash_pattern, vec![4.0, 2.0]);
}

#[test]
fn figma_style_dash_offset_maps_to_stroke_style_dash_offset() {
    let figma_style: FigmaStyle = serde_json::from_str(
        r#"{
            "strokes":[{"type":"SOLID","color":[1.0,1.0,1.0,1.0]}],
            "strokeWeight": 2.0,
            "strokeDashes": [4.0, 2.0],
            "dashOffset": 3.5
        }"#,
    )
    .expect("failed to deserialize figma style");
    let style = figma_style.into_visual_style();
    let stroke = style.stroke.expect("expected stroke");
    assert_eq!(stroke.dash_pattern, vec![4.0, 2.0]);
    assert!(
        (stroke.dash_offset - 3.5).abs() < 1e-6,
        "dashOffset should map to StrokeStyle.dash_offset"
    );
}

#[test]
fn figma_gradient_variants_map_to_style_engine_gradient_paints() {
    let radial: FigmaPaint = serde_json::from_str(
        r#"{
            "type":"GRADIENT_RADIAL",
            "center":[0.5,0.5],
            "radius":0.4,
            "stops":[
                {"position":0.0,"color":[1.0,0.0,0.0,1.0]},
                {"position":1.0,"color":[0.0,0.0,1.0,1.0]}
            ]
        }"#,
    )
    .expect("failed to deserialize radial gradient paint");
    assert!(matches!(radial.into_paint(), Paint::Radial(_)));

    let angular: FigmaPaint = serde_json::from_str(
        r#"{
            "type":"GRADIENT_ANGULAR",
            "center":[0.5,0.5],
            "angle":1.2,
            "stops":[
                {"position":0.0,"color":[1.0,1.0,0.0,1.0]},
                {"position":1.0,"color":[0.0,1.0,1.0,1.0]}
            ]
        }"#,
    )
    .expect("failed to deserialize angular gradient paint");
    assert!(matches!(angular.into_paint(), Paint::Angular(_)));

    let diamond: FigmaPaint = serde_json::from_str(
        r#"{
            "type":"GRADIENT_DIAMOND",
            "center":[0.5,0.5],
            "scale":0.8,
            "stops":[
                {"position":0.0,"color":[1.0,0.5,0.0,1.0]},
                {"position":1.0,"color":[0.0,0.2,1.0,1.0]}
            ]
        }"#,
    )
    .expect("failed to deserialize diamond gradient paint");
    assert!(matches!(diamond.into_paint(), Paint::Diamond(_)));
}

#[test]
fn figma_style_fill_geometry_maps_svg_paths_with_winding_rule() {
    let figma_style: FigmaStyle = serde_json::from_str(
        r#"{
            "fills":[{"type":"SOLID","color":[0.9,0.2,0.3,1.0]}],
            "fillGeometry": [
                {
                    "path": "M 0 0 L 10 0 L 10 10 L 0 10 Z",
                    "windingRule": "EVENODD"
                }
            ]
        }"#,
    )
    .expect("failed to deserialize figma style");

    let style = figma_style.into_visual_style();
    let geometry = style
        .fill_geometry
        .as_ref()
        .expect("expected fill geometry from figma fillGeometry");
    assert_eq!(geometry.len(), 1, "expected one mapped fill geometry path");
    assert!(
        !geometry[0].commands.is_empty(),
        "mapped fill geometry path should contain parsed commands"
    );
    assert_eq!(
        geometry[0].winding_rule,
        WindingRule::EvenOdd,
        "windingRule should map from figma geometry"
    );
}

#[test]
fn figma_style_fill_geometry_path_data_alias_maps() {
    let figma_style: FigmaStyle = serde_json::from_str(
        r#"{
            "fills":[{"type":"SOLID","color":[0.4,0.9,0.6,1.0]}],
            "fillGeometry": [
                {
                    "pathData": "M 0 0 L 10 0 L 10 10 Z",
                    "windingRule": "NONZERO"
                }
            ]
        }"#,
    )
    .expect("failed to deserialize figma style");

    let style = figma_style.into_visual_style();
    let geometry = style
        .fill_geometry
        .as_ref()
        .expect("expected fill geometry from pathData alias");
    assert_eq!(geometry.len(), 1, "expected one mapped fill geometry path");
    assert!(
        !geometry[0].commands.is_empty(),
        "mapped fill geometry path should contain parsed commands"
    );
    assert_eq!(
        geometry[0].winding_rule,
        WindingRule::NonZero,
        "NONZERO winding should map to NonZero"
    );
}

#[test]
fn figma_style_fill_geometry_even_odd_winding_alias_maps() {
    let figma_style: FigmaStyle = serde_json::from_str(
        r#"{
            "fills":[{"type":"SOLID","color":[0.7,0.6,0.9,1.0]}],
            "fillGeometry": [
                {
                    "path": "M 0 0 L 10 0 L 10 10 Z",
                    "windingRule": "EVEN_ODD"
                }
            ]
        }"#,
    )
    .expect("failed to deserialize figma style");

    let style = figma_style.into_visual_style();
    let geometry = style
        .fill_geometry
        .as_ref()
        .expect("expected fill geometry from EVEN_ODD alias");
    assert_eq!(geometry.len(), 1, "expected one mapped fill geometry path");
    assert_eq!(
        geometry[0].winding_rule,
        WindingRule::EvenOdd,
        "EVEN_ODD should map to EvenOdd winding rule"
    );
}

#[test]
fn figma_style_stroke_geometry_maps_svg_path_strings() {
    let figma_style: FigmaStyle = serde_json::from_str(
        r#"{
            "strokes":[{"type":"SOLID","color":[1.0,1.0,1.0,1.0]}],
            "strokeWeight": 1.0,
            "strokeGeometry": ["M 0 0 L 12 0 L 6 8 Z"]
        }"#,
    )
    .expect("failed to deserialize figma style");

    let style = figma_style.into_visual_style();
    let geometry = style
        .stroke_geometry
        .as_ref()
        .expect("expected stroke geometry from figma strokeGeometry");
    assert_eq!(
        geometry.len(),
        1,
        "expected one mapped stroke geometry path"
    );
    assert!(
        !geometry[0].commands.is_empty(),
        "mapped stroke geometry path should contain parsed commands"
    );
}

#[test]
fn figma_style_stroke_miter_angle_maps_to_miter_limit() {
    let figma_style: FigmaStyle = serde_json::from_str(
        r#"{
            "strokes":[{"type":"SOLID","color":[1.0,1.0,1.0,1.0]}],
            "strokeWeight": 2.0,
            "strokeJoin": "MITER",
            "strokeMiterAngle": 60.0
        }"#,
    )
    .expect("failed to deserialize figma style");

    let style = figma_style.into_visual_style();
    let stroke = style.stroke.expect("expected stroke");
    assert!(
        (stroke.miter_limit - 2.0).abs() < 1e-3,
        "expected strokeMiterAngle=60deg to map near miter_limit=2.0, got {}",
        stroke.miter_limit
    );
}

#[test]
fn figma_style_explicit_stroke_miter_limit_overrides_angle_mapping() {
    let figma_style: FigmaStyle = serde_json::from_str(
        r#"{
            "strokes":[{"type":"SOLID","color":[1.0,1.0,1.0,1.0]}],
            "strokeWeight": 2.0,
            "strokeJoin": "MITER",
            "strokeMiterAngle": 60.0,
            "strokeMiterLimit": 6.5
        }"#,
    )
    .expect("failed to deserialize figma style");

    let style = figma_style.into_visual_style();
    let stroke = style.stroke.expect("expected stroke");
    assert!(
        (stroke.miter_limit - 6.5).abs() < 1e-6,
        "explicit strokeMiterLimit should take precedence over angle mapping"
    );
}

#[test]
fn figma_node_parent_id_maps_to_scene_hierarchy() {
    let fixture: FigmaSceneFixture = serde_json::from_str(
        r#"{
            "width": 100,
            "height": 100,
            "clearColor": [0.0, 0.0, 0.0, 1.0],
            "nodes": [
                {
                    "id": 1,
                    "bounds": [0.0, 0.0, 100.0, 100.0],
                    "style": {"fills":[{"type":"SOLID","color":[0.1,0.1,0.1,1.0]}]}
                },
                {
                    "id": 2,
                    "parentId": 1,
                    "bounds": [5.0, 5.0, 60.0, 60.0],
                    "style": {"clipsContent": true}
                },
                {
                    "id": 3,
                    "parentId": 2,
                    "bounds": [10.0, 10.0, 80.0, 80.0],
                    "style": {"fills":[{"type":"SOLID","color":[1.0,0.0,0.0,1.0]}]}
                },
                {
                    "id": 4,
                    "parentId": 1,
                    "bounds": [70.0, 70.0, 20.0, 20.0],
                    "style": {"fills":[{"type":"SOLID","color":[0.0,1.0,0.0,1.0]}]}
                }
            ]
        }"#,
    )
    .expect("failed to deserialize figma fixture");

    let scene = build_scene(&fixture);
    let root = scene.root();
    let root_children = scene
        .get_node(root)
        .expect("scene root should exist")
        .children
        .clone();
    assert_eq!(
        root_children.len(),
        1,
        "expected one top-level imported node under scene root"
    );

    let top_level = root_children[0];
    let top_level_children = scene
        .get_node(top_level)
        .expect("top-level imported node should exist")
        .children
        .clone();
    assert_eq!(
        top_level_children.len(),
        2,
        "expected parentId children to attach under their declared parent"
    );

    let nested_parent = top_level_children[0];
    let nested_leaf = scene
        .get_node(nested_parent)
        .expect("nested parent should exist")
        .children
        .first()
        .copied()
        .expect("nested parent should contain leaf child");
    assert_eq!(
        scene.parent(nested_leaf),
        Some(nested_parent),
        "leaf should remain attached to parentId chain"
    );
}

#[test]
fn figma_fixture_includes_vector_geometry_render_cases() {
    let fixture = load_fixture(Path::new(FIXTURE_PATH));

    let mut fill_geometry_nodes = 0usize;
    let mut stroke_geometry_nodes = 0usize;

    for node in &fixture.nodes {
        let style = node.style.clone().into_visual_style();
        if style
            .fill_geometry
            .as_ref()
            .is_some_and(|paths| !paths.is_empty())
        {
            fill_geometry_nodes += 1;
        }
        if style
            .stroke_geometry
            .as_ref()
            .is_some_and(|paths| !paths.is_empty())
        {
            stroke_geometry_nodes += 1;
        }
    }

    assert!(
        fill_geometry_nodes > 0,
        "fixture should include at least one node with mapped fillGeometry"
    );
    assert!(
        stroke_geometry_nodes > 0,
        "fixture should include at least one node with mapped strokeGeometry"
    );
}

impl From<FigmaStrokeAlign> for StrokeAlign {
    fn from(value: FigmaStrokeAlign) -> Self {
        match value {
            FigmaStrokeAlign::Inside => Self::Inside,
            FigmaStrokeAlign::Center => Self::Center,
            FigmaStrokeAlign::Outside => Self::Outside,
        }
    }
}

impl From<FigmaWindingRule> for WindingRule {
    fn from(value: FigmaWindingRule) -> Self {
        match value {
            FigmaWindingRule::NonZero => Self::NonZero,
            FigmaWindingRule::EvenOdd => Self::EvenOdd,
        }
    }
}

impl From<FigmaStrokeCap> for StrokeCap {
    fn from(value: FigmaStrokeCap) -> Self {
        match value {
            FigmaStrokeCap::None => Self::Butt,
            FigmaStrokeCap::Round => Self::Round,
            FigmaStrokeCap::Square => Self::Square,
        }
    }
}

impl From<FigmaStrokeJoin> for StrokeJoin {
    fn from(value: FigmaStrokeJoin) -> Self {
        match value {
            FigmaStrokeJoin::Miter => Self::Miter,
            FigmaStrokeJoin::Round => Self::Round,
            FigmaStrokeJoin::Bevel => Self::Bevel,
        }
    }
}

impl From<FigmaMaskType> for MaskType {
    fn from(value: FigmaMaskType) -> Self {
        match value {
            FigmaMaskType::Alpha => Self::Alpha,
            FigmaMaskType::Vector => Self::Vector,
            FigmaMaskType::Luminance => Self::Luminance,
        }
    }
}

impl From<FigmaBlendMode> for BlendMode {
    fn from(value: FigmaBlendMode) -> Self {
        match value {
            FigmaBlendMode::Normal => Self::Normal,
            FigmaBlendMode::Darken => Self::Darken,
            FigmaBlendMode::Multiply => Self::Multiply,
            FigmaBlendMode::ColorBurn => Self::ColorBurn,
            FigmaBlendMode::Lighten => Self::Lighten,
            FigmaBlendMode::Screen => Self::Screen,
            FigmaBlendMode::ColorDodge => Self::ColorDodge,
            FigmaBlendMode::Overlay => Self::Overlay,
            FigmaBlendMode::SoftLight => Self::SoftLight,
            FigmaBlendMode::HardLight => Self::HardLight,
            FigmaBlendMode::Difference => Self::Difference,
            FigmaBlendMode::Exclusion => Self::Exclusion,
            FigmaBlendMode::Hue => Self::Hue,
            FigmaBlendMode::Saturation => Self::Saturation,
            FigmaBlendMode::Color => Self::Color,
            FigmaBlendMode::Luminosity => Self::Luminosity,
            FigmaBlendMode::LinearBurn => Self::LinearBurn,
            FigmaBlendMode::LinearDodge => Self::LinearDodge,
            FigmaBlendMode::PassThrough => Self::PassThrough,
        }
    }
}

impl FigmaStyle {
    fn into_visual_style(self) -> VisualStyle {
        let mut style = VisualStyle::new();
        for fill in self.fills {
            style = style.fill(fill.into_paint());
        }
        if let Some(fill_geometry) = self.fill_geometry {
            let paths: Vec<_> = fill_geometry
                .into_iter()
                .filter_map(FigmaPathGeometry::into_vector_path)
                .collect();
            if !paths.is_empty() {
                style = style.fill_geometry(paths);
            }
        }

        if let Some(corner_radius) = self.corner_radius {
            style = style.corner_radius(corner_radius);
        }
        if let Some([tl, tr, br, bl]) = self.rectangle_corner_radii {
            style = style.corner_radii(CornerRadii::new(tl, tr, br, bl));
        }
        if let Some(corner_smoothing) = self.corner_smoothing {
            style = style.corner_smoothing(corner_smoothing);
        }

        for effect in self.effects {
            if let Some(effect) = effect.into_effect() {
                style = style.effect(effect);
            }
        }

        if !self.strokes.is_empty() || self.stroke_weight.unwrap_or_default() > 0.0 {
            let mut stroke = StrokeStyle {
                paints: self
                    .strokes
                    .into_iter()
                    .map(FigmaPaint::into_paint)
                    .collect::<Vec<_>>(),
                weight: self.stroke_weight.unwrap_or(1.0),
                align: self.stroke_align.unwrap_or(FigmaStrokeAlign::Center).into(),
                cap: self.stroke_cap.unwrap_or(FigmaStrokeCap::None).into(),
                join: self.stroke_join.unwrap_or(FigmaStrokeJoin::Miter).into(),
                miter_limit: self
                    .stroke_miter_limit
                    .filter(|v| v.is_finite() && *v > 0.0)
                    .or_else(|| {
                        self.stroke_miter_angle
                            .and_then(figma_stroke_miter_limit_from_angle)
                    })
                    .unwrap_or(StrokeStyle::default().miter_limit),
                dash_pattern: self.stroke_dashes.unwrap_or_default(),
                dash_offset: self
                    .dash_offset
                    .filter(|v| v.is_finite())
                    .unwrap_or(StrokeStyle::default().dash_offset),
                ..StrokeStyle::default()
            };
            if stroke.paints.is_empty() {
                stroke.paints.push(Paint::Solid(Vec4::ONE));
            }
            if let Some(side_weights) = self.individual_stroke_weights {
                stroke.side_weights = Some(SideWeights {
                    top: side_weights.top,
                    right: side_weights.right,
                    bottom: side_weights.bottom,
                    left: side_weights.left,
                });
            }
            style = style.stroke(stroke);
        }
        if let Some(stroke_geometry) = self.stroke_geometry {
            let paths: Vec<_> = stroke_geometry
                .into_iter()
                .filter_map(FigmaPathGeometry::into_vector_path)
                .collect();
            if !paths.is_empty() {
                style = style.stroke_geometry(paths);
            }
        }

        if let Some(opacity) = self.opacity {
            style = style.opacity(opacity);
        }
        if let Some(blend_mode) = self.blend_mode {
            style = style.blend_mode(blend_mode.into());
        }
        if let Some(clips_content) = self.clips_content {
            style = style.clips_content(clips_content);
        }
        if let Some(is_mask) = self.is_mask {
            style = style.is_mask(is_mask);
        }
        if let Some(mask_type) = self.mask_type {
            style = style.mask_type(mask_type.into());
        }

        style
    }
}

impl FigmaPathGeometry {
    fn into_vector_path(self) -> Option<VectorPath> {
        let (path_data, winding_rule) = match self {
            Self::SvgPathData(path) => (path, None),
            Self::PathObject(obj) => (obj.path, obj.winding_rule.map(Into::into)),
        };

        let mut path = VectorPath::from_svg_path_data(&path_data).ok()?;
        if let Some(winding_rule) = winding_rule {
            path.winding_rule = winding_rule;
        }
        Some(path)
    }
}

fn vec2(v: [f32; 2]) -> Vec2 {
    Vec2::new(v[0], v[1])
}

fn vec4(v: [f32; 4]) -> Vec4 {
    Vec4::new(v[0], v[1], v[2], v[3])
}

fn load_fixture(path: &Path) -> FigmaSceneFixture {
    let json = std::fs::read_to_string(path)
        .unwrap_or_else(|err| panic!("failed to read fixture {}: {err}", path.display()));
    serde_json::from_str::<FigmaSceneFixture>(&json)
        .unwrap_or_else(|err| panic!("failed to deserialize fixture {}: {err}", path.display()))
}

fn build_scene(fixture: &FigmaSceneFixture) -> Scene {
    let mut scene = Scene::new();
    let root = scene.root();

    let mut used_ids = HashSet::new();
    let mut resolved_ids = Vec::with_capacity(fixture.nodes.len());
    for (index, node) in fixture.nodes.iter().enumerate() {
        let fallback_id = 1_000_000_000_u64.saturating_add(index as u64);
        let mut stable_id = node.id.unwrap_or(fallback_id);
        while !used_ids.insert(stable_id) {
            stable_id = stable_id.saturating_add(1);
        }
        resolved_ids.push(stable_id);
    }

    let mut figma_to_scene = HashMap::new();
    let mut pending: Vec<usize> = (0..fixture.nodes.len()).collect();

    while !pending.is_empty() {
        let mut progressed = false;
        let mut unresolved = Vec::new();

        for index in pending {
            let node = &fixture.nodes[index];
            let parent = match node.parent_id {
                Some(parent_id) => figma_to_scene.get(&parent_id).copied(),
                None => Some(root),
            };

            if let Some(parent) = parent {
                let mut scene_node = SceneNode::new(NodeContent::Styled {
                    style: Box::new(node.style.clone().into_visual_style()),
                });
                scene_node.bounds = Rect::new(
                    node.bounds[0],
                    node.bounds[1],
                    node.bounds[2].max(0.0),
                    node.bounds[3].max(0.0),
                );
                let scene_id = scene.add_node(parent, scene_node);
                figma_to_scene.insert(resolved_ids[index], scene_id);
                progressed = true;
            } else {
                unresolved.push(index);
            }
        }

        if !progressed {
            for index in unresolved {
                let node = &fixture.nodes[index];
                let mut scene_node = SceneNode::new(NodeContent::Styled {
                    style: Box::new(node.style.clone().into_visual_style()),
                });
                scene_node.bounds = Rect::new(
                    node.bounds[0],
                    node.bounds[1],
                    node.bounds[2].max(0.0),
                    node.bounds[3].max(0.0),
                );
                let scene_id = scene.add_node(root, scene_node);
                figma_to_scene.insert(resolved_ids[index], scene_id);
            }
            break;
        }

        pending = unresolved;
    }

    scene
}

fn figma_stroke_miter_limit_from_angle(angle_degrees: f32) -> Option<f32> {
    if !angle_degrees.is_finite() || angle_degrees <= 0.0 {
        return None;
    }
    let half_radians = 0.5 * angle_degrees.to_radians();
    let sin = half_radians.sin().abs();
    if sin <= f32::EPSILON {
        return None;
    }
    Some(1.0 / sin)
}

fn register_fixture_images(fixture: &FigmaSceneFixture, backend: &mut WgpuBackend) {
    for image in &fixture.images {
        let bytes = match image.pattern {
            FigmaImagePattern::Phase4Marker => phase4_marker_bytes(image.width, image.height),
        };
        backend
            .register_image_rgba8(ImageId(image.id), image.width, image.height, bytes)
            .unwrap_or_else(|err| {
                panic!(
                    "failed to register fixture image {} ({}x{}): {err}",
                    image.id, image.width, image.height
                )
            });
    }
}

fn phase4_marker_bytes(width: u32, height: u32) -> Vec<u8> {
    let mut rgba = vec![0u8; (width * height * 4) as usize];
    let margin_x = 6u32;
    let margin_y = 6u32;
    let gap_x = 4u32;
    let gap_y = 4u32;
    let block_w = (width.saturating_sub(margin_x * 2 + gap_x)) / 2;
    let block_h = (height.saturating_sub(margin_y * 2 + gap_y)) / 2;
    let cx = (width / 2) as i32;
    let cy = (height / 2) as i32;
    let r2 = 7_i32 * 7_i32;

    for y in 0..height {
        for x in 0..width {
            let idx = ((y * width + x) * 4) as usize;
            let mut color = [28u8, 36u8, 52u8, 255u8];

            let in_tl =
                x >= margin_x && x < margin_x + block_w && y >= margin_y && y < margin_y + block_h;
            let in_tr = x >= margin_x + block_w + gap_x
                && x < margin_x + block_w + gap_x + block_w
                && y >= margin_y
                && y < margin_y + block_h;
            let in_bl = x >= margin_x
                && x < margin_x + block_w
                && y >= margin_y + block_h + gap_y
                && y < margin_y + block_h + gap_y + block_h;
            let in_br = x >= margin_x + block_w + gap_x
                && x < margin_x + block_w + gap_x + block_w
                && y >= margin_y + block_h + gap_y
                && y < margin_y + block_h + gap_y + block_h;

            if in_tl {
                color = [224, 86, 86, 255];
            } else if in_tr {
                color = [83, 188, 236, 255];
            } else if in_bl {
                color = [237, 196, 85, 255];
            } else if in_br {
                color = [163, 116, 229, 255];
            }

            if x == 0 || y == 0 || x + 1 == width || y + 1 == height {
                color = [245, 245, 245, 255];
            }

            if x.abs_diff(width / 2) <= 1 || y.abs_diff(height / 2) <= 1 {
                color = [22, 22, 22, 255];
            }

            let dx = x as i32 - cx;
            let dy = y as i32 - cy;
            if dx * dx + dy * dy <= r2 {
                color = [245, 245, 245, 255];
            }

            rgba[idx] = color[0];
            rgba[idx + 1] = color[1];
            rgba[idx + 2] = color[2];
            rgba[idx + 3] = color[3];
        }
    }

    rgba
}

fn should_update_goldens() -> bool {
    std::env::var("ARTHROPOD_UPDATE_GOLDENS")
        .map(|value| {
            matches!(
                value.to_ascii_lowercase().as_str(),
                "1" | "true" | "yes" | "on"
            )
        })
        .unwrap_or(false)
}

fn env_u8(name: &str, default: u8) -> u8 {
    std::env::var(name)
        .ok()
        .and_then(|value| value.parse::<u8>().ok())
        .unwrap_or(default)
}

fn env_f32(name: &str, default: f32) -> f32 {
    std::env::var(name)
        .ok()
        .and_then(|value| value.parse::<f32>().ok())
        .unwrap_or(default)
}

#[test]
fn figma_json_render_regression_matches_golden() {
    let _guard = acquire_visual_test_lock();
    let fixture = load_fixture(Path::new(FIXTURE_PATH));
    let scene = build_scene(&fixture);

    let event_loop = EventLoop::new().expect("failed to create event loop");
    let window = event_loop
        .create_window(WindowConfig {
            title: "Figma JSON Render Regression".to_string(),
            size: Size::new(fixture.width, fixture.height),
            visible: false,
            ..Default::default()
        })
        .expect("failed to create window");

    // SAFETY: backend drops before window in this scope.
    let mut backend = unsafe { WgpuBackend::new(&window, fixture.width, fixture.height, false) }
        .expect("failed to initialize backend");
    backend.set_clear_color(Color::from_vec4(vec4(fixture.clear_color)));
    register_fixture_images(&fixture, &mut backend);

    let rendered = backend
        .render_scene_to_rgba(&scene, fixture.width, fixture.height)
        .expect("failed to render fixture scene");

    let artifact_path = PathBuf::from(ARTIFACT_PATH);
    if let Some(dir) = artifact_path.parent() {
        std::fs::create_dir_all(dir).expect("failed to create artifact directory");
    }
    save_image(&artifact_path, &rendered, fixture.width, fixture.height)
        .expect("failed to save artifact image");

    let golden_path = Path::new(GOLDEN_PATH);
    if should_update_goldens() || !golden_path.exists() {
        if let Some(dir) = golden_path.parent() {
            std::fs::create_dir_all(dir).expect("failed to create golden directory");
        }
        save_image(golden_path, &rendered, fixture.width, fixture.height)
            .expect("failed to write golden image");
        assert!(
            should_update_goldens(),
            "missing golden image: {}. generated from current render. rerun with ARTHROPOD_UPDATE_GOLDENS=1 to accept it.",
            golden_path.display()
        );
    }

    let (golden, golden_w, golden_h) =
        load_image(golden_path).expect("failed to load golden image");
    assert_eq!(
        (golden_w, golden_h),
        (fixture.width, fixture.height),
        "golden dimensions must match fixture dimensions"
    );

    let channel_tolerance = env_u8(
        "ARTHROPOD_VISUAL_CHANNEL_TOLERANCE",
        DEFAULT_CHANNEL_TOLERANCE,
    );
    let max_difference_ratio = env_f32(
        "ARTHROPOD_VISUAL_MAX_DIFF_RATIO",
        DEFAULT_MAX_DIFFERENCE_RATIO,
    );
    let diff_ratio = compare_images_with_tolerance(
        &rendered,
        &golden,
        fixture.width,
        fixture.height,
        channel_tolerance,
    )
    .expect("failed to compare images");

    assert!(
        diff_ratio <= max_difference_ratio,
        "figma JSON render regression exceeded threshold: diff_ratio={diff_ratio:.4}, threshold={max_difference_ratio:.4}, channel_tolerance={channel_tolerance}. artifact: {}",
        artifact_path.display()
    );
}
