//! MD3 MaterialProgressBar -- linear progress with track + indicator.

use crate::theme::MaterialTheme;
use flux_state::ReadSignal;
use glam::Vec4;
use layout_engine::FlexStyle;
use render_engine::node::NodeContent;
use render_engine::{NodeId, VisualStyle};
use widget_core::WidgetContext;
use widget_core::widget_trait::Widget;

// ---------------------------------------------------------------------------
// MD3 fallback colors
// ---------------------------------------------------------------------------

/// MD3 primary (#6750A4)
const FALLBACK_PRIMARY: Vec4 = Vec4::new(0.404, 0.314, 0.643, 1.0);

/// MD3 surface_variant (#E7E0EC)
const FALLBACK_SURFACE_VARIANT: Vec4 = Vec4::new(0.906, 0.878, 0.925, 1.0);

/// Default height in dp (MD3).
const DEFAULT_HEIGHT: f32 = 4.0;

/// Default width in dp.
const DEFAULT_WIDTH: f32 = 200.0;

/// MD3 progress bar corner radius.
const CORNER_RADIUS: f32 = 2.0;

// ---------------------------------------------------------------------------
// MaterialProgressBar
// ---------------------------------------------------------------------------

/// MD3 linear progress indicator.
///
/// Displays a track (surface_variant) with an indicator (primary) whose width
/// is proportional to the progress value (0.0..1.0).
///
/// # Example
///
/// ```rust,no_run
/// use flux_state::{Runtime, Signal};
/// use material_ui::components::MaterialProgressBar;
///
/// let runtime = Runtime::new();
/// let progress = Signal::new(runtime, 0.5_f32);
/// let (read, _write) = progress.split();
///
/// let bar = MaterialProgressBar::new(read)
///     .width(300.0)
///     .height(6.0);
/// ```
pub struct MaterialProgressBar {
    progress: ReadSignal<f32>,
    height: f32,
    width: f32,
}

impl MaterialProgressBar {
    /// Create a new progress bar from a reactive progress signal (0.0..1.0).
    pub fn new(progress: ReadSignal<f32>) -> Self {
        Self {
            progress,
            height: DEFAULT_HEIGHT,
            width: DEFAULT_WIDTH,
        }
    }

    /// Override the bar height in dp (default: 4.0).
    pub fn height(mut self, h: f32) -> Self {
        self.height = h;
        self
    }

    /// Override the bar width in dp (default: 200.0).
    pub fn width(mut self, w: f32) -> Self {
        self.width = w;
        self
    }
}

impl Widget for MaterialProgressBar {
    fn build(&self, ctx: &mut WidgetContext) -> NodeId {
        // -- Resolve theme colors --
        let theme = ctx.get_extension::<MaterialTheme>().cloned();
        let primary = theme
            .as_ref()
            .map(|t| t.color.primary)
            .unwrap_or(FALLBACK_PRIMARY);
        let surface_variant = theme
            .as_ref()
            .map(|t| t.color.surface_variant)
            .unwrap_or(FALLBACK_SURFACE_VARIANT);

        let progress = self.progress.get_untracked().clamp(0.0, 1.0);
        let indicator_width = self.width * progress;

        // -- Track --
        let track_style = VisualStyle::new()
            .solid_fill(surface_variant)
            .corner_radius(CORNER_RADIUS);
        let track = ctx.create_node(
            ctx.root(),
            NodeContent::Styled {
                style: Box::new(track_style),
            },
        );
        ctx.set_layout_style(
            track,
            FlexStyle {
                width: Some(self.width),
                height: Some(self.height),
                ..Default::default()
            },
        );

        // -- Indicator (only rendered if progress > 0) --
        if progress > 0.0 {
            let indicator_style = VisualStyle::new()
                .solid_fill(primary)
                .corner_radius(CORNER_RADIUS);
            let indicator = ctx.create_node(
                track,
                NodeContent::Styled {
                    style: Box::new(indicator_style),
                },
            );
            ctx.set_layout_style(
                indicator,
                FlexStyle {
                    width: Some(indicator_width),
                    height: Some(self.height),
                    ..Default::default()
                },
            );
        }

        track
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use flux_state::{Runtime, Signal};
    use render_engine::Paint;

    #[test]
    fn test_progress_bar_builds() {
        let runtime = Runtime::new();
        let signal = Signal::new(runtime, 0.5_f32);
        let (read, _) = signal.split();

        let mut ctx = WidgetContext::new_test();
        let bar = MaterialProgressBar::new(read);
        let root_id = bar.build(&mut ctx);

        assert!(
            ctx.scene().get_node(root_id).is_some(),
            "ProgressBar root node should exist"
        );

        // Track should have the indicator child
        let track = ctx.scene().get_node(root_id).unwrap();
        assert!(
            !track.children.is_empty(),
            "50% progress should produce an indicator child"
        );

        // Track should have surface_variant fill
        if let NodeContent::Styled { ref style } = track.content
            && let Paint::Solid(color) = &style.fills[0]
        {
            assert!(
                (color.x - FALLBACK_SURFACE_VARIANT.x).abs() < 0.01,
                "Track should use surface_variant fill, got {color:?}"
            );
        }
    }

    #[test]
    fn test_custom_height() {
        let runtime = Runtime::new();
        let signal = Signal::new(runtime, 0.5_f32);
        let (read, _) = signal.split();

        let bar = MaterialProgressBar::new(read).height(8.0);
        assert!(
            (bar.height - 8.0).abs() < f32::EPSILON,
            "Custom height should be 8.0, got {}",
            bar.height
        );
    }

    #[test]
    fn test_zero_and_full_progress() {
        let runtime = Runtime::new();

        // Zero progress
        let zero_signal = Signal::new(runtime.clone(), 0.0_f32);
        let (zero_read, _) = zero_signal.split();

        let mut ctx = WidgetContext::new_test();
        let bar = MaterialProgressBar::new(zero_read);
        let root_id = bar.build(&mut ctx);

        let track = ctx.scene().get_node(root_id).unwrap();
        assert!(
            track.children.is_empty(),
            "0% progress should not produce an indicator child"
        );

        // Full progress
        let full_signal = Signal::new(runtime, 1.0_f32);
        let (full_read, _) = full_signal.split();

        let mut ctx2 = WidgetContext::new_test();
        let bar2 = MaterialProgressBar::new(full_read);
        let root_id2 = bar2.build(&mut ctx2);

        let track2 = ctx2.scene().get_node(root_id2).unwrap();
        assert!(
            !track2.children.is_empty(),
            "100% progress should produce an indicator child"
        );
    }
}
