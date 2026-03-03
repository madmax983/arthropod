struct VertexOut {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

struct BlendParams {
    blend_mode: u32,
    _pad0: u32,
    _pad1: u32,
    _pad2: u32,
}

@group(0) @binding(0)
var src_tex: texture_2d<f32>;
@group(0) @binding(1)
var dst_tex: texture_2d<f32>;
@group(0) @binding(2)
var tex_smp: sampler;
@group(0) @binding(3)
var<uniform> blend: BlendParams;

fn blend_multiply(src: vec3<f32>, dst: vec3<f32>) -> vec3<f32> {
    return src * dst;
}

fn blend_screen(src: vec3<f32>, dst: vec3<f32>) -> vec3<f32> {
    return src + dst - src * dst;
}

fn blend_overlay(src: vec3<f32>, dst: vec3<f32>) -> vec3<f32> {
    return select(
        2.0 * src * dst,
        1.0 - 2.0 * (1.0 - src) * (1.0 - dst),
        dst > vec3<f32>(0.5),
    );
}

fn blend_darken(src: vec3<f32>, dst: vec3<f32>) -> vec3<f32> {
    return min(src, dst);
}

fn blend_lighten(src: vec3<f32>, dst: vec3<f32>) -> vec3<f32> {
    return max(src, dst);
}

fn blend_difference(src: vec3<f32>, dst: vec3<f32>) -> vec3<f32> {
    return abs(src - dst);
}

fn blend_exclusion(src: vec3<f32>, dst: vec3<f32>) -> vec3<f32> {
    return src + dst - 2.0 * src * dst;
}

fn blend_color_burn(src: vec3<f32>, dst: vec3<f32>) -> vec3<f32> {
    // Guard near-zero source terms.
    let eps = vec3<f32>(1e-6);
    return 1.0 - min(vec3<f32>(1.0), (1.0 - dst) / max(src, eps));
}

fn blend_color_dodge(src: vec3<f32>, dst: vec3<f32>) -> vec3<f32> {
    let eps = vec3<f32>(1e-6);
    return min(vec3<f32>(1.0), dst / max(1.0 - src, eps));
}

fn blend_linear_burn(src: vec3<f32>, dst: vec3<f32>) -> vec3<f32> {
    return max(vec3<f32>(0.0), src + dst - vec3<f32>(1.0));
}

fn blend_linear_dodge(src: vec3<f32>, dst: vec3<f32>) -> vec3<f32> {
    return min(vec3<f32>(1.0), src + dst);
}

fn rgb_to_hsl(rgb: vec3<f32>) -> vec3<f32> {
    let max_c = max(max(rgb.r, rgb.g), rgb.b);
    let min_c = min(min(rgb.r, rgb.g), rgb.b);
    let l = (max_c + min_c) * 0.5;
    let delta = max_c - min_c;
    let eps = 1e-6;
    if delta <= eps {
        return vec3<f32>(0.0, 0.0, l);
    }
    let denom = 1.0 - abs(2.0 * l - 1.0);
    let s = select(delta / denom, 0.0, denom <= eps);
    var h: f32;
    if abs(max_c - rgb.r) <= eps {
        h = (rgb.g - rgb.b) / delta + select(0.0, 6.0, rgb.g < rgb.b);
    } else if abs(max_c - rgb.g) <= eps {
        h = (rgb.b - rgb.r) / delta + 2.0;
    } else {
        h = (rgb.r - rgb.g) / delta + 4.0;
    }
    h = h / 6.0;
    if h < 0.0 {
        h = h + 1.0;
    }
    return vec3<f32>(h, clamp(s, 0.0, 1.0), clamp(l, 0.0, 1.0));
}

fn hue_to_rgb(p: f32, q: f32, t_in: f32) -> f32 {
    var t = t_in;
    if t < 0.0 {
        t = t + 1.0;
    }
    if t > 1.0 {
        t = t - 1.0;
    }
    if t < 1.0 / 6.0 {
        return p + (q - p) * 6.0 * t;
    }
    if t < 1.0 / 2.0 {
        return q;
    }
    if t < 2.0 / 3.0 {
        return p + (q - p) * (2.0 / 3.0 - t) * 6.0;
    }
    return p;
}

fn hsl_to_rgb(hsl: vec3<f32>) -> vec3<f32> {
    let h = hsl.x;
    let s = hsl.y;
    let l = hsl.z;
    let eps = 1e-6;
    if s <= eps {
        return vec3<f32>(l);
    }
    var q: f32;
    if l < 0.5 {
        q = l * (1.0 + s);
    } else {
        q = l + s - l * s;
    }
    let p = 2.0 * l - q;
    return vec3<f32>(
        clamp(hue_to_rgb(p, q, h + 1.0 / 3.0), 0.0, 1.0),
        clamp(hue_to_rgb(p, q, h), 0.0, 1.0),
        clamp(hue_to_rgb(p, q, h - 1.0 / 3.0), 0.0, 1.0),
    );
}

fn blend_hue(src: vec3<f32>, dst: vec3<f32>) -> vec3<f32> {
    let src_hsl = rgb_to_hsl(src);
    let dst_hsl = rgb_to_hsl(dst);
    return hsl_to_rgb(vec3<f32>(src_hsl.x, dst_hsl.y, dst_hsl.z));
}

fn blend_saturation(src: vec3<f32>, dst: vec3<f32>) -> vec3<f32> {
    let src_hsl = rgb_to_hsl(src);
    let dst_hsl = rgb_to_hsl(dst);
    return hsl_to_rgb(vec3<f32>(dst_hsl.x, src_hsl.y, dst_hsl.z));
}

fn blend_color(src: vec3<f32>, dst: vec3<f32>) -> vec3<f32> {
    let src_hsl = rgb_to_hsl(src);
    let dst_hsl = rgb_to_hsl(dst);
    return hsl_to_rgb(vec3<f32>(src_hsl.x, src_hsl.y, dst_hsl.z));
}

fn blend_luminosity(src: vec3<f32>, dst: vec3<f32>) -> vec3<f32> {
    let src_hsl = rgb_to_hsl(src);
    let dst_hsl = rgb_to_hsl(dst);
    return hsl_to_rgb(vec3<f32>(dst_hsl.x, dst_hsl.y, src_hsl.z));
}

fn apply_blend(mode: u32, src: vec3<f32>, dst: vec3<f32>) -> vec3<f32> {
    // style-engine::BlendMode flag mapping:
    // 0=Normal, 1=Darken, 2=Multiply, 3=ColorBurn, 4=Lighten, 5=Screen,
    // 6=ColorDodge, 7=Overlay, 8=SoftLight, 9=HardLight, 10=Difference, 11=Exclusion,
    // 12=Hue, 13=Saturation, 14=Color, 15=Luminosity, 16=LinearBurn, 17=LinearDodge, 18=PassThrough
    switch mode {
        case 1u: { return blend_darken(src, dst); }
        case 2u: { return blend_multiply(src, dst); }
        case 3u: { return blend_color_burn(src, dst); }
        case 4u: { return blend_lighten(src, dst); }
        case 5u: { return blend_screen(src, dst); }
        case 6u: { return blend_color_dodge(src, dst); }
        case 7u: { return blend_overlay(src, dst); }
        // SoftLight fallback to screen-like behavior.
        case 8u: { return blend_screen(src, dst); }
        // HardLight is overlay with src/dst swapped.
        case 9u: { return blend_overlay(dst, src); }
        case 10u: { return blend_difference(src, dst); }
        case 11u: { return blend_exclusion(src, dst); }
        case 12u: { return blend_hue(src, dst); }
        case 13u: { return blend_saturation(src, dst); }
        case 14u: { return blend_color(src, dst); }
        case 15u: { return blend_luminosity(src, dst); }
        case 16u: { return blend_linear_burn(src, dst); }
        case 17u: { return blend_linear_dodge(src, dst); }
        // PassThrough behaves like normal at this compositing stage.
        case 18u: { return src; }
        default: { return src; } // Normal and unknown fallback
    }
}

fn unpremultiply(color: vec4<f32>) -> vec3<f32> {
    if color.a <= 1e-6 {
        return vec3<f32>(0.0);
    }
    return color.rgb / color.a;
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
    let src = textureSample(src_tex, tex_smp, input.uv);
    let dst = textureSample(dst_tex, tex_smp, input.uv);

    // Offscreen targets are stored as premultiplied color due fixed-function alpha
    // blending in primitive/path passes. Convert back to straight color for
    // blend-mode math, then re-premultiply in the final output.
    let src_rgb = unpremultiply(src);
    let dst_rgb = unpremultiply(dst);
    let blended_rgb = apply_blend(blend.blend_mode, src_rgb, dst_rgb);

    // Porter-Duff source-over with blend-mode color function:
    // Co = as*(1-ab)*Cs + as*ab*B(Cb,Cs) + (1-as)*ab*Cb
    let out_rgb = src.a * (1.0 - dst.a) * src_rgb
        + src.a * dst.a * blended_rgb
        + (1.0 - src.a) * dst.a * dst_rgb;
    let out_a = src.a + dst.a - src.a * dst.a;
    return vec4<f32>(out_rgb, out_a);
}
