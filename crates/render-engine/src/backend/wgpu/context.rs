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
    pub transform: [[f32; 4]; 4], // 4x4 projection matrix
}

/// Offscreen render target (color + optional stencil).
pub struct RenderTarget {
    pub key: RenderTargetKey,
    pub color_texture: wgpu::Texture,
    pub color_view: wgpu::TextureView,
    pub stencil_texture: Option<wgpu::Texture>,
    pub stencil_view: Option<wgpu::TextureView>,
}

impl RenderTarget {
    #[must_use]
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
    pub webgpu_available: bool,
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
            // SAFETY: Propagates caller safety requirement into shared async initializer.
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
        // SAFETY: Propagates caller safety requirement into shared async initializer.
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
        let surface = unsafe {
            let target = wgpu::SurfaceTargetUnsafe::from_window(window)?;
            instance.create_surface_unsafe(target)?
        };

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
    pub fn get_render_target(&self, handle: RenderTargetHandle) -> Option<&RenderTarget> {
        self.render_targets.get(&handle)
    }

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
        let padded_bytes_per_row = aligned_bytes_per_row(width, bytes_per_pixel);
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
        );
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
        let padded_bytes_per_row = aligned_bytes_per_row(width, bytes_per_pixel);
        let output_size = padded_bytes_per_row as u64 * height as u64;
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
        );
        drop(mapped);
        output_buffer.unmap();
        Ok(rgba)
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

#[cfg(not(target_arch = "wasm32"))]
fn aligned_bytes_per_row(width: u32, bytes_per_pixel: u32) -> u32 {
    let unaligned = width.saturating_mul(bytes_per_pixel);
    let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
    let rem = unaligned % align;
    if rem == 0 {
        unaligned
    } else {
        unaligned + (align - rem)
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn unpack_readback_pixels(
    readback: &[u8],
    width: u32,
    height: u32,
    padded_bytes_per_row: u32,
    format: wgpu::TextureFormat,
) -> Vec<u8> {
    let row_len = (width * 4) as usize;
    let padded = padded_bytes_per_row as usize;
    let mut out = vec![0u8; row_len * height as usize];

    for row in 0..height as usize {
        let src_start = row * padded;
        let src_end = src_start + row_len;
        let dst_start = row * row_len;
        let dst_end = dst_start + row_len;
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

    out
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

    #[test]
    fn test_aligned_bytes_per_row_rounds_up_to_256_bytes() {
        assert_eq!(aligned_bytes_per_row(1, 4), 256);
        assert_eq!(aligned_bytes_per_row(64, 4), 256);
        assert_eq!(aligned_bytes_per_row(65, 4), 512);
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
        );
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
        );
        assert_eq!(out, vec![1, 2, 3, 255]);
    }
}
