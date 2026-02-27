# Figma-Compatible GPU Rendering Pipeline

**Date**: 2026-02-08
**Status**: Draft
**Author**: Mark + Claude
**Timeline**: ~16 weeks (Phases 1-5)

---

## Executive Summary

Arthropod's current rendering pipeline handles solid rectangles, rounded rectangles, and text. To deliver on the vision of a **beautiful, Figma-native GUI framework**, we need a rendering pipeline that can express everything Figma can express - gradients, shadows, strokes, blur, blend modes, per-corner radii, vector paths, and more.

This design describes a **5-phase rendering pipeline upgrade** that:

1. **Mirrors the Figma visual API** - Every Figma visual property has a 1:1 Arthropod equivalent
2. **Owns the GPU rendering** - Purpose-built SDF + tessellation pipeline (no external renderer dependency)
3. **Targets WASM from day one** - No compute shaders required; works on WebGPU AND WebGL2
4. **Scales to editor-class workloads** - Tessellation caching enables design-tool-level complexity
5. **Clean break** - No backwards compatibility baggage; `NodeContent::Styled` replaces `Rect`/`RoundedRect` entirely
6. **Preserves existing performance** - Instanced rendering, <1ms for 1,000 widgets

### Why Not Vello?

| Factor | Vello | Our Approach |
|--------|-------|--------------|
| wgpu version | 27 (we're on 28) | Same as ours |
| WASM/WebGL2 | No (requires compute) | Yes |
| Architecture coupling | External alpha dependency | We own it |
| GUI optimization | General-purpose vector | Purpose-built for UI |
| Figma data model | Not included | First-class |
| Bevy integration | Separate crate | Same wgpu/ecs stack |

### Why Not Just Extend What We Have Piecemeal?

We could bolt on gradients here, shadows there. But Figma's visual model is a **coherent system** - fills, strokes, and effects stack and compose. Designing the data model and pipeline together ensures correctness and avoids rework.

---

## Architecture Overview

### New Crate: `style-engine`

A renderer-agnostic crate that defines the Figma-compatible visual data model.

```
arthropod/
├── crates/
│   ├── style-engine/          ← NEW: Figma-compatible visual types
│   │   ├── src/
│   │   │   ├── lib.rs         # Re-exports
│   │   │   ├── paint.rs       # Paint, Gradient, ColorStop
│   │   │   ├── stroke.rs      # StrokeStyle, StrokeAlign, caps/joins
│   │   │   ├── effect.rs      # Shadow, Blur effects
│   │   │   ├── blend.rs       # BlendMode enum
│   │   │   ├── corner.rs      # CornerRadii (per-corner)
│   │   │   ├── path.rs        # VectorPath, PathCommand
│   │   │   ├── text.rs        # TextContent, FontStyle, TextAlign, LineHeight
│   │   │   └── visual.rs      # VisualStyle (the unified type)
│   │   └── Cargo.toml
│   ├── render-engine/
│   │   ├── src/
│   │   │   ├── backend/
│   │   │   │   ├── shaders/
│   │   │   │   │   ├── primitive.wgsl     ← NEW: replaces rect.wgsl (SDF + glyph unified)
│   │   │   │   │   ├── shadow.wgsl        ← NEW: drop/inner shadow
│   │   │   │   │   ├── blur.wgsl          ← NEW: gaussian blur pass
│   │   │   │   │   ├── path.wgsl          ← NEW: tessellated paths
│   │   │   │   │   └── blend.wgsl         ← NEW: blend mode compositing
│   │   │   │   ├── wgpu/
│   │   │   │   │   ├── mod.rs             # EXISTING (rewritten)
│   │   │   │   │   ├── context.rs         # EXISTING (render target support)
│   │   │   │   │   └── pipelines/
│   │   │   │   │       ├── primitive_pipeline.rs ← NEW: replaces rect + glyph
│   │   │   │   │       ├── shadow_pipeline.rs    ← NEW
│   │   │   │   │       ├── blur_pipeline.rs      ← NEW
│   │   │   │   │       ├── path_pipeline.rs      ← NEW
│   │   │   │   │       └── stencil_pipeline.rs   ← NEW
```

### Dependency Graph

```
widget-core ──→ style-engine ──→ render-engine
                     │
                     ↓
              (pure Rust types,
               no GPU dependency,
               WASM-safe)
```

### Rendering Order (per frame)

```
1. Collect VisualStyle per node from scene graph
2. Sort by z-order (Painter's Algorithm, existing)
3. For each node:
   a. Render drop shadows (shadow pipeline, offset + blur)
   b. Render fills bottom-to-top (primitive pipeline)
   c. Render strokes bottom-to-top (primitive pipeline)
   d. Render inner shadows (shadow pipeline, inverted SDF)
   e. Render text glyphs if present (primitive pipeline, glyph atlas sampling)
   f. Render children (recurse)
4. Apply layer effects (blur, blend modes) via render-to-texture
```

**Unified pipeline**: Text glyphs render through the same `PrimitivePipeline` as shapes. Each glyph is a `PrimitiveInstance` with the `is_glyph` flag set, sampling from the glyph atlas texture. Gradient text is achieved by combining glyph alpha with gradient LUT sampling - no render-to-texture needed.

---

## Phase 1: Extended SDF Pipeline (~3 weeks)

**Goal**: Extend the existing rect shader to handle gradients, per-corner radii, strokes, and drop shadows - all within the instanced rendering model.

### 1.1 Per-Corner Radii

Extend the SDF from uniform radius to per-corner.

**Current `RectInstance`** (36 bytes):
```rust
pub struct RectInstance {
    pub pos: [f32; 2],
    pub size: [f32; 2],
    pub color: [f32; 4],
    pub corner_radius: f32,  // uniform
}
```

**New `PrimitiveInstance`** (96 bytes):
```rust
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct PrimitiveInstance {
    // Geometry (16 bytes)
    pub pos: [f32; 2],
    pub size: [f32; 2],

    // Fill color (16 bytes) - used for solid fills
    pub color: [f32; 4],

    // Corner radii (16 bytes) - [TL, TR, BR, BL]
    pub corner_radii: [f32; 4],

    // Stroke (16 bytes)
    pub stroke_color: [f32; 4],

    // Glyph texture coordinates (16 bytes) - used when is_glyph flag set
    pub tex_coords: [f32; 4],   // [u_min, v_min, u_max, v_max] in glyph atlas

    // Stroke properties + flags (16 bytes)
    pub stroke_width: f32,
    pub stroke_align: f32,      // 0=center, 1=inside, -1=outside
    pub flags: u32,             // bitfield: fill_type, has_stroke, blend_mode, is_glyph
    pub _padding: f32,
}
```

**`flags` bitfield layout:**
```
bits 0-3:   fill_type (0=solid, 1=linear_grad, 2=radial_grad, 3=angular_grad)
bits 4-4:   has_stroke (0=no, 1=yes)
bits 5-9:   blend_mode (0=normal, 1=multiply, 2=screen, ...)
bits 10-11: stroke_cap (0=butt, 1=round, 2=square)
bits 12-13: stroke_join (0=miter, 1=bevel, 2=round)
bits 14-14: is_glyph (0=shape, 1=glyph - samples from glyph atlas)
bits 15-31: reserved
```

**Shader change** (`rect.wgsl` + `glyph.wgsl` → `primitive.wgsl`):

```wgsl
// Per-corner rounded rectangle SDF
// Based on https://iquilezles.org/articles/distfunctions2d/
fn rounded_rect_sdf_4(pos: vec2<f32>, half_size: vec2<f32>, radii: vec4<f32>) -> f32 {
    // radii = [TL, TR, BR, BL]
    // Select radius based on quadrant
    var r: vec2<f32>;
    if pos.x > 0.0 {
        r = vec2<f32>(radii.y, radii.z);  // TR, BR
    } else {
        r = vec2<f32>(radii.x, radii.w);  // TL, BL
    }
    let radius = select(r.x, r.y, pos.y > 0.0);

    let q = abs(pos) - half_size + vec2<f32>(radius);
    return min(max(q.x, q.y), 0.0) + length(max(q, vec2<f32>(0.0))) - radius;
}
```

**Corner smoothing** (iOS squircle): We can add this later as a shader parameter. The SDF modification for superellipse smoothing is:
```wgsl
// Squircle blend: mix between circular arc and superellipse
fn smoothed_corner_sdf(..., smoothing: f32) -> f32 {
    let circular = rounded_rect_sdf_4(...);
    let superellipse = superellipse_sdf(...);
    return mix(circular, superellipse, smoothing);
}
```

### 1.2 Unified Glyph Rendering

Text glyphs render through the same `PrimitivePipeline` as shapes. Each glyph is emitted as a `PrimitiveInstance` with `is_glyph` set in `flags` and `tex_coords` pointing into the glyph atlas.

**Fragment shader** (inside `primitive.wgsl`):
```wgsl
@group(2) @binding(0)
var glyph_atlas: texture_2d<f32>;
@group(2) @binding(1)
var glyph_sampler: sampler;

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    if is_glyph(input.flags) {
        // Sample glyph alpha from atlas (R8 single-channel)
        let uv = mix(input.tex_min, input.tex_max, input.local_uv);
        let alpha = textureSample(glyph_atlas, glyph_sampler, uv).r;

        // Determine fill color: solid or gradient
        var fill_color: vec4<f32>;
        let fill_type = input.flags & 0xFu;
        if fill_type == 0u {
            fill_color = input.color;
        } else {
            fill_color = sample_gradient(input.local_uv, get_gradient_params(input.gradient_index));
        }

        return vec4(fill_color.rgb, fill_color.a * alpha);
    }

    // ... normal shape rendering (SDF, stroke, etc.)
}

fn is_glyph(flags: u32) -> bool {
    return (flags & (1u << 14u)) != 0u;
}
```

**Why unified**: Gradient text (a common Figma pattern) comes for free - the glyph alpha is multiplied by the gradient color, no render-to-texture compositing needed. Solid text overhead is ~4μs for 1000 glyphs (GPU branch coherence: all glyphs take the same path).

**Glyph instance generation** (CPU side):
```rust
fn emit_glyph_instances(
    text_content: &TextContent,
    style: &VisualStyle,
    shaped_glyphs: &[ShapedGlyph],
) -> Vec<PrimitiveInstance> {
    shaped_glyphs.iter().map(|glyph| {
        PrimitiveInstance {
            pos: [glyph.x, glyph.y],
            size: [glyph.width, glyph.height],
            color: style.fills.first().map(|f| f.solid_color()).unwrap_or(Color::BLACK).to_array(),
            corner_radii: [0.0; 4],
            stroke_color: [0.0; 4],
            tex_coords: [glyph.u_min, glyph.v_min, glyph.u_max, glyph.v_max],
            stroke_width: 0.0,
            stroke_align: 0.0,
            flags: (1 << 14) | fill_type_bits(&style.fills),
            _padding: 0.0,
        }
    }).collect()
}
```

### 1.3 Linear Gradients (+ All Gradient Types)

**Approach**: Encode gradient parameters in a per-instance uniform buffer. The fragment shader computes the gradient color based on the fragment's position along the gradient axis.

**Gradient data: Texture-based LUT (unlimited stops)**

Gradients use a 1D texture LUT (Look-Up Table) rather than an in-shader stop array.
This is both faster (1 texture sample vs N comparisons) and unlimited in stop count.

**LUT specification:**
- **Format**: `Rgba16Float` (16-bit half-float per channel - zero perceptible quantization)
- **Width**: 1024 texels per gradient
- **Layout**: Multiple gradients packed into rows of a 2D texture atlas
- **Worst case**: Hard stop on a 1000px gradient has ~1px edge softening (sub-pixel, imperceptible)

```rust
/// Gradient atlas: packs multiple gradient LUTs into a 2D texture.
/// Each row is one gradient, 1024 texels wide.
pub struct GradientAtlas {
    texture: wgpu::Texture,         // Rgba16Float, 1024 x max_gradients
    view: wgpu::TextureView,
    sampler: wgpu::Sampler,         // Linear filtering for smooth interpolation
    row_allocator: u32,             // Next free row
    cache: HashMap<u64, u32>,       // gradient_hash → row index
}

impl GradientAtlas {
    const LUT_WIDTH: u32 = 1024;

    /// Rasterize gradient stops into a 1024-texel LUT row.
    pub fn rasterize_gradient(&mut self, stops: &[ColorStop]) -> u32 {
        let row = self.row_allocator;
        let mut pixels = vec![[0.0f32; 4]; Self::LUT_WIDTH as usize];

        for x in 0..Self::LUT_WIDTH {
            let t = x as f32 / (Self::LUT_WIDTH - 1) as f32;
            pixels[x as usize] = interpolate_stops(t, stops);
        }

        // Upload row to GPU texture
        self.upload_row(row, &pixels);
        self.row_allocator += 1;
        row
    }
}

/// Per-gradient metadata (stored in a storage buffer, indexed by instance)
#[repr(C)]
pub struct GradientParams {
    pub start: [f32; 2],          // gradient start point (normalized 0-1 within rect)
    pub end: [f32; 2],            // gradient end point (normalized 0-1 within rect)
    pub atlas_row: f32,           // row in gradient atlas texture (normalized v coord)
    pub gradient_type: u32,       // 0=linear, 1=radial, 2=angular, 3=diamond
    pub _padding: [f32; 2],
}
```

**Shader** (`gradient.wgsl`):
```wgsl
@group(1) @binding(0)
var gradient_atlas: texture_2d<f32>;
@group(1) @binding(1)
var gradient_sampler: sampler;
@group(1) @binding(2)
var<storage, read> gradient_params: array<GradientParams>;

fn sample_gradient(uv: vec2<f32>, params: GradientParams) -> vec4<f32> {
    // Compute t based on gradient type
    var t: f32;
    switch params.gradient_type {
        case 0u: { // Linear
            let dir = params.end - params.start;
            t = clamp(dot(uv - params.start, dir) / dot(dir, dir), 0.0, 1.0);
        }
        case 1u: { // Radial
            t = clamp(length(uv - params.start) / length(params.end - params.start), 0.0, 1.0);
        }
        case 2u: { // Angular
            let d = uv - params.start;
            t = (atan2(d.y, d.x) + 3.14159265) / (2.0 * 3.14159265);
        }
        case 3u, default: { // Diamond
            let d = abs(uv - params.start);
            let scale = abs(params.end - params.start);
            t = clamp((d.x / scale.x + d.y / scale.y), 0.0, 1.0);
        }
    }

    // Single texture sample from pre-rasterized LUT - fast and unlimited stops
    let atlas_uv = vec2<f32>(t, params.atlas_row);
    return textureSample(gradient_atlas, gradient_sampler, atlas_uv);
}
```

**Why LUT over in-shader array:**

| Factor | Array (8 stops) | LUT 1024px Rgba16Float |
|--------|-----------------|------------------------|
| Max stops | 8 | Unlimited |
| GPU cost/pixel | Loop + N comparisons | 1 texture sample |
| Color precision | Float32 (perfect) | Float16 (65,536 levels - imperceptible loss) |
| Hard stop sharpness | Perfect | ~1px softening on 1000px gradient (sub-pixel) |
| Memory | 256 bytes/gradient | 8KB/gradient (1024 x 8 bytes) |
| Complexity | Branch-heavy shader | Simple shader, CPU-side rasterization |

### 1.4 SDF Strokes

Strokes on SDF shapes are elegant: the stroke region is where `abs(sdf) < stroke_width/2`.

```wgsl
fn render_stroke(dist: f32, stroke_width: f32, stroke_align: f32) -> f32 {
    // stroke_align: 0=center, 1=inside, -1=outside
    var d = dist;
    if stroke_align > 0.0 {
        // Inside: shift SDF outward
        d = d + stroke_width * 0.5;
    } else if stroke_align < 0.0 {
        // Outside: shift SDF inward
        d = d - stroke_width * 0.5;
    }

    // Stroke is where abs(d) < half_width
    let half_w = stroke_width * 0.5;
    let stroke_alpha = 1.0 - smoothstep(half_w - 0.5, half_w + 0.5, abs(d));
    return stroke_alpha;
}
```

**For INSIDE/OUTSIDE strokes**, the quad size must be expanded by `stroke_width` (outside) or kept the same (inside). This adjustment happens on the CPU side when generating instances.

### 1.5 Drop Shadows (SDF-based)

Drop shadows are the shape's SDF, offset and blurred. For SDF shapes, blur approximation via smoothstep width works well for small blur radii.

**Approach**: Render shadow as a separate instance behind the main shape.

```rust
// CPU side: generate shadow instance
fn create_shadow_instance(
    node: &SceneNode,
    shadow: &DropShadow,
) -> PrimitiveInstance {
    let expand = shadow.blur_radius + shadow.spread;
    PrimitiveInstance {
        pos: [node.bounds.x + shadow.offset.x - expand,
              node.bounds.y + shadow.offset.y - expand],
        size: [node.bounds.width + expand * 2.0,
               node.bounds.height + expand * 2.0],
        color: shadow.color.to_array(),
        corner_radii: node.corner_radii_expanded(expand),
        // ... shadow-specific flags
    }
}
```

**Shader**: The shadow fragment shader uses a wider smoothstep for the blur:

```wgsl
fn shadow_alpha(dist: f32, blur_radius: f32) -> f32 {
    // Approximate gaussian blur with smoothstep
    // This is accurate for blur_radius < ~20px
    // For larger blurs, use the multi-pass blur pipeline (Phase 4)
    return 1.0 - smoothstep(-blur_radius, blur_radius, dist);
}
```

**For high-quality large blur radii** (>20px), Phase 4's multi-pass Gaussian blur is used instead. The SDF approach is a fast path for the common case of subtle shadows (2-8px blur).

### 1.6 Phase 1 Deliverables

| Deliverable | Description |
|-------------|-------------|
| `PrimitiveInstance` struct | Extended GPU instance with per-corner radii, stroke, glyph tex_coords, flags |
| `primitive.wgsl` shader | Per-corner SDF, stroke, gradient sampling, glyph atlas sampling (unified) |
| `GradientAtlas` | 1024px Rgba16Float texture LUT for unlimited gradient stops |
| `GradientParams` buffer | Storage buffer for per-gradient parameters |
| `PrimitivePipeline` | Replaces both `RectPipeline` and `GlyphPipeline` |
| Unified glyph rendering | Text glyphs as `PrimitiveInstance` with `is_glyph` flag |
| Drop shadow fast path | SDF-based shadow for blur < 20px |
| Corner smoothing | Squircle SDF variant (superellipse blend) |
| Diamond gradient | SDF-based diamond gradient in shader |
| Unit tests | Per-corner SDF, gradient interpolation, glyph rendering |
| Benchmarks | Regression tests vs current pipelines |
| Visual test | Example showing all new primitives including gradient text |

### 1.7 RectInstance and GlyphPipeline Removal

`RectInstance`, `GlyphInstance`, `RectPipeline`, and `GlyphPipeline` are all **deleted**. `PrimitiveInstance` is the only GPU instance type. All call sites are updated:

- `WgpuBackend::collect_instances()` → produces `Vec<PrimitiveInstance>` (shapes AND glyphs unified)
- `collect_renderables_system` in `arthropod-ecs` → produces `PrimitiveInstance`
- `create_rect_instance()` helper → replaced by `create_primitive_instance()`
- `RenderCommands` resource → `Vec<PrimitiveInstance>`
- `RectPipeline` → deleted, replaced by `PrimitivePipeline`
- `GlyphPipeline` → deleted, replaced by `PrimitivePipeline`
- `glyph.wgsl` → deleted, functionality merged into `primitive.wgsl`
- `rect.wgsl` → deleted, functionality merged into `primitive.wgsl`
- Text shaping (cosmic-text) still runs on CPU; shaped glyphs are emitted as glyph-flagged `PrimitiveInstance`s

---

## Phase 2: Figma-Compatible Data Model (~2 weeks)

**Goal**: Create `style-engine` crate with types that map 1:1 to Figma's visual properties.

### 2.1 Core Types

```rust
// crates/style-engine/src/paint.rs

/// A paint describes how a fill or stroke is colored.
/// Maps to Figma's Paint type.
#[derive(Debug, Clone)]
pub enum Paint {
    /// Solid color fill.
    Solid {
        color: Color,
        opacity: f32,
        blend_mode: BlendMode,
    },
    /// Linear gradient between two points.
    LinearGradient {
        start: Vec2,
        end: Vec2,
        stops: Vec<ColorStop>,
        opacity: f32,
        blend_mode: BlendMode,
    },
    /// Radial gradient from center outward.
    RadialGradient {
        center: Vec2,
        radius: Vec2, // x and y radii (elliptical)
        stops: Vec<ColorStop>,
        opacity: f32,
        blend_mode: BlendMode,
    },
    /// Angular/conic gradient around a point.
    AngularGradient {
        center: Vec2,
        stops: Vec<ColorStop>,
        opacity: f32,
        blend_mode: BlendMode,
    },
    /// Diamond-shaped gradient.
    DiamondGradient {
        center: Vec2,
        stops: Vec<ColorStop>,
        opacity: f32,
        blend_mode: BlendMode,
    },
    /// Image fill (Phase 5+).
    Image {
        image_id: ImageId,
        scale_mode: ImageScaleMode,
        transform: Affine2,
        opacity: f32,
        blend_mode: BlendMode,
    },
}

/// A color stop in a gradient.
#[derive(Debug, Clone, Copy)]
pub struct ColorStop {
    pub position: f32,  // 0.0 to 1.0
    pub color: Color,
}
```

```rust
// crates/style-engine/src/stroke.rs

/// Stroke styling properties.
/// Maps to Figma's stroke properties.
#[derive(Debug, Clone)]
pub struct StrokeStyle {
    /// Paints applied to the stroke (bottom-to-top).
    pub paints: Vec<Paint>,
    /// Stroke weight in pixels.
    pub weight: f32,
    /// Per-side stroke weights (overrides `weight` if set).
    pub individual_weights: Option<SideWeights>,
    /// Where the stroke is drawn relative to the path.
    pub align: StrokeAlign,
    /// Shape at open ends of paths.
    pub cap: StrokeCap,
    /// Shape at path segment joints.
    pub join: StrokeJoin,
    /// Dash pattern (alternating dash/gap lengths in pixels).
    pub dash_pattern: Vec<f32>,
    /// Angle threshold for miter-to-bevel fallback (degrees).
    pub miter_angle: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum StrokeAlign {
    Inside,
    #[default]
    Center,
    Outside,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum StrokeCap {
    #[default]
    Butt,
    Round,
    Square,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum StrokeJoin {
    #[default]
    Miter,
    Bevel,
    Round,
}

#[derive(Debug, Clone, Copy)]
pub struct SideWeights {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}
```

```rust
// crates/style-engine/src/effect.rs

/// Visual effects applied to a node.
/// Maps to Figma's Effect type.
#[derive(Debug, Clone)]
pub enum Effect {
    /// Shadow cast outside the shape.
    DropShadow {
        offset: Vec2,
        blur_radius: f32,
        spread: f32,
        color: Color,
        blend_mode: BlendMode,
        /// Whether shadow renders behind the node (Figma: showShadowBehindNode).
        show_behind_node: bool,
        visible: bool,
    },
    /// Shadow cast inside the shape.
    InnerShadow {
        offset: Vec2,
        blur_radius: f32,
        spread: f32,
        color: Color,
        blend_mode: BlendMode,
        visible: bool,
    },
    /// Blur applied to the node's content.
    LayerBlur {
        radius: f32,
        visible: bool,
    },
    /// Blur applied to content behind the node.
    BackgroundBlur {
        radius: f32,
        visible: bool,
    },
}
```

```rust
// crates/style-engine/src/corner.rs

/// Per-corner radii.
/// Maps to Figma's rectangleCornerRadii.
#[derive(Debug, Clone, Copy, Default)]
pub struct CornerRadii {
    pub top_left: f32,
    pub top_right: f32,
    pub bottom_right: f32,
    pub bottom_left: f32,
}

impl CornerRadii {
    /// Uniform radius on all corners.
    pub fn uniform(radius: f32) -> Self {
        Self {
            top_left: radius,
            top_right: radius,
            bottom_right: radius,
            bottom_left: radius,
        }
    }

    /// All zeros (sharp corners).
    pub const ZERO: Self = Self {
        top_left: 0.0,
        top_right: 0.0,
        bottom_right: 0.0,
        bottom_left: 0.0,
    };

    /// Convert to GPU array [TL, TR, BR, BL].
    pub fn to_array(&self) -> [f32; 4] {
        [self.top_left, self.top_right, self.bottom_right, self.bottom_left]
    }

    /// Whether all corners are the same.
    pub fn is_uniform(&self) -> bool {
        self.top_left == self.top_right
            && self.top_right == self.bottom_right
            && self.bottom_right == self.bottom_left
    }

    /// Whether all corners are zero.
    pub fn is_zero(&self) -> bool {
        self.top_left == 0.0
            && self.top_right == 0.0
            && self.bottom_right == 0.0
            && self.bottom_left == 0.0
    }
}
```

```rust
// crates/style-engine/src/blend.rs

/// Blend mode for compositing layers.
/// Maps to Figma's BlendMode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BlendMode {
    // Normal
    #[default]
    Normal,
    PassThrough, // groups only

    // Darken
    Darken,
    Multiply,
    LinearBurn,
    ColorBurn,

    // Lighten
    Lighten,
    Screen,
    LinearDodge,
    ColorDodge,

    // Contrast
    Overlay,
    SoftLight,
    HardLight,

    // Inversion
    Difference,
    Exclusion,

    // Component
    Hue,
    Saturation,
    ColorMode, // "Color" in Figma (renamed to avoid conflict with Color type)
    Luminosity,
}
```

```rust
// crates/style-engine/src/path.rs

/// A vector path composed of SVG-style commands.
/// Maps to Figma's fillGeometry/strokeGeometry paths.
#[derive(Debug, Clone)]
pub struct VectorPath {
    pub commands: Vec<PathCommand>,
    pub winding_rule: WindingRule,
    pub closed: bool,
}

#[derive(Debug, Clone, Copy)]
pub enum PathCommand {
    MoveTo { x: f32, y: f32 },
    LineTo { x: f32, y: f32 },
    CubicTo { x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32 },
    QuadTo { x1: f32, y1: f32, x: f32, y: f32 },
    Close,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WindingRule {
    #[default]
    NonZero,
    EvenOdd,
}
```

### 2.2 The Unified `VisualStyle` Type

```rust
// crates/style-engine/src/visual.rs

/// Complete visual style for a scene node.
///
/// This is the central type that maps 1:1 to Figma's visual properties.
/// Every visual property a Figma node can have is representable here.
///
/// # Rendering Order
///
/// For a node with this style, the rendering order is:
/// 1. Drop shadows (bottom-to-top)
/// 2. Fills (bottom-to-top)
/// 3. Strokes (bottom-to-top)
/// 4. Inner shadows (bottom-to-top)
/// 5. Children
/// 6. Layer effects (blur, blend) applied to entire result
///
/// # Example
///
/// ```
/// use style_engine::*;
///
/// let card_style = VisualStyle::new()
///     .fill(Paint::Solid {
///         color: Color::WHITE,
///         opacity: 1.0,
///         blend_mode: BlendMode::Normal,
///     })
///     .corner_radii(CornerRadii::uniform(12.0))
///     .effect(Effect::DropShadow {
///         offset: Vec2::new(0.0, 4.0),
///         blur_radius: 12.0,
///         spread: 0.0,
///         color: Color::rgba(0.0, 0.0, 0.0, 0.15),
///         blend_mode: BlendMode::Normal,
///         show_behind_node: false,
///         visible: true,
///     })
///     .stroke(StrokeStyle {
///         paints: vec![Paint::Solid {
///             color: Color::rgba(0.0, 0.0, 0.0, 0.1),
///             opacity: 1.0,
///             blend_mode: BlendMode::Normal,
///         }],
///         weight: 1.0,
///         align: StrokeAlign::Inside,
///         ..Default::default()
///     });
/// ```
#[derive(Debug, Clone, Default)]
pub struct VisualStyle {
    /// Fill paints, rendered bottom-to-top.
    pub fills: Vec<Paint>,

    /// Stroke styling.
    pub stroke: Option<StrokeStyle>,

    /// Visual effects (shadows, blur), rendered in order.
    pub effects: Vec<Effect>,

    /// Corner radii (per-corner).
    pub corner_radii: CornerRadii,

    /// Corner smoothing (0.0 = circular, 0.6 = iOS squircle, 1.0 = full superellipse).
    pub corner_smoothing: f32,

    /// Node-level opacity (multiplied with paint/color opacity).
    pub opacity: f32,

    /// Blend mode for compositing this node.
    pub blend_mode: BlendMode,

    /// Whether children are clipped to this node's bounds.
    pub clips_content: bool,

    /// Vector paths for custom shapes (None = rectangular bounds).
    pub fill_geometry: Option<Vec<VectorPath>>,
    pub stroke_geometry: Option<Vec<VectorPath>>,

    /// Text content (if this node contains text).
    /// When set, glyphs are shaped and emitted as glyph-flagged PrimitiveInstances.
    /// Fills apply to the text (enabling gradient text, etc.).
    pub text: Option<TextContent>,
}

/// Text content for a styled node.
/// Maps to Figma's TypeStyle properties.
#[derive(Debug, Clone)]
pub struct TextContent {
    pub text: String,
    pub font_size: f32,
    pub font_family: Option<String>,
    pub font_weight: u16,       // 100-900 (400 = normal, 700 = bold)
    pub font_style: FontStyle,
    pub text_align: TextAlign,
    pub line_height: LineHeight,
    pub letter_spacing: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FontStyle { #[default] Normal, Italic }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TextAlign { #[default] Left, Center, Right, Justified }

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LineHeight {
    /// Automatic line height based on font metrics.
    Auto,
    /// Fixed line height in pixels.
    Pixels(f32),
    /// Line height as a percentage of font size (1.0 = 100%).
    Percent(f32),
}

impl Default for LineHeight {
    fn default() -> Self { Self::Auto }
}

impl TextContent {
    pub fn new(text: impl Into<String>, font_size: f32) -> Self {
        Self {
            text: text.into(),
            font_size,
            font_family: None,
            font_weight: 400,
            font_style: FontStyle::Normal,
            text_align: TextAlign::Left,
            line_height: LineHeight::Auto,
            letter_spacing: 0.0,
        }
    }

    pub fn bold(mut self) -> Self { self.font_weight = 700; self }
    pub fn italic(mut self) -> Self { self.font_style = FontStyle::Italic; self }
    pub fn weight(mut self, weight: u16) -> Self { self.font_weight = weight; self }
    pub fn align(mut self, align: TextAlign) -> Self { self.text_align = align; self }
    pub fn family(mut self, family: impl Into<String>) -> Self { self.font_family = Some(family.into()); self }
    pub fn line_height_px(mut self, px: f32) -> Self { self.line_height = LineHeight::Pixels(px); self }
    pub fn letter_spacing(mut self, spacing: f32) -> Self { self.letter_spacing = spacing; self }
}

impl VisualStyle {
    pub fn new() -> Self {
        Self {
            opacity: 1.0,
            ..Default::default()
        }
    }

    // Builder methods
    pub fn fill(mut self, paint: Paint) -> Self {
        self.fills.push(paint);
        self
    }

    pub fn solid_fill(self, color: Color) -> Self {
        self.fill(Paint::Solid {
            color,
            opacity: 1.0,
            blend_mode: BlendMode::Normal,
        })
    }

    pub fn linear_gradient(self, start: Vec2, end: Vec2, stops: Vec<ColorStop>) -> Self {
        self.fill(Paint::LinearGradient {
            start,
            end,
            stops,
            opacity: 1.0,
            blend_mode: BlendMode::Normal,
        })
    }

    pub fn stroke(mut self, style: StrokeStyle) -> Self {
        self.stroke = Some(style);
        self
    }

    pub fn effect(mut self, effect: Effect) -> Self {
        self.effects.push(effect);
        self
    }

    pub fn drop_shadow(self, offset: Vec2, blur: f32, color: Color) -> Self {
        self.effect(Effect::DropShadow {
            offset,
            blur_radius: blur,
            spread: 0.0,
            color,
            blend_mode: BlendMode::Normal,
            show_behind_node: false,
            visible: true,
        })
    }

    pub fn corner_radii(mut self, radii: CornerRadii) -> Self {
        self.corner_radii = radii;
        self
    }

    pub fn corner_radius(mut self, radius: f32) -> Self {
        self.corner_radii = CornerRadii::uniform(radius);
        self
    }

    pub fn opacity(mut self, opacity: f32) -> Self {
        self.opacity = opacity;
        self
    }

    pub fn clips(mut self, clips: bool) -> Self {
        self.clips_content = clips;
        self
    }

    pub fn text(mut self, content: TextContent) -> Self {
        self.text = Some(content);
        self
    }
}
```

### 2.3 NodeContent Replacement

`NodeContent::Rect`, `NodeContent::RoundedRect`, and `NodeContent::Text` are **all removed entirely**. All visual content - including text - uses `VisualStyle`.

```rust
// render-engine/src/node.rs (REPLACED)

pub enum NodeContent {
    /// Empty container (for grouping/layout).
    Empty,

    /// Styled node using the Figma-compatible visual system.
    /// Replaces the old Rect/RoundedRect/Text variants.
    /// Text is represented via VisualStyle.text field.
    Styled { style: VisualStyle },
}
```

All existing widgets, examples, and tests are updated in the same pass. No migration period - clean break.

```rust
// Before (deleted)
NodeContent::Rect { color: Color::RED }
NodeContent::RoundedRect { color: Color::BLUE, corner_radius: 8.0 }
NodeContent::Text { text: "Hello".into(), font_size: 16.0, color: Color::BLACK }

// After (only way)
NodeContent::Styled {
    style: VisualStyle::new().solid_fill(Color::RED),
}
NodeContent::Styled {
    style: VisualStyle::new()
        .solid_fill(Color::BLUE)
        .corner_radius(8.0),
}
NodeContent::Styled {
    style: VisualStyle::new()
        .solid_fill(Color::BLACK)
        .text(TextContent::new("Hello", 16.0)),
}
// Gradient text - the killer feature of unified pipeline:
NodeContent::Styled {
    style: VisualStyle::new()
        .linear_gradient(
            Vec2::ZERO, Vec2::X,
            vec![
                ColorStop { position: 0.0, color: Color::BLUE },
                ColorStop { position: 1.0, color: Color::RED },
            ],
        )
        .text(TextContent::new("Gradient Text!", 32.0)),
}
```

### 2.4 Widget API Integration

`WidgetContext::create_node()` is updated to accept `NodeContent` (which now uses `Styled`). A convenience method is added:

```rust
impl WidgetContext {
    /// Create a styled node. This is the primary way to create visual content.
    pub fn create_styled_node(
        &mut self,
        parent: NodeId,
        style: VisualStyle,
    ) -> NodeId {
        let node = SceneNode::new(NodeContent::Styled { style });
        self.scene.add_node(parent, node)
    }
}
```

**Example: Button with full visual styling:**

```rust
impl Button {
    pub fn build(&self, ctx: &mut WidgetContext) -> NodeId {
        let tokens = ctx.design_tokens();

        let style = VisualStyle::new()
            .solid_fill(tokens.accent)
            .corner_radius(tokens.radius_md)
            .drop_shadow(Vec2::new(0.0, 2.0), 4.0, Color::rgba(0.0, 0.0, 0.0, 0.15))
            .stroke(StrokeStyle::solid(
                Color::rgba(0.0, 0.0, 0.0, 0.08),
                1.0,
                StrokeAlign::Inside,
            ));

        let node_id = ctx.create_styled_node(ctx.root(), style);
        // ... build text child, set layout, etc.
        node_id
    }
}
```

**Example: Card with glassmorphism:**

```rust
let card_style = VisualStyle::new()
    .solid_fill(Color::rgba(1.0, 1.0, 1.0, 0.7))
    .corner_radius(16.0)
    .effect(Effect::BackgroundBlur { radius: 20.0, visible: true })
    .effect(Effect::DropShadow {
        offset: Vec2::new(0.0, 8.0),
        blur_radius: 24.0,
        spread: 0.0,
        color: Color::rgba(0.0, 0.0, 0.0, 0.12),
        blend_mode: BlendMode::Normal,
        show_behind_node: false,
        visible: true,
    })
    .stroke(StrokeStyle::solid(
        Color::rgba(1.0, 1.0, 1.0, 0.3),
        1.0,
        StrokeAlign::Inside,
    ))
    .clips(true);
```

### 2.5 Figma Import Compatibility

The `VisualStyle` is designed to be directly deserializable from Figma's REST API JSON. A future `figma-import` crate would:

```rust
// Future: crates/figma-import/src/lib.rs

pub fn figma_node_to_visual_style(node: &FigmaNode) -> VisualStyle {
    VisualStyle {
        fills: node.fills.iter().map(figma_paint_to_paint).collect(),
        stroke: convert_stroke(&node.strokes, node.stroke_weight, ...),
        effects: node.effects.iter().map(figma_effect_to_effect).collect(),
        corner_radii: match &node.rectangle_corner_radii {
            Some(r) => CornerRadii { top_left: r[0], top_right: r[1], ... },
            None => CornerRadii::uniform(node.corner_radius),
        },
        opacity: node.opacity,
        blend_mode: figma_blend_to_blend(node.blend_mode),
        ..Default::default()
    }
}
```

### 2.6 Phase 2 Deliverables

| Deliverable | Description |
|-------------|-------------|
| `style-engine` crate | Pure Rust types, no GPU dependency |
| `VisualStyle` unified type | Builder API with all Figma properties |
| `TextContent` | Font size, family, weight, style, alignment, line height, letter spacing |
| `Paint` enum | Solid, Linear, Radial, Angular, Diamond, Image |
| `StrokeStyle` | Weight, align, cap, join, dash pattern |
| `Effect` enum | DropShadow, InnerShadow, LayerBlur, BackgroundBlur |
| `CornerRadii` | Per-corner radius with helpers |
| `BlendMode` | All 19 Figma blend modes |
| `VectorPath` | SVG-style path commands |
| `NodeContent` reduced to `Empty` + `Styled` | `Text` variant removed; text via `VisualStyle.text` |
| Serde support | JSON serialization matching Figma's format |
| Comprehensive tests | Every type, every builder method |
| Figma mapping doc | Property-by-property mapping reference |

---

## Phase 3: Tessellated Path Pipeline (~5 weeks)

**Goal**: Render arbitrary vector paths using CPU tessellation + GPU triangle rendering. This enables custom shapes, icons, SVG content, and editor-class vector workloads.

### 3.1 Lyon Integration

[Lyon](https://github.com/nical/lyon) is a mature, pure-Rust path tessellation library. It's:
- Well-tested and widely used
- Compiles to WASM
- Handles fill and stroke tessellation
- Supports all winding rules

```toml
# style-engine/Cargo.toml (or render-engine)
[dependencies]
lyon = "1.0"
```

### 3.2 Path Pipeline Architecture

```
VectorPath (style-engine)
    │
    ↓  convert
lyon::path::Path
    │
    ↓  tessellate (CPU)
VertexBuffers<PathVertex, u32>
    │
    ↓  upload to GPU
Vertex Buffer + Index Buffer
    │
    ↓  render
path.wgsl fragment shader (color/gradient)
```

**`PathVertex`**:
```rust
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct PathVertex {
    pub position: [f32; 2],  // Tessellated vertex position
    pub normal: [f32; 2],    // Vertex normal (for anti-aliasing)
}
```

### 3.3 Tessellation Caching

For editor-class workloads, tessellation must be cached:

```rust
pub struct TessellationCache {
    /// Cached tessellation results, keyed by path hash.
    cache: HashMap<u64, CachedMesh>,
    /// LRU tracking for eviction.
    lru: VecDeque<u64>,
    /// Max cache size in vertices.
    max_vertices: usize,
    current_vertices: usize,
}

pub struct CachedMesh {
    pub vertices: Vec<PathVertex>,
    pub indices: Vec<u32>,
    /// Hash of the path that generated this mesh.
    pub path_hash: u64,
    /// Whether this mesh needs re-upload to GPU.
    pub dirty: bool,
}

impl TessellationCache {
    /// Get or tessellate a path.
    pub fn get_or_tessellate(
        &mut self,
        path: &VectorPath,
        stroke: Option<&StrokeStyle>,
    ) -> &CachedMesh {
        let hash = path.content_hash();
        if !self.cache.contains_key(&hash) {
            let mesh = tessellate(path, stroke);
            self.insert(hash, mesh);
        }
        self.touch(hash);
        &self.cache[&hash]
    }
}
```

### 3.4 Fill and Stroke Tessellation

```rust
fn tessellate_fill(path: &VectorPath) -> (Vec<PathVertex>, Vec<u32>) {
    let mut builder = lyon::path::Path::builder();

    for cmd in &path.commands {
        match cmd {
            PathCommand::MoveTo { x, y } => builder.begin(point(*x, *y)),
            PathCommand::LineTo { x, y } => builder.line_to(point(*x, *y)),
            PathCommand::CubicTo { x1, y1, x2, y2, x, y } =>
                builder.cubic_bezier_to(point(*x1, *y1), point(*x2, *y2), point(*x, *y)),
            PathCommand::QuadTo { x1, y1, x, y } =>
                builder.quadratic_bezier_to(point(*x1, *y1), point(*x, *y)),
            PathCommand::Close => builder.close(),
        }
    }

    let lyon_path = builder.build();
    let mut buffers: VertexBuffers<PathVertex, u32> = VertexBuffers::new();
    let mut tessellator = FillTessellator::new();

    let options = FillOptions::tolerance(0.1)
        .with_fill_rule(match path.winding_rule {
            WindingRule::NonZero => lyon::tessellation::FillRule::NonZero,
            WindingRule::EvenOdd => lyon::tessellation::FillRule::EvenOdd,
        });

    tessellator.tessellate_path(
        &lyon_path,
        &options,
        &mut BuffersBuilder::new(&mut buffers, |vertex: FillVertex| {
            PathVertex {
                position: vertex.position().to_array(),
                normal: [0.0, 0.0],
            }
        }),
    ).unwrap();

    (buffers.vertices, buffers.indices)
}

fn tessellate_stroke(path: &VectorPath, style: &StrokeStyle) -> (Vec<PathVertex>, Vec<u32>) {
    // Similar to fill, but uses StrokeTessellator with width/cap/join from StrokeStyle
    let mut tessellator = StrokeTessellator::new();
    let options = StrokeOptions::tolerance(0.1)
        .with_line_width(style.weight)
        .with_line_cap(match style.cap {
            StrokeCap::Butt => lyon::tessellation::LineCap::Butt,
            StrokeCap::Round => lyon::tessellation::LineCap::Round,
            StrokeCap::Square => lyon::tessellation::LineCap::Square,
        })
        .with_line_join(match style.join {
            StrokeJoin::Miter => lyon::tessellation::LineJoin::Miter,
            StrokeJoin::Bevel => lyon::tessellation::LineJoin::Bevel,
            StrokeJoin::Round => lyon::tessellation::LineJoin::Round,
        });

    // ... tessellate and return buffers
}
```

### 3.5 Path Pipeline GPU Side

```rust
pub struct PathPipeline {
    pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    // Per-path instance data (transform, color/gradient reference)
    instance_buffer: wgpu::Buffer,
}
```

**Shader** (`path.wgsl`):
```wgsl
struct PathVertexInput {
    @location(0) position: vec2<f32>,
    @location(1) normal: vec2<f32>,
    // Per-instance:
    @location(2) transform_col0: vec2<f32>,
    @location(3) transform_col1: vec2<f32>,
    @location(4) translation: vec2<f32>,
    @location(5) color: vec4<f32>,
}

@vertex
fn vs_main(input: PathVertexInput) -> VertexOutput {
    // Apply per-path transform
    let local = mat2x2<f32>(
        input.transform_col0,
        input.transform_col1,
    ) * input.position + input.translation;

    var out: VertexOutput;
    out.position = globals.transform * vec4<f32>(local, 0.0, 1.0);
    out.color = input.color;
    return out;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    return input.color;
}
```

### 3.6 Boolean Operations

For path boolean operations (union, subtract, intersect, exclude), we use Lyon's built-in support or integrate a dedicated boolean ops crate:

```rust
pub fn boolean_op(a: &VectorPath, b: &VectorPath, op: BooleanOp) -> VectorPath {
    // Convert to lyon paths, perform operation, convert back
    match op {
        BooleanOp::Union => lyon_algorithms::boolean::union(&a_lyon, &b_lyon),
        BooleanOp::Subtract => lyon_algorithms::boolean::difference(&a_lyon, &b_lyon),
        BooleanOp::Intersect => lyon_algorithms::boolean::intersection(&a_lyon, &b_lyon),
        BooleanOp::Exclude => lyon_algorithms::boolean::xor(&a_lyon, &b_lyon),
    }
}
```

### 3.7 Hit Testing for Paths

For interactive path selection (editor use case):

```rust
impl VectorPath {
    /// Test if a point is inside this path.
    pub fn contains_point(&self, x: f32, y: f32) -> bool {
        // Use winding number algorithm on the path commands
        // This is O(n) where n = number of path segments
        let point = Vec2::new(x, y);
        winding_number(&self.commands, point, self.winding_rule) != 0
    }
}
```

### 3.8 Phase 3 Deliverables

| Deliverable | Description |
|-------------|-------------|
| Lyon integration | Path tessellation for fills and strokes |
| `PathPipeline` | GPU pipeline for tessellated triangles |
| `TessellationCache` | LRU cache for tessellated meshes |
| `path.wgsl` shader | Per-path transforms, color/gradient |
| Boolean operations | Union, subtract, intersect, exclude |
| Path hit testing | Point-in-path for interactive selection |
| SVG path parsing | Parse SVG `d` attribute strings |
| Benchmarks | Tessellation time, cache hit rate |

---

## Phase 4: Multi-Pass Effects (~3 weeks)

**Goal**: Implement render-to-texture effects (layer blur, background blur, inner shadows, clipping/masking, advanced blend modes) without compute shaders so the same design runs on native wgpu and WASM WebGL2 fallback.

### 4.1 Effect Classification and Render Plan

Phase 4 introduces a per-frame **effect plan** generated from scene nodes after z-sort. This avoids ad hoc branching in draw code and gives deterministic ordering.

```rust
pub enum EffectPassKind {
    DirectPrimitive,       // no offscreen pass needed
    OffscreenLayer,        // render node subtree into target
    BackgroundCapture,     // copy backdrop region for background blur
    BlurHorizontal,
    BlurVertical,
    InnerShadow,
    BlendComposite,
    StencilPush,
    StencilPop,
}

pub struct EffectPass {
    pub node_id: NodeId,
    pub kind: EffectPassKind,
    pub target: Option<RenderTargetHandle>,
    pub bounds_px: UVec4, // [x, y, width, height]
    pub blend_mode: BlendMode,
}
```

**Planning rules**:
- `BlendMode::Normal` + no heavy effects -> `DirectPrimitive`
- `LayerBlur` -> `OffscreenLayer` + `BlurHorizontal` + `BlurVertical` + composite
- `BackgroundBlur` -> `BackgroundCapture` + blur passes + masked composite
- `InnerShadow` -> offscreen mask + `InnerShadow` pass
- `clips_content` and masks -> `StencilPush` / `StencilPop` around children

### 4.2 Render Target Infrastructure and Pooling

Offscreen rendering must be pooled to avoid per-frame texture churn.

```rust
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub struct RenderTargetKey {
    pub width: u32,
    pub height: u32,
    pub format: wgpu::TextureFormat,
    pub has_stencil: bool,
}

pub struct RenderTargetPool {
    free: HashMap<RenderTargetKey, Vec<RenderTarget>>,
    in_use: Vec<RenderTarget>,
    bytes_in_use: u64,
    soft_budget_bytes: u64, // desktop default: 256 MB, web default: 96 MB
}

impl RenderTargetPool {
    pub fn acquire(&mut self, ctx: &WgpuContext, key: RenderTargetKey) -> RenderTarget;
    pub fn release(&mut self, target: RenderTarget);
    pub fn end_frame(&mut self);
}
```

**File changes**:
- `crates/render-engine/src/backend/wgpu/context.rs` (render target creation helpers)
- `crates/render-engine/src/backend/wgpu/mod.rs` (pool owned by backend)
- `crates/render-engine/src/backend/wgpu/render_target_pool.rs` (new)

### 4.3 Gaussian Blur Pipeline (Separable + Radius Tiers)

Blur stays two-pass separable, but uses two quality tiers:
- **Tier A** (`radius <= 24`): full-resolution two-pass blur
- **Tier B** (`radius > 24`): downsample to half-res, blur, then upsample (large perf win)

```wgsl
const MAX_TAPS: u32 = 25u;

struct BlurParams {
    direction: vec2<f32>,     // (1,0) horizontal, (0,1) vertical
    texel_size: vec2<f32>,    // 1.0 / target_size
    tap_count: u32,
    _pad: u32,
    weights: array<f32, 25>,  // symmetric kernel precomputed on CPU
}

@group(1) @binding(0) var source_texture: texture_2d<f32>;
@group(1) @binding(1) var source_sampler: sampler;
@group(1) @binding(2) var<uniform> blur: BlurParams;
```

**Execution**:
1. Render source into `A`.
2. Pass 1 (`BlurHorizontal`): `A -> B`.
3. Pass 2 (`BlurVertical`): `B -> A`.
4. Composite `A` into destination using node opacity/blend.

### 4.4 Layer Blur

`Effect::LayerBlur` blurs the node's own pixels (and children), not the backdrop.

```
1. Render node subtree into offscreen A
2. Blur A -> B -> A
3. Composite A back into parent target
```

Important details:
- Blur bounds are inflated by `ceil(radius * 2.0)` to prevent edge clipping.
- Clear color is transparent black to avoid halo artifacts.
- Layer blur participates in clip stack before compositing.

### 4.5 Background Blur (Backdrop Filter)

`Effect::BackgroundBlur` samples pixels *already rendered behind* the node:

```
1. Resolve/copy current destination into backdrop texture
2. Crop node bounds (inflated by blur radius) into A
3. Blur A -> B -> A
4. Composite blurred result through node shape mask
5. Render node fills/strokes/text on top
```

This requires an intermediate color target for the frame; swapchain textures cannot be sampled directly in all backends.

### 4.6 Inner Shadows

Inner shadows are implemented as masked blur:

```
1. Render node alpha mask (shape coverage) into A
2. Offset mask by shadow offset into B
3. Blur B -> C -> B
4. Subtract original mask from blurred mask
5. Multiply by shadow color and composite inside shape only
```

```wgsl
fn inner_shadow_alpha(mask: f32, blurred_offset_mask: f32) -> f32 {
    // Only keep blur that falls inside the original shape.
    return clamp(blurred_offset_mask - (1.0 - mask), 0.0, 1.0) * mask;
}
```

### 4.7 Stencil Clipping and Mask Stack

`clips_content: true` and mask nodes use an explicit stencil stack:

```rust
pub struct ClipStack {
    depth: u8, // 0..255 stencil levels
}
```

**Protocol**:
1. `StencilPush`: draw clip geometry writing `depth + 1`.
2. Render child passes with stencil compare `Equal(depth + 1)`.
3. Nested clips increment depth.
4. `StencilPop`: decrement depth after children.

This supports nested frame clipping and mask groups with deterministic behavior.

### 4.8 Blend Mode Compositing

Blend modes are implemented in `blend.wgsl` with two categories:
- **Separable modes**: multiply, screen, overlay, darken, lighten, dodge, burn, hard/soft light, difference, exclusion
- **Non-separable modes**: hue, saturation, color, luminosity (HSL conversion path)

```wgsl
fn blend_multiply(src: vec3<f32>, dst: vec3<f32>) -> vec3<f32> {
    return src * dst;
}

fn blend_screen(src: vec3<f32>, dst: vec3<f32>) -> vec3<f32> {
    return src + dst - src * dst;
}
```

Compositing rule:
- `BlendMode::Normal` -> regular alpha pipeline (no offscreen)
- all other modes -> source rendered to offscreen, then blended against destination sample

### 4.9 Phase 4 Verification (Tests + Benchmarks)

**Unit/integration tests**:
- `test_render_target_pool_reuses_same_key`
- `test_blur_kernel_weights_sum_to_one`
- `test_background_blur_respects_clip_bounds`
- `test_inner_shadow_only_inside_shape`
- `test_nested_clip_stack_depth`
- `test_blend_mode_multiply_matches_reference`

**Visual tests**:
```bash
cargo run --example visual_test_phase4_blur
cargo run --example visual_test_phase4_blend
cargo run --example visual_test_phase4_clipping
```

**Benchmarks**:
```bash
cargo bench -p render-engine blur_pass_1080p
cargo bench -p render-engine background_blur_500_nodes
cargo bench -p render-engine blend_composite_1000_layers
```

Exit gates:
- 1080p 2-pass blur <= 2.0 ms
- blend composite <= 1.0 ms for 1,000 layers
- no >10% regression in existing primitive/path throughput benches

### 4.10 Phase 4 Deliverables

| Deliverable | Description |
|-------------|-------------|
| Effect planner | Deterministic per-frame `EffectPass` plan |
| Render target pool | Reuse offscreen textures with memory budget |
| `BlurPipeline` | Separable blur with large-radius tiering |
| Layer blur | Node-local blur compositing |
| Background blur | Backdrop capture + masked blur |
| Inner shadow pass | Offset+blur+mask implementation |
| Stencil clip stack | Nested clip/mask behavior |
| `BlendPipeline` | All 19 Figma blend modes |
| Visual tests | Blur/blend/clip regression examples |
| Performance suite | Phase 4 criterion benchmarks + gates |

---

## Phase 5: WASM Target (~3 weeks)

**Goal**: Ship Arthropod in browsers with WebAssembly + WebGPU, with automatic fallback to WebGL2 where required.

### 5.1 Target Constraints and Compatibility

Phase 5 must preserve one renderer architecture:
- same `VisualStyle` data model
- same `PrimitivePipeline`, `PathPipeline`, and Phase 4 effect passes
- no compute-only features
- predictable behavior across native + web

Current readiness snapshot:

| Component | WASM Ready? | Notes |
|-----------|-------------|-------|
| `style-engine` | Yes | Pure Rust, no platform deps |
| `flux-state` | Yes | Pure Rust signals |
| `render-engine` | Mostly | Requires web surface init and runtime capability checks |
| `plat-core` | No | Needs `web` platform backend |
| `text` stack | Mostly | Font loading and caching path must be web-safe |
| tessellation (`lyon`) | Yes | Pure Rust |

### 5.2 Cargo Features and Target-Specific Dependencies

Use explicit feature gates for native and web backends:

```toml
[features]
default = ["native"]
native = ["plat-core/native", "render-engine/native"]
web = ["plat-core/web", "render-engine/web"]
```

`plat-core` adds web-only dependencies:

```toml
[target.'cfg(target_arch = "wasm32")'.dependencies]
wasm-bindgen = "0.2"
web-sys = { version = "0.3", features = [
  "Window",
  "Document",
  "HtmlCanvasElement",
  "Performance",
  "MouseEvent",
  "PointerEvent",
  "WheelEvent",
  "KeyboardEvent",
] }
```

### 5.3 Web Platform Backend (`plat-core`)

Add `crates/plat-core/src/platform/web.rs` and register it in `platform/mod.rs`.

```rust
#[cfg(target_arch = "wasm32")]
pub struct WebPlatform {
    canvas: web_sys::HtmlCanvasElement,
    window: web_sys::Window,
    scale_factor: f64,
    event_queue: Vec<PlatformEvent>,
}
```

Responsibilities:
- attach to existing `<canvas>` or create one
- map DOM events to `PlatformEvent`
- forward resize + DPR updates
- request animation frames for redraw
- expose surface handle for wgpu/winit initialization

### 5.4 Bootstrapping and Canvas Integration

Create a web entrypoint for the gallery:
- `examples/widget_gallery_web.rs` (new, `wasm_bindgen(start)`)
- optional `examples/web/index.html` and `examples/web/trunk.toml`

```rust
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub async fn start() -> Result<(), JsValue> {
    let app = WidgetGalleryApp::new_web("arthropod-canvas").await?;
    app.run();
    Ok(())
}
```

### 5.5 Event and Input Mapping

Browser events map into existing framework input types:

| Browser Event | Arthropod Event |
|---------------|-----------------|
| `pointerdown/move/up` | `PointerDown/Move/Up` |
| `wheel` | `MouseWheel` (pixel/line normalized) |
| `keydown/keyup` | `KeyDown/KeyUp` |
| `resize` | `WindowResized` |
| `blur/focus` | `WindowBlur/Focus` |

Requirements:
- preserve pointer capture semantics
- normalize wheel delta modes
- keep text input IME-safe (composition events staged for later phase if needed)

### 5.6 Font and Asset Loading on Web

Filesystem-based loading is replaced with async HTTP fetch:

```rust
pub enum FontSource {
    Bytes(Vec<u8>),       // native/tests
    Url(String),          // web runtime
}
```

Plan:
1. `fetch()` font bytes via `web_sys`.
2. Cache bytes in-memory (hash by URL + etag/version).
3. Hand bytes to text shaping/atlas as today.
4. Preload default UI font before first frame to avoid layout jumps.

### 5.7 WebGPU + WebGL2 Runtime Path

Renderer startup probes backend capabilities:

```text
try WebGPU adapter -> if unavailable, initialize wgpu WebGL2 backend
```

Compatibility rules:
- no storage textures required
- no compute passes
- conservative uniform/storage buffer sizes
- shader code validated on both backends in CI

| Feature | WebGPU | WebGL2 |
|---------|--------|--------|
| Primitive pipeline | Yes | Yes |
| Gradient LUT sampling | Yes | Yes |
| Path rendering | Yes | Yes |
| Blur / blend / stencil effects | Yes | Yes |
| Compute shaders | Yes | No (unused) |

### 5.8 Build, Tooling, and CI

Local workflows:

```bash
rustup target add wasm32-unknown-unknown
cargo check --target wasm32-unknown-unknown --features web
wasm-pack build --target web --features web
trunk serve --features web
```

CI additions:
- `cargo check --target wasm32-unknown-unknown --features web`
- `cargo test -p style-engine --target wasm32-unknown-unknown` (where supported)
- browser smoke test for `widget_gallery` load + first frame render

### 5.9 Phase 5 Verification (Tests + Performance)

**Tests**:
- `test_web_canvas_bootstrap`
- `test_web_pointer_event_mapping`
- `test_font_fetch_and_cache`
- `test_web_resize_updates_surface`
- `test_webgl2_fallback_initializes`

**Smoke benchmarks**:
- time-to-first-frame (TTFF) for widget gallery
- steady-state frame time with 1,000 styled nodes
- wasm bundle size tracking per commit

Exit gates:
- gallery loads and renders on Chrome, Firefox, Safari
- fallback path works on browsers without WebGPU
- no correctness differences in visual regression set vs native

### 5.10 Phase 5 Deliverables

| Deliverable | Description |
|-------------|-------------|
| `plat-core` web backend | Canvas lifecycle + event bridge |
| Web entrypoint | `wasm_bindgen(start)` example app |
| Font/asset loader | Async HTTP fetch + caching |
| WASM feature gating | Native/web split in workspace features |
| Runtime backend probe | WebGPU primary, WebGL2 fallback |
| Web CI checks | wasm check + browser smoke test |
| Web visual regression suite | Compare output vs native references |
| Browser-ready widget gallery | Public demo target for ongoing work |

---
## Figma Property -> Arthropod Mapping Reference

| Figma Property | Arthropod Type | Phase |
|---------------|----------------|-------|
| `fills: Paint[]` | `VisualStyle.fills: Vec<Paint>` | 1-2 |
| `fills[].type: SOLID` | `Paint::Solid { color, opacity }` | 1 |
| `fills[].type: GRADIENT_LINEAR` | `Paint::LinearGradient { ... }` | 1 |
| `fills[].type: GRADIENT_RADIAL` | `Paint::RadialGradient { ... }` | 1 |
| `fills[].type: GRADIENT_ANGULAR` | `Paint::AngularGradient { ... }` | 2 |
| `fills[].type: GRADIENT_DIAMOND` | `Paint::DiamondGradient { ... }` | 2 |
| `fills[].type: IMAGE` | `Paint::Image { ... }` | 5+ |
| `fills[].imageTransform` | `Paint::Image.transform: Option<[f32; 9]>` | 5+ |
| `strokes: Paint[]` | `StrokeStyle.paints: Vec<Paint>` | 1-2 |
| `strokeWeight` | `StrokeStyle.weight` | 1 |
| `strokeAlign` | `StrokeStyle.align: StrokeAlign` | 1 |
| `strokeCap` | `StrokeStyle.cap: StrokeCap` | 3 |
| `strokeJoin` | `StrokeStyle.join: StrokeJoin` | 3 |
| `strokeDashes` | `StrokeStyle.dash_pattern: Vec<f32>` | 3 |
| `dashOffset` | `StrokeStyle.dash_offset: f32` | 3 |
| `individualStrokeWeights` | `StrokeStyle.individual_weights` | 2 |
| `effects[].DROP_SHADOW` | `Effect::DropShadow { ... }` | 1 (fast) / 4 (full) |
| `effects[].INNER_SHADOW` | `Effect::InnerShadow { ... }` | 4 |
| `effects[].LAYER_BLUR` | `Effect::LayerBlur { ... }` | 4 |
| `effects[].BACKGROUND_BLUR` | `Effect::BackgroundBlur { ... }` | 4 |
| `cornerRadius` | `CornerRadii::uniform(r)` | 1 |
| `rectangleCornerRadii` | `CornerRadii { tl, tr, br, bl }` | 1 |
| `cornerSmoothing` | `VisualStyle.corner_smoothing` | 2+ |
| `opacity` | `VisualStyle.opacity` | Existing |
| `blendMode` | `VisualStyle.blend_mode: BlendMode` | 2 (type) / 4 (render) |
| `clipsContent` | `VisualStyle.clips_content` | 4 |
| `isMask` / `maskType` | Mask system (stencil-based) | 4 |
| `fillGeometry` | `VisualStyle.fill_geometry` | 3 |
| `strokeGeometry` | `VisualStyle.stroke_geometry` | 3 |

---

## Testing Strategy

### Unit Tests (per phase)

```rust
// Phase 1: SDF correctness
#[test]
fn test_per_corner_sdf_top_left_only() {
    // Verify SDF produces correct distance for asymmetric radii
}

#[test]
fn test_gradient_interpolation_2_stops() {
    // Verify linear interpolation between 2 color stops
}

#[test]
fn test_gradient_interpolation_multi_stop() {
    // Verify correct stop selection for N stops
}

// Phase 2: Data model
#[test]
fn test_visual_style_builder_fills() {
    let style = VisualStyle::new()
        .solid_fill(Color::RED)
        .solid_fill(Color::BLUE);
    assert_eq!(style.fills.len(), 2);
}

#[test]
fn test_visual_style_serde_roundtrip() {
    let style = VisualStyle::new().solid_fill(Color::RED);
    let json = serde_json::to_string(&style).unwrap();
    let back: VisualStyle = serde_json::from_str(&json).unwrap();
    // Verify equality
}

// Phase 3: Tessellation
#[test]
fn test_tessellate_triangle() {
    let path = VectorPath {
        commands: vec![
            PathCommand::MoveTo { x: 0.0, y: 0.0 },
            PathCommand::LineTo { x: 100.0, y: 0.0 },
            PathCommand::LineTo { x: 50.0, y: 100.0 },
            PathCommand::Close,
        ],
        winding_rule: WindingRule::NonZero,
        closed: true,
    };
    let (vertices, indices) = tessellate_fill(&path);
    assert!(!vertices.is_empty());
    assert!(!indices.is_empty());
    assert_eq!(indices.len() % 3, 0); // triangles
}

// Phase 4: Effects
#[test]
fn test_render_target_creation() {
    // Verify render target has correct dimensions and format
}
```

### Visual Regression Tests

Each phase includes a visual test example that renders all new primitives. These can be screenshot-compared:

```bash
cargo run --example visual_test_phase1  # Gradients, corners, strokes, shadows
cargo run --example visual_test_phase3  # Vector paths, tessellation
cargo run --example visual_test_phase4  # Blur, blend modes, clipping
```

### Benchmark Tests

```rust
// Phase 1: Instance throughput
#[bench]
fn bench_1000_gradient_primitives(b: &mut Bencher) {
    // Must stay under 1ms
}

// Phase 3: Tessellation
#[bench]
fn bench_tessellate_complex_path(b: &mut Bencher) {
    // Measure tessellation time for a 100-segment bezier path
}

#[bench]
fn bench_tessellation_cache_hit(b: &mut Bencher) {
    // Verify cache lookup is < 100ns
}

// Phase 4: Multi-pass
#[bench]
fn bench_blur_pass_1080p(b: &mut Bencher) {
    // Two-pass blur on 1920x1080 target
}
```

---

## Performance Budget

| Operation | Budget | Phase |
|-----------|--------|-------|
| 1,000 primitives (solid) | < 200 μs | 1 |
| 1,000 primitives (gradient) | < 400 μs | 1 |
| 1,000 primitives (stroked) | < 300 μs | 1 |
| Shadow SDF (per shadow) | < 5 μs | 1 |
| Tessellation (100 segments) | < 500 μs | 3 |
| Tessellation cache hit | < 100 ns | 3 |
| Blur pass (1080p) | < 2 ms | 4 |
| Blend mode composite | < 1 ms | 4 |
| Total frame (1000 styled nodes) | < 4 ms (24% of 60fps) | All |

**Target**: 60fps with 1,000 fully-styled nodes (fills + strokes + shadows + corners). This leaves 76% of the frame budget for application logic.

---

## Migration Plan

### Clean Break - No Backwards Compatibility

This is a **breaking change by design**. All old rendering types are removed:

| Removed | Replaced By |
|---------|-------------|
| `NodeContent::Rect` | `NodeContent::Styled { style }` |
| `NodeContent::RoundedRect` | `NodeContent::Styled { style }` |
| `NodeContent::Text` | `NodeContent::Styled { style }` with `style.text` |
| `RectInstance` | `PrimitiveInstance` |
| `GlyphInstance` | `PrimitiveInstance` with `is_glyph` flag |
| `RectPipeline` | `PrimitivePipeline` |
| `GlyphPipeline` | `PrimitivePipeline` |
| `rect.wgsl` | `primitive.wgsl` |
| `glyph.wgsl` | `primitive.wgsl` |
| `create_rect_instance()` | `create_primitive_instance()` |

### What Gets Updated

All of these are updated in a single pass during Phase 1+2:

**Widgets** (widget-core):
- `Button` → uses `VisualStyle` with fill + corner_radius + shadow + stroke
- `Card` → uses `VisualStyle` with surface fill + shadow
- `Divider` → uses `VisualStyle` with thin fill
- `Checkbox` → uses `VisualStyle` with border stroke + fill
- `TextInput` → uses `VisualStyle` with border + focus ring

**ECS Systems** (arthropod-ecs):
- `collect_renderables_system` → emits `PrimitiveInstance`
- `RenderCommands` → `Vec<PrimitiveInstance>`

**Render Backend** (render-engine):
- `WgpuBackend::collect_instances()` → converts `NodeContent::Styled` to `Vec<PrimitiveInstance>` (shapes + glyphs)
- `PrimitivePipeline` replaces both `RectPipeline` and `GlyphPipeline`
- Text shaping still uses cosmic-text; shaped glyphs emitted as glyph-flagged instances
- `GlyphInstance` removed; glyph data carried via `PrimitiveInstance.tex_coords` + `is_glyph` flag

**Examples** (all updated):
- Every example that creates `NodeContent::Rect` or `RoundedRect` is updated

**Tests** (all updated):
- Every test asserting on `RectInstance` is rewritten for `PrimitiveInstance`

### Execution Order

1. **Phase 1+2** (parallel): Build `style-engine` + `PrimitivePipeline`, update ALL call sites
2. **Phase 3**: `PathPipeline` added alongside `PrimitivePipeline` (additive)
3. **Phase 4**: Effects pipelines added (additive)
4. **Phase 5**: WASM target (additive)

---

## Timeline Summary

| Phase | Duration | Focus | Key Deliverable |
|-------|----------|-------|-----------------|
| **Phase 1** | 3 weeks | Extended SDF | Per-corner radii, gradients, strokes, shadows |
| **Phase 2** | 2 weeks | Data Model | `style-engine` crate, `VisualStyle`, Figma mapping |
| **Phase 3** | 5 weeks | Vector Paths | Lyon tessellation, caching, boolean ops, hit testing |
| **Phase 4** | 3 weeks | Effects | Blur, background blur, clipping, blend modes |
| **Phase 5** | 3 weeks | WASM | Web platform, WebGPU/WebGL2, browser deployment |
| **Total** | **~16 weeks** | | Complete Figma-compatible GPU rendering pipeline |

### Suggested Execution Order

Phases 1 and 2 can run in parallel (data model is renderer-agnostic):

```
Week 1-3:  Phase 1 (SDF) + Phase 2 (Data Model) [parallel]
Week 4-8:  Phase 3 (Paths)
Week 9-11: Phase 4 (Effects)
Week 12-14: Phase 5 (WASM)
Week 15-16: Polish, benchmarks, documentation
```

This brings the realistic timeline to **~14 weeks** with parallelization.

---

## Open Questions

1. ~~**Gradient stop limit**~~: **RESOLVED** - Use 1024px `Rgba16Float` texture LUT. Unlimited stops, faster than array, zero perceptible quality loss.

2. ~~**Diamond gradient**~~: **RESOLVED** - Ship in Phase 1. Shader already written above.

3. ~~**Corner smoothing**~~: **RESOLVED** - Ship in Phase 1. Superellipse SDF variant in primitive shader.

4. ~~**Image fills**~~: **RESOLVED** - Phase 5. Substantial feature (FILL/FIT/CROP/TILE), pairs with WASM work.

5. ~~**Text integration**~~: **RESOLVED** - Unified pipeline. Text glyphs render through `PrimitivePipeline` as glyph-flagged instances. Glyph alpha from atlas multiplied by fill color (solid or gradient). ~4μs overhead for 1000 glyphs. Enables gradient text without render-to-texture. `NodeContent::Text` removed; text is `VisualStyle.text: Option<TextContent>`.

6. **Render target pooling**: How aggressively should we pool/reuse offscreen render targets? This affects GPU memory vs allocation overhead.

---

## Success Criteria

1. Every Figma visual property has a documented Arthropod equivalent
2. A Figma design exported to JSON can be rendered in Arthropod
3. WASM build runs widget gallery in Chrome, Firefox, Safari
4. Performance: 60fps with 1,000 fully-styled nodes
5. All examples and tests updated and passing with new `VisualStyle` API
6. `NodeContent::Rect`, `RoundedRect`, and `Text` fully removed - zero legacy code
7. Single unified `PrimitivePipeline` handles all rendering (shapes + text)
8. Gradient text renders without render-to-texture compositing
9. Zero clippy warnings
10. Comprehensive benchmark suite for regression detection
