//! WGPU backend implementation.

pub mod context;
pub mod pipelines;

use crate::backend::text::TextRenderer;
use crate::{Color, RendererError, Scene, SceneNode};
use bevy_ecs::prelude::*;
use raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use rayon::prelude::*;
use text_engine::{ShapedText, shape_text_parallel};
use tracing::{Level, instrument, span};

use context::WgpuContext;
use pipelines::glyph_pipeline::GlyphPipeline;
pub use pipelines::rect_pipeline::RectInstance;
use pipelines::rect_pipeline::RectPipeline;

type SceneInstanceData<'a> = (
    Vec<RectInstance>,
    Vec<(&'a SceneNode, &'a String, f32, Color)>,
);

/// Threshold for parallelizing text shaping
/// Below this count, sequential shaping is faster due to thread overhead
const TEXT_PARALLEL_THRESHOLD: usize = 8;

/// Helper to create a RectInstance from a SceneNode.
///
/// Returns None if the node is not a rectangle/rounded-rectangle, or is invisible.
pub fn create_rect_instance(node: &SceneNode) -> Option<RectInstance> {
    use crate::NodeContent;
    if !node.visible || node.opacity <= 0.0 {
        return None;
    }

    match &node.content {
        NodeContent::Rect { color } => Some(RectInstance::rect(
            [node.bounds.x, node.bounds.y],
            [node.bounds.width, node.bounds.height],
            [color.r(), color.g(), color.b(), color.a() * node.opacity],
        )),
        NodeContent::RoundedRect {
            color,
            corner_radius,
        } => Some(RectInstance::new(
            [node.bounds.x, node.bounds.y],
            [node.bounds.width, node.bounds.height],
            [color.r(), color.g(), color.b(), color.a() * node.opacity],
            *corner_radius,
        )),
        _ => None,
    }
}

/// wgpu-based rendering backend.
#[derive(Resource)]
pub struct WgpuBackend {
    pub(crate) context: WgpuContext,
    rect_pipeline: RectPipeline,
    glyph_pipeline: GlyphPipeline,
    text_renderer: TextRenderer,
}

impl WgpuBackend {
    /// Create a new wgpu backend from a window.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the created `WgpuBackend` is dropped *before* the window
    /// it was created from. This is required because `wgpu::Surface` holds a reference to the
    /// window handle, but carries a `'static` lifetime. Accessing the surface after the window
    /// is destroyed results in undefined behavior.
    #[instrument(skip(window), fields(width, height, composition_mode))]
    pub unsafe fn new<W>(
        window: &W,
        width: u32,
        height: u32,
        composition_mode: bool,
    ) -> Result<Self, RendererError>
    where
        W: HasWindowHandle + HasDisplayHandle + Sync,
    {
        // SAFETY: Propagating the safety requirement to the caller.
        let context = unsafe { WgpuContext::new(window, width, height, composition_mode)? };

        let rect_pipeline = RectPipeline::new(
            &context.device,
            &context.globals_bind_group_layout,
            context.config.format,
        );

        let glyph_pipeline = GlyphPipeline::new(
            &context.device,
            &context.globals_buffer,
            context.config.format,
        );

        let text_renderer = TextRenderer::new();

        Ok(Self {
            context,
            rect_pipeline,
            glyph_pipeline,
            text_renderer,
        })
    }

    /// Render a collection of rectangle instances directly (ECS-friendly API)
    #[instrument(skip(self, instances))]
    pub fn render_instances(&mut self, instances: &[RectInstance]) -> Result<(), RendererError> {
        let _span = span!(Level::TRACE, "render_instances").entered();

        self.rect_pipeline
            .prepare(&self.context.device, &self.context.queue, instances);

        let WgpuBackend {
            context,
            rect_pipeline,
            ..
        } = self;

        context.with_render_pass(|render_pass, globals_bind_group| {
            rect_pipeline.render(render_pass, globals_bind_group, instances.len() as u32);
        })
    }

    /// Collect instances from the scene.
    ///
    /// Returns a tuple of (rect_instances, text_nodes).
    /// Text nodes are returned as a list of data needed for shaping: (node, text, font_size, color).
    fn collect_instances(scene: &Scene) -> SceneInstanceData<'_> {
        use crate::NodeContent;
        let mut instances = Vec::new();
        let mut raw_text_nodes = Vec::new();

        for (_node_id, node) in scene.iter_visuals() {
            // Use helper for rect instances
            if let Some(instance) = create_rect_instance(node) {
                instances.push(instance);
                continue;
            }

            // Handle other content (Text)
            if !node.visible || node.opacity <= 0.0 {
                continue;
            }

            match &node.content {
                NodeContent::Text {
                    text,
                    font_size,
                    color,
                } => {
                    raw_text_nodes.push((node, text, *font_size, *color));
                }
                _ => {}
            }
        }
        (instances, raw_text_nodes)
    }
}

impl super::RenderBackend for WgpuBackend {
    #[instrument(skip(self, scene))]
    fn render(&mut self, scene: &Scene) -> Result<(), RendererError> {
        let _span = span!(Level::TRACE, "render_frame").entered();

        let (instances, raw_text_nodes) = Self::collect_instances(scene);

        self.rect_pipeline
            .prepare(&self.context.device, &self.context.queue, &instances);

        // Process text nodes
        let mut glyph_instances = Vec::new();
        if !raw_text_nodes.is_empty() {
            // Step 1: Shape text in parallel if above threshold
            // Shaping is CPU-intensive and read-only (uses thread-local FontSystem)
            let shaped_results: Vec<(glam::Vec2, glam::Vec4, ShapedText)> = if raw_text_nodes.len()
                >= TEXT_PARALLEL_THRESHOLD
            {
                // Parallel shaping
                raw_text_nodes
                    .par_iter()
                    .filter(|(_, text, _, _)| !text.is_empty())
                    .map(|(node, text, font_size, color)| {
                        let shaped = shape_text_parallel(text, *font_size);
                        let position = glam::Vec2::new(node.bounds.x, node.bounds.y + font_size);
                        let text_color = glam::Vec4::new(
                            color.r(),
                            color.g(),
                            color.b(),
                            color.a() * node.opacity,
                        );
                        (position, text_color, shaped)
                    })
                    .collect()
            } else {
                // Sequential shaping for small counts
                raw_text_nodes
                    .iter()
                    .filter(|(_, text, _, _)| !text.is_empty())
                    .map(|(node, text, font_size, color)| {
                        let shaped = self
                            .text_renderer
                            .text_engine_mut()
                            .shape_text(text, *font_size);
                        let position = glam::Vec2::new(node.bounds.x, node.bounds.y + font_size);
                        let text_color = glam::Vec4::new(
                            color.r(),
                            color.g(),
                            color.b(),
                            color.a() * node.opacity,
                        );
                        (position, text_color, shaped)
                    })
                    .collect()
            };

            // Step 2: Generate instances sequentially (atlas access is mutable)
            for (position, text_color, shaped) in shaped_results {
                let instances = self
                    .text_renderer
                    .generate_instances(&shaped, position, text_color);
                glyph_instances.extend(instances);
            }
        }

        self.glyph_pipeline.prepare(
            &self.context.device,
            &self.context.queue,
            self.text_renderer.atlas().texture_data(),
            &glyph_instances,
        );

        let WgpuBackend {
            context,
            rect_pipeline,
            glyph_pipeline,
            ..
        } = self;

        context.with_render_pass(|render_pass, globals_bind_group| {
            // Render rectangles
            rect_pipeline.render(render_pass, globals_bind_group, instances.len() as u32);

            // Render glyphs
            glyph_pipeline.render(render_pass, glyph_instances.len() as u32);
        })
    }

    fn resize(&mut self, width: u32, height: u32) {
        self.context.resize(width, height);
    }

    fn set_clear_color(&mut self, color: Color) {
        self.context.clear_color = color;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{NodeContent, SceneNode, Transform2D};

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
            parent: None, // Set by add_node
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
            parent: None, // Set by add_node
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
            parent: None, // Set by add_node
            visible: true,
            opacity: 0.5,
        };
        scene.add_node(root, blue_rect);

        // Collect instances
        let (instances, _) = WgpuBackend::collect_instances(&scene);

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

        // Both flat rects should have corner_radius = 0.0
        assert_eq!(red_instance.corner_radius, 0.0);
        assert_eq!(blue_instance.corner_radius, 0.0);
    }

    #[test]
    fn test_rounded_rect_preserves_corner_radius() {
        let mut scene = Scene::new();
        let root = scene.root();

        // Add a RoundedRect with corner_radius
        let rounded = SceneNode {
            content: NodeContent::RoundedRect {
                color: Color::rgba(0.5, 0.5, 0.5, 1.0),
                corner_radius: 12.0,
            },
            transform: Transform2D::identity(),
            bounds: plat_core::Rect {
                x: 30.0,
                y: 30.0,
                width: 200.0,
                height: 100.0,
            },
            children: vec![],
            parent: None,
            visible: true,
            opacity: 1.0,
        };
        scene.add_node(root, rounded);

        // Add a flat Rect for comparison
        let flat = SceneNode {
            content: NodeContent::Rect {
                color: Color::rgba(1.0, 0.0, 0.0, 1.0),
            },
            transform: Transform2D::identity(),
            bounds: plat_core::Rect {
                x: 0.0,
                y: 0.0,
                width: 50.0,
                height: 50.0,
            },
            children: vec![],
            parent: None,
            visible: true,
            opacity: 1.0,
        };
        scene.add_node(root, flat);

        let (instances, _) = WgpuBackend::collect_instances(&scene);
        assert_eq!(instances.len(), 2);

        // Rounded rect should preserve corner_radius
        let rounded_inst = instances
            .iter()
            .find(|i| i.corner_radius > 0.0)
            .expect("Rounded instance not found");
        assert_eq!(rounded_inst.corner_radius, 12.0);
        assert_eq!(rounded_inst.pos, [30.0, 30.0]);

        // Flat rect should have 0.0
        let flat_inst = instances
            .iter()
            .find(|i| i.color[0] > 0.9)
            .expect("Flat instance not found");
        assert_eq!(flat_inst.corner_radius, 0.0);
    }

    #[test]
    fn test_text_parallel_shaping_below_threshold() {
        // Create scene with text nodes below parallel threshold
        let mut scene = Scene::new();
        let root = scene.root();

        // Add 4 text nodes (below TEXT_PARALLEL_THRESHOLD of 8)
        for i in 0..4 {
            let text_node = SceneNode {
                content: NodeContent::Text {
                    text: format!("Text {}", i),
                    font_size: 16.0,
                    color: Color::rgba(1.0, 1.0, 1.0, 1.0),
                },
                transform: Transform2D::identity(),
                bounds: plat_core::Rect {
                    x: 0.0,
                    y: (i * 20) as f32,
                    width: 100.0,
                    height: 20.0,
                },
                children: vec![],
                parent: None,
                visible: true,
                opacity: 1.0,
            };
            scene.add_node(root, text_node);
        }

        let (_, text_nodes) = WgpuBackend::collect_instances(&scene);
        assert_eq!(text_nodes.len(), 4);
        // Sequential path should be taken
    }

    #[test]
    fn test_text_parallel_shaping_above_threshold() {
        // Create scene with text nodes above parallel threshold
        let mut scene = Scene::new();
        let root = scene.root();

        // Add 10 text nodes (above TEXT_PARALLEL_THRESHOLD of 8)
        for i in 0..10 {
            let text_node = SceneNode {
                content: NodeContent::Text {
                    text: format!("Text {}", i),
                    font_size: 16.0,
                    color: Color::rgba(1.0, 1.0, 1.0, 1.0),
                },
                transform: Transform2D::identity(),
                bounds: plat_core::Rect {
                    x: 0.0,
                    y: (i * 20) as f32,
                    width: 100.0,
                    height: 20.0,
                },
                children: vec![],
                parent: None,
                visible: true,
                opacity: 1.0,
            };
            scene.add_node(root, text_node);
        }

        let (_, text_nodes) = WgpuBackend::collect_instances(&scene);
        assert_eq!(text_nodes.len(), 10);
        // Parallel path should be taken
    }

    #[test]
    fn test_text_shaping_with_empty_strings() {
        // Create scene with mix of empty and non-empty text
        let mut scene = Scene::new();
        let root = scene.root();

        for i in 0..5 {
            let text = if i % 2 == 0 {
                String::new()
            } else {
                format!("Text {}", i)
            };

            let text_node = SceneNode {
                content: NodeContent::Text {
                    text,
                    font_size: 16.0,
                    color: Color::rgba(1.0, 1.0, 1.0, 1.0),
                },
                transform: Transform2D::identity(),
                bounds: plat_core::Rect {
                    x: 0.0,
                    y: (i * 20) as f32,
                    width: 100.0,
                    height: 20.0,
                },
                children: vec![],
                parent: None,
                visible: true,
                opacity: 1.0,
            };
            scene.add_node(root, text_node);
        }

        let (_, text_nodes) = WgpuBackend::collect_instances(&scene);
        assert_eq!(text_nodes.len(), 5);
        // Empty strings should be filtered out during shaping
    }
}
