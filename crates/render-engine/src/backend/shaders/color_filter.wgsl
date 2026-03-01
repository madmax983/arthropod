struct VertexOut {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

struct ColorFilterParams {
    grayscale: f32,
    contrast: f32,
    invert: f32,
    _pad: f32,
}

@group(0) @binding(0)
var source_tex: texture_2d<f32>;
@group(0) @binding(1)
var source_smp: sampler;
@group(0) @binding(2)
var<uniform> params: ColorFilterParams;

fn linear_to_srgb_channel(channel: f32) -> f32 {
    let c = clamp(channel, 0.0, 1.0);
    if c <= 0.0031308 {
        return c * 12.92;
    }
    return 1.055 * pow(c, 1.0 / 2.4) - 0.055;
}

fn srgb_to_linear_channel(channel: f32) -> f32 {
    let c = clamp(channel, 0.0, 1.0);
    if c <= 0.04045 {
        return c / 12.92;
    }
    return pow((c + 0.055) / 1.055, 2.4);
}

fn linear_to_srgb(rgb: vec3<f32>) -> vec3<f32> {
    return vec3<f32>(
        linear_to_srgb_channel(rgb.x),
        linear_to_srgb_channel(rgb.y),
        linear_to_srgb_channel(rgb.z),
    );
}

fn srgb_to_linear(rgb: vec3<f32>) -> vec3<f32> {
    return vec3<f32>(
        srgb_to_linear_channel(rgb.x),
        srgb_to_linear_channel(rgb.y),
        srgb_to_linear_channel(rgb.z),
    );
}

fn apply_color_filter(srgb_rgb: vec3<f32>) -> vec3<f32> {
    var rgb = srgb_rgb;

    let g = clamp(params.grayscale, 0.0, 1.0);
    if g > 0.0 {
        let luma = dot(rgb, vec3<f32>(0.2126, 0.7152, 0.0722));
        rgb = mix(rgb, vec3<f32>(luma), g);
    }

    let contrast = max(params.contrast, 0.0);
    rgb = clamp((rgb - vec3<f32>(0.5)) * contrast + vec3<f32>(0.5), vec3<f32>(0.0), vec3<f32>(1.0));

    let inv = clamp(params.invert, 0.0, 1.0);
    if inv > 0.0 {
        rgb = mix(rgb, vec3<f32>(1.0) - rgb, inv);
    }

    return rgb;
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
    let src = textureSample(source_tex, source_smp, input.uv);
    if src.a <= 1e-6 {
        return vec4<f32>(0.0, 0.0, 0.0, 0.0);
    }

    // Multipass layers are stored premultiplied linear in render targets.
    // Figma/CSS-like color filters are authored in display (sRGB) space.
    let straight_linear = clamp(src.rgb / src.a, vec3<f32>(0.0), vec3<f32>(1.0));
    let straight_srgb = linear_to_srgb(straight_linear);
    let filtered_srgb = apply_color_filter(straight_srgb);
    let filtered_linear = clamp(srgb_to_linear(filtered_srgb), vec3<f32>(0.0), vec3<f32>(1.0));
    return vec4<f32>(filtered_linear * src.a, src.a);
}
