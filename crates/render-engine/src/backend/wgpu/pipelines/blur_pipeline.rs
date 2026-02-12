//! Two-pass separable blur pipeline (Phase 4).

use crate::backend::wgpu::effects::gaussian_kernel_1d;

/// Blur pass direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlurDirection {
    Horizontal,
    Vertical,
}

/// Blur quality tier based on blur radius.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlurTier {
    /// Full-resolution two-pass blur path.
    FullRes,
    /// Half-resolution blur path for large radii.
    HalfRes,
}

/// Select blur tier based on radius.
#[must_use]
pub fn select_blur_tier(radius: f32) -> BlurTier {
    if radius > 24.0 {
        BlurTier::HalfRes
    } else {
        BlurTier::FullRes
    }
}

/// Uniform payload consumed by `blur.wgsl`.
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct BlurParams {
    pub direction: [f32; 2],
    pub texel_size: [f32; 2],
    pub tap_count: u32,
    pub _pad: u32,
    // Match WGSL uniform alignment before packed_weights (offset 32).
    pub _pad2: [u32; 2],
    // WGSL uniform arrays require 16-byte stride; pack scalar weights into vec4 lanes.
    pub packed_weights: [[f32; 4]; 7],
}

impl BlurParams {
    /// Build params from radius, target size, and pass direction.
    #[must_use]
    pub fn from_radius(radius: f32, width: u32, height: u32, direction: BlurDirection) -> Self {
        let kernel = gaussian_kernel_1d(radius, 25);
        let mut packed_weights = [[0.0; 4]; 7];
        for (idx, w) in kernel.iter().copied().enumerate().take(25) {
            packed_weights[idx / 4][idx % 4] = w;
        }

        let direction = match direction {
            BlurDirection::Horizontal => [1.0, 0.0],
            BlurDirection::Vertical => [0.0, 1.0],
        };

        Self {
            direction,
            texel_size: [
                if width == 0 { 0.0 } else { 1.0 / width as f32 },
                if height == 0 {
                    0.0
                } else {
                    1.0 / height as f32
                },
            ],
            tap_count: kernel.len() as u32,
            _pad: 0,
            _pad2: [0; 2],
            packed_weights,
        }
    }
}

/// GPU blur pipeline.
///
/// This is intentionally standalone and can be composed by the backend effect planner.
pub struct BlurPipeline {
    pipeline: wgpu::RenderPipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    params_buffer: wgpu::Buffer,
}

fn blur_color_target_state(format: wgpu::TextureFormat) -> wgpu::ColorTargetState {
    // Blur shader writes the final filtered sample directly, so blend state must be
    // disabled to avoid double-applying source alpha during fullscreen compositing.
    wgpu::ColorTargetState {
        format,
        blend: None,
        write_mask: wgpu::ColorWrites::ALL,
    }
}

impl BlurPipeline {
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Blur Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../../shaders/blur.wgsl").into()),
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Blur Bind Group Layout"),
            entries: &[
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
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: Some(
                            wgpu::BufferSize::new(std::mem::size_of::<BlurParams>() as u64)
                                .expect("valid blur params size"),
                        ),
                    },
                    count: None,
                },
            ],
        });

        let params_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Blur Params Buffer"),
            size: std::mem::size_of::<BlurParams>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Blur Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            immediate_size: 0,
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Blur Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(blur_color_target_state(format))],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        Self {
            pipeline,
            bind_group_layout,
            params_buffer,
        }
    }

    #[must_use]
    pub fn create_bind_group(
        &self,
        device: &wgpu::Device,
        source_view: &wgpu::TextureView,
        source_sampler: &wgpu::Sampler,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Blur Bind Group"),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(source_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(source_sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: self.params_buffer.as_entire_binding(),
                },
            ],
        })
    }

    pub fn update_params(&self, queue: &wgpu::Queue, params: &BlurParams) {
        queue.write_buffer(&self.params_buffer, 0, bytemuck::bytes_of(params));
    }

    pub fn render(&self, render_pass: &mut wgpu::RenderPass<'_>, bind_group: &wgpu::BindGroup) {
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, bind_group, &[]);
        render_pass.draw(0..3, 0..1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blur_params_zero_dimension_texel_size_is_zero() {
        let p = BlurParams::from_radius(8.0, 0, 0, BlurDirection::Horizontal);
        assert_eq!(p.texel_size, [0.0, 0.0]);
    }

    #[test]
    fn test_blur_params_direction() {
        let h = BlurParams::from_radius(8.0, 100, 100, BlurDirection::Horizontal);
        let v = BlurParams::from_radius(8.0, 100, 100, BlurDirection::Vertical);
        assert_eq!(h.direction, [1.0, 0.0]);
        assert_eq!(v.direction, [0.0, 1.0]);
    }

    #[test]
    fn test_blur_params_uniform_size_matches_wgsl() {
        assert_eq!(std::mem::size_of::<BlurParams>(), 144);
    }

    #[test]
    fn test_blur_params_pack_weights() {
        let p = BlurParams::from_radius(8.0, 100, 100, BlurDirection::Horizontal);
        let sum: f32 = p
            .packed_weights
            .iter()
            .flat_map(|v| v.iter())
            .take(p.tap_count as usize)
            .copied()
            .sum();
        assert!((sum - 1.0).abs() < 1.0e-5, "sum={sum}");
    }

    #[test]
    fn test_blur_pipeline_uses_replace_target_state() {
        let state = super::blur_color_target_state(wgpu::TextureFormat::Rgba8UnormSrgb);
        assert!(
            state.blend.is_none(),
            "blur pipeline must write replace output without hardware alpha blending"
        );
    }
}
