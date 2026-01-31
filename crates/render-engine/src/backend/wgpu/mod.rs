//! WGPU backend implementation.

pub mod context;
pub mod pipelines;

use crate::backend::text::TextRenderer;
use crate::{Color, RendererError, Scene, SceneNode};
use bevy_ecs::prelude::*;
use raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use tracing::{instrument, span, Level};

use context::WgpuContext;
pub use pipelines::rect_pipeline::RectInstance;
use pipelines::rect_pipeline::RectPipeline;
use pipelines::glyph_pipeline::GlyphPipeline;

/// wgpu-based rendering backend.
#[derive(Resource)]
pub struct WgpuBackend {
    pub context: WgpuContext,
    rect_pipeline: RectPipeline,
    glyph_pipeline: GlyphPipeline,
    text_renderer: TextRenderer,
}

impl WgpuBackend {
    /// Create a new wgpu backend from a window.
    #[instrument(skip(window), fields(width, height, composition_mode))]
    pub fn new<W>(
        window: &W,
        width: u32,
        height: u32,
        composition_mode: bool,
    ) -> Result<Self, RendererError>
    where
        W: HasWindowHandle + HasDisplayHandle + Sync,
    {
        let context = WgpuContext::new(window, width, height, composition_mode)?;

        let rect_pipeline = RectPipeline::new(
            &context.device,
            &context.globals_bind_group_layout,
            context.config.format
        );

        let glyph_pipeline = GlyphPipeline::new(
            &context.device,
            &context.globals_buffer,
            context.config.format
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

        self.rect_pipeline.prepare(&self.context.device, &self.context.queue, instances);

        let WgpuBackend {
            context,
            rect_pipeline,
            ..
        } = self;

        context.with_render_pass(|render_pass, globals_bind_group| {
            rect_pipeline.render(
                render_pass,
                globals_bind_group,
                instances.len() as u32
            );
        })
    }

    /// Collect instances from the scene.
    ///
    /// Returns a tuple of (rect_instances, text_nodes).
    /// Text nodes are returned as a list of data needed for shaping: (node, text, font_size, color).
    fn collect_instances<'a>(scene: &'a Scene) -> (Vec<RectInstance>, Vec<(&'a SceneNode, &'a String, f32, Color)>) {
        use crate::NodeContent;
        let mut instances = Vec::new();
        let mut raw_text_nodes = Vec::new();

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
                NodeContent::Text {
                    text,
                    font_size,
                    color,
                } => {
                    raw_text_nodes.push((node, text, *font_size, *color));
                }
                NodeContent::Empty => {}
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

        self.rect_pipeline.prepare(&self.context.device, &self.context.queue, &instances);

        // Process text nodes
        let mut glyph_instances = Vec::new();
        if !raw_text_nodes.is_empty() {
            for (node, text, font_size, color) in &raw_text_nodes {
                if text.is_empty() {
                    continue;
                }

                let shaped = self
                    .text_renderer
                    .text_engine_mut()
                    .shape_text(text, *font_size);

                let position = glam::Vec2::new(node.bounds.x, node.bounds.y);
                let text_color =
                    glam::Vec4::new(color.r(), color.g(), color.b(), color.a() * node.opacity);

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
            &glyph_instances
        );

        let WgpuBackend {
            context,
            rect_pipeline,
            glyph_pipeline,
            ..
        } = self;

        context.with_render_pass(|render_pass, globals_bind_group| {
            // Render rectangles
            rect_pipeline.render(
                render_pass,
                globals_bind_group,
                instances.len() as u32
            );

            // Render glyphs
            glyph_pipeline.render(
                render_pass,
                glyph_instances.len() as u32
            );
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
    }
}
