struct VertexOut {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

struct BlurParams {
    direction: vec2<f32>,
    texel_size: vec2<f32>,
    tap_count: u32,
    _pad: u32,
    packed_weights: array<vec4<f32>, 7>,
}

@group(0) @binding(0)
var src_tex: texture_2d<f32>;
@group(0) @binding(1)
var src_smp: sampler;
@group(0) @binding(2)
var<uniform> blur: BlurParams;

fn inner_shadow_alpha(mask: f32, blurred_offset_mask: f32) -> f32 {
    return clamp(blurred_offset_mask - (1.0 - mask), 0.0, 1.0) * mask;
}

fn blur_weight(index: u32) -> f32 {
    let block = index / 4u;
    let lane = index % 4u;
    let packed = blur.packed_weights[block];
    if lane == 0u {
        return packed.x;
    }
    if lane == 1u {
        return packed.y;
    }
    if lane == 2u {
        return packed.z;
    }
    return packed.w;
}

@vertex
fn vs_main(@builtin(vertex_index) vi: u32) -> VertexOut {
    var pos = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -3.0),
        vec2<f32>(-1.0, 1.0),
        vec2<f32>(3.0, 1.0)
    );
    let p = pos[vi];
    var out: VertexOut;
    out.position = vec4<f32>(p, 0.0, 1.0);
    out.uv = vec2<f32>(p.x * 0.5 + 0.5, 1.0 - (p.y * 0.5 + 0.5));
    return out;
}

@fragment
fn fs_main(input: VertexOut) -> @location(0) vec4<f32> {
    let taps = min(blur.tap_count, 25u);
    let center = i32(taps / 2u);
    var color = vec4<f32>(0.0);
    var total = 0.0;

    for (var i: i32 = 0; i < i32(taps); i++) {
        let weight = blur_weight(u32(i));
        let offset = vec2<f32>(f32(i - center)) * blur.direction * blur.texel_size;
        color += textureSample(src_tex, src_smp, input.uv + offset) * weight;
        total += weight;
    }

    if total <= 0.0 {
        return textureSample(src_tex, src_smp, input.uv);
    }
    return color / total;
}
