//! Frame capture utilities for debugging rendering

use anyhow::{Context, Result};
use std::path::Path;
use wgpu;

/// Captures the current framebuffer to a PNG file
pub struct FrameCapture {
    device: wgpu::Device,
    queue: wgpu::Queue,
}

impl FrameCapture {
    /// Create a new frame capture instance
    pub fn new(device: wgpu::Device, queue: wgpu::Queue) -> Self {
        Self { device, queue }
    }

    /// Capture a texture to a PNG file
    pub async fn capture_texture_to_file(
        &self,
        texture: &wgpu::Texture,
        output_path: impl AsRef<Path>,
    ) -> Result<()> {
        let size = texture.size();
        let format = texture.format();

        // Create a buffer to copy the texture data into
        let buffer_size = (size.width * size.height * 4) as u64; // RGBA8
        let buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Frame Capture Buffer"),
            size: buffer_size,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        // Create a command encoder
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Frame Capture Encoder"),
        });

        // Copy the texture to the buffer
        encoder.copy_texture_to_buffer(
            wgpu::ImageCopyTexture {
                texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::ImageCopyBuffer {
                buffer: &buffer,
                layout: wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(size.width * 4),
                    rows_per_image: Some(size.height),
                },
            },
            size,
        );

        self.queue.submit(Some(encoder.finish()));

        // Map the buffer and read the data
        let buffer_slice = buffer.slice(..);
        let (tx, rx) = std::sync::mpsc::channel();
        buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
            tx.send(result).unwrap();
        });

        self.device.poll(wgpu::Maintain::Wait);
        rx.recv().context("Failed to map buffer")??;

        // Get the data
        let data = buffer_slice.get_mapped_range();

        // Convert to RGBA8 if needed
        let rgba_data: Vec<u8> = match format {
            wgpu::TextureFormat::Rgba8Unorm | wgpu::TextureFormat::Rgba8UnormSrgb => {
                data.to_vec()
            }
            wgpu::TextureFormat::Bgra8Unorm | wgpu::TextureFormat::Bgra8UnormSrgb => {
                // Convert BGRA to RGBA
                data.chunks(4)
                    .flat_map(|pixel| [pixel[2], pixel[1], pixel[0], pixel[3]])
                    .collect()
            }
            _ => {
                anyhow::bail!("Unsupported texture format: {:?}", format);
            }
        };

        drop(data);
        buffer.unmap();

        // Save as PNG
        image::save_buffer(
            output_path,
            &rgba_data,
            size.width,
            size.height,
            image::ColorType::Rgba8,
        )?;

        Ok(())
    }

    /// Capture a texture and return the pixel data
    pub async fn capture_texture_to_buffer(
        &self,
        texture: &wgpu::Texture,
    ) -> Result<Vec<u8>> {
        let size = texture.size();

        let buffer_size = (size.width * size.height * 4) as u64;
        let buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Frame Capture Buffer"),
            size: buffer_size,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Frame Capture Encoder"),
        });

        encoder.copy_texture_to_buffer(
            wgpu::ImageCopyTexture {
                texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::ImageCopyBuffer {
                buffer: &buffer,
                layout: wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(size.width * 4),
                    rows_per_image: Some(size.height),
                },
            },
            size,
        );

        self.queue.submit(Some(encoder.finish()));

        let buffer_slice = buffer.slice(..);
        let (tx, rx) = std::sync::mpsc::channel();
        buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
            tx.send(result).unwrap();
        });

        self.device.poll(wgpu::Maintain::Wait);
        rx.recv().context("Failed to map buffer")??;

        let data = buffer_slice.get_mapped_range().to_vec();
        drop(buffer_slice);
        buffer.unmap();

        Ok(data)
    }
}
