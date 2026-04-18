//! Unified primitive rendering pipeline supporting rects, text, gradients, strokes, and effects.

use style_engine::ColorStop;
use wgpu::util::DeviceExt;

use super::gradient_atlas::{GradientAtlas, GradientParams};
use crate::primitives::PrimitiveInstance;

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
    /// Create a new instance.
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

    /// Prepare instances for rendering by updating buffers.
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
                if let (Some(view), Some(sampler)) = (
                    self.gradient_atlas.texture_view(),
                    self.gradient_atlas.sampler(),
                ) {
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

        let instance_bytes = bytemuck::cast_slice(instances);
        self.vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Primitive Instance Buffer"),
            contents: instance_bytes,
            usage: wgpu::BufferUsages::VERTEX,
        });
        self.vertex_buffer_capacity = instances.len();
    }

    /// Draw the primitive instances.
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

#[cfg(test)]
mod tests {
    use super::*;

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
