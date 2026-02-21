struct Globals {
    transform: mat4x4<f32>,
};

@group(0) @binding(0)
var<uniform> globals: Globals;

@group(1) @binding(0)
var gradient_texture: texture_2d<f32>;

@group(1) @binding(1)
var gradient_sampler: sampler;

struct GradientParams {
    start: vec2<f32>,
    end: vec2<f32>,
    atlas_row: f32,
    gradient_type: u32, // 0=linear, 1=radial, 2=angular, 3=diamond
    _padding: vec2<f32>,
}

@group(1) @binding(2)
var<storage, read> gradient_params_buffer: array<GradientParams>;

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) normal: vec2<f32>,
    @location(2) color: vec4<f32>,
    @location(3) uv: vec2<f32>,
    @location(4) fill_type: u32,
    @location(5) gradient_index: u32,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) @interpolate(flat) fill_type: u32,
    @location(3) @interpolate(flat) gradient_index: u32,
}

fn sample_gradient(uv: vec2<f32>, params: GradientParams) -> vec4<f32> {
    var t: f32;
    switch params.gradient_type {
        case 0u: {
            let dir = params.end - params.start;
            let dir_len_sq = dot(dir, dir);
            if dir_len_sq < 0.0001 {
                t = 0.0;
            } else {
                t = clamp(dot(uv - params.start, dir) / dir_len_sq, 0.0, 1.0);
            }
        }
        case 1u: {
            let radius = length(params.end - params.start);
            if radius < 0.0001 {
                t = 0.0;
            } else {
                t = clamp(length(uv - params.start) / radius, 0.0, 1.0);
            }
        }
        case 2u: {
            let d = uv - params.start;
            t = (atan2(d.y, d.x) + 3.14159265) / (2.0 * 3.14159265);
        }
        case 3u, default: {
            let d = abs(uv - params.start);
            let scale = abs(params.end - params.start);
            if all(scale < vec2<f32>(0.0001)) {
                t = 0.0;
            } else {
                let dist_x = select(0.0, d.x / scale.x, scale.x > 0.0001);
                let dist_y = select(0.0, d.y / scale.y, scale.y > 0.0001);
                t = clamp(dist_x + dist_y, 0.0, 1.0);
            }
        }
    }
    let atlas_uv = vec2<f32>(t, params.atlas_row);
    return textureSampleLevel(gradient_texture, gradient_sampler, atlas_uv, 0.0);
}

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    let _n = input.normal;
    out.position = globals.transform * vec4<f32>(input.position, 0.0, 1.0);
    out.color = input.color;
    out.uv = input.uv;
    out.fill_type = input.fill_type;
    out.gradient_index = input.gradient_index;
    return out;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    if input.fill_type == 0u {
        return input.color;
    }

    let params = gradient_params_buffer[input.gradient_index];
    let gradient_color = sample_gradient(input.uv, params);
    return vec4<f32>(gradient_color.rgb, gradient_color.a * input.color.a);
}
