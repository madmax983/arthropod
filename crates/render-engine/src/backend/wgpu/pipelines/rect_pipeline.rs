//! Rectangle rendering pipeline.

use tracing::{debug, info};
use wgpu;

/// Per-rectangle instance data (uploaded to GPU as vertex attributes)
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct RectInstance {
    pub pos: [f32; 2],      // Position (x, y)
    pub size: [f32; 2],     // Size (width, height)
    pub color: [f32; 4],    // Color (r, g, b, a)
    pub corner_radius: f32, // Corner radius for rounded rectangles (0.0 = sharp)
}

impl RectInstance {
    /// Create a new instance with explicit corner radius.
    #[inline]
    pub fn new(pos: [f32; 2], size: [f32; 2], color: [f32; 4], corner_radius: f32) -> Self {
        Self {
            pos,
            size,
            color,
            corner_radius,
        }
    }

    /// Create a sharp-cornered rectangle (corner_radius = 0.0).
    #[inline]
    pub fn rect(pos: [f32; 2], size: [f32; 2], color: [f32; 4]) -> Self {
        Self::new(pos, size, color, 0.0)
    }
}

pub struct RectPipeline {
    pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    vertex_buffer_capacity: usize,
}

impl RectPipeline {
    pub fn new(
        device: &wgpu::Device,
        globals_layout: &wgpu::BindGroupLayout,
        format: wgpu::TextureFormat,
    ) -> Self {
        info!("Loading shader from rect.wgsl");
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Rect Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../../shaders/rect.wgsl").into()),
        });
        info!("Shader module created successfully");

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[globals_layout],
            immediate_size: 0,
        });

        let vertex_buffer_layout = wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<RectInstance>() as u64,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &wgpu::vertex_attr_array![
                0 => Float32x2,  // pos
                1 => Float32x2,  // size
                2 => Float32x4,  // color
                3 => Float32,    // corner_radius
            ],
        };

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
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
                cull_mode: None, // Disabled culling - negative Y projection flips winding
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

        // Create vertex buffer for rectangle instances (start with capacity for 1024 rectangles)
        const INITIAL_CAPACITY: usize = 1024;
        let instance_size = std::mem::size_of::<RectInstance>();
        let vertex_buffer_size = (INITIAL_CAPACITY * instance_size) as u64;
        info!(
            "Creating vertex buffer: {} instances, {} bytes per instance, {} bytes total",
            INITIAL_CAPACITY, instance_size, vertex_buffer_size
        );
        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Rect Instance Buffer"),
            size: vertex_buffer_size,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            pipeline,
            vertex_buffer,
            vertex_buffer_capacity: INITIAL_CAPACITY,
        }
    }

    pub fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        instances: &[RectInstance],
    ) {
        if instances.is_empty() {
            return;
        }

        // Resize vertex buffer if needed
        if instances.len() > self.vertex_buffer_capacity {
            let new_capacity = instances.len().next_power_of_two();
            info!(
                "Resizing vertex buffer from {} to {} instances",
                self.vertex_buffer_capacity, new_capacity
            );
            self.vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Rect Instance Buffer"),
                size: (new_capacity * std::mem::size_of::<RectInstance>()) as u64,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            self.vertex_buffer_capacity = new_capacity;
        }

        // Upload instance data
        let instance_bytes = bytemuck::cast_slice(instances);
        debug!(
            "Uploading {} instances ({} bytes)",
            instances.len(),
            instance_bytes.len()
        );
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
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.draw(0..6, 0..instance_count);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rect_instance_is_pod() {
        // Verify RectInstance can be safely cast to bytes
        let instance = RectInstance::rect([10.0, 20.0], [100.0, 200.0], [1.0, 0.0, 0.0, 1.0]);

        let bytes = bytemuck::bytes_of(&instance);
        assert_eq!(bytes.len(), std::mem::size_of::<RectInstance>());
        assert_eq!(instance.corner_radius, 0.0);

        let instances = vec![instance];
        let slice = bytemuck::cast_slice::<RectInstance, u8>(&instances);
        assert_eq!(slice.len(), std::mem::size_of::<RectInstance>());
    }

    #[test]
    fn test_corner_radius_preserved() {
        let instance = RectInstance::new([10.0, 20.0], [100.0, 50.0], [1.0, 1.0, 1.0, 1.0], 12.0);
        assert_eq!(instance.corner_radius, 12.0);
        assert_eq!(instance.pos, [10.0, 20.0]);
        assert_eq!(instance.size, [100.0, 50.0]);
    }
}
