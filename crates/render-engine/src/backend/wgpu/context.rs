//! WGPU context management.

use crate::RendererError;
use raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use tracing::{Level, error, info, span};

/// Global uniform data (shared across all rectangles and glyphs)
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Globals {
    pub transform: [[f32; 4]; 4], // 4x4 projection matrix
}

/// Wraps the WGPU instance, surface, device, queue, and global resources.
pub struct WgpuContext {
    #[allow(dead_code)]
    pub instance: wgpu::Instance,
    pub surface: wgpu::Surface<'static>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub globals_buffer: wgpu::Buffer,
    pub globals_bind_group: wgpu::BindGroup,
    pub globals_bind_group_layout: wgpu::BindGroupLayout,
    pub clear_color: crate::Color,
}

impl WgpuContext {
    /// Create a new WGPU context.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the created `WgpuContext` is dropped *before* the window
    /// it was created from. This is required because `wgpu::Surface` holds a reference to the
    /// window handle, but carries a `'static` lifetime to avoid infecting the entire codebase
    /// with lifetimes. Accessing the surface after the window is destroyed results in undefined behavior.
    pub unsafe fn new<W>(
        window: &W,
        width: u32,
        height: u32,
        composition_mode: bool,
    ) -> Result<Self, RendererError>
    where
        W: HasWindowHandle + HasDisplayHandle + Sync,
    {
        let _span = span!(Level::INFO, "wgpu_context_init").entered();

        // Create wgpu instance with DirectComposition support
        println!("🚀 Creating wgpu Instance (DX12 with DirectComposition)");

        #[cfg(target_os = "windows")]
        let backend_options = {
            let mut dx12_options = if composition_mode {
                wgpu::Dx12BackendOptions {
                    presentation_system: wgpu::Dx12SwapchainKind::DxgiFromVisual,
                    ..Default::default()
                }
            } else {
                Default::default()
            };

            // Apply environment variable overrides
            dx12_options = dx12_options.with_env();

            // Log final presentation system
            match dx12_options.presentation_system {
                wgpu::Dx12SwapchainKind::DxgiFromVisual => {
                    println!("   Using DxgiFromVisual (transparency, no RenderDoc)");
                }
                wgpu::Dx12SwapchainKind::DxgiFromHwnd => {
                    println!("   Using DxgiFromHwnd (RenderDoc compatible)");
                }
            }

            wgpu::BackendOptions {
                dx12: dx12_options,
                ..Default::default()
            }
        };

        #[cfg(not(target_os = "windows"))]
        let backend_options = Default::default();

        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::DX12,
            flags: wgpu::InstanceFlags::empty(),
            backend_options,
            ..Default::default()
        });

        // Create surface
        // SAFETY: wgpu::Surface requires 'static lifetime, but we're borrowing the window.
        // This is guaranteed by the caller ensuring backend drops before window.
        let surface = unsafe {
            let target = wgpu::SurfaceTargetUnsafe::from_window(window)?;
            instance.create_surface_unsafe(target)?
        };

        // Request adapter
        info!("Requesting GPU adapter");
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }))
        .map_err(|e| {
            RendererError::InitializationFailed(format!("Failed to request adapter: {:?}", e))
        })?;

        let adapter_info = adapter.get_info();
        println!(
            "🎮 Adapter: {} ({:?})",
            adapter_info.name, adapter_info.backend
        );
        info!(
            "Selected adapter: {} ({:?})",
            adapter_info.name, adapter_info.backend
        );

        // Request device and queue
        info!("Creating device and queue");
        let (device, queue) =
            pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
                label: Some("Arthropod Device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                memory_hints: wgpu::MemoryHints::default(),
                experimental_features: Default::default(),
                trace: Default::default(),
            }))?;

        // Set up error callback
        device.on_uncaptured_error(std::sync::Arc::new(|err| {
            error!("wgpu uncaptured error: {}", err);
        }));

        // Configure surface
        let surface_caps = surface.get_capabilities(&adapter);

        #[cfg(target_os = "windows")]
        let config = if composition_mode
            && crate::backend::composition_swap_chain::supports_composition(&surface_caps)
        {
            println!("✨ Using DirectComposition: Bgra8UnormSrgb + PreMultiplied");
            info!("Using DirectComposition-compatible surface configuration");
            crate::backend::composition_swap_chain::create_composition_surface_config(
                width,
                height,
                surface_caps.present_modes[0],
            )
        } else {
            if composition_mode {
                tracing::warn!(
                    "Composition mode requested but surface doesn't support BGRA + PreMultiplied. \
                     Falling back to standard config."
                );
            }
            // Standard surface configuration
            let surface_format = surface_caps
                .formats
                .iter()
                .copied()
                .find(|f| f.is_srgb())
                .unwrap_or(surface_caps.formats[0]);
            let alpha_mode = surface_caps.alpha_modes[0];

            wgpu::SurfaceConfiguration {
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                format: surface_format,
                width,
                height,
                present_mode: surface_caps.present_modes[0],
                alpha_mode,
                view_formats: vec![],
                desired_maximum_frame_latency: 2,
            }
        };

        #[cfg(not(target_os = "windows"))]
        let config = {
            let _ = composition_mode;
            let surface_format = surface_caps
                .formats
                .iter()
                .copied()
                .find(|f| f.is_srgb())
                .unwrap_or(surface_caps.formats[0]);
            let alpha_mode = surface_caps.alpha_modes[0];

            wgpu::SurfaceConfiguration {
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                format: surface_format,
                width,
                height,
                present_mode: surface_caps.present_modes[0],
                alpha_mode,
                view_formats: vec![],
                desired_maximum_frame_latency: 2,
            }
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

        // Create projection matrix
        let projection = create_projection_matrix(width, height);

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

        let clear_color = if composition_mode {
            crate::Color::rgba(0.0, 0.0, 0.0, 0.0)
        } else {
            crate::Color::rgba(0.0, 0.0, 0.0, 1.0)
        };

        Ok(Self {
            instance,
            surface,
            device,
            queue,
            config,
            globals_buffer,
            globals_bind_group,
            globals_bind_group_layout,
            clear_color,
        })
    }

    /// Helper to create a render pass and execute a closure.
    pub fn with_render_pass<F>(&mut self, f: F) -> Result<(), RendererError>
    where
        F: FnOnce(&mut wgpu::RenderPass, &wgpu::BindGroup),
    {
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
                    depth_slice: None,
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
                multiview_mask: None,
            });

            f(&mut render_pass, &self.globals_bind_group);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.config.width = width;
            self.config.height = height;
            self.surface.configure(&self.device, &self.config);

            let projection = create_projection_matrix(width, height);

            let globals = Globals {
                transform: projection,
            };

            self.queue
                .write_buffer(&self.globals_buffer, 0, bytemuck::cast_slice(&[globals]));
        }
    }
}

fn create_projection_matrix(width: u32, height: u32) -> [[f32; 4]; 4] {
    [
        [2.0 / width as f32, 0.0, 0.0, 0.0],
        [0.0, -2.0 / height as f32, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [-1.0, 1.0, 0.0, 1.0],
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

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

        let projection = create_projection_matrix(width as u32, height as u32);

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
    fn test_projection_matrix_updates_on_resize() {
        // Test that projection matrix should change when dimensions change
        let width1 = 800.0f32;
        let height1 = 600.0f32;

        let projection1 = create_projection_matrix(width1 as u32, height1 as u32);

        let width2 = 1024.0f32;
        let height2 = 768.0f32;

        let projection2 = create_projection_matrix(width2 as u32, height2 as u32);

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
