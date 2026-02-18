#[cfg(test)]
use glam::Vec4;
use hashbrown::HashMap;
use style_engine::{ColorStop, Paint};

/// Gradient parameters for shader (32 bytes)
///
/// Passed to shader via storage buffer to describe how to sample the gradient atlas.
/// Must match WGSL struct layout exactly (16-byte alignment for vec2/vec3/vec4).
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GradientParams {
    pub start: [f32; 2],    // 8 bytes: Gradient start point (normalized 0-1)
    pub end: [f32; 2],      // 8 bytes: Gradient end point (normalized 0-1)
    pub atlas_row: f32,     // 4 bytes: Row in atlas (normalized v coord)
    pub gradient_type: u32, // 4 bytes: 0=linear, 1=radial, 2=angular, 3=diamond
    pub _padding: [f32; 2], // 8 bytes: Padding to 32 bytes
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
    pub const ATLAS_SIZE: usize = 1024;
    pub const TEXELS_PER_ROW: usize = 1024;

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

#[cfg(test)]
mod tests {
    use super::*;

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
        assert_eq!(
            angular.gradient_type, 2,
            "Angular gradient type should be 2"
        );

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
}
