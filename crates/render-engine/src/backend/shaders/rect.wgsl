// Rectangle shader with SDF rounded corners
// Follows iced's instanced rendering pattern

// Global uniforms (shared across all rectangles)
struct Globals {
    transform: mat4x4<f32>,  // Projection matrix for NDC conversion
}

@group(0) @binding(0)
var<uniform> globals: Globals;

// Per-instance vertex attributes
struct VertexInput {
    @builtin(vertex_index) vertex_index: u32,
    // Per-instance attributes
    @location(0) pos: vec2<f32>,           // Rectangle position (screen space)
    @location(1) size: vec2<f32>,          // Rectangle size
    @location(2) color: vec4<f32>,         // Rectangle color with alpha
    @location(3) corner_radius: f32,       // Corner radius (0 = sharp)
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) local_pos: vec2<f32>,     // Position within rect (0..size)
    @location(2) rect_size: vec2<f32>,     // Rect dimensions for SDF
    @location(3) corner_radius: f32,       // Passed to fragment shader
}

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

// Signed distance function for a rounded rectangle
// Based on Inigo Quilez's box SDF: https://iquilezles.org/articles/distfunctions2d/
fn rounded_rect_sdf(pos: vec2<f32>, half_size: vec2<f32>, radius: f32) -> f32 {
    let q = abs(pos) - half_size + vec2<f32>(radius);
    return min(max(q.x, q.y), 0.0) + length(max(q, vec2<f32>(0.0))) - radius;
}

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    // Get unit quad vertex position (0-1 range)
    let unit_pos = vertex_position(input.vertex_index);

    // Scale to rectangle size and offset to position
    let screen_pos = input.pos + unit_pos * input.size;

    // Convert to clip space using projection matrix
    out.position = globals.transform * vec4<f32>(screen_pos, 0.0, 1.0);
    out.color = input.color;
    out.local_pos = unit_pos * input.size;
    out.rect_size = input.size;
    out.corner_radius = input.corner_radius;

    return out;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    // Fast path: sharp corners, no SDF needed
    if input.corner_radius <= 0.0 {
        return input.color;
    }

    // Clamp radius so it doesn't exceed half the smallest dimension
    let max_radius = min(input.rect_size.x, input.rect_size.y) * 0.5;
    let radius = min(input.corner_radius, max_radius);

    // Compute SDF from center of rectangle
    let half_size = input.rect_size * 0.5;
    let centered_pos = input.local_pos - half_size;
    let dist = rounded_rect_sdf(centered_pos, half_size, radius);

    // Anti-aliased alpha: smoothstep over ~1px at the boundary
    let alpha = 1.0 - smoothstep(-0.5, 0.5, dist);

    return vec4<f32>(input.color.rgb, input.color.a * alpha);
}
