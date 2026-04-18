//! GPU render pipelines for drawing, filtering, and compositing.

/// Blend compositing pass for layer opacity and non-normal blend modes.
pub mod blend_pipeline;
/// Two-pass separable blur pass for layer and background blurs.
pub mod blur_pipeline;
/// Color filter post-process pass (grayscale, contrast, invert).
pub mod color_filter_pipeline;
/// Gradient atlas texture management.
pub mod gradient_atlas;
/// Vector path tessellation and rendering.
pub mod path_pipeline;
/// Builder for turning styles into primitive instances.
pub mod primitive_builder;
/// Raw GPU vertex and instance layouts.
pub mod primitive_instance;
/// Main 2D primitive rendering pass.
pub mod primitive_pipeline;
/// Stencil masking and clipping pass.
pub mod stencil_pipeline;
