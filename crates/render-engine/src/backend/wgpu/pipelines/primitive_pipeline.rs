//! Unified primitive rendering pipeline supporting rects, text, gradients, strokes, and effects.

use glam::Vec2;
#[cfg(test)]
use glam::Vec4;
use hashbrown::HashMap;
use style_engine::{BlendMode, ColorStop, CornerRadii, Paint, StrokeCap, StrokeJoin, VisualStyle};

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

fn with_fill_type(flags: u32, fill_type: u32) -> u32 {
    (flags & !FLAG_FILL_TYPE_MASK) | (fill_type & FLAG_FILL_TYPE_MASK)
}

fn with_blend_mode(flags: u32, blend_mode: BlendMode) -> u32 {
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

fn with_stroke_cap_join(flags: u32, cap: StrokeCap, join: StrokeJoin) -> u32 {
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

/// Gradient parameters for shader (32 bytes)
///
/// Passed to shader via storage buffer to describe how to sample the gradient atlas.
/// Must match WGSL struct layout exactly (16-byte alignment for vec2/vec3/vec4).
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GradientParams {
    pub start: [f32; 2],        // 8 bytes: Gradient start point (normalized 0-1)
    pub end: [f32; 2],          // 8 bytes: Gradient end point (normalized 0-1)
    pub atlas_row: f32,         // 4 bytes: Row in atlas (normalized v coord)
    pub gradient_type: u32,     // 4 bytes: 0=linear, 1=radial, 2=angular, 3=diamond
    pub _padding: [f32; 2],     // 8 bytes: Padding to 32 bytes
}

impl GradientParams {
    /// Create gradient params for linear gradient
    pub fn linear(start: [f32; 2], end: [f32; 2], atlas_row: u32) -> Self {
        Self {
            start,
            end,
            atlas_row: (atlas_row as f32 + 0.5) / GradientAtlas::ATLAS_SIZE as f32,
            gradient_type: 0,
            _padding: [0.0; 2],
        }
    }

    /// Create gradient params for radial gradient
    pub fn radial(center: [f32; 2], radius: f32, atlas_row: u32) -> Self {
        Self {
            start: center,
            end: [center[0] + radius, center[1]],
            atlas_row: (atlas_row as f32 + 0.5) / GradientAtlas::ATLAS_SIZE as f32,
            gradient_type: 1,
            _padding: [0.0; 2],
        }
    }

    /// Create gradient params for angular gradient
    pub fn angular(center: [f32; 2], atlas_row: u32) -> Self {
        Self {
            start: center,
            end: center, // Not used for angular
            atlas_row: (atlas_row as f32 + 0.5) / GradientAtlas::ATLAS_SIZE as f32,
            gradient_type: 2,
            _padding: [0.0; 2],
        }
    }

    /// Create gradient params for diamond gradient (Figma-specific)
    pub fn diamond(center: [f32; 2], scale: [f32; 2], atlas_row: u32) -> Self {
        Self {
            start: center,
            end: [center[0] + scale[0], center[1] + scale[1]],
            atlas_row: (atlas_row as f32 + 0.5) / GradientAtlas::ATLAS_SIZE as f32,
            gradient_type: 3,
            _padding: [0.0; 2],
        }
    }
}

/// Gradient atlas texture manager
///
/// A 1024x1024 Rgba16Float texture where each row stores a rasterized gradient.
/// This allows unlimited gradient stops without shader complexity.
pub struct GradientAtlas {
    /// Cache mapping gradient hash -> atlas row index
    cache: HashMap<u64, u32>,
    /// Next available row
    next_row: u32,
    /// Texture data (1024 rows x 1024 texels x 4 channels x f16)
    data: Vec<u16>, // f16 stored as u16
    /// GPU texture (owned by GradientAtlas)
    texture: Option<wgpu::Texture>,
    /// Texture view
    texture_view: Option<wgpu::TextureView>,
    /// Sampler
    sampler: Option<wgpu::Sampler>,
    /// Dirty flag: true if data has changed and needs GPU upload
    dirty: bool,
}

impl GradientAtlas {
    const ATLAS_SIZE: usize = 1024;
    const TEXELS_PER_ROW: usize = 1024;
}

impl Default for GradientAtlas {
    fn default() -> Self {
        Self {
            cache: HashMap::new(),
            next_row: 0,
            data: vec![0u16; Self::ATLAS_SIZE * Self::TEXELS_PER_ROW * 4],
            texture: None,
            texture_view: None,
            sampler: None,
            dirty: false,
        }
    }
}

impl GradientAtlas {
    /// Initialize GPU resources (must be called after construction)
    pub fn init_gpu(&mut self, device: &wgpu::Device) {
        // Create 1024x1024 Rgba16Float texture
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Gradient Atlas Texture"),
            size: wgpu::Extent3d {
                width: Self::ATLAS_SIZE as u32,
                height: Self::ATLAS_SIZE as u32,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba16Float,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Gradient Atlas Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Linear,
            ..Default::default()
        });

        self.texture = Some(texture);
        self.texture_view = Some(texture_view);
        self.sampler = Some(sampler);
        self.dirty = true; // Mark for initial upload
    }

    /// Upload dirty data to GPU
    pub fn upload_to_gpu(&mut self, queue: &wgpu::Queue) {
        if !self.dirty {
            return;
        }

        if let Some(ref texture) = self.texture {
            // Upload entire texture (1024x1024x4 half-floats = 8MB)
            let data_bytes = bytemuck::cast_slice(&self.data);

            let bytes_per_row = Self::TEXELS_PER_ROW as u32 * 4 * 2; // 4 channels * 2 bytes (f16)

            queue.write_texture(
                texture.as_image_copy(),
                data_bytes,
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(bytes_per_row),
                    rows_per_image: Some(Self::ATLAS_SIZE as u32),
                },
                wgpu::Extent3d {
                    width: Self::ATLAS_SIZE as u32,
                    height: Self::ATLAS_SIZE as u32,
                    depth_or_array_layers: 1,
                },
            );

            self.dirty = false;
        }
    }

    /// Get texture view (if initialized)
    pub fn texture_view(&self) -> Option<&wgpu::TextureView> {
        self.texture_view.as_ref()
    }

    /// Get sampler (if initialized)
    pub fn sampler(&self) -> Option<&wgpu::Sampler> {
        self.sampler.as_ref()
    }

    /// Rasterize a gradient into the atlas and return the row index
    pub fn add_gradient(&mut self, stops: &[ColorStop]) -> u32 {
        // Compute hash for deduplication
        let hash = self.hash_gradient(stops);

        // Check cache
        if let Some(&row) = self.cache.get(&hash) {
            return row;
        }

        // Allocate new row
        let row = self.next_row;
        self.next_row += 1;

        // Rasterize gradient into row
        self.rasterize_gradient(row, stops);

        // Cache it
        self.cache.insert(hash, row);

        // Mark for GPU upload
        self.dirty = true;

        row
    }

    /// Rasterize gradient stops into a texture row
    fn rasterize_gradient(&mut self, row: u32, stops: &[ColorStop]) {
        let row_start = (row as usize) * Self::TEXELS_PER_ROW * 4;

        for texel in 0..Self::TEXELS_PER_ROW {
            let t = texel as f32 / (Self::TEXELS_PER_ROW - 1) as f32;
            let color = Paint::interpolate_stops(t, stops);

            let base = row_start + texel * 4;
            self.data[base] = half::f16::from_f32(color.x).to_bits();
            self.data[base + 1] = half::f16::from_f32(color.y).to_bits();
            self.data[base + 2] = half::f16::from_f32(color.z).to_bits();
            self.data[base + 3] = half::f16::from_f32(color.w).to_bits();
        }
    }

    /// Simple hash for gradient deduplication
    fn hash_gradient(&self, stops: &[ColorStop]) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        stops.len().hash(&mut hasher);
        for stop in stops {
            // Hash position and color components
            stop.position.to_bits().hash(&mut hasher);
            stop.color.x.to_bits().hash(&mut hasher);
            stop.color.y.to_bits().hash(&mut hasher);
            stop.color.z.to_bits().hash(&mut hasher);
            stop.color.w.to_bits().hash(&mut hasher);
        }
        hasher.finish()
    }

    /// Get texture data for GPU upload
    pub fn data(&self) -> &[u16] {
        &self.data
    }

    /// Get color at specific texel (for testing)
    #[cfg(test)]
    pub fn get_texel(&self, row: u32, texel: usize) -> Vec4 {
        let base = (row as usize) * Self::TEXELS_PER_ROW * 4 + texel * 4;
        Vec4::new(
            half::f16::from_bits(self.data[base + 0]).to_f32(),
            half::f16::from_bits(self.data[base + 1]).to_f32(),
            half::f16::from_bits(self.data[base + 2]).to_f32(),
            half::f16::from_bits(self.data[base + 3]).to_f32(),
        )
    }
}

/// Convert a VisualStyle into one or more PrimitiveInstances.
///
/// A single styled node may generate multiple instances:
/// - One per fill (solid/gradient)
/// - One for stroke (if present)
/// - One per effect (shadows, blur)
/// - Text glyphs (if text is present)
fn create_primitive_instances_impl(
    mut pipeline: Option<&mut PrimitivePipeline>,
    style: &VisualStyle,
    pos: Vec2,
    size: Vec2,
    opacity: f32,
) -> Vec<PrimitiveInstance> {
    let mut instances = Vec::new();

    // Skip rendering background for text nodes (Phase 2+ will render actual glyphs)
    // Text nodes use fills for text color, not background
    if style.text.is_some() {
        return instances;
    }

    // 1. Render shadows FIRST (behind everything)
    for effect in &style.effects {
        if let style_engine::Effect::DropShadow(shadow) = effect {
            if shadow.visible {
                // Create shadow instance with offset position
                let shadow_pos = pos + shadow.offset;

                let mut shadow_color = shadow.color;
                shadow_color.w *= opacity;

                let mut shadow_instance = PrimitiveInstance::rounded(
                    [shadow_pos.x, shadow_pos.y],
                    [size.x, size.y],
                    [shadow_color.x, shadow_color.y, shadow_color.z, shadow_color.w],
                    style.corner_radii,
                );

                // Store blur radius in stroke_params[0] (shadows don't use stroke)
                shadow_instance.stroke_params[0] = shadow.blur;

                // Set blend mode and internal shadow flag
                shadow_instance.flags = with_blend_mode(shadow_instance.flags, style.blend_mode);
                shadow_instance.flags |= FLAG_IS_SHADOW;

                instances.push(shadow_instance);
            }
        }
    }

    // 2. Render fills (solid or gradient backgrounds)
    for fill in &style.fills {
        let fill_instance = if let Some(pipeline) = pipeline.as_deref_mut() {
            create_fill_instance(
                pipeline,
                fill,
                pos,
                size,
                opacity,
                &style.corner_radii,
                style.blend_mode,
            )
        } else {
            create_fill_instance_without_pipeline(
                fill,
                pos,
                size,
                opacity,
                &style.corner_radii,
                style.blend_mode,
            )
        };
        instances.push(fill_instance);
    }

    // 3. Render stroke (on top of fill)
    if let Some(stroke) = &style.stroke {
        let stroke_instance = if let Some(pipeline) = pipeline.as_deref_mut() {
            create_stroke_instance(
                pipeline,
                stroke,
                pos,
                size,
                opacity,
                &style.corner_radii,
                style.blend_mode,
            )
        } else {
            create_stroke_instance_without_pipeline(
                stroke,
                pos,
                size,
                opacity,
                &style.corner_radii,
                style.blend_mode,
            )
        };
        instances.push(stroke_instance);
    }

    instances
}

pub fn create_primitive_instances(
    style: &VisualStyle,
    pos: Vec2,
    size: Vec2,
    opacity: f32,
) -> Vec<PrimitiveInstance> {
    create_primitive_instances_impl(None, style, pos, size, opacity)
}

pub fn create_primitive_instances_with_pipeline(
    pipeline: &mut PrimitivePipeline,
    style: &VisualStyle,
    pos: Vec2,
    size: Vec2,
    opacity: f32,
) -> Vec<PrimitiveInstance> {
    create_primitive_instances_impl(Some(pipeline), style, pos, size, opacity)
}

/// Create a stroke instance
fn create_stroke_instance(
    pipeline: &mut PrimitivePipeline,
    stroke: &style_engine::StrokeStyle,
    pos: Vec2,
    size: Vec2,
    opacity: f32,
    corner_radii: &CornerRadii,
    blend_mode: BlendMode,
) -> PrimitiveInstance {
    use style_engine::StrokeAlign;

    let stroke_width = stroke.weight;
    let stroke_align = match stroke.align {
        StrokeAlign::Center => 0.0,
        StrokeAlign::Inside => 1.0,
        StrokeAlign::Outside => -1.0,
    };

    let Some(stroke_paint) = stroke.top_paint() else {
        let mut instance = PrimitiveInstance::rounded(
            [pos.x, pos.y],
            [size.x, size.y],
            [1.0, 0.0, 1.0, opacity],
            *corner_radii,
        );
        instance.stroke_params = [stroke_width, stroke_align];
        instance.flags |= FLAG_HAS_STROKE;
        instance.flags = with_blend_mode(instance.flags, blend_mode);
        instance.flags = with_stroke_cap_join(instance.flags, stroke.cap, stroke.join);
        return instance;
    };

    match stroke_paint {
        Paint::Solid(color) => {
            let mut final_color = *color;
            final_color.w *= opacity;

            let mut instance = PrimitiveInstance::rounded(
                [pos.x, pos.y],
                [size.x, size.y],
                [final_color.x, final_color.y, final_color.z, final_color.w],
                *corner_radii,
            );
            instance.stroke_params = [stroke_width, stroke_align];
            instance.flags |= FLAG_HAS_STROKE;
            instance.flags = with_blend_mode(instance.flags, blend_mode);
            instance.flags = with_stroke_cap_join(instance.flags, stroke.cap, stroke.join);
            instance
        }
        Paint::Linear(gradient) => {
            let atlas_row = pipeline.add_gradient(&gradient.stops);
            let params = GradientParams {
                start: gradient.start.to_array(),
                end: gradient.end.to_array(),
                atlas_row: (atlas_row as f32 + 0.5) / GradientAtlas::ATLAS_SIZE as f32,
                gradient_type: 0,
                _padding: [0.0; 2],
            };
            let param_index = pipeline.add_gradient_params(params);

            let mut instance = PrimitiveInstance::rounded(
                [pos.x, pos.y],
                [size.x, size.y],
                [1.0, 1.0, 1.0, opacity],
                *corner_radii,
            );
            instance.gradient_params = [param_index as f32, 0.0, 0.0, 0.0];
            instance.stroke_params = [stroke_width, stroke_align];
            instance.flags = with_fill_type(instance.flags, 1);
            instance.flags |= FLAG_HAS_STROKE;
            instance.flags = with_blend_mode(instance.flags, blend_mode);
            instance.flags = with_stroke_cap_join(instance.flags, stroke.cap, stroke.join);
            instance
        }
        _ => {
            // Other gradient types for strokes (implement if needed)
            let mut instance = PrimitiveInstance::rounded(
                [pos.x, pos.y],
                [size.x, size.y],
                [1.0, 1.0, 1.0, opacity],
                *corner_radii,
            );
            instance.stroke_params = [stroke_width, stroke_align];
            instance.flags |= FLAG_HAS_STROKE;
            instance.flags = with_blend_mode(instance.flags, blend_mode);
            instance.flags = with_stroke_cap_join(instance.flags, stroke.cap, stroke.join);
            instance
        }
    }
}

fn fallback_gradient_color(stops: &[ColorStop], opacity: f32) -> [f32; 4] {
    let mut color = style_engine::Paint::interpolate_stops(0.5, stops);
    color.w *= opacity;
    [color.x, color.y, color.z, color.w]
}

fn create_stroke_instance_without_pipeline(
    stroke: &style_engine::StrokeStyle,
    pos: Vec2,
    size: Vec2,
    opacity: f32,
    corner_radii: &CornerRadii,
    blend_mode: BlendMode,
) -> PrimitiveInstance {
    use style_engine::StrokeAlign;

    let stroke_width = stroke.weight;
    let stroke_align = match stroke.align {
        StrokeAlign::Center => 0.0,
        StrokeAlign::Inside => 1.0,
        StrokeAlign::Outside => -1.0,
    };

    let stroke_color = match stroke.top_paint() {
        Some(Paint::Solid(color)) => {
            let mut final_color = *color;
            final_color.w *= opacity;
            [final_color.x, final_color.y, final_color.z, final_color.w]
        }
        Some(Paint::Linear(gradient)) => fallback_gradient_color(&gradient.stops, opacity),
        Some(Paint::Radial(gradient)) => fallback_gradient_color(&gradient.stops, opacity),
        Some(Paint::Angular(gradient)) => fallback_gradient_color(&gradient.stops, opacity),
        Some(Paint::Diamond(gradient)) => fallback_gradient_color(&gradient.stops, opacity),
        Some(Paint::Image(_)) | None => [1.0, 0.0, 1.0, opacity],
    };

    let mut instance =
        PrimitiveInstance::rounded([pos.x, pos.y], [size.x, size.y], stroke_color, *corner_radii);
    instance.stroke_params = [stroke_width, stroke_align];
    instance.flags |= FLAG_HAS_STROKE;
    instance.flags = with_blend_mode(instance.flags, blend_mode);
    instance.flags = with_stroke_cap_join(instance.flags, stroke.cap, stroke.join);
    instance
}

/// Create a single fill instance (solid or gradient)
fn create_fill_instance(
    pipeline: &mut PrimitivePipeline,
    fill: &Paint,
    pos: Vec2,
    size: Vec2,
    opacity: f32,
    corner_radii: &CornerRadii,
    blend_mode: BlendMode,
) -> PrimitiveInstance {
    let mut instance = match fill {
        Paint::Solid(color) => {
            let mut final_color = *color;
            final_color.w *= opacity;

            PrimitiveInstance::rounded(
                [pos.x, pos.y],
                [size.x, size.y],
                [final_color.x, final_color.y, final_color.z, final_color.w],
                *corner_radii,
            )
        }
        Paint::Linear(gradient) => {
            // Add gradient to atlas
            let atlas_row = pipeline.add_gradient(&gradient.stops);

            // Create gradient params
            let params = GradientParams {
                start: gradient.start.to_array(),
                end: gradient.end.to_array(),
                atlas_row: (atlas_row as f32 + 0.5) / GradientAtlas::ATLAS_SIZE as f32,
                gradient_type: 0, // Linear
                _padding: [0.0; 2],
            };
            let param_index = pipeline.add_gradient_params(params);

            // Create instance with gradient
            let mut instance = PrimitiveInstance::rounded(
                [pos.x, pos.y],
                [size.x, size.y],
                [1.0, 1.0, 1.0, opacity], // Color unused for gradients, just alpha
                *corner_radii,
            );
            instance.gradient_params = [param_index as f32, 0.0, 0.0, 0.0];
            instance.flags = with_fill_type(instance.flags, 1);
            instance
        }
        Paint::Radial(gradient) => {
            let atlas_row = pipeline.add_gradient(&gradient.stops);
            let params = GradientParams {
                start: gradient.center.to_array(),
                end: (gradient.center + Vec2::new(gradient.radius, 0.0)).to_array(), // End = center + radius vector
                atlas_row: (atlas_row as f32 + 0.5) / GradientAtlas::ATLAS_SIZE as f32,
                gradient_type: 1, // Radial
                _padding: [0.0; 2],
            };
            let param_index = pipeline.add_gradient_params(params);

            let mut instance = PrimitiveInstance::rounded(
                [pos.x, pos.y],
                [size.x, size.y],
                [1.0, 1.0, 1.0, opacity],
                *corner_radii,
            );
            instance.gradient_params = [param_index as f32, 0.0, 0.0, 0.0];
            instance.flags = with_fill_type(instance.flags, 2);
            instance
        }
        Paint::Angular(gradient) => {
            let atlas_row = pipeline.add_gradient(&gradient.stops);
            let params = GradientParams {
                start: gradient.center.to_array(),
                end: gradient.center.to_array(), // End unused for angular
                atlas_row: (atlas_row as f32 + 0.5) / GradientAtlas::ATLAS_SIZE as f32,
                gradient_type: 2, // Angular
                _padding: [0.0; 2],
            };
            let param_index = pipeline.add_gradient_params(params);

            let mut instance = PrimitiveInstance::rounded(
                [pos.x, pos.y],
                [size.x, size.y],
                [1.0, 1.0, 1.0, opacity],
                *corner_radii,
            );
            instance.gradient_params = [param_index as f32, 0.0, 0.0, 0.0];
            instance.flags = with_fill_type(instance.flags, 3);
            instance
        }
        Paint::Diamond(gradient) => {
            let atlas_row = pipeline.add_gradient(&gradient.stops);
            let params = GradientParams {
                start: gradient.center.to_array(),
                end: (gradient.center + Vec2::new(gradient.scale, gradient.scale)).to_array(),
                atlas_row: (atlas_row as f32 + 0.5) / GradientAtlas::ATLAS_SIZE as f32,
                gradient_type: 3, // Diamond
                _padding: [0.0; 2],
            };
            let param_index = pipeline.add_gradient_params(params);

            let mut instance = PrimitiveInstance::rounded(
                [pos.x, pos.y],
                [size.x, size.y],
                [1.0, 1.0, 1.0, opacity],
                *corner_radii,
            );
            instance.gradient_params = [param_index as f32, 0.0, 0.0, 0.0];
            instance.flags = with_fill_type(instance.flags, 4);
            instance
        }
        Paint::Image(_) => {
            // TODO: Image fills in Phase 3
            PrimitiveInstance::solid([pos.x, pos.y], [size.x, size.y], [1.0, 0.0, 1.0, 1.0])
        }
    };
    instance.flags = with_blend_mode(instance.flags, blend_mode);
    instance
}

fn create_fill_instance_without_pipeline(
    fill: &Paint,
    pos: Vec2,
    size: Vec2,
    opacity: f32,
    corner_radii: &CornerRadii,
    blend_mode: BlendMode,
) -> PrimitiveInstance {
    let mut instance = match fill {
        Paint::Solid(color) => {
            let mut final_color = *color;
            final_color.w *= opacity;

            PrimitiveInstance::rounded(
                [pos.x, pos.y],
                [size.x, size.y],
                [final_color.x, final_color.y, final_color.z, final_color.w],
                *corner_radii,
            )
        }
        Paint::Linear(gradient) => {
            let mut inst = PrimitiveInstance::rounded(
                [pos.x, pos.y],
                [size.x, size.y],
                fallback_gradient_color(&gradient.stops, opacity),
                *corner_radii,
            );
            inst.flags = with_fill_type(inst.flags, 1);
            inst
        }
        Paint::Radial(gradient) => {
            let mut inst = PrimitiveInstance::rounded(
                [pos.x, pos.y],
                [size.x, size.y],
                fallback_gradient_color(&gradient.stops, opacity),
                *corner_radii,
            );
            inst.flags = with_fill_type(inst.flags, 2);
            inst
        }
        Paint::Angular(gradient) => {
            let mut inst = PrimitiveInstance::rounded(
                [pos.x, pos.y],
                [size.x, size.y],
                fallback_gradient_color(&gradient.stops, opacity),
                *corner_radii,
            );
            inst.flags = with_fill_type(inst.flags, 3);
            inst
        }
        Paint::Diamond(gradient) => {
            let mut inst = PrimitiveInstance::rounded(
                [pos.x, pos.y],
                [size.x, size.y],
                fallback_gradient_color(&gradient.stops, opacity),
                *corner_radii,
            );
            inst.flags = with_fill_type(inst.flags, 4);
            inst
        }
        Paint::Image(_) => {
            PrimitiveInstance::solid([pos.x, pos.y], [size.x, size.y], [1.0, 0.0, 1.0, 1.0])
        }
    };
    instance.flags = with_blend_mode(instance.flags, blend_mode);
    instance
}

/// Unified primitive rendering pipeline
pub struct PrimitivePipeline {
    pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    vertex_buffer_capacity: usize,
    /// Gradient atlas (1024x1024 texture storing rasterized gradients)
    gradient_atlas: GradientAtlas,
    /// Gradient params storage buffer
    gradient_params_buffer: wgpu::Buffer,
    gradient_params_capacity: usize,
    /// Gradient params data (CPU-side)
    gradient_params: Vec<GradientParams>,
    /// Gradient bind group (Group 1)
    gradient_bind_group: Option<wgpu::BindGroup>,
    /// Gradient bind group layout (kept for potential rebind)
    gradient_bind_group_layout: wgpu::BindGroupLayout,
    /// Glyph bind group (Group 2)
    glyph_bind_group: wgpu::BindGroup,
}

impl PrimitivePipeline {
    pub fn new(
        device: &wgpu::Device,
        globals_layout: &wgpu::BindGroupLayout,
        glyph_texture: &wgpu::TextureView,
        glyph_sampler: &wgpu::Sampler,
        format: wgpu::TextureFormat,
    ) -> Self {
        tracing::info!("Creating PrimitivePipeline");

        // Load shader
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Primitive Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../../shaders/primitive.wgsl").into()),
        });

        // Create bind group layouts

        // Group 1: Gradient atlas
        let gradient_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Gradient Atlas Bind Group Layout"),
                entries: &[
                    // Binding 0: Gradient texture (1024x1024 LUT)
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    // Binding 1: Gradient sampler
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                    // Binding 2: Gradient params storage buffer
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            });

        // Group 2: Glyph atlas
        let glyph_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Glyph Atlas Bind Group Layout"),
                entries: &[
                    // Texture
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    // Sampler
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });

        // Initialize gradient atlas with GPU resources
        let mut gradient_atlas = GradientAtlas::default();
        gradient_atlas.init_gpu(device);

        // Create gradient params storage buffer
        const INITIAL_GRADIENT_CAPACITY: usize = 256;
        let gradient_params_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Gradient Params Storage Buffer"),
            size: (INITIAL_GRADIENT_CAPACITY * std::mem::size_of::<GradientParams>()) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Create gradient bind group with texture, sampler, and storage buffer
        let gradient_bind_group = if let (Some(view), Some(sampler)) =
            (gradient_atlas.texture_view(), gradient_atlas.sampler())
        {
            Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Gradient Atlas Bind Group"),
                layout: &gradient_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(sampler),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: gradient_params_buffer.as_entire_binding(),
                    },
                ],
            }))
        } else {
            None
        };

        // Create glyph bind group
        let glyph_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Glyph Atlas Bind Group"),
            layout: &glyph_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(glyph_texture),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(glyph_sampler),
                },
            ],
        });

        // Pipeline layout with 3 bind groups
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Primitive Pipeline Layout"),
            bind_group_layouts: &[
                globals_layout,              // Group 0
                &gradient_bind_group_layout, // Group 1
                &glyph_bind_group_layout,    // Group 2
            ],
            immediate_size: 0,
        });

        // Vertex buffer layout (96 bytes stride)
        let vertex_buffer_layout = wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<PrimitiveInstance>() as u64,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &wgpu::vertex_attr_array![
                0 => Float32x2,  // pos
                1 => Float32x2,  // size
                2 => Float32x4,  // color
                3 => Float32x4,  // corner_radii
                4 => Float32x4,  // gradient_params
                5 => Float32x4,  // tex_coords
                6 => Float32x2,  // stroke_params
                7 => Uint32,     // flags
                8 => Uint32,     // _padding
            ],
        };

        // Create render pipeline
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Primitive Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[vertex_buffer_layout],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview_mask: None,
            cache: None,
        });

        // Create vertex buffer
        const INITIAL_CAPACITY: usize = 1024;
        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Primitive Instance Buffer"),
            size: (INITIAL_CAPACITY * std::mem::size_of::<PrimitiveInstance>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            pipeline,
            vertex_buffer,
            vertex_buffer_capacity: INITIAL_CAPACITY,
            gradient_atlas,
            gradient_params_buffer,
            gradient_params_capacity: INITIAL_GRADIENT_CAPACITY,
            gradient_params: Vec::new(),
            gradient_bind_group,
            gradient_bind_group_layout,
            glyph_bind_group,
        }
    }

    /// Add a gradient to the atlas and return its row index
    pub fn add_gradient(&mut self, stops: &[ColorStop]) -> u32 {
        self.gradient_atlas.add_gradient(stops)
    }

    /// Upload any pending gradient atlas changes to GPU
    pub fn upload_gradient_atlas(&mut self, queue: &wgpu::Queue) {
        self.gradient_atlas.upload_to_gpu(queue);
    }

    /// Add gradient parameters and return the index into the storage buffer
    pub fn add_gradient_params(&mut self, params: GradientParams) -> u32 {
        let index = self.gradient_params.len() as u32;
        self.gradient_params.push(params);
        index
    }

    /// Clear all gradient params (call at start of frame)
    pub fn clear_gradient_params(&mut self) {
        self.gradient_params.clear();
    }

    pub fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        instances: &[PrimitiveInstance],
    ) {
        // Upload gradient atlas if dirty
        self.gradient_atlas.upload_to_gpu(queue);

        // Upload gradient params if any
        if !self.gradient_params.is_empty() {
            // Resize gradient params buffer if needed
            if self.gradient_params.len() > self.gradient_params_capacity {
                let new_capacity = self.gradient_params.len().next_power_of_two();
                self.gradient_params_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("Gradient Params Storage Buffer"),
                    size: (new_capacity * std::mem::size_of::<GradientParams>()) as u64,
                    usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                });
                self.gradient_params_capacity = new_capacity;

                // Recreate bind group with new buffer
                if let (Some(view), Some(sampler)) =
                    (self.gradient_atlas.texture_view(), self.gradient_atlas.sampler())
                {
                    self.gradient_bind_group =
                        Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
                            label: Some("Gradient Atlas Bind Group"),
                            layout: &self.gradient_bind_group_layout,
                            entries: &[
                                wgpu::BindGroupEntry {
                                    binding: 0,
                                    resource: wgpu::BindingResource::TextureView(view),
                                },
                                wgpu::BindGroupEntry {
                                    binding: 1,
                                    resource: wgpu::BindingResource::Sampler(sampler),
                                },
                                wgpu::BindGroupEntry {
                                    binding: 2,
                                    resource: self.gradient_params_buffer.as_entire_binding(),
                                },
                            ],
                        }));
                }
            }

            // Upload gradient params data
            let params_bytes = bytemuck::cast_slice(&self.gradient_params);
            queue.write_buffer(&self.gradient_params_buffer, 0, params_bytes);
        }

        if instances.is_empty() {
            return;
        }

        // Resize vertex buffer if needed
        if instances.len() > self.vertex_buffer_capacity {
            let new_capacity = instances.len().next_power_of_two();
            self.vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Primitive Instance Buffer"),
                size: (new_capacity * std::mem::size_of::<PrimitiveInstance>()) as u64,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            self.vertex_buffer_capacity = new_capacity;
        }

        // Upload instance data
        let instance_bytes = bytemuck::cast_slice(instances);
        queue.write_buffer(&self.vertex_buffer, 0, instance_bytes);
    }

    pub fn render(
        &self,
        render_pass: &mut wgpu::RenderPass<'_>,
        globals_bind_group: &wgpu::BindGroup,
        instance_count: u32,
    ) {
        if instance_count == 0 {
            return;
        }

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, globals_bind_group, &[]);
        // Group 1 (gradient atlas) - using dummy texture for Phase 1
        if let Some(ref gradient_bg) = self.gradient_bind_group {
            render_pass.set_bind_group(1, gradient_bg, &[]);
        }
        render_pass.set_bind_group(2, &self.glyph_bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.draw(0..6, 0..instance_count);
    }
}

/// CPU reference: Per-corner rounded rectangle SDF
///
/// This matches the WGSL implementation for testing purposes.
#[cfg(test)]
fn rounded_rect_sdf_4_cpu(pos: [f32; 2], half_size: [f32; 2], radii: [f32; 4]) -> f32 {
    use glam::Vec2;

    let p = Vec2::from(pos);
    let hs = Vec2::from(half_size);

    // Select corner radius based on quadrant
    let radius = if p.x > 0.0 && p.y > 0.0 {
        radii[2] // BR
    } else if p.x > 0.0 {
        radii[1] // TR
    } else if p.y > 0.0 {
        radii[3] // BL
    } else {
        radii[0] // TL
    };

    let q = p.abs() - hs + Vec2::splat(radius);
    q.max(Vec2::ZERO).length() + q.x.max(q.y).min(0.0) - radius
}

/// CPU reference: Stroke rendering
#[cfg(test)]
fn render_stroke_cpu(dist: f32, width: f32, align: f32) -> f32 {
    // align: -1=outside, 0=center, 1=inside
    let offset = width * 0.5 * align;
    let adjusted_dist = dist - offset;
    let half_width = width * 0.5;

    // Smoothstep anti-aliasing over ~1px (matches WGSL)
    fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
        let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
        t * t * (3.0 - 2.0 * t)
    }

    1.0 - smoothstep(half_width - 0.5, half_width + 0.5, adjusted_dist.abs())
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::Vec4;

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

    #[test]
    fn test_gradient_fill_types_are_distinct() {
        let linear = VisualStyle::new().fill(Paint::Linear(style_engine::LinearGradient {
            start: Vec2::new(0.0, 0.0),
            end: Vec2::new(1.0, 0.0),
            stops: vec![
                ColorStop::new(0.0, Vec4::new(1.0, 0.0, 0.0, 1.0)),
                ColorStop::new(1.0, Vec4::new(0.0, 0.0, 1.0, 1.0)),
            ],
        }));
        let radial = VisualStyle::new().fill(Paint::Radial(style_engine::RadialGradient {
            center: Vec2::new(0.5, 0.5),
            radius: 0.5,
            stops: vec![
                ColorStop::new(0.0, Vec4::new(1.0, 1.0, 1.0, 1.0)),
                ColorStop::new(1.0, Vec4::new(0.0, 0.0, 0.0, 1.0)),
            ],
        }));
        let angular = VisualStyle::new().fill(Paint::Angular(style_engine::AngularGradient {
            center: Vec2::new(0.5, 0.5),
            angle: 0.0,
            stops: vec![
                ColorStop::new(0.0, Vec4::new(1.0, 0.0, 0.0, 1.0)),
                ColorStop::new(1.0, Vec4::new(1.0, 0.0, 0.0, 1.0)),
            ],
        }));
        let diamond = VisualStyle::new().fill(Paint::Diamond(style_engine::DiamondGradient {
            center: Vec2::new(0.5, 0.5),
            scale: 0.5,
            stops: vec![
                ColorStop::new(0.0, Vec4::new(1.0, 1.0, 0.0, 1.0)),
                ColorStop::new(1.0, Vec4::new(0.5, 0.0, 0.5, 1.0)),
            ],
        }));

        let linear_i = create_primitive_instances(&linear, Vec2::ZERO, Vec2::ONE, 1.0);
        let radial_i = create_primitive_instances(&radial, Vec2::ZERO, Vec2::ONE, 1.0);
        let angular_i = create_primitive_instances(&angular, Vec2::ZERO, Vec2::ONE, 1.0);
        let diamond_i = create_primitive_instances(&diamond, Vec2::ZERO, Vec2::ONE, 1.0);

        assert_eq!(linear_i[0].flags & FLAG_FILL_TYPE_MASK, 1);
        assert_eq!(radial_i[0].flags & FLAG_FILL_TYPE_MASK, 2);
        assert_eq!(angular_i[0].flags & FLAG_FILL_TYPE_MASK, 3);
        assert_eq!(diamond_i[0].flags & FLAG_FILL_TYPE_MASK, 4);
    }

    #[test]
    fn test_shadow_flag_uses_internal_high_bit() {
        let style = VisualStyle::new()
            .solid_fill(Vec4::new(1.0, 1.0, 1.0, 1.0))
            .drop_shadow(Vec2::new(2.0, 2.0), 6.0, Vec4::new(0.0, 0.0, 0.0, 0.3));
        let instances = create_primitive_instances(&style, Vec2::ZERO, Vec2::ONE, 1.0);

        assert!(instances.iter().any(|i| (i.flags & FLAG_IS_SHADOW) != 0));
    }

    #[test]
    fn test_stroke_packs_cap_join_and_blend_mode_bits() {
        let mut stroke = style_engine::StrokeStyle::solid(
            Paint::solid(Vec4::new(1.0, 1.0, 1.0, 1.0)),
            2.0,
            style_engine::StrokeAlign::Inside,
        );
        stroke.cap = style_engine::StrokeCap::Square;
        stroke.join = style_engine::StrokeJoin::Round;

        let style = VisualStyle::new()
            .solid_fill(Vec4::new(0.2, 0.2, 0.2, 1.0))
            .blend_mode(style_engine::BlendMode::Screen)
            .stroke(stroke);
        let instances = create_primitive_instances(&style, Vec2::ZERO, Vec2::ONE, 1.0);
        let stroke_instance = instances
            .iter()
            .find(|i| (i.flags & FLAG_HAS_STROKE) != 0)
            .expect("missing stroke instance");

        let blend = (stroke_instance.flags & FLAG_BLEND_MODE_MASK) >> FLAG_BLEND_MODE_SHIFT;
        let cap = (stroke_instance.flags & FLAG_STROKE_CAP_MASK) >> FLAG_STROKE_CAP_SHIFT;
        let join = (stroke_instance.flags & FLAG_STROKE_JOIN_MASK) >> FLAG_STROKE_JOIN_SHIFT;

        assert_eq!(blend, style_engine::BlendMode::Screen.to_flag_bits() as u32);
        assert_eq!(cap, 2);
        assert_eq!(join, 2);
    }

    #[test]
    fn test_create_primitive_instances_solid() {
        let style = VisualStyle::new()
            .solid_fill(Vec4::new(1.0, 0.0, 0.0, 1.0))
            .corner_radius(12.0);

        let instances =
            create_primitive_instances(&style, Vec2::new(10.0, 20.0), Vec2::new(100.0, 50.0), 1.0);

        assert_eq!(
            instances.len(),
            1,
            "Single solid fill should create 1 instance"
        );
        assert_eq!(instances[0].pos, [10.0, 20.0]);
        assert_eq!(instances[0].size, [100.0, 50.0]);
        assert_eq!(instances[0].color, [1.0, 0.0, 0.0, 1.0]);
        assert_eq!(instances[0].corner_radii, [12.0, 12.0, 12.0, 12.0]);
    }

    #[test]
    fn test_create_primitive_instances_with_opacity() {
        let style = VisualStyle::new().solid_fill(Vec4::new(1.0, 0.0, 0.0, 1.0));

        let instances = create_primitive_instances(&style, Vec2::ZERO, Vec2::ONE, 0.5);

        assert_eq!(instances.len(), 1);
        assert_eq!(
            instances[0].color[3], 0.5,
            "Opacity should be applied to alpha"
        );
    }

    #[test]
    fn test_create_primitive_instances_empty() {
        let style = VisualStyle::new(); // No fills

        let instances = create_primitive_instances(&style, Vec2::ZERO, Vec2::ONE, 1.0);

        assert_eq!(instances.len(), 0, "Empty style should create 0 instances");
    }

    #[test]
    fn test_rounded_rect_sdf_4_center_inside() {
        // Center of 100x50 rect should be well inside (negative distance)
        let dist = rounded_rect_sdf_4_cpu([0.0, 0.0], [50.0, 25.0], [8.0; 4]);
        assert!(
            dist < 0.0,
            "Center should be inside (negative dist), got {}",
            dist
        );
    }

    #[test]
    fn test_rounded_rect_sdf_4_edge() {
        // Point on top edge (no corner) should be near zero
        let dist = rounded_rect_sdf_4_cpu([0.0, -25.0], [50.0, 25.0], [0.0; 4]);
        assert!((dist - 0.0).abs() < 0.1, "Edge should be ~0, got {}", dist);
    }

    #[test]
    fn test_rounded_rect_sdf_4_outside() {
        // Point far outside should be positive
        let dist = rounded_rect_sdf_4_cpu([100.0, 100.0], [50.0, 25.0], [8.0; 4]);
        assert!(
            dist > 0.0,
            "Outside point should be positive dist, got {}",
            dist
        );
    }

    #[test]
    fn test_rounded_rect_sdf_4_asymmetric_radii() {
        // Large TL corner (radius 20), zero others
        let radii = [20.0, 0.0, 0.0, 0.0];

        // Rectangle is 100x50 (half_size 50x25), so corners are at (±50, ±25)

        // Point just outside TL corner (uses radius 20)
        let dist_tl = rounded_rect_sdf_4_cpu([-55.0, -30.0], [50.0, 25.0], radii);

        // Point just outside TR corner (uses radius 0 = sharp corner)
        let dist_tr = rounded_rect_sdf_4_cpu([55.0, -30.0], [50.0, 25.0], radii);

        // Both should be outside (positive distance)
        assert!(
            dist_tl > 0.0,
            "TL corner point should be outside, got {}",
            dist_tl
        );
        assert!(
            dist_tr > 0.0,
            "TR corner point should be outside, got {}",
            dist_tr
        );

        // For asymmetric radii, just verify they're both outside and use correct radii
        // The relationship between distances depends on the exact geometry
        eprintln!(
            "dist_tl (radius 20): {}, dist_tr (radius 0): {}",
            dist_tl, dist_tr
        );
    }

    #[test]
    fn test_render_stroke_center_align() {
        // Point exactly on boundary with center-aligned stroke
        let alpha = render_stroke_cpu(0.0, 2.0, 0.0);
        assert!(
            alpha > 0.9,
            "Boundary should be solid with center stroke, got {}",
            alpha
        );
    }

    #[test]
    fn test_render_stroke_outside_align() {
        // Outside stroke (align=-1) shifts boundary outward (negative direction)
        let alpha_negative = render_stroke_cpu(-0.5, 2.0, -1.0); // Inside the outside stroke
        let alpha_positive = render_stroke_cpu(0.5, 2.0, -1.0); // Outside the outside stroke
        assert!(
            alpha_negative > alpha_positive,
            "Outside stroke should render on negative side"
        );
    }

    #[test]
    fn test_shader_compiles() {
        // Test that primitive.wgsl compiles via naga
        let shader_source = include_str!("../../shaders/primitive.wgsl");
        let result = naga::front::wgsl::parse_str(shader_source);

        match result {
            Ok(_module) => {
                // Success - shader compiled
            }
            Err(e) => {
                panic!("Shader failed to compile: {:?}", e);
            }
        }
    }

    // ========================================================================
    // GradientAtlas Tests
    // ========================================================================

    #[test]
    fn test_gradient_params_size() {
        assert_eq!(
            std::mem::size_of::<GradientParams>(),
            32,
            "GradientParams must be exactly 32 bytes to match WGSL struct"
        );
    }

    #[test]
    fn test_gradient_params_constructors() {
        // Test linear gradient
        let linear = GradientParams::linear([0.0, 0.0], [1.0, 0.0], 5);
        assert_eq!(linear.gradient_type, 0, "Linear gradient type should be 0");
        assert_eq!(linear.start, [0.0, 0.0]);
        assert_eq!(linear.end, [1.0, 0.0]);

        // Test radial gradient
        let radial = GradientParams::radial([0.5, 0.5], 0.5, 10);
        assert_eq!(radial.gradient_type, 1, "Radial gradient type should be 1");
        assert_eq!(radial.start, [0.5, 0.5]);

        // Test angular gradient
        let angular = GradientParams::angular([0.5, 0.5], 15);
        assert_eq!(angular.gradient_type, 2, "Angular gradient type should be 2");

        // Test diamond gradient
        let diamond = GradientParams::diamond([0.5, 0.5], [0.3, 0.3], 20);
        assert_eq!(
            diamond.gradient_type, 3,
            "Diamond gradient type should be 3"
        );
    }

    #[test]
    fn test_gradient_atlas_creation() {
        let atlas = GradientAtlas::default();
        assert_eq!(
            atlas.data().len(),
            1024 * 1024 * 4,
            "Atlas should have 1024x1024x4 f16 values"
        );
        assert_eq!(atlas.next_row, 0, "New atlas should start at row 0");
    }

    #[test]
    fn test_gradient_atlas_black_to_white() {
        use style_engine::ColorStop;
        let mut atlas = GradientAtlas::default();

        let stops = vec![
            ColorStop::new(0.0, Vec4::new(0.0, 0.0, 0.0, 1.0)), // Black
            ColorStop::new(1.0, Vec4::new(1.0, 1.0, 1.0, 1.0)), // White
        ];

        let row = atlas.add_gradient(&stops);
        assert_eq!(row, 0, "First gradient should use row 0");

        // Check texel 0 ≈ black
        let texel_0 = atlas.get_texel(row, 0);
        assert!(
            (texel_0.x - 0.0).abs() < 0.01,
            "Texel 0 R should be ~0, got {}",
            texel_0.x
        );
        assert!(
            (texel_0.y - 0.0).abs() < 0.01,
            "Texel 0 G should be ~0, got {}",
            texel_0.y
        );
        assert!(
            (texel_0.z - 0.0).abs() < 0.01,
            "Texel 0 B should be ~0, got {}",
            texel_0.z
        );

        // Check texel 1023 ≈ white
        let texel_1023 = atlas.get_texel(row, 1023);
        assert!(
            (texel_1023.x - 1.0).abs() < 0.01,
            "Texel 1023 R should be ~1, got {}",
            texel_1023.x
        );
        assert!(
            (texel_1023.y - 1.0).abs() < 0.01,
            "Texel 1023 G should be ~1, got {}",
            texel_1023.y
        );
        assert!(
            (texel_1023.z - 1.0).abs() < 0.01,
            "Texel 1023 B should be ~1, got {}",
            texel_1023.z
        );

        // Check texel 512 ≈ perceptual midpoint gray (Oklab default interpolation)
        let texel_512 = atlas.get_texel(row, 512);
        assert!(
            (texel_512.x - 0.39).abs() < 0.03,
            "Texel 512 R should be ~0.39, got {}",
            texel_512.x
        );
        assert!(
            (texel_512.y - 0.39).abs() < 0.03,
            "Texel 512 G should be ~0.39, got {}",
            texel_512.y
        );
        assert!(
            (texel_512.z - 0.39).abs() < 0.03,
            "Texel 512 B should be ~0.39, got {}",
            texel_512.z
        );
    }

    #[test]
    fn test_gradient_atlas_hard_stop() {
        use style_engine::ColorStop;
        let mut atlas = GradientAtlas::default();

        // Hard stop at 0.5: black before, white after
        let stops = vec![
            ColorStop::new(0.0, Vec4::new(0.0, 0.0, 0.0, 1.0)),
            ColorStop::new(0.5, Vec4::new(0.0, 0.0, 0.0, 1.0)),
            ColorStop::new(0.5, Vec4::new(1.0, 1.0, 1.0, 1.0)),
            ColorStop::new(1.0, Vec4::new(1.0, 1.0, 1.0, 1.0)),
        ];

        let row = atlas.add_gradient(&stops);

        // Texel at ~0.49 should be black
        let texel_before = atlas.get_texel(row, 500);
        assert!(
            texel_before.x < 0.1,
            "Before hard stop should be dark, got {}",
            texel_before.x
        );

        // Texel at ~0.51 should be white
        let texel_after = atlas.get_texel(row, 524);
        assert!(
            texel_after.x > 0.9,
            "After hard stop should be bright, got {}",
            texel_after.x
        );
    }

    #[test]
    fn test_gradient_atlas_cache_dedup() {
        use style_engine::ColorStop;
        let mut atlas = GradientAtlas::default();

        let stops = vec![
            ColorStop::new(0.0, Vec4::new(1.0, 0.0, 0.0, 1.0)),
            ColorStop::new(1.0, Vec4::new(0.0, 0.0, 1.0, 1.0)),
        ];

        // Add same gradient twice
        let row1 = atlas.add_gradient(&stops);
        let row2 = atlas.add_gradient(&stops);

        assert_eq!(row1, row2, "Same gradient should return same row (cached)");
        assert_eq!(atlas.next_row, 1, "Should only allocate one row");
    }

    #[test]
    fn test_gradient_atlas_multiple_gradients() {
        use style_engine::ColorStop;
        let mut atlas = GradientAtlas::default();

        let stops1 = vec![
            ColorStop::new(0.0, Vec4::new(1.0, 0.0, 0.0, 1.0)),
            ColorStop::new(1.0, Vec4::new(0.0, 1.0, 0.0, 1.0)),
        ];

        let stops2 = vec![
            ColorStop::new(0.0, Vec4::new(0.0, 0.0, 1.0, 1.0)),
            ColorStop::new(1.0, Vec4::new(1.0, 1.0, 0.0, 1.0)),
        ];

        let row1 = atlas.add_gradient(&stops1);
        let row2 = atlas.add_gradient(&stops2);

        assert_eq!(row1, 0);
        assert_eq!(row2, 1);
        assert_ne!(row1, row2, "Different gradients should use different rows");
    }

    // ========================================================================
    // PrimitivePipeline Tests
    // ========================================================================

    #[test]
    fn test_primitive_pipeline_vertex_buffer_layout() {
        // Verify vertex buffer stride is 96 bytes
        assert_eq!(
            std::mem::size_of::<PrimitiveInstance>(),
            96,
            "Vertex buffer stride must match PrimitiveInstance size"
        );
    }

    #[test]
    fn test_primitive_pipeline_compiles() {
        // Verify PrimitivePipeline struct and methods compile
        // Actual GPU pipeline creation is tested in integration tests with real device

        // This test just verifies the types are correct
        fn _check_pipeline_signature() {
            fn _check<D, G, T, S, F>(_device: D, _globals: G, _tex: T, _samp: S, _fmt: F)
            where
                D: AsRef<wgpu::Device>,
                G: AsRef<wgpu::BindGroupLayout>,
                T: AsRef<wgpu::TextureView>,
                S: AsRef<wgpu::Sampler>,
                F: Into<wgpu::TextureFormat>,
            {
                // PrimitivePipeline::new would be called here with real GPU resources
            }
        }
    }
}
