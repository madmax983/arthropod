//! WGPU context management.

use crate::RendererError;
use crate::backend::wgpu::effects::{RenderTargetHandle, RenderTargetKey};
use hashbrown::HashMap;
use raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use tracing::{Level, error, info, span};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;

/// Global uniform data (shared across all rectangles and glyphs)
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Globals {
    /// The 4x4 orthographic projection matrix used to map scene coordinates to normalized device coordinates.
    pub transform: [[f32; 4]; 4], // 4x4 projection matrix
}

/// Offscreen render target (color + optional stencil).
pub struct RenderTarget {
    /// The unique configuration key for this render target.
    pub key: RenderTargetKey,
    /// The actual WGPU color texture.
    pub color_texture: wgpu::Texture,
    /// The view into the color texture used for rendering.
    pub color_view: wgpu::TextureView,
    /// An optional stencil texture for clipping masks.
    pub stencil_texture: Option<wgpu::Texture>,
    /// The view into the optional stencil texture.
    pub stencil_view: Option<wgpu::TextureView>,
}

impl RenderTarget {
    #[must_use]
    /// Estimate the VRAM usage of this render target in bytes.
    pub fn estimated_bytes(&self) -> u64 {
        self.key.estimated_bytes()
    }
}

/// Runtime web backend choice for wasm.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WebBackend {
    /// Browser WebGPU backend.
    WebGpu,
    /// WebGL2 compatibility backend.
    WebGl2,
}

/// Capability probe output used to choose a web backend.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProbeCaps {
    /// True if WebGPU is supported by the browser.
    pub webgpu_available: bool,
    /// True if WebGL2 is supported by the browser.
    pub webgl2_available: bool,
}

/// Select web backend using WebGPU-first policy with WebGL2 fallback.
#[must_use]
pub fn select_web_backend(caps: ProbeCaps) -> WebBackend {
    if caps.webgpu_available {
        return WebBackend::WebGpu;
    }
    if caps.webgl2_available {
        return WebBackend::WebGl2;
    }
    WebBackend::WebGpu
}

#[cfg(target_arch = "wasm32")]
fn probe_web_backend_caps() -> ProbeCaps {
    let Some(window) = web_sys::window() else {
        return ProbeCaps {
            webgpu_available: false,
            webgl2_available: false,
        };
    };

    let navigator = window.navigator();
    let webgpu_available = js_sys::Reflect::has(&navigator, &"gpu".into()).unwrap_or(false);
    let webgl2_available = window
        .document()
        .and_then(|doc| doc.create_element("canvas").ok())
        .and_then(|element| element.dyn_into::<web_sys::HtmlCanvasElement>().ok())
        .and_then(|canvas| canvas.get_context("webgl2").ok().flatten())
        .is_some();

    ProbeCaps {
        webgpu_available,
        webgl2_available,
    }
}

/// Wraps the WGPU instance, surface, device, queue, and global resources.
pub struct WgpuContext {
    #[allow(dead_code)]
    pub(crate) instance: wgpu::Instance,
    pub(crate) surface: wgpu::Surface<'static>,
    pub(crate) device: wgpu::Device,
    pub(crate) queue: wgpu::Queue,
    pub(crate) config: wgpu::SurfaceConfiguration,
    pub(crate) globals_buffer: wgpu::Buffer,
    pub(crate) globals_bind_group: wgpu::BindGroup,
    pub(crate) globals_bind_group_layout: wgpu::BindGroupLayout,
    pub(crate) clear_color: crate::Color,
    design_space: Option<(f32, f32)>,
    next_render_target_handle: RenderTargetHandle,
    render_targets: HashMap<RenderTargetHandle, RenderTarget>,
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
    #[cfg(not(target_arch = "wasm32"))]
    pub unsafe fn new<W>(
        window: &W,
        width: u32,
        height: u32,
        composition_mode: bool,
    ) -> Result<Self, RendererError>
    where
        W: HasWindowHandle + HasDisplayHandle,
    {
        pollster::block_on(async {
            // SAFETY: The caller provides the guarantee that `window` will outlive
            // the returned `WgpuContext`. We safely propagate this lifetime guarantee
            // into the asynchronous initialization implementation.
            unsafe { Self::new_impl(window, width, height, composition_mode).await }
        })
    }

    /// Create a new WGPU context asynchronously on wasm targets.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the created `WgpuContext` is dropped *before* the window
    /// it was created from. This is required because `wgpu::Surface` holds a reference to the
    /// window handle, but carries a `'static` lifetime to avoid infecting the entire codebase
    /// with lifetimes. Accessing the surface after the window is destroyed results in undefined behavior.
    #[cfg(target_arch = "wasm32")]
    pub async unsafe fn new_async<W>(
        window: &W,
        width: u32,
        height: u32,
        composition_mode: bool,
    ) -> Result<Self, RendererError>
    where
        W: HasWindowHandle + HasDisplayHandle,
    {
        // SAFETY: The caller provides the guarantee that `window` will outlive
        // the returned `WgpuContext`. We safely propagate this lifetime guarantee
        // into the asynchronous initialization implementation.
        unsafe { Self::new_impl(window, width, height, composition_mode).await }
    }

    async unsafe fn new_impl<W>(
        window: &W,
        width: u32,
        height: u32,
        composition_mode: bool,
    ) -> Result<Self, RendererError>
    where
        W: HasWindowHandle + HasDisplayHandle,
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

        #[cfg(target_arch = "wasm32")]
        let selected_web_backend = select_web_backend(probe_web_backend_caps());
        #[cfg(target_arch = "wasm32")]
        let backends = match selected_web_backend {
            WebBackend::WebGpu => wgpu::Backends::BROWSER_WEBGPU,
            WebBackend::WebGl2 => wgpu::Backends::GL,
        };
        #[cfg(not(target_arch = "wasm32"))]
        let backends = wgpu::Backends::DX12;

        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends,
            flags: wgpu::InstanceFlags::empty(),
            backend_options,
            ..Default::default()
        });

        // Create surface
        // SAFETY: wgpu::Surface requires 'static lifetime, but we're borrowing the window.
        // This is guaranteed by the caller ensuring backend drops before window.
        let target = unsafe { wgpu::SurfaceTargetUnsafe::from_window(window)? };
        // SAFETY: The target uses the window handle, which outlives the surface.
        let surface = unsafe { instance.create_surface_unsafe(target)? };

        // Request adapter
        info!("Requesting GPU adapter");
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
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
        #[cfg(target_arch = "wasm32")]
        let required_limits = match selected_web_backend {
            WebBackend::WebGpu => wgpu::Limits::downlevel_defaults(),
            WebBackend::WebGl2 => wgpu::Limits::downlevel_webgl2_defaults(),
        };
        #[cfg(not(target_arch = "wasm32"))]
        let required_limits = wgpu::Limits::default();

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("Arthropod Device"),
                required_features: wgpu::Features::empty(),
                required_limits,
                memory_hints: wgpu::MemoryHints::default(),
                experimental_features: Default::default(),
                trace: Default::default(),
            })
            .await?;

        // Set up error callback
        device.on_uncaptured_error(std::sync::Arc::new(|err| {
            error!("wgpu uncaptured error: {}", err);
            eprintln!("wgpu uncaptured error: {err}");
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
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
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
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
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
        let projection = create_projection_matrix_with_design_space(width, height, None);

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
            design_space: None,
            next_render_target_handle: 1,
            render_targets: HashMap::new(),
        })
    }

    /// Create an offscreen render target for Phase 4 multipass effects.
    pub fn create_render_target(&mut self, key: RenderTargetKey) -> RenderTargetHandle {
        let format = self.config.format;
        let color_texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Phase4 RenderTarget Color"),
            size: wgpu::Extent3d {
                width: key.width.max(1),
                height: key.height.max(1),
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_SRC
                | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let color_view = color_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let (stencil_texture, stencil_view) = if key.has_stencil {
            let tex = self.device.create_texture(&wgpu::TextureDescriptor {
                label: Some("Phase4 RenderTarget Stencil"),
                size: wgpu::Extent3d {
                    width: key.width.max(1),
                    height: key.height.max(1),
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Stencil8,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            });
            let view = tex.create_view(&wgpu::TextureViewDescriptor::default());
            (Some(tex), Some(view))
        } else {
            (None, None)
        };

        let handle = self.next_render_target_handle;
        self.next_render_target_handle = self.next_render_target_handle.saturating_add(1);

        self.render_targets.insert(
            handle,
            RenderTarget {
                key,
                color_texture,
                color_view,
                stencil_texture,
                stencil_view,
            },
        );

        handle
    }

    #[must_use]
    /// Retrieve a reference to a render target by its handle.
    pub fn get_render_target(&self, handle: RenderTargetHandle) -> Option<&RenderTarget> {
        self.render_targets.get(&handle)
    }

    /// Remove and return a render target from the context.
    pub fn remove_render_target(&mut self, handle: RenderTargetHandle) -> Option<RenderTarget> {
        self.render_targets.remove(&handle)
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

    /// Render into an offscreen texture and return RGBA8 pixel data.
    ///
    /// Intended for native visual regression capture.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn with_offscreen_render_pass<F>(
        &mut self,
        width: u32,
        height: u32,
        f: F,
    ) -> Result<Vec<u8>, RendererError>
    where
        F: FnOnce(&mut wgpu::RenderPass, &wgpu::BindGroup),
    {
        if width == 0 || height == 0 {
            return Ok(Vec::new());
        }

        let format = self.config.format;
        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Offscreen Capture Texture"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let bytes_per_pixel = 4u32;
        let padded_bytes_per_row = aligned_bytes_per_row(width, bytes_per_pixel)?;
        let output_size = padded_bytes_per_row as u64 * height as u64;
        let output_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Offscreen Capture Readback Buffer"),
            size: output_size,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Offscreen Capture Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Offscreen Render Pass"),
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

        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &output_buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(padded_bytes_per_row),
                    rows_per_image: Some(height),
                },
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );

        self.queue.submit(std::iter::once(encoder.finish()));
        let slice = output_buffer.slice(..);
        let (tx, rx) = std::sync::mpsc::channel();
        slice.map_async(wgpu::MapMode::Read, move |result| {
            let _ = tx.send(result);
        });

        self.device
            .poll(wgpu::PollType::wait_indefinitely())
            .map_err(|err| RendererError::InitializationFailed(err.to_string()))?;
        rx.recv()
            .map_err(|err| RendererError::InitializationFailed(err.to_string()))?
            .map_err(|err| RendererError::InitializationFailed(err.to_string()))?;

        let mapped = slice.get_mapped_range();
        let rgba = unpack_readback_pixels(
            &mapped,
            width,
            height,
            padded_bytes_per_row,
            self.config.format,
        )?;
        drop(mapped);
        output_buffer.unmap();

        Ok(rgba)
    }

    /// Read a texture into tightly-packed RGBA8 bytes.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn read_texture_to_rgba(
        &self,
        texture: &wgpu::Texture,
        width: u32,
        height: u32,
    ) -> Result<Vec<u8>, RendererError> {
        if width == 0 || height == 0 {
            return Ok(Vec::new());
        }

        let bytes_per_pixel = 4u32;
        let padded_bytes_per_row = aligned_bytes_per_row(width, bytes_per_pixel)?;
        let output_size = (padded_bytes_per_row as u64)
            .checked_mul(height as u64)
            .ok_or_else(|| {
                RendererError::InitializationFailed(
                    "Overflow calculating output buffer size for texture readback".to_string(),
                )
            })?;
        let output_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Texture Readback Buffer"),
            size: output_size,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Texture Readback Encoder"),
            });

        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &output_buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(padded_bytes_per_row),
                    rows_per_image: Some(height),
                },
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );
        self.queue.submit(std::iter::once(encoder.finish()));

        let slice = output_buffer.slice(..);
        let (tx, rx) = std::sync::mpsc::channel();
        slice.map_async(wgpu::MapMode::Read, move |result| {
            let _ = tx.send(result);
        });

        self.device
            .poll(wgpu::PollType::wait_indefinitely())
            .map_err(|err| RendererError::InitializationFailed(err.to_string()))?;
        rx.recv()
            .map_err(|err| RendererError::InitializationFailed(err.to_string()))?
            .map_err(|err| RendererError::InitializationFailed(err.to_string()))?;

        let mapped = slice.get_mapped_range();
        let rgba = unpack_readback_pixels(
            &mapped,
            width,
            height,
            padded_bytes_per_row,
            self.config.format,
        )?;
        drop(mapped);
        output_buffer.unmap();
        Ok(rgba)
    }

    /// Resize the window surface and update projection matrices.
    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.config.width = width;
            self.config.height = height;
            self.surface.configure(&self.device, &self.config);
            self.update_globals_transform();
        }
    }

    /// Set the design space coordinate system.
    pub fn set_design_space(&mut self, width: f32, height: f32) {
        self.design_space =
            if width.is_finite() && height.is_finite() && width > 0.0 && height > 0.0 {
                Some((width, height))
            } else {
                None
            };
        self.update_globals_transform();
    }

    /// Clear the design space and return to pixel coordinates.
    pub fn clear_design_space(&mut self) {
        self.design_space = None;
        self.update_globals_transform();
    }

    fn update_globals_transform(&mut self) {
        let projection = create_projection_matrix_with_design_space(
            self.config.width,
            self.config.height,
            self.design_space,
        );
        let globals = Globals {
            transform: projection,
        };
        self.queue
            .write_buffer(&self.globals_buffer, 0, bytemuck::cast_slice(&[globals]));
    }

    /// Map a rectangle from scene space to surface pixel coordinates.
    pub fn map_scene_rect_to_surface(&self, rect: plat_core::Rect) -> plat_core::Rect {
        map_scene_rect_to_surface_with_design_space(
            rect,
            self.config.width,
            self.config.height,
            self.design_space,
        )
    }
}

fn map_scene_rect_to_surface_with_design_space(
    rect: plat_core::Rect,
    frame_width: u32,
    frame_height: u32,
    design_space: Option<(f32, f32)>,
) -> plat_core::Rect {
    let Some((design_width, design_height)) = design_space else {
        return rect;
    };
    let width_f = frame_width as f32;
    let height_f = frame_height as f32;
    if width_f <= 0.0 || height_f <= 0.0 || design_width <= 0.0 || design_height <= 0.0 {
        return rect;
    }

    let scale = (width_f / design_width).min(height_f / design_height);
    if !scale.is_finite() || scale <= 0.0 {
        return rect;
    }

    let fitted_width = design_width * scale;
    let fitted_height = design_height * scale;
    let offset_x = (width_f - fitted_width) * 0.5;
    let offset_y = (height_f - fitted_height) * 0.5;

    plat_core::Rect::new(
        rect.x * scale + offset_x,
        rect.y * scale + offset_y,
        rect.width * scale,
        rect.height * scale,
    )
}

fn create_projection_matrix_with_design_space(
    width: u32,
    height: u32,
    design_space: Option<(f32, f32)>,
) -> [[f32; 4]; 4] {
    if width == 0 || height == 0 {
        return [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ];
    }

    let width_f = width as f32;
    let height_f = height as f32;
    let (sx, sy, tx, ty) = if let Some((design_width, design_height)) = design_space {
        if design_width > 0.0 && design_height > 0.0 {
            let scale = (width_f / design_width).min(height_f / design_height);
            let fitted_width = design_width * scale;
            let fitted_height = design_height * scale;
            let offset_x = (width_f - fitted_width) * 0.5;
            let offset_y = (height_f - fitted_height) * 0.5;
            (scale, scale, offset_x, offset_y)
        } else {
            (1.0, 1.0, 0.0, 0.0)
        }
    } else {
        (1.0, 1.0, 0.0, 0.0)
    };

    [
        [2.0 * sx / width_f, 0.0, 0.0, 0.0],
        [0.0, -2.0 * sy / height_f, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [
            2.0 * tx / width_f - 1.0,
            1.0 - 2.0 * ty / height_f,
            0.0,
            1.0,
        ],
    ]
}

#[cfg(not(target_arch = "wasm32"))]
fn aligned_bytes_per_row(width: u32, bytes_per_pixel: u32) -> Result<u32, RendererError> {
    let unaligned = width.checked_mul(bytes_per_pixel).ok_or_else(|| {
        RendererError::InitializationFailed(
            "Overflow calculating unaligned bytes per row".to_string(),
        )
    })?;
    let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
    let rem = unaligned % align;
    if rem == 0 {
        Ok(unaligned)
    } else {
        unaligned.checked_add(align - rem).ok_or_else(|| {
            RendererError::InitializationFailed(
                "Overflow calculating aligned bytes per row".to_string(),
            )
        })
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn unpack_readback_pixels(
    readback: &[u8],
    width: u32,
    height: u32,
    padded_bytes_per_row: u32,
    format: wgpu::TextureFormat,
) -> Result<Vec<u8>, RendererError> {
    let row_len = (width as usize).checked_mul(4).ok_or_else(|| {
        RendererError::InitializationFailed("Overflow calculating row length".to_string())
    })?;

    let padded = padded_bytes_per_row as usize;

    // Validate buffer size early before trying to reserve gigabytes of memory
    let min_required_src_len = (height as usize)
        .saturating_sub(1)
        .checked_mul(padded)
        .and_then(|h| h.checked_add(row_len))
        .ok_or_else(|| {
            RendererError::InitializationFailed(
                "Overflow calculating required buffer size".to_string(),
            )
        })?;

    if readback.len() < min_required_src_len {
        return Err(RendererError::InitializationFailed(
            "Readback buffer too small for requested dimensions".to_string(),
        ));
    }

    let total_size = row_len.checked_mul(height as usize).ok_or_else(|| {
        RendererError::InitializationFailed("Overflow calculating output buffer size".to_string())
    })?;

    let mut out = Vec::new();
    // Use try_reserve to prevent panic on OOM for large allocations
    if out.try_reserve(total_size).is_err() {
        return Err(RendererError::InitializationFailed(
            "Out of memory during texture readback".to_string(),
        ));
    }
    // Safe to resize now as we reserved capacity
    out.resize(total_size, 0u8);

    for row in 0..height as usize {
        let src_start = row.checked_mul(padded).ok_or_else(|| {
            RendererError::InitializationFailed("Overflow calculating source offset".to_string())
        })?;
        let src_end = src_start.checked_add(row_len).ok_or_else(|| {
            RendererError::InitializationFailed("Overflow calculating source end".to_string())
        })?;

        let dst_start = row.checked_mul(row_len).ok_or_else(|| {
            RendererError::InitializationFailed("Overflow calculating dest offset".to_string())
        })?;
        let dst_end = dst_start.checked_add(row_len).ok_or_else(|| {
            RendererError::InitializationFailed("Overflow calculating dest end".to_string())
        })?;

        if src_end > readback.len() {
            return Err(RendererError::InitializationFailed(
                "Readback buffer too small for requested dimensions".to_string(),
            ));
        }

        out[dst_start..dst_end].copy_from_slice(&readback[src_start..src_end]);
    }

    match format {
        wgpu::TextureFormat::Bgra8Unorm | wgpu::TextureFormat::Bgra8UnormSrgb => {
            for px in out.chunks_exact_mut(4) {
                px.swap(0, 2);
            }
        }
        _ => {}
    }

    Ok(out)
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

        let projection =
            create_projection_matrix_with_design_space(width as u32, height as u32, None);

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
    fn test_projection_matrix_handles_zero_dimensions() {
        let matrix = create_projection_matrix_with_design_space(0, 0, None);
        // Should return identity matrix (no NaNs or Infs)
        assert_eq!(matrix[0][0], 1.0);
        assert_eq!(matrix[1][1], 1.0);
        assert_eq!(matrix[2][2], 1.0);
        assert_eq!(matrix[3][3], 1.0);
        assert_eq!(matrix[0][1], 0.0);
    }

    #[test]
    fn test_projection_matrix_updates_on_resize() {
        // Test that projection matrix should change when dimensions change
        let width1 = 800.0f32;
        let height1 = 600.0f32;

        let projection1 =
            create_projection_matrix_with_design_space(width1 as u32, height1 as u32, None);

        let width2 = 1024.0f32;
        let height2 = 768.0f32;

        let projection2 =
            create_projection_matrix_with_design_space(width2 as u32, height2 as u32, None);

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

    #[test]
    fn test_projection_matrix_with_design_space_letterboxes_to_center() {
        let projection =
            create_projection_matrix_with_design_space(1366, 768, Some((1366.0, 884.0)));

        // Top of authored design should still land at top of viewport.
        let top_left_x = 0.0f32;
        let top_left_y = 0.0f32;
        let ndc_x = projection[0][0] * top_left_x + projection[3][0];
        let ndc_y = projection[1][1] * top_left_y + projection[3][1];
        assert!(ndc_x > -1.0, "expected horizontal letterbox inset");
        assert!((ndc_y - 1.0).abs() < 0.001);

        // Bottom of authored design should map to viewport bottom exactly.
        let bottom_y = 884.0f32;
        let ndc_bottom_y = projection[1][1] * bottom_y + projection[3][1];
        assert!((ndc_bottom_y - (-1.0)).abs() < 0.001);

        // Authored design center should remain screen center.
        let center_x = 1366.0f32 * 0.5;
        let center_y = 884.0f32 * 0.5;
        let ndc_center_x = projection[0][0] * center_x + projection[3][0];
        let ndc_center_y = projection[1][1] * center_y + projection[3][1];
        assert!(ndc_center_x.abs() < 0.001);
        assert!(ndc_center_y.abs() < 0.001);
    }

    #[test]
    fn test_map_scene_rect_to_surface_applies_design_space_scale_and_offset() {
        let mapped = map_scene_rect_to_surface_with_design_space(
            plat_core::Rect::new(1185.2, 516.4, 132.8, 43.0),
            1366,
            768,
            Some((1366.0, 884.0)),
        );
        let scale = 768.0 / 884.0;
        let offset_x = (1366.0 - 1366.0 * scale) * 0.5;
        assert!((mapped.x - (1185.2 * scale + offset_x)).abs() < 1e-3);
        assert!((mapped.y - (516.4 * scale)).abs() < 1e-3);
        assert!((mapped.width - (132.8 * scale)).abs() < 1e-3);
        assert!((mapped.height - (43.0 * scale)).abs() < 1e-3);

        let same = map_scene_rect_to_surface_with_design_space(
            plat_core::Rect::new(1.0, 2.0, 3.0, 4.0),
            1366,
            768,
            None,
        );
        assert_eq!(same, plat_core::Rect::new(1.0, 2.0, 3.0, 4.0));
    }

    #[test]
    fn test_aligned_bytes_per_row_rounds_up_to_256_bytes() {
        assert_eq!(aligned_bytes_per_row(1, 4).unwrap(), 256);
        assert_eq!(aligned_bytes_per_row(64, 4).unwrap(), 256);
        assert_eq!(aligned_bytes_per_row(65, 4).unwrap(), 512);
    }

    #[test]
    fn test_unpack_readback_pixels_removes_padding() {
        let width = 2;
        let height = 2;
        let padded_bpr = 256;
        let mut readback = vec![0u8; padded_bpr as usize * height as usize];

        // Row 0 pixels: [1,2,3,4] [5,6,7,8]
        readback[0..8].copy_from_slice(&[1, 2, 3, 4, 5, 6, 7, 8]);
        // Row 1 pixels: [9,10,11,12] [13,14,15,16]
        let row1_start = padded_bpr as usize;
        readback[row1_start..row1_start + 8].copy_from_slice(&[9, 10, 11, 12, 13, 14, 15, 16]);

        let out = unpack_readback_pixels(
            &readback,
            width,
            height,
            padded_bpr,
            wgpu::TextureFormat::Rgba8Unorm,
        )
        .unwrap();
        assert_eq!(
            out,
            vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16]
        );
    }

    #[test]
    fn test_unpack_readback_pixels_swizzles_bgra_to_rgba() {
        let width = 1;
        let height = 1;
        let padded_bpr = 256;
        let mut readback = vec![0u8; padded_bpr as usize];
        // BGRA pixel
        readback[0..4].copy_from_slice(&[3, 2, 1, 255]);

        let out = unpack_readback_pixels(
            &readback,
            width,
            height,
            padded_bpr,
            wgpu::TextureFormat::Bgra8Unorm,
        )
        .unwrap();
        assert_eq!(out, vec![1, 2, 3, 255]);
    }

    #[test]
    fn test_unpack_readback_pixels_overflow_repro() {
        // width * 4 overflows u32 (approx 4GB width)
        // 1_073_741_824 * 4 = 4_294_967_296 (just over u32::MAX)
        let width = 1_073_741_824 + 10;
        let height = 1;
        let padded_bpr = 256;
        let readback = vec![0u8; 1000];

        // This should return an Error now, instead of panicking or returning garbage
        let result = unpack_readback_pixels(
            &readback,
            width,
            height,
            padded_bpr,
            wgpu::TextureFormat::Rgba8Unorm,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_aligned_bytes_per_row_overflow() {
        // width that causes saturation: u32::MAX / 4 + 100
        // u32::MAX is 4,294,967,295.
        // aligned_bytes_per_row uses saturating_mul, so it hits u32::MAX.
        // u32::MAX % 256 = 255.
        // padding needed = 256 - 255 = 1.
        // u32::MAX + 1 overflows.
        let width = u32::MAX / 4 + 100;
        let bytes_per_pixel = 4;
        let result = aligned_bytes_per_row(width, bytes_per_pixel);
        assert!(result.is_err());
    }

    #[test]
    fn test_unpack_readback_pixels_oom() {
        // Request allocation of ~4GB buffer (should fail on most CI runners or return error)
        // width = 32768, height = 32768 -> 1073741824 pixels * 4 bytes = 4GB
        let width = 32768;
        let height = 32768;
        let padded_bpr = width * 4; // aligned
        let readback = vec![]; // Empty readback, we expect allocation error before reading it

        // We expect an error due to OOM or readback buffer size mismatch (if allocation succeeded)
        // But since readback is empty, it fails early if it checks readback len?
        // unpack_readback_pixels calculates dst_end using row math.
        // But allocation happens first.
        let result = unpack_readback_pixels(
            &readback,
            width,
            height,
            padded_bpr,
            wgpu::TextureFormat::Rgba8Unorm,
        );

        // If allocation fails, it returns InitializationFailed("Out of memory...")
        // If allocation succeeds (on 64GB RAM machine), it will hit "Readback buffer too small"
        // Either way, it must not panic.
        assert!(result.is_err());
        let err = result.unwrap_err();
        match err {
            RendererError::InitializationFailed(msg) => {
                assert!(msg.contains("Out of memory") || msg.contains("Readback buffer too small"));
            }
            _ => panic!("Unexpected error type: {:?}", err),
        }
    }
}
