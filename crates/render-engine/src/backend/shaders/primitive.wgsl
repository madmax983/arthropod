// Unified primitive shader supporting rectangles, text, gradients, strokes, and effects
//
// This replaces both rect.wgsl and glyph.wgsl with a single shader that handles:
// - Solid fills and gradients (linear, radial, angular, diamond)
// - Per-corner rounded rectangles (asymmetric corner radii)
// - Strokes (center, inside, outside alignment)
// - Text rendering via glyph atlas
// - Effects (shadows, blur)

// ============================================================================
// Bind Groups
// ============================================================================

// Group 0: Global uniforms (shared with old shaders)
struct Globals {
    transform: mat4x4<f32>,  // Projection matrix for NDC conversion
}

@group(0) @binding(0)
var<uniform> globals: Globals;

// Group 1: Gradient atlas (TODO: Step 11)
// @group(1) @binding(0) var gradient_texture: texture_2d<f32>;
// @group(1) @binding(1) var gradient_sampler: sampler;

// Group 2: Glyph atlas (moved from old glyph shader)
@group(2) @binding(0)
var glyph_texture: texture_2d<f32>;

@group(2) @binding(1)
var glyph_sampler: sampler;

// ============================================================================
// Vertex Input/Output
// ============================================================================

// Per-instance vertex attributes (matches PrimitiveInstance: 96 bytes)
struct VertexInput {
    @builtin(vertex_index) vertex_index: u32,
    // Instance attributes (96 bytes total)
    @location(0) pos: vec2<f32>,           // 8 bytes: Position
    @location(1) size: vec2<f32>,          // 8 bytes: Size
    @location(2) color: vec4<f32>,         // 16 bytes: Primary color
    @location(3) corner_radii: vec4<f32>,  // 16 bytes: Corner radii (TL, TR, BR, BL)
    @location(4) gradient_params: vec4<f32>, // 16 bytes: Gradient params (TODO: Step 11)
    @location(5) tex_coords: vec4<f32>,    // 16 bytes: Texture coordinates (x0, y0, x1, y1)
    @location(6) stroke_params: vec2<f32>, // 8 bytes: Stroke (width, align)
    @location(7) flags: u32,               // 4 bytes: Bitflags
    @location(8) _padding: u32,            // 4 bytes: Padding
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) local_pos: vec2<f32>,     // Position within primitive (0..size)
    @location(2) prim_size: vec2<f32>,     // Primitive dimensions
    @location(3) corner_radii: vec4<f32>,  // Per-corner radii
    @location(4) tex_coord: vec2<f32>,     // Glyph texture coordinates
    @location(5) stroke_params: vec2<f32>, // Stroke parameters
    @location(6) flags: u32,               // Bitflags
}

// ============================================================================
// Utility Functions
// ============================================================================

// Generate quad vertex positions (0,0 to 1,1)
fn vertex_position(index: u32) -> vec2<f32> {
    // Two triangles forming a quad:
    // Triangle 1: (0,0), (1,0), (1,1)
    // Triangle 2: (0,0), (1,1), (0,1)
    switch index {
        case 0u: { return vec2<f32>(0.0, 0.0); }
        case 1u: { return vec2<f32>(1.0, 0.0); }
        case 2u: { return vec2<f32>(1.0, 1.0); }
        case 3u: { return vec2<f32>(0.0, 0.0); }
        case 4u: { return vec2<f32>(1.0, 1.0); }
        case 5u, default: { return vec2<f32>(0.0, 1.0); }
    }
}

// Per-corner rounded rectangle SDF
// radii = vec4(TL, TR, BR, BL)
fn rounded_rect_sdf_4(pos: vec2<f32>, half_size: vec2<f32>, radii: vec4<f32>) -> f32 {
    // Select corner radius based on quadrant
    var radius: f32;
    if pos.x > 0.0 && pos.y > 0.0 {
        radius = radii.z; // BR
    } else if pos.x > 0.0 {
        radius = radii.y; // TR
    } else if pos.y > 0.0 {
        radius = radii.w; // BL
    } else {
        radius = radii.x; // TL
    }

    let q = abs(pos) - half_size + vec2<f32>(radius);
    return min(max(q.x, q.y), 0.0) + length(max(q, vec2<f32>(0.0))) - radius;
}

// Render stroke around an SDF boundary
// align: -1=outside, 0=center, 1=inside
fn render_stroke(dist: f32, width: f32, align: f32) -> f32 {
    let offset = width * 0.5 * align;
    let adjusted_dist = dist - offset;
    let half_width = width * 0.5;

    // Smoothstep anti-aliasing over ~1px
    return 1.0 - smoothstep(half_width - 0.5, half_width + 0.5, abs(adjusted_dist));
}

// Extract flags
fn is_glyph(flags: u32) -> bool {
    return (flags & (1u << 14u)) != 0u;
}

fn has_stroke(flags: u32) -> bool {
    return (flags & (1u << 4u)) != 0u;
}

// ============================================================================
// Vertex Shader
// ============================================================================

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    // Get unit quad vertex position (0-1 range)
    let unit_pos = vertex_position(input.vertex_index);

    // Scale to primitive size and offset to position
    let screen_pos = input.pos + unit_pos * input.size;

    // Convert to clip space using projection matrix
    out.position = globals.transform * vec4<f32>(screen_pos, 0.0, 1.0);
    out.color = input.color;
    out.local_pos = unit_pos * input.size;
    out.prim_size = input.size;
    out.corner_radii = input.corner_radii;
    out.stroke_params = input.stroke_params;
    out.flags = input.flags;

    // Interpolate texture coordinates for glyphs
    let u = mix(input.tex_coords.x, input.tex_coords.z, unit_pos.x);
    let v = mix(input.tex_coords.y, input.tex_coords.w, unit_pos.y);
    out.tex_coord = vec2<f32>(u, v);

    return out;
}

// ============================================================================
// Fragment Shader
// ============================================================================

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    var final_color = input.color;

    // Branch: Glyph rendering (text)
    if is_glyph(input.flags) {
        // Sample glyph atlas texture
        let alpha = textureSample(glyph_texture, glyph_sampler, input.tex_coord).r;
        return vec4<f32>(final_color.rgb, final_color.a * alpha);
    }

    // Branch: Shape rendering (rectangles, gradients, strokes)

    // Fast path: sharp corners and no stroke
    // Use epsilon tolerance instead of exact comparison to avoid precision issues
    let all_radii_zero = all(input.corner_radii < vec4<f32>(0.01));
    if all_radii_zero && !has_stroke(input.flags) {
        return final_color;
    }

    // Compute SDF from center of rectangle
    let half_size = input.prim_size * 0.5;
    let centered_pos = input.local_pos - half_size;

    // Clamp radii so they don't exceed half the smallest dimension
    let max_radius = min(input.prim_size.x, input.prim_size.y) * 0.5;
    let clamped_radii = min(input.corner_radii, vec4<f32>(max_radius));

    let dist = rounded_rect_sdf_4(centered_pos, half_size, clamped_radii);

    // Apply stroke if present
    if has_stroke(input.flags) {
        let stroke_width = input.stroke_params.x;
        let stroke_align = input.stroke_params.y;
        let stroke_alpha = render_stroke(dist, stroke_width, stroke_align);
        final_color = vec4<f32>(final_color.rgb, final_color.a * stroke_alpha);
    } else {
        // Fill: anti-aliased alpha based on SDF
        let fill_alpha = 1.0 - smoothstep(-0.5, 0.5, dist);
        final_color = vec4<f32>(final_color.rgb, final_color.a * fill_alpha);
    }

    return final_color;
}
