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

// Group 1: Gradient atlas
@group(1) @binding(0)
var gradient_texture: texture_2d<f32>;

@group(1) @binding(1)
var gradient_sampler: sampler;

// Per-gradient parameters (indexed via instance gradient_params)
struct GradientParams {
    start: vec2<f32>,           // Gradient start point (normalized 0-1 within rect)
    end: vec2<f32>,             // Gradient end point (normalized 0-1 within rect)
    atlas_row: f32,             // Row in gradient atlas (normalized v coord)
    gradient_type: u32,         // 0=linear, 1=radial, 2=angular, 3=diamond
    _padding: vec2<f32>,
}

@group(1) @binding(2)
var<storage, read> gradient_params_buffer: array<GradientParams>;

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
    @location(4) gradient_params: vec4<f32>, // Gradient parameters
    @location(5) tex_coord: vec2<f32>,     // Glyph texture coordinates
    @location(6) stroke_params: vec2<f32>, // Stroke parameters
    @location(7) flags: u32,               // Bitflags
    @location(8) world_pos: vec2<f32>,     // Fragment position in scene space
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

fn is_shadow(flags: u32) -> bool {
    return (flags & (1u << 31u)) != 0u;
}

fn get_fill_type(flags: u32) -> u32 {
    return flags & 0xFu;  // bits 0-3
}

fn get_gradient_index(gradient_params: vec4<f32>) -> u32 {
    return u32(gradient_params.x);
}

// ============================================================================
// Shadow Rendering
// ============================================================================

/// Render drop shadow with SDF-based blur approximation.
/// This is the fast path for small blur radii (< 20px).
/// For larger blurs, Phase 4's multi-pass Gaussian blur is used.
///
/// # Parameters
/// - dist: Signed distance from the SDF boundary
/// - blur_radius: Shadow blur radius in pixels
///
/// # Returns
/// Alpha value for the shadow at this distance
fn shadow_alpha(dist: f32, blur_radius: f32) -> f32 {
    // Approximate gaussian blur with smoothstep
    // Shadow is visible where dist < blur_radius
    // Smooth falloff from -blur_radius to +blur_radius
    return 1.0 - smoothstep(-blur_radius, blur_radius, dist);
}

// ============================================================================
// Gradient Sampling
// ============================================================================

/// Sample a gradient color from the gradient atlas LUT.
/// uv: Position within primitive (0-1 range)
/// params: Gradient parameters from storage buffer
fn sample_gradient(uv: vec2<f32>, params: GradientParams) -> vec4<f32> {
    // Compute gradient position t (0.0 to 1.0) based on gradient type
    var t: f32;

    switch params.gradient_type {
        case 0u: {
            // Linear gradient: dot product along axis
            let dir = params.end - params.start;
            let dir_len_sq = dot(dir, dir);
            if dir_len_sq < 0.0001 {
                // Degenerate gradient (start == end): use start color
                t = 0.0;
            } else {
                t = clamp(dot(uv - params.start, dir) / dir_len_sq, 0.0, 1.0);
            }
        }
        case 1u: {
            // Radial gradient: distance from center
            let radius = length(params.end - params.start);
            if radius < 0.0001 {
                // Degenerate gradient (zero radius): use start color
                t = 0.0;
            } else {
                t = clamp(length(uv - params.start) / radius, 0.0, 1.0);
            }
        }
        case 2u: {
            // Angular gradient: sweep around center (0-360 degrees)
            let d = uv - params.start;
            // atan2 returns [-π, π], normalize to [0, 1]
            t = (atan2(d.y, d.x) + 3.14159265) / (2.0 * 3.14159265);
        }
        case 3u, default: {
            // Diamond gradient: Manhattan distance (Figma-specific)
            let d = abs(uv - params.start);
            let scale = abs(params.end - params.start);
            if all(scale < vec2<f32>(0.0001)) {
                // Degenerate gradient: use start color
                t = 0.0;
            } else {
                // Manhattan distance normalized by scale
                let dist_x = select(0.0, d.x / scale.x, scale.x > 0.0001);
                let dist_y = select(0.0, d.y / scale.y, scale.y > 0.0001);
                t = clamp(dist_x + dist_y, 0.0, 1.0);
            }
        }
    }

    // Sample from gradient atlas LUT
    // x: gradient position (0-1), y: atlas row (normalized v coordinate)
    let atlas_uv = vec2<f32>(t, params.atlas_row);
    return textureSample(gradient_texture, gradient_sampler, atlas_uv);
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
    out.gradient_params = input.gradient_params;
    out.stroke_params = input.stroke_params;
    out.flags = input.flags;
    out.world_pos = input.pos + unit_pos * input.size;

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
    // Determine fill color: solid or gradient
    let fill_type = get_fill_type(input.flags);
    var base_color: vec4<f32>;

    if fill_type == 0u {
        // Solid fill
        base_color = input.color;
    } else {
        // Gradient fill (linear=1, radial=2, angular=3, diamond=4)
        // Get gradient parameters from storage buffer
        let gradient_index = get_gradient_index(input.gradient_params);
        let params = gradient_params_buffer[gradient_index];

        // Compute UV in normalized rect space (0-1).
        // For glyphs, map against the parent text node bounds:
        // - gradient_params.yz = text x/y
        // - gradient_params.w = text width
        // - stroke_params.x = text height
        // For shapes, map against primitive local size.
        var uv: vec2<f32>;
        if is_glyph(input.flags) {
            let text_pos = input.gradient_params.yz;
            let text_size = max(vec2<f32>(input.gradient_params.w, input.stroke_params.x), vec2<f32>(1.0, 1.0));
            uv = clamp((input.world_pos - text_pos) / text_size, vec2<f32>(0.0), vec2<f32>(1.0));
        } else {
            uv = input.local_pos / input.prim_size;
        }
        base_color = sample_gradient(uv, params);

        // Apply instance color alpha (for opacity control)
        base_color = vec4<f32>(base_color.rgb, base_color.a * input.color.a);
    }

    var final_color = base_color;

    // Branch: Glyph rendering (text)
    if is_glyph(input.flags) {
        // Sample glyph atlas texture
        let alpha = textureSample(glyph_texture, glyph_sampler, input.tex_coord).r;
        // Gradient text: multiply glyph alpha with gradient color
        return vec4<f32>(final_color.rgb, final_color.a * alpha);
    }

    // Branch: Shadow rendering
    if is_shadow(input.flags) {
        // Compute SDF from center of rectangle
        let half_size = input.prim_size * 0.5;
        let centered_pos = input.local_pos - half_size;

        // Clamp radii so they don't exceed half the smallest dimension
        let max_radius = min(input.prim_size.x, input.prim_size.y) * 0.5;
        let clamped_radii = min(input.corner_radii, vec4<f32>(max_radius));

        let dist = rounded_rect_sdf_4(centered_pos, half_size, clamped_radii);

        // Get blur radius from stroke_params[0] (shadows don't use stroke)
        let blur_radius = input.stroke_params.x;

        // Apply shadow blur
        let shadow_alpha_value = shadow_alpha(dist, blur_radius);

        return vec4<f32>(final_color.rgb, final_color.a * shadow_alpha_value);
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
