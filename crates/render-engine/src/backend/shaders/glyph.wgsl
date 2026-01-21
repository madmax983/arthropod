// Glyph shader for text rendering with texture atlas

// Global uniforms (shared across all glyphs)
struct Globals {
    transform: mat4x4<f32>,  // Projection matrix for NDC conversion
}

@group(0) @binding(0)
var<uniform> globals: Globals;

@group(0) @binding(1)
var glyph_texture: texture_2d<f32>;

@group(0) @binding(2)
var glyph_sampler: sampler;

// Per-instance vertex attributes
struct VertexInput {
    @builtin(vertex_index) vertex_index: u32,
    // Per-instance attributes
    @location(0) pos: vec2<f32>,        // Glyph position (screen space)
    @location(1) size: vec2<f32>,       // Glyph size
    @location(2) color: vec4<f32>,      // Glyph color with alpha
    @location(3) tex_coords: vec4<f32>, // Texture coords (u0, v0, u1, v1)
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) tex_coord: vec2<f32>,
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

    // Scale to glyph size and offset to position
    let screen_pos = input.pos + unit_pos * input.size;

    // Convert to clip space using projection matrix
    out.position = globals.transform * vec4<f32>(screen_pos, 0.0, 1.0);
    out.color = input.color;

    // Interpolate texture coordinates
    let u = mix(input.tex_coords.x, input.tex_coords.z, unit_pos.x);
    let v = mix(input.tex_coords.y, input.tex_coords.w, unit_pos.y);
    out.tex_coord = vec2<f32>(u, v);

    return out;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    // Sample glyph atlas texture
    let alpha = textureSample(glyph_texture, glyph_sampler, input.tex_coord).r;

    // Multiply color by texture alpha
    return vec4<f32>(input.color.rgb, input.color.a * alpha);
}
