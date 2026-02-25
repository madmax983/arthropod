use style_engine::{BlendMode, CornerRadii, StrokeCap, StrokeJoin};

pub const FLAG_FILL_TYPE_MASK: u32 = 0xF;
pub const FLAG_HAS_STROKE: u32 = 1 << 4;
pub const FLAG_BLEND_MODE_SHIFT: u32 = 5;
pub const FLAG_BLEND_MODE_MASK: u32 = 0x1F << FLAG_BLEND_MODE_SHIFT;
pub const FLAG_STROKE_CAP_SHIFT: u32 = 10;
pub const FLAG_STROKE_CAP_MASK: u32 = 0x3 << FLAG_STROKE_CAP_SHIFT;
pub const FLAG_STROKE_JOIN_SHIFT: u32 = 12;
pub const FLAG_STROKE_JOIN_MASK: u32 = 0x3 << FLAG_STROKE_JOIN_SHIFT;
pub const FLAG_IS_GLYPH: u32 = 1 << 14;
pub const FLAG_IS_SHADOW: u32 = 1 << 31;

pub fn with_fill_type(flags: u32, fill_type: u32) -> u32 {
    (flags & !FLAG_FILL_TYPE_MASK) | (fill_type & FLAG_FILL_TYPE_MASK)
}

pub fn with_blend_mode(flags: u32, blend_mode: BlendMode) -> u32 {
    let blend_bits = (blend_mode.to_flag_bits() as u32) << FLAG_BLEND_MODE_SHIFT;
    (flags & !FLAG_BLEND_MODE_MASK) | (blend_bits & FLAG_BLEND_MODE_MASK)
}

fn stroke_cap_bits(cap: StrokeCap) -> u32 {
    match cap {
        StrokeCap::Butt => 0,
        StrokeCap::Round => 1,
        StrokeCap::Square => 2,
    }
}

fn stroke_join_bits(join: StrokeJoin) -> u32 {
    match join {
        StrokeJoin::Miter => 0,
        StrokeJoin::Bevel => 1,
        StrokeJoin::Round => 2,
    }
}

pub fn with_stroke_cap_join(flags: u32, cap: StrokeCap, join: StrokeJoin) -> u32 {
    let cap_bits = stroke_cap_bits(cap) << FLAG_STROKE_CAP_SHIFT;
    let join_bits = stroke_join_bits(join) << FLAG_STROKE_JOIN_SHIFT;
    (flags & !(FLAG_STROKE_CAP_MASK | FLAG_STROKE_JOIN_MASK))
        | (cap_bits & FLAG_STROKE_CAP_MASK)
        | (join_bits & FLAG_STROKE_JOIN_MASK)
}

/// Unified primitive instance data (96 bytes)
///
/// This replaces legacy Rect/Glyph instance payloads with a single
/// unified type that supports solid fills, gradients, strokes, shadows, and text.
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct PrimitiveInstance {
    pub pos: [f32; 2],             // 8 bytes: Position (x, y)
    pub size: [f32; 2],            // 8 bytes: Size (width, height)
    pub color: [f32; 4],           // 16 bytes: Primary color (r, g, b, a)
    pub corner_radii: [f32; 4],    // 16 bytes: Corner radii (TL, TR, BR, BL)
    pub gradient_params: [f32; 4], // 16 bytes: Gradient parameters (atlas row, blend mode, etc.)
    pub tex_coords: [f32; 4],      // 16 bytes: Texture coordinates (x0, y0, x1, y1) for glyph atlas
    pub stroke_params: [f32; 2],   // 8 bytes: Stroke (width, align: -1=outside, 0=center, 1=inside)
    pub flags: u32, // 4 bytes: Bitflags (fill_type, has_stroke, blend_mode, is_glyph)
    pub _padding: u32, // 4 bytes: Padding to 96 bytes
}

impl PrimitiveInstance {
    /// Create a solid-fill rectangle with sharp corners.
    #[inline]
    pub fn solid(pos: [f32; 2], size: [f32; 2], color: [f32; 4]) -> Self {
        Self {
            pos,
            size,
            color,
            corner_radii: [0.0; 4],
            gradient_params: [0.0; 4],
            tex_coords: [0.0; 4],
            stroke_params: [0.0; 2],
            flags: 0, // fill_type=0 (solid)
            _padding: 0,
        }
    }

    /// Create a solid-fill rectangle with per-corner radii.
    #[inline]
    pub fn rounded(pos: [f32; 2], size: [f32; 2], color: [f32; 4], radii: CornerRadii) -> Self {
        Self {
            pos,
            size,
            color,
            corner_radii: radii.to_array(),
            gradient_params: [0.0; 4],
            tex_coords: [0.0; 4],
            stroke_params: [0.0; 2],
            flags: 0, // fill_type=0 (solid)
            _padding: 0,
        }
    }

    /// Create a glyph instance for text rendering.
    #[inline]
    pub fn glyph(pos: [f32; 2], size: [f32; 2], color: [f32; 4], tex_coords: [f32; 4]) -> Self {
        Self {
            pos,
            size,
            color,
            corner_radii: [0.0; 4],
            gradient_params: [0.0; 4],
            tex_coords,
            stroke_params: [0.0; 2],
            flags: FLAG_IS_GLYPH, // is_glyph flag (bit 14)
            _padding: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_primitive_instance_size() {
        assert_eq!(
            std::mem::size_of::<PrimitiveInstance>(),
            96,
            "PrimitiveInstance must be exactly 96 bytes"
        );
    }

    #[test]
    fn test_primitive_instance_pod() {
        // If this compiles, bytemuck::Pod and Zeroable are correctly implemented
        let instance = PrimitiveInstance::solid([0.0, 0.0], [100.0, 100.0], [1.0, 0.0, 0.0, 1.0]);
        let _bytes = bytemuck::bytes_of(&instance);
        assert_eq!(_bytes.len(), 96);
    }

    #[test]
    fn test_solid_constructor() {
        let instance = PrimitiveInstance::solid([10.0, 20.0], [100.0, 50.0], [1.0, 0.0, 0.0, 1.0]);

        assert_eq!(instance.pos, [10.0, 20.0]);
        assert_eq!(instance.size, [100.0, 50.0]);
        assert_eq!(instance.color, [1.0, 0.0, 0.0, 1.0]);
        assert_eq!(
            instance.corner_radii, [0.0; 4],
            "Solid should have zero radii"
        );
        assert_eq!(instance.flags, 0, "Solid should have fill_type=0");
    }

    #[test]
    fn test_rounded_constructor() {
        let radii = CornerRadii::uniform(8.0);
        let instance =
            PrimitiveInstance::rounded([10.0, 20.0], [100.0, 50.0], [0.0, 1.0, 0.0, 1.0], radii);

        assert_eq!(instance.pos, [10.0, 20.0]);
        assert_eq!(instance.corner_radii, [8.0, 8.0, 8.0, 8.0]);
        assert_eq!(instance.flags, 0, "Rounded should have fill_type=0");
    }

    #[test]
    fn test_glyph_constructor() {
        let instance = PrimitiveInstance::glyph(
            [10.0, 20.0],
            [16.0, 16.0],
            [0.0, 0.0, 0.0, 1.0],
            [0.0, 0.0, 0.5, 0.5],
        );

        assert_eq!(instance.tex_coords, [0.0, 0.0, 0.5, 0.5]);
        assert_eq!(
            instance.flags,
            1 << 14,
            "Glyph should have is_glyph bit 14 set"
        );
    }

    #[test]
    fn test_flags_bitfield() {
        let instance = PrimitiveInstance::glyph([0.0, 0.0], [10.0, 10.0], [1.0; 4], [0.0; 4]);

        // Bit 14 should be set for glyph
        assert_eq!(instance.flags & (1 << 14), 1 << 14);

        // Bits 0-3 (fill_type) should be 0
        assert_eq!(instance.flags & 0xF, 0);
    }
}
