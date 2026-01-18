//! wgpu rendering backend implementation.

use bevy_ecs::prelude::*;
use crate::{Color, RendererError, Scene};
use raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use tracing::{Level, debug, error, info, instrument, span, warn};
use wgpu;

/// Per-rectangle instance data (uploaded to GPU as vertex attributes)
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct RectInstance {
    pub pos: [f32; 2],   // Position (x, y)
    pub size: [f32; 2],  // Size (width, height)
    pub color: [f32; 4], // Color (r, g, b, a)
}

/// Global uniform data (shared across all rectangles)
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Globals {
    transform: [[f32; 4]; 4], // 4x4 projection matrix
}

/// wgpu-based rendering backend.
///
/// Can be stored in ECS World as a Resource.
/// The backend manages GPU resources and performs rendering operations.
#[derive(Resource)]
pub struct WgpuBackend {
    #[allow(dead_code)]
    instance: wgpu::Instance,
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    pipeline: wgpu::RenderPipeline,
    globals_buffer: wgpu::Buffer,
    globals_bind_group: wgpu::BindGroup,
    vertex_buffer: wgpu::Buffer,
    vertex_buffer_capacity: usize, // Number of instances
    clear_color: Color,
}

impl WgpuBackend {
    /// Create a new wgpu backend from a window.
    ///
    /// # Safety Requirements
    ///
    /// The returned `WgpuBackend` must not outlive the window. The caller is responsible
    /// for ensuring the backend is dropped before the window is destroyed. This is typically
    /// achieved by storing both the window and backend in the same struct, relying on Rust's
    /// drop order (fields are dropped in declaration order).
    ///
    /// # Example Structure
    ///
    /// ```ignore
    /// struct Application {
    ///     window: Window,        // Dropped second
    ///     backend: WgpuBackend,  // Dropped first
    /// }
    /// ```
    #[instrument(skip(window), fields(width, height))]
    pub fn new<W>(window: &W, width: u32, height: u32) -> Result<Self, RendererError>
    where
        W: HasWindowHandle + HasDisplayHandle + Sync,
    {
        let _span = span!(Level::INFO, "wgpu_backend_init").entered();

        // Create wgpu instance with validation enabled in debug builds
        info!("Creating wgpu instance");
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            flags: if cfg!(debug_assertions) {
                wgpu::InstanceFlags::VALIDATION | wgpu::InstanceFlags::DEBUG
            } else {
                wgpu::InstanceFlags::empty()
            },
            ..Default::default()
        });

        // Create surface
        // SAFETY: wgpu::Surface requires 'static lifetime, but we're borrowing the window.
        // This is the standard wgpu pattern for surface creation.
        //
        // Safety invariant: The surface must not outlive the window.
        //
        // This is guaranteed by the caller:
        // - The caller creates the window
        // - The caller creates the backend with a reference to the window
        // - The caller must ensure the backend is dropped before the window is destroyed
        //
        // In practice, the backend will typically be owned by the application struct
        // alongside the window, ensuring proper drop order.
        let surface = unsafe {
            let target = wgpu::SurfaceTargetUnsafe::from_window(window)?;
            instance.create_surface_unsafe(target)?
        };

        // Request adapter
        info!("Requesting GPU adapter");
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }))
        .ok_or(RendererError::NoAdapter)?;

        let adapter_info = adapter.get_info();
        info!(
            "Selected adapter: {} ({:?})",
            adapter_info.name, adapter_info.backend
        );

        // Request device and queue
        info!("Creating device and queue");
        let (device, queue) = pollster::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: Some("Arthropod Device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                memory_hints: wgpu::MemoryHints::default(),
            },
            None,
        ))?;

        // Set up error callback
        device.on_uncaptured_error(Box::new(|err| {
            error!("wgpu uncaptured error: {}", err);
        }));

        // Configure surface
        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width,
            height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        surface.configure(&device, &config);

        // Create bind group layout for globals uniform
        let globals_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Globals Bind Group Layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(
                            std::mem::size_of::<Globals>() as u64
                        ),
                    },
                    count: None,
                }],
            });

        // Create projection matrix for screen space to NDC conversion
        // Screen space: (0, 0) top-left to (width, height) bottom-right
        // NDC: (-1, -1) bottom-left to (1, 1) top-right
        let projection = [
            [2.0 / width as f32, 0.0, 0.0, 0.0],
            [0.0, -2.0 / height as f32, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [-1.0, 1.0, 0.0, 1.0],
        ];

        let globals = Globals {
            transform: projection,
        };

        // Create globals uniform buffer
        let globals_size = std::mem::size_of::<Globals>() as u64;
        info!("Creating globals buffer: {} bytes", globals_size);
        let globals_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Globals Uniform Buffer"),
            size: globals_size,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Upload initial globals
        info!("Uploading globals: {} bytes", globals_size);
        queue.write_buffer(&globals_buffer, 0, bytemuck::cast_slice(&[globals]));

        // Create globals bind group
        let globals_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Globals Bind Group"),
            layout: &globals_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: globals_buffer.as_entire_binding(),
            }],
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

        // Create shader module
        info!("Loading shader from rect.wgsl");
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Rect Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/rect.wgsl").into()),
        });
        info!("Shader module created successfully");

        // Create render pipeline
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[&globals_bind_group_layout],
            push_constant_ranges: &[],
        });

        // Define vertex buffer layout (per-instance attributes)
        let vertex_buffer_layout = wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<RectInstance>() as u64,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &wgpu::vertex_attr_array![
                0 => Float32x2,  // pos
                1 => Float32x2,  // size
                2 => Float32x4,  // color
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
                    format: config.format,
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
            multiview: None,
            cache: None,
        });

        Ok(Self {
            instance,
            surface,
            device,
            queue,
            config,
            pipeline,
            globals_buffer,
            globals_bind_group,
            vertex_buffer,
            vertex_buffer_capacity: INITIAL_CAPACITY,
            clear_color: Color::rgba(0.0, 0.0, 0.0, 1.0),
        })
    }
}

impl super::RenderBackend for WgpuBackend {
    #[instrument(skip(self, scene))]
    fn render(&mut self, scene: &Scene) -> Result<(), RendererError> {
        let _span = span!(Level::TRACE, "render_frame").entered();
        use crate::NodeContent;

        // Collect all visible rectangles into instances
        let mut instances = Vec::new();
        for (_node_id, node) in scene.nodes() {
            if !node.visible || node.opacity <= 0.0 {
                continue;
            }

            match &node.content {
                NodeContent::Rect { color } | NodeContent::RoundedRect { color, .. } => {
                    instances.push(RectInstance {
                        pos: [node.bounds.x, node.bounds.y],
                        size: [node.bounds.width, node.bounds.height],
                        color: [color.r(), color.g(), color.b(), color.a() * node.opacity],
                    });
                }
                NodeContent::Empty => {}
            }
        }

        if instances.is_empty() {
            debug!("No visible rectangles to render");
        } else {
            debug!("Rendering {} rectangles", instances.len());
        }

        // Resize vertex buffer if needed
        if instances.len() > self.vertex_buffer_capacity {
            let new_capacity = instances.len().next_power_of_two();
            info!(
                "Resizing vertex buffer from {} to {} instances",
                self.vertex_buffer_capacity, new_capacity
            );
            self.vertex_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Rect Instance Buffer"),
                size: (new_capacity * std::mem::size_of::<RectInstance>()) as u64,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            self.vertex_buffer_capacity = new_capacity;
        }

        // Upload instance data
        if !instances.is_empty() {
            let instance_bytes = bytemuck::cast_slice(&instances);
            debug!(
                "Uploading {} instances ({} bytes): {:?}",
                instances.len(),
                instance_bytes.len(),
                &instances[0]
            );
            self.queue
                .write_buffer(&self.vertex_buffer, 0, instance_bytes);
        }

        // Render
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: self.clear_color.r() as f64,
                            g: self.clear_color.g() as f64,
                            b: self.clear_color.b() as f64,
                            a: self.clear_color.a() as f64,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            render_pass.set_pipeline(&self.pipeline);
            render_pass.set_bind_group(0, &self.globals_bind_group, &[]);

            if !instances.is_empty() {
                render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
                render_pass.draw(0..6, 0..instances.len() as u32);
            }
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.config.width = width;
            self.config.height = height;
            self.surface.configure(&self.device, &self.config);

            // Recalculate projection matrix for new dimensions
            let projection = [
                [2.0 / width as f32, 0.0, 0.0, 0.0],
                [0.0, -2.0 / height as f32, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [-1.0, 1.0, 0.0, 1.0],
            ];

            let globals = Globals {
                transform: projection,
            };

            // Update globals buffer with new projection
            self.queue
                .write_buffer(&self.globals_buffer, 0, bytemuck::cast_slice(&[globals]));
        }
    }

    fn set_clear_color(&mut self, color: Color) {
        self.clear_color = color;
    }
}

// Additional WgpuBackend methods (not part of RenderBackend trait)
impl WgpuBackend {
    /// Render a collection of rectangle instances directly (ECS-friendly API)
    ///
    /// This method accepts pre-computed RectInstances instead of a Scene,
    /// making it suitable for use with ECS systems that generate instances.
    ///
    /// # Arguments
    ///
    /// * `instances` - Slice of RectInstance structs to render
    ///
    /// # Errors
    ///
    /// Returns an error if surface texture acquisition or rendering fails
    #[instrument(skip(self, instances))]
    pub fn render_instances(&mut self, instances: &[RectInstance]) -> Result<(), RendererError> {
        let _span = span!(Level::TRACE, "render_instances").entered();

        if instances.is_empty() {
            debug!("No instances to render");
        } else {
            debug!("Rendering {} instances", instances.len());
        }

        // Resize vertex buffer if needed
        if instances.len() > self.vertex_buffer_capacity {
            let new_capacity = instances.len().next_power_of_two();
            info!(
                "Resizing vertex buffer from {} to {} instances",
                self.vertex_buffer_capacity, new_capacity
            );
            self.vertex_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Rect Instance Buffer"),
                size: (new_capacity * std::mem::size_of::<RectInstance>()) as u64,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            self.vertex_buffer_capacity = new_capacity;
        }

        // Upload instance data
        if !instances.is_empty() {
            let instance_bytes = bytemuck::cast_slice(instances);
            debug!(
                "Uploading {} instances ({} bytes)",
                instances.len(),
                instance_bytes.len()
            );
            self.queue
                .write_buffer(&self.vertex_buffer, 0, instance_bytes);
        }

        // Render
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: self.clear_color.r() as f64,
                            g: self.clear_color.g() as f64,
                            b: self.clear_color.b() as f64,
                            a: self.clear_color.a() as f64,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            render_pass.set_pipeline(&self.pipeline);
            render_pass.set_bind_group(0, &self.globals_bind_group, &[]);

            if !instances.is_empty() {
                render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
                render_pass.draw(0..6, 0..instances.len() as u32);
            }
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{NodeContent, SceneNode, Transform2D};

    #[test]
    fn test_rect_instance_is_pod() {
        // Verify RectInstance can be safely cast to bytes
        let instance = RectInstance {
            pos: [10.0, 20.0],
            size: [100.0, 200.0],
            color: [1.0, 0.0, 0.0, 1.0],
        };

        let bytes = bytemuck::bytes_of(&instance);
        assert_eq!(bytes.len(), std::mem::size_of::<RectInstance>());

        let instances = vec![instance];
        let slice = bytemuck::cast_slice::<RectInstance, u8>(&instances);
        assert_eq!(slice.len(), std::mem::size_of::<RectInstance>());
    }

    #[test]
    fn test_globals_is_pod() {
        // Verify Globals can be safely cast to bytes
        let globals = Globals {
            transform: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        };

        let bytes = bytemuck::bytes_of(&globals);
        assert_eq!(bytes.len(), std::mem::size_of::<Globals>());

        let globals_array = vec![globals];
        let slice = bytemuck::cast_slice::<Globals, u8>(&globals_array);
        assert_eq!(slice.len(), std::mem::size_of::<Globals>());
    }

    #[test]
    fn test_projection_matrix_calculation() {
        // Test projection matrix converts screen space to NDC correctly
        let width = 800.0f32;
        let height = 600.0f32;

        let projection = [
            [2.0 / width, 0.0, 0.0, 0.0],
            [0.0, -2.0 / height, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [-1.0, 1.0, 0.0, 1.0],
        ];

        // Top-left corner (0, 0) should map to (-1, 1) in NDC
        let x = 0.0;
        let y = 0.0;
        let ndc_x = projection[0][0] * x + projection[3][0];
        let ndc_y = projection[1][1] * y + projection[3][1];
        assert!((ndc_x - (-1.0)).abs() < 0.001);
        assert!((ndc_y - 1.0).abs() < 0.001);

        // Bottom-right corner (800, 600) should map to (1, -1) in NDC
        let x = width;
        let y = height;
        let ndc_x = projection[0][0] * x + projection[3][0];
        let ndc_y = projection[1][1] * y + projection[3][1];
        assert!((ndc_x - 1.0).abs() < 0.001);
        assert!((ndc_y - (-1.0)).abs() < 0.001);

        // Center (400, 300) should map to (0, 0) in NDC
        let x = width / 2.0;
        let y = height / 2.0;
        let ndc_x = projection[0][0] * x + projection[3][0];
        let ndc_y = projection[1][1] * y + projection[3][1];
        assert!(ndc_x.abs() < 0.001);
        assert!(ndc_y.abs() < 0.001);
    }

    #[test]
    fn test_instance_collection_from_scene() {
        // Create a test scene with rectangles
        let mut scene = Scene::new();
        let root = scene.root();

        // Add visible rectangle
        let red_rect = SceneNode {
            content: NodeContent::Rect {
                color: Color::rgba(1.0, 0.0, 0.0, 1.0),
            },
            transform: Transform2D::identity(),
            bounds: plat_core::Rect {
                x: 10.0,
                y: 20.0,
                width: 100.0,
                height: 200.0,
            },
            children: vec![],
            visible: true,
            opacity: 1.0,
        };
        scene.add_node(root, red_rect);

        // Add invisible rectangle (should be skipped)
        let invisible_rect = SceneNode {
            content: NodeContent::Rect {
                color: Color::rgba(0.0, 1.0, 0.0, 1.0),
            },
            transform: Transform2D::identity(),
            bounds: plat_core::Rect {
                x: 0.0,
                y: 0.0,
                width: 50.0,
                height: 50.0,
            },
            children: vec![],
            visible: false,
            opacity: 1.0,
        };
        scene.add_node(root, invisible_rect);

        // Add rectangle with opacity
        let blue_rect = SceneNode {
            content: NodeContent::Rect {
                color: Color::rgba(0.0, 0.0, 1.0, 1.0),
            },
            transform: Transform2D::identity(),
            bounds: plat_core::Rect {
                x: 200.0,
                y: 100.0,
                width: 150.0,
                height: 150.0,
            },
            children: vec![],
            visible: true,
            opacity: 0.5,
        };
        scene.add_node(root, blue_rect);

        // Collect instances (mimicking the render method logic)
        let mut instances = Vec::new();
        for (_node_id, node) in scene.nodes() {
            if !node.visible || node.opacity <= 0.0 {
                continue;
            }

            match &node.content {
                NodeContent::Rect { color } | NodeContent::RoundedRect { color, .. } => {
                    instances.push(RectInstance {
                        pos: [node.bounds.x, node.bounds.y],
                        size: [node.bounds.width, node.bounds.height],
                        color: [color.r(), color.g(), color.b(), color.a() * node.opacity],
                    });
                }
                NodeContent::Empty => {}
            }
        }

        // Should have 2 instances (red and blue, not invisible)
        assert_eq!(instances.len(), 2);

        // Find red rectangle instance
        let red_instance = instances
            .iter()
            .find(|i| i.color[0] > 0.9 && i.color[1] < 0.1)
            .expect("Red instance not found");
        assert_eq!(red_instance.pos, [10.0, 20.0]);
        assert_eq!(red_instance.size, [100.0, 200.0]);
        assert_eq!(red_instance.color, [1.0, 0.0, 0.0, 1.0]);

        // Find blue rectangle instance (with opacity applied)
        let blue_instance = instances
            .iter()
            .find(|i| i.color[2] > 0.9 && i.color[0] < 0.1)
            .expect("Blue instance not found");
        assert_eq!(blue_instance.pos, [200.0, 100.0]);
        assert_eq!(blue_instance.size, [150.0, 150.0]);
        assert_eq!(blue_instance.color, [0.0, 0.0, 1.0, 0.5]); // opacity applied
    }

    #[test]
    fn test_quad_vertex_generation() {
        // Test that quad vertices are generated correctly
        // Expected pattern for two triangles:
        // Triangle 1: (0,0), (1,0), (1,1)
        // Triangle 2: (0,0), (1,1), (0,1)
        let expected = [
            (0.0, 0.0), // 0
            (1.0, 0.0), // 1
            (1.0, 1.0), // 2
            (0.0, 0.0), // 3
            (1.0, 1.0), // 4
            (0.0, 1.0), // 5
        ];

        // This would need to be tested by actually running the shader
        // For now, just document the expected pattern
        for (i, &(ex, ey)) in expected.iter().enumerate() {
            println!("Vertex {}: expected ({}, {})", i, ex, ey);
        }
    }

    #[test]
    fn test_projection_matrix_updates_on_resize() {
        // Test that projection matrix should change when dimensions change
        let width1 = 800.0f32;
        let height1 = 600.0f32;

        let projection1 = [
            [2.0 / width1, 0.0, 0.0, 0.0],
            [0.0, -2.0 / height1, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [-1.0, 1.0, 0.0, 1.0],
        ];

        let width2 = 1024.0f32;
        let height2 = 768.0f32;

        let projection2 = [
            [2.0 / width2, 0.0, 0.0, 0.0],
            [0.0, -2.0 / height2, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [-1.0, 1.0, 0.0, 1.0],
        ];

        // Matrices should be different
        assert_ne!(projection1[0][0], projection2[0][0]);
        assert_ne!(projection1[1][1], projection2[1][1]);

        // Bottom-right corner should still map to (1, -1) with new dimensions
        let x = width2;
        let y = height2;
        let ndc_x = projection2[0][0] * x + projection2[3][0];
        let ndc_y = projection2[1][1] * y + projection2[3][1];
        assert!((ndc_x - 1.0).abs() < 0.001);
        assert!((ndc_y - (-1.0)).abs() < 0.001);
    }
}
