//! Image widget - displays a bitmap image

use crate::{Widget, WidgetContext};
use render_engine::{NodeContent, NodeId, Paint, VisualStyle};
use layout_engine::FlexStyle;
use style_engine::{ImageId, ImageScaleMode, ImageFill};

/// Image widget
/// 
/// Displays an image asset loaded into the renderer's image store.
/// 
/// # Example
/// 
/// ```no_run
/// use widget_core::Image;
/// use style_engine::ImageId;
/// 
/// let widget = Image::new(ImageId(1)).width(100.0).height(100.0);
/// ```
#[derive(crate::Widget)]
#[widget(name = "img", skip_impl)]
pub struct Image {
    #[positional]
    image_id: ImageId,
    
    #[param]
    width: Option<f32>,
    
    #[param]
    height: Option<f32>,
    
    #[param(default = ImageScaleMode::Fit)]
    scale_mode: ImageScaleMode,
}

impl Image {
    /// Create a new Image widget with the given asset ID
    pub fn new(image_id: ImageId) -> Self {
        Self {
            image_id,
            width: None,
            height: None,
            scale_mode: ImageScaleMode::Fit,
        }
    }

    /// Set fixed width
    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    /// Set fixed height
    pub fn height(mut self, height: f32) -> Self {
        self.height = Some(height);
        self
    }

    /// Set scaling mode (Fill, Fit, Crop, Tile, Stretch)
    pub fn scale_mode(mut self, mode: ImageScaleMode) -> Self {
        self.scale_mode = mode;
        self
    }
}

impl Widget for Image {
    fn build(&self, ctx: &mut WidgetContext) -> NodeId {
        // Create node with image fill
        let node_id = ctx.create_node(
            ctx.root(),
            NodeContent::Styled {
                style: Box::new(
                    VisualStyle::new()
                        .fill(Paint::Image(ImageFill {
                            image_id: self.image_id,
                            scale_mode: self.scale_mode,
                            transform: None,
                        })),
                ),
            },
        );

        // Configure layout
        ctx.set_layout_style(node_id, FlexStyle {
            width: self.width,
            height: self.height,
            ..Default::default()
        });

        node_id
    }
}
