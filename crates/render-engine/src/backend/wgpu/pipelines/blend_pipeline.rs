//! Blend compositing pipeline (Phase 4).

use style_engine::BlendMode;

/// Uniform payload consumed by `blend.wgsl`.
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct BlendParams {
    pub blend_mode: u32,
    pub _pad: [u32; 3],
}

impl BlendParams {
    #[must_use]
    pub fn new(mode: BlendMode) -> Self {
        Self {
            blend_mode: mode.to_flag_bits() as u32,
            _pad: [0; 3],
        }
    }
}

/// Whether the shader has an explicit branch for this blend mode.
#[must_use]
pub fn has_explicit_shader_branch(mode: BlendMode) -> bool {
    !matches!(
        mode,
        BlendMode::Normal
            | BlendMode::PassThrough
            | BlendMode::Hue
            | BlendMode::Saturation
            | BlendMode::Color
            | BlendMode::Luminosity
    )
}

/// GPU pipeline for source/destination compositing.
pub struct BlendPipeline {
    pipeline: wgpu::RenderPipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    params_buffer: wgpu::Buffer,
}

fn blend_color_target_state(format: wgpu::TextureFormat) -> wgpu::ColorTargetState {
    // The shader computes final src-over-dst output explicitly, so hardware blending
    // must be disabled to avoid double-applying blend math.
    wgpu::ColorTargetState {
        format,
        blend: None,
        write_mask: wgpu::ColorWrites::ALL,
    }
}

impl BlendPipeline {
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Blend Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../../shaders/blend.wgsl").into()),
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Blend Bind Group Layout"),
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
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: Some(
                            wgpu::BufferSize::new(std::mem::size_of::<BlendParams>() as u64)
                                .expect("valid blend params size"),
                        ),
                    },
                    count: None,
                },
            ],
        });

        let params_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Blend Params Buffer"),
            size: std::mem::size_of::<BlendParams>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Blend Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            immediate_size: 0,
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Blend Pipeline"),
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
                targets: &[Some(blend_color_target_state(format))],
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
        src_view: &wgpu::TextureView,
        dst_view: &wgpu::TextureView,
        sampler: &wgpu::Sampler,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Blend Bind Group"),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(src_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(dst_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: self.params_buffer.as_entire_binding(),
                },
            ],
        })
    }

    pub fn update_params(&self, queue: &wgpu::Queue, params: BlendParams) {
        queue.write_buffer(&self.params_buffer, 0, bytemuck::bytes_of(&params));
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
    fn test_blend_params_encode_mode_bits() {
        let params = BlendParams::new(BlendMode::Overlay);
        assert_eq!(params.blend_mode, BlendMode::Overlay.to_flag_bits() as u32);
    }

    #[test]
    fn test_has_explicit_shader_branch_for_common_modes() {
        assert!(has_explicit_shader_branch(BlendMode::Multiply));
        assert!(has_explicit_shader_branch(BlendMode::Screen));
        assert!(has_explicit_shader_branch(BlendMode::Overlay));
        assert!(!has_explicit_shader_branch(BlendMode::Hue));
    }

    #[test]
    fn test_blend_pipeline_uses_replace_target_state() {
        let state = super::blend_color_target_state(wgpu::TextureFormat::Rgba8UnormSrgb);
        assert!(
            state.blend.is_none(),
            "blend composite pipeline must use replace writes"
        );
    }
}
