//! Layer-based compositor API for selective transparency.
//!
//! This module provides a high-level API for creating composition layers
//! with different backdrop materials (Mica, Acrylic, etc.).
//!
//! # Example
//!
//! ```ignore
//! // Create compositor from window
//! let compositor = Compositor::new(&window)?;
//!
//! // Create a Mica layer for the sidebar
//! let sidebar_layer = compositor.create_layer(BackdropMaterial::Mica)?;
//! sidebar_layer.set_bounds(Rect::new(0.0, 0.0, 200.0, 600.0));
//!
//! // Create a solid layer for content
//! let content_layer = compositor.create_layer(BackdropMaterial::None)?;
//! content_layer.set_bounds(Rect::new(200.0, 0.0, 600.0, 600.0));
//!
//! // Commit changes to make them visible
//! compositor.commit()?;
//! ```

use crate::materials::BackdropMaterial;
use crate::window::Rect;
use crate::PlatformError;

/// A compositor manages layers for selective transparency effects.
///
/// The compositor creates a visual tree where each layer can have
/// a different backdrop material. Layers are rendered back-to-front
/// in the order they were created.
pub struct Compositor {
    #[cfg(target_os = "windows")]
    inner: Option<platform::CompositorImpl>,
    #[cfg(not(target_os = "windows"))]
    inner: Option<()>,
}

impl Compositor {
    /// Creates a new compositor for a window.
    ///
    /// Returns `None` if the window doesn't have composition mode enabled.
    pub fn new(window: &crate::Window) -> Result<Option<Self>, PlatformError> {
        #[cfg(target_os = "windows")]
        {
            platform::CompositorImpl::new(window).map(|inner| Some(Compositor { inner }))
        }
        #[cfg(not(target_os = "windows"))]
        {
            let _ = window;
            Ok(None)
        }
    }

    /// Creates a new layer with the specified backdrop material.
    ///
    /// Layers are rendered in creation order (first created = bottom).
    pub fn create_layer(&mut self, material: BackdropMaterial) -> Result<Layer, PlatformError> {
        #[cfg(target_os = "windows")]
        {
            if let Some(ref mut inner) = self.inner {
                inner.create_layer(material)
            } else {
                Err(PlatformError::Initialization(
                    "Compositor not initialized".into(),
                ))
            }
        }
        #[cfg(not(target_os = "windows"))]
        {
            let _ = material;
            Err(PlatformError::Initialization(
                "Compositor not supported on this platform".into(),
            ))
        }
    }

    /// Commits all pending changes to make them visible.
    ///
    /// Call this after creating/modifying layers.
    pub fn commit(&self) -> Result<(), PlatformError> {
        #[cfg(target_os = "windows")]
        {
            if let Some(ref inner) = self.inner {
                inner.commit()
            } else {
                Ok(())
            }
        }
        #[cfg(not(target_os = "windows"))]
        {
            Ok(())
        }
    }
}

/// A composition layer with a backdrop material.
///
/// Each layer can have its own material (Mica, Acrylic, None) and bounds.
/// Content rendered to this layer will show the backdrop material behind it.
pub struct Layer {
    #[cfg(target_os = "windows")]
    inner: platform::LayerImpl,
    #[cfg(not(target_os = "windows"))]
    inner: (),
}

impl Layer {
    /// Sets the bounds of this layer in window coordinates.
    pub fn set_bounds(&mut self, bounds: Rect) -> Result<(), PlatformError> {
        #[cfg(target_os = "windows")]
        {
            self.inner.set_bounds(bounds)
        }
        #[cfg(not(target_os = "windows"))]
        {
            let _ = bounds;
            Ok(())
        }
    }

    /// Gets the current bounds of this layer.
    pub fn bounds(&self) -> Rect {
        #[cfg(target_os = "windows")]
        {
            self.inner.bounds()
        }
        #[cfg(not(target_os = "windows"))]
        {
            Rect::default()
        }
    }

    /// Gets the backdrop material for this layer.
    pub fn material(&self) -> BackdropMaterial {
        #[cfg(target_os = "windows")]
        {
            self.inner.material()
        }
        #[cfg(not(target_os = "windows"))]
        {
            BackdropMaterial::None
        }
    }
}

#[cfg(target_os = "windows")]
mod platform {
    use super::*;
    use crate::platform::windows::composition::{CompositionDevice, CompositionVisual};

    pub struct CompositorImpl {
        device: CompositionDevice,
        layers: Vec<LayerImpl>,
    }

    // Wrapper to make CompositionVisual clonable via Arc
    struct CompositionVisualWrapper {
        #[allow(dead_code)]
        visual: CompositionVisual,
    }

    #[derive(Clone)]
    pub struct LayerImpl {
        #[allow(dead_code)]
        visual: std::sync::Arc<CompositionVisualWrapper>,
        material: BackdropMaterial,
        bounds: Rect,
    }

    impl LayerImpl {
        pub fn set_bounds(&mut self, bounds: Rect) -> Result<(), PlatformError> {
            self.bounds = bounds;
            // TODO: Update visual transform/clip
            Ok(())
        }

        pub fn bounds(&self) -> Rect {
            self.bounds
        }

        pub fn material(&self) -> BackdropMaterial {
            self.material
        }
    }

    impl CompositorImpl {
        pub fn new(_window: &crate::Window) -> Result<Option<Self>, PlatformError> {
            // For now, create a standalone device
            // TODO: Share device with WindowComposition when available
            unsafe {
                let _ = windows::Win32::System::Com::CoInitializeEx(
                    None,
                    windows::Win32::System::Com::COINIT_APARTMENTTHREADED,
                );
            }

            let device = CompositionDevice::new()
                .map_err(|e| PlatformError::Initialization(format!("CompositionDevice: {}", e)))?;

            Ok(Some(Self {
                device,
                layers: Vec::new(),
            }))
        }

        pub fn create_layer(&mut self, material: BackdropMaterial) -> Result<Layer, PlatformError> {
            let visual = self
                .device
                .create_visual()
                .map_err(|e| PlatformError::Initialization(format!("Create visual: {}", e)))?;

            let wrapped = std::sync::Arc::new(CompositionVisualWrapper { visual });

            let layer_impl = LayerImpl {
                visual: wrapped,
                material,
                bounds: Rect::default(),
            };

            self.layers.push(layer_impl.clone());

            Ok(Layer { inner: layer_impl })
        }

        pub fn commit(&self) -> Result<(), PlatformError> {
            // TODO: Call IDCompositionDevice::Commit()
            // For now, this is a placeholder
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layer_bounds() {
        let bounds = Rect::new(10.0, 20.0, 100.0, 200.0);

        #[cfg(not(target_os = "windows"))]
        {
            let mut layer = Layer { inner: () };
            assert!(layer.set_bounds(bounds).is_ok());
        }
    }
}
