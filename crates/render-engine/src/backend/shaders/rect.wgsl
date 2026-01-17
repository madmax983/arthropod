// Rectangle shader following iced's instanced rendering pattern

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
    @location(0) pos: vec2<f32>,      // Rectangle position (screen space)
    @location(1) size: vec2<f32>,     // Rectangle size
    @location(2) color: vec4<f32>,    // Rectangle color with alpha
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
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

    return out;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    return input.color;
}
