//! MD3 Backdrop (scrim overlay).

use render_engine::NodeId;
use render_engine::node::NodeContent;
use std::sync::Arc;
use widget_core::WidgetContext;
use widget_core::layer::Layer;
use widget_core::widget_trait::Widget;

/// MD3 Backdrop -- full-screen scrim overlay.
///
/// Renders in the [`Layer::Dialog`] layer as a semi-transparent black overlay,
/// typically used behind modal dialogs and bottom sheets.
///
/// # Defaults
///
/// * Opacity: 0.32 (per MD3 scrim spec)
///
/// # Example
///
/// ```no_run
/// use material_ui::feedback::Backdrop;
///
/// let backdrop = Backdrop::new()
///     .opacity(0.5)
///     .on_click(|| println!("backdrop tapped -- dismiss modal"));
/// ```
pub struct Backdrop {
    opacity: f32,
    on_click: Option<Arc<dyn Fn() + Send + Sync>>,
}

impl Backdrop {
    /// Create a new backdrop with the MD3 default scrim opacity (0.32).
    pub fn new() -> Self {
        Self {
            opacity: 0.32,
            on_click: None,
        }
    }

    /// Override the scrim opacity (0.0 = fully transparent, 1.0 = fully opaque).
    pub fn opacity(mut self, opacity: f32) -> Self {
        self.opacity = opacity;
        self
    }

    /// Set a click handler, typically used to dismiss the modal when
    /// the user taps outside the content area.
    pub fn on_click(mut self, callback: impl Fn() + Send + Sync + 'static) -> Self {
        self.on_click = Some(Arc::new(callback));
        self
    }
}

impl Default for Backdrop {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for Backdrop {
    fn build(&self, ctx: &mut WidgetContext) -> NodeId {
        let color = render_engine::Color::rgba(0.0, 0.0, 0.0, self.opacity);
        let node_id = ctx.add_to_layer(Layer::Dialog, NodeContent::SolidColor { color });
        if let Some(ref callback) = self.on_click {
            ctx.add_clickable(node_id, callback.clone());
        }
        node_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, Ordering};

    #[test]
    fn test_backdrop_builds_in_dialog_layer() {
        let mut ctx = WidgetContext::new_test();
        let backdrop = Backdrop::new();
        let node_id = backdrop.build(&mut ctx);

        let dialog_root = ctx.layer_root(Layer::Dialog);
        let scene = ctx.scene();
        let dialog_node = scene.get_node(dialog_root).unwrap();
        assert!(
            dialog_node.children.contains(&node_id),
            "Backdrop node should be a child of the Dialog layer root"
        );
    }

    #[test]
    fn test_backdrop_default_opacity() {
        let mut ctx = WidgetContext::new_test();
        let backdrop = Backdrop::new();
        let node_id = backdrop.build(&mut ctx);

        let scene = ctx.scene();
        let node = scene.get_node(node_id).unwrap();
        match &node.content {
            NodeContent::SolidColor { color } => {
                assert!(color.r() < 0.01, "Backdrop red channel should be ~0");
                assert!(color.g() < 0.01, "Backdrop green channel should be ~0");
                assert!(color.b() < 0.01, "Backdrop blue channel should be ~0");
                assert!(
                    (color.a() - 0.32).abs() < 0.01,
                    "Backdrop alpha should be ~0.32, got {}",
                    color.a()
                );
            }
            other => panic!("Backdrop should use SolidColor content, got {other:?}"),
        }
    }

    #[test]
    fn test_backdrop_custom_opacity() {
        let mut ctx = WidgetContext::new_test();
        let backdrop = Backdrop::new().opacity(0.5);
        let node_id = backdrop.build(&mut ctx);

        let scene = ctx.scene();
        let node = scene.get_node(node_id).unwrap();
        match &node.content {
            NodeContent::SolidColor { color } => {
                assert!(
                    (color.a() - 0.5).abs() < 0.01,
                    "Backdrop alpha should be ~0.5, got {}",
                    color.a()
                );
            }
            other => panic!("Backdrop should use SolidColor content, got {other:?}"),
        }
    }

    #[test]
    fn test_backdrop_on_click_registers_handler() {
        let clicked = Arc::new(AtomicBool::new(false));
        let clicked_clone = clicked.clone();

        let mut ctx = WidgetContext::new_test();
        let backdrop = Backdrop::new().on_click(move || {
            clicked_clone.store(true, Ordering::SeqCst);
        });
        let node_id = backdrop.build(&mut ctx);

        assert!(
            ctx.has_clickable(node_id),
            "Backdrop with on_click should register a clickable"
        );
    }

    #[test]
    fn test_backdrop_without_on_click_has_no_handler() {
        let mut ctx = WidgetContext::new_test();
        let backdrop = Backdrop::new();
        let node_id = backdrop.build(&mut ctx);

        assert!(
            !ctx.has_clickable(node_id),
            "Backdrop without on_click should not register a clickable"
        );
    }

    #[test]
    fn test_backdrop_on_click_fires_callback() {
        let clicked = Arc::new(AtomicBool::new(false));
        let clicked_clone = clicked.clone();

        let mut ctx = WidgetContext::new_test();
        let backdrop = Backdrop::new().on_click(move || {
            clicked_clone.store(true, Ordering::SeqCst);
        });
        let node_id = backdrop.build(&mut ctx);

        // Simulate click via the context helper
        ctx.trigger_click(node_id);

        assert!(
            clicked.load(Ordering::SeqCst),
            "Backdrop click callback should have been invoked"
        );
    }

    #[test]
    fn test_backdrop_default_is_same_as_new() {
        let a = Backdrop::new();
        let b = Backdrop::default();
        // Both should produce the same opacity and no callback
        assert!((a.opacity - b.opacity).abs() < f32::EPSILON);
        assert!(a.on_click.is_none());
        assert!(b.on_click.is_none());
    }
}
