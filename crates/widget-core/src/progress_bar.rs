//! ProgressBar widget - visual indicator of completion progress

use crate::{style, DesignTokens, Style};
use crate::{Widget, WidgetContext};
use flux_state::ReadSignal;
use layout_engine::{FlexDirection, FlexStyle};
use render_engine::{NodeContent, NodeId};

/// ProgressBar widget
///
/// Displays a horizontal track with a filled bar representing progress (0.0 to 1.0).
///
/// # Example
///
/// ```no_run
/// use widget_core::ProgressBar;
/// # use flux_state::{Runtime, Signal};
/// # let runtime = Runtime::new();
/// let progress = Signal::new(runtime, 0.5);
/// let (read, _) = progress.split();
///
/// let widget = ProgressBar::new(read);
/// ```
#[derive(crate::Widget)]
#[widget(name = "progress_bar", skip_impl)]
pub struct ProgressBar {
    #[positional]
    progress: ReadSignal<f32>,

    #[param(default = 8.0)]
    height: f32,

    #[param]
    style: Option<Style>,
}

impl ProgressBar {
    /// Create a new ProgressBar wired to a reactive progress signal (0.0 to 1.0)
    pub fn new(progress: ReadSignal<f32>) -> Self {
        Self {
            progress,
            height: 8.0,
            style: None,
        }
    }

    /// Set the height of the progress bar
    pub fn height(mut self, height: f32) -> Self {
        self.height = height;
        self
    }

    /// Set a high-level style override for the track
    pub fn style(mut self, style: Style) -> Self {
        self.style = Some(style);
        self
    }

    fn create_default_style(&self, tokens: Option<&DesignTokens>) -> Style {
        match tokens {
            Some(t) => {
                style! {
                    background: t.surface_secondary.clone();
                    border_radius: self.height / 2.0;
                    height: self.height;
                }
            }
            None => {
                style! {
                    background: glam::Vec4::new(0.0, 0.0, 0.0, 0.1); // Transparent black track
                    border_radius: self.height / 2.0;
                    height: self.height;
                }
            }
        }
    }
}

impl Widget for ProgressBar {
    fn build(&self, ctx: &mut WidgetContext) -> NodeId {
        let tokens = ctx.design_tokens().cloned();
        let track_style = self
            .style
            .clone()
            .unwrap_or_else(|| self.create_default_style(tokens.as_ref()));

        // 1. Create Track Node (Static Background)
        let track_node = ctx.create_node(
            ctx.root(),
            NodeContent::Styled {
                style: Box::new(
                    render_engine::VisualStyle::new()
                        .solid_fill(glam::Vec4::new(0.0, 0.0, 0.0, 0.1)),
                ),
            },
        );
        ctx.set_widget_style(track_node, track_style);

        ctx.set_layout_style(
            track_node,
            FlexStyle {
                direction: FlexDirection::Row,
                width: Some(200.0),
                height: Some(self.height),
                ..Default::default()
            },
        );

        // 2. Create Progress Bar Fill Node (Styled, width driven by ECS)
        let bar_color = tokens
            .as_ref()
            .map(|t| t.accent)
            .unwrap_or_else(|| glam::Vec4::new(0.0, 0.47, 0.84, 1.0));

        let bar_node = ctx.create_node(
            track_node,
            NodeContent::Styled {
                style: Box::new(
                    render_engine::VisualStyle::new()
                        .solid_fill(bar_color)
                        .corner_radius(self.height / 2.0),
                ),
            },
        );

        // Initial width from current progress value
        let initial_p = self.progress.get_untracked().clamp(0.0, 1.0);
        ctx.set_layout_style(
            bar_node,
            FlexStyle {
                width: Some(initial_p * 200.0),
                height: Some(self.height),
                ..Default::default()
            },
        );

        // Register progress bar state — the ECS system updates layout width each frame
        ctx.add_progress_bar_state(bar_node, self.progress.clone(), 200.0);

        track_node
    }
}
