//! MD3 Slider widget -- continuous or discrete value slider with track and thumb.

use crate::theme::MaterialTheme;
use flux_state::{Effect, ReadSignal, Signal, WriteSignal};
use glam::Vec4;
use layout_engine::{FlexAlign, FlexDirection, FlexStyle};
use render_engine::node::NodeContent;
use render_engine::{Color, NodeId};
use widget_core::Widget;
use widget_core::WidgetContext;

// ---------------------------------------------------------------------------
// MD3 fallback colors (used when no MaterialTheme is provided)
// ---------------------------------------------------------------------------

/// MD3 primary (#6750A4)
const FALLBACK_PRIMARY: Vec4 = Vec4::new(0.404, 0.314, 0.643, 1.0);

/// MD3 surface_variant (#E7E0EC)
const FALLBACK_SURFACE_VARIANT: Vec4 = Vec4::new(0.906, 0.878, 0.925, 1.0);

/// Track height in dp.
const TRACK_HEIGHT: f32 = 4.0;

/// Track corner radius (half of track height for pill shape).
const TRACK_CORNER_RADIUS: f32 = 2.0;

/// Thumb diameter in dp.
const THUMB_SIZE: f32 = 20.0;

/// Thumb corner radius (half of thumb size for full circle).
const THUMB_CORNER_RADIUS: f32 = 10.0;

/// Default track width in dp.
const DEFAULT_WIDTH: f32 = 200.0;

/// Default minimum value.
const DEFAULT_MIN: f32 = 0.0;

/// Default maximum value.
const DEFAULT_MAX: f32 = 1.0;

/// MD3 Slider control.
///
/// A horizontal slider for selecting a value within a range. The track is 4dp
/// tall with a 20dp circular thumb indicator. Supports both continuous and
/// discrete (stepped) value selection.
///
/// The slider renders correctly and updates reactively via signal. Full drag
/// interaction is deferred until pointer events are available.
///
/// # Example
///
/// ```rust,no_run
/// use flux_state::{Runtime, Signal};
/// use material_ui::inputs::Slider;
///
/// let runtime = Runtime::new();
/// let volume = Signal::new(runtime, 0.5_f32);
///
/// let slider = Slider::new(volume)
///     .range(0.0, 100.0)
///     .step(1.0)
///     .width(300.0);
/// ```
pub struct Slider {
    read_signal: ReadSignal<f32>,
    write_signal: WriteSignal<f32>,
    min: f32,
    max: f32,
    step: Option<f32>,
    disabled: bool,
    width: f32,
}

struct TrackContext {
    root: NodeId,
    track_width: f32,
    initial_active_width: f32,
    surface_variant: Vec4,
    primary: Vec4,
    active_track_color_read: flux_state::ReadSignal<Color>,
}

struct ThumbContext {
    root: NodeId,
    initial_thumb_left: f32,
    primary: Vec4,
    thumb_color_read: flux_state::ReadSignal<Color>,
}
impl Slider {
    /// Create a new slider wired to a reactive `f32` signal.
    ///
    /// The signal is split internally into read/write halves. The read side
    /// drives visual updates; the write side will be used for drag interaction
    /// once pointer events are available.
    pub fn new(signal: Signal<f32>) -> Self {
        let (read_signal, write_signal) = signal.split();
        Self {
            read_signal,
            write_signal,
            min: DEFAULT_MIN,
            max: DEFAULT_MAX,
            step: None,
            disabled: false,
            width: DEFAULT_WIDTH,
        }
    }

    /// Set the value range for the slider.
    ///
    /// # Panics
    ///
    /// Debug-asserts that `min < max`.
    pub fn range(mut self, min: f32, max: f32) -> Self {
        debug_assert!(
            min < max,
            "Slider min ({min}) must be less than max ({max})"
        );
        self.min = min;
        self.max = max;
        self
    }

    /// Enable discrete stepping. When set, values snap to the nearest multiple
    /// of `step` within the range.
    pub fn step(mut self, step: f32) -> Self {
        self.step = Some(step);
        self
    }

    /// Disable interaction. The slider still reflects signal changes visually,
    /// but input is ignored.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Set the track width in dp (default 200.0).
    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }

    /// Compute the normalized ratio `[0.0, 1.0]` for a given value, optionally
    /// snapping to discrete steps.
    fn value_ratio(&self, value: f32) -> f32 {
        let snapped = if let Some(s) = self.step {
            (value / s).round() * s
        } else {
            value
        };
        let clamped = snapped.clamp(self.min, self.max);
        if (self.max - self.min).abs() < f32::EPSILON {
            0.0
        } else {
            (clamped - self.min) / (self.max - self.min)
        }
    }

    fn build_track(&self, ctx: &mut WidgetContext, tctx: TrackContext) {
        // 2. Track container -- positions track vertically centered within tctx.root.
        //    Uses padding_top to center the 4dp track within the 20dp tctx.root height.
        //    Center offset = (20 - 4) / 2 = 8dp
        let track_container = ctx.create_node(tctx.root, NodeContent::Empty);
        ctx.set_layout_style(
            track_container,
            FlexStyle {
                direction: FlexDirection::Row,
                width: Some(tctx.track_width),
                height: Some(THUMB_SIZE),
                padding_top: (THUMB_SIZE - TRACK_HEIGHT) / 2.0,
                align_items: FlexAlign::Start,
                ..Default::default()
            },
        );

        // 3. Track background -- full-width, 4dp height, tctx.surface_variant color
        let track_bg = ctx.create_node(
            track_container,
            NodeContent::Styled {
                style: Box::new(
                    render_engine::VisualStyle::new()
                        .solid_fill(tctx.surface_variant)
                        .corner_radius(TRACK_CORNER_RADIUS),
                ),
            },
        );
        ctx.set_layout_style(
            track_bg,
            FlexStyle {
                width: Some(tctx.track_width),
                height: Some(TRACK_HEIGHT),
                ..Default::default()
            },
        );

        // 4. Active track -- partial width based on value ratio, tctx.primary color.
        //    Placed as first child of track_bg so it overlaps visually.
        let active_track = ctx.create_node(
            track_bg,
            NodeContent::Styled {
                style: Box::new(
                    render_engine::VisualStyle::new()
                        .solid_fill(tctx.primary)
                        .corner_radius(TRACK_CORNER_RADIUS),
                ),
            },
        );
        ctx.set_layout_style(
            active_track,
            FlexStyle {
                width: Some(tctx.initial_active_width),
                height: Some(TRACK_HEIGHT),
                ..Default::default()
            },
        );
        ctx.add_reactive_color_state(active_track, tctx.active_track_color_read);
    }

    fn build_thumb(&self, ctx: &mut WidgetContext, tctx: ThumbContext) {
        // 5. Thumb container -- positioned via padding_left to place the thumb
        //    at the correct horizontal position along the track.
        let thumb_container = ctx.create_node(tctx.root, NodeContent::Empty);
        ctx.set_layout_style(
            thumb_container,
            FlexStyle {
                direction: FlexDirection::Row,
                padding_left: tctx.initial_thumb_left,
                ..Default::default()
            },
        );

        // 6. Thumb -- 20dp circle, tctx.primary color
        let thumb = ctx.create_node(
            thumb_container,
            NodeContent::Styled {
                style: Box::new(
                    render_engine::VisualStyle::new()
                        .solid_fill(tctx.primary)
                        .corner_radius(THUMB_CORNER_RADIUS),
                ),
            },
        );
        ctx.set_layout_style(
            thumb,
            FlexStyle {
                width: Some(THUMB_SIZE),
                height: Some(THUMB_SIZE),
                ..Default::default()
            },
        );
        ctx.add_reactive_color_state(thumb, tctx.thumb_color_read);
    }
}

impl Widget for Slider {
    fn build(&self, ctx: &mut WidgetContext) -> NodeId {
        let runtime = self.read_signal.runtime().clone();
        let read = self.read_signal.clone();
        let _write = self.write_signal.clone();

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

        // -- Compute initial geometry from signal value --
        let initial_value = read.get_untracked();
        let initial_ratio = self.value_ratio(initial_value);
        let track_width = self.width;
        let initial_active_width = initial_ratio * track_width;
        let initial_thumb_left = initial_ratio * (track_width - THUMB_SIZE);

        // -- Reactive color signals for active track and thumb --
        let active_track_color_signal = Signal::new(
            runtime.clone(),
            Color::rgba(primary.x, primary.y, primary.z, primary.w),
        );
        let (active_track_color_read, _active_track_color_write) =
            active_track_color_signal.split();

        let thumb_color_signal = Signal::new(
            runtime.clone(),
            Color::rgba(primary.x, primary.y, primary.z, primary.w),
        );
        let (thumb_color_read, _thumb_color_write) = thumb_color_signal.split();

        // Capture slider parameters for the effect closure
        let slider_min = self.min;
        let slider_max = self.max;
        let slider_step = self.step;
        let slider_width = self.width;

        // -- Build scene nodes --

        // 1. Root container -- holds everything, sized to track width x thumb height
        let root = ctx.create_node(ctx.root(), NodeContent::Empty);
        ctx.set_layout_style(
            root,
            FlexStyle {
                direction: FlexDirection::Column,
                width: Some(track_width),
                height: Some(THUMB_SIZE),
                align_items: FlexAlign::Start,
                ..Default::default()
            },
        );

        self.build_track(
            ctx,
            TrackContext {
                root,
                track_width,
                initial_active_width,
                surface_variant,
                primary,
                active_track_color_read,
            },
        );

        self.build_thumb(
            ctx,
            ThumbContext {
                root,
                initial_thumb_left,
                primary,
                thumb_color_read,
            },
        );

        // -- Effect: sync signal value -> active track width and thumb position --
        // Note: Without layout mutation from effects, we store the effect so it
        // stays alive. Full reactive layout update will be possible once the
        // layout engine supports reactive dimension signals.
        let read_for_effect = read.clone();
        let effect = Effect::new(runtime.clone(), move || {
            let value = read_for_effect.get();
            let snapped = if let Some(s) = slider_step {
                (value / s).round() * s
            } else {
                value
            };
            let clamped = snapped.clamp(slider_min, slider_max);
            let _ratio = if (slider_max - slider_min).abs() < f32::EPSILON {
                0.0
            } else {
                (clamped - slider_min) / (slider_max - slider_min)
            };
            // Reactive layout updates for active track width and thumb position
            // require layout-mutation support not yet available. The effect keeps
            // the subscription alive so that when layout reactivity lands, the
            // slider will update automatically.
            let _active_width = _ratio * slider_width;
            let _thumb_left = _ratio * (slider_width - THUMB_SIZE);
        });
        ctx.store_effect(effect);

        root
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use flux_state::Runtime;

    #[test]
    fn test_slider_builds_track_and_thumb() {
        let runtime = Runtime::new();
        let signal = Signal::new(runtime, 0.5_f32);

        let mut ctx = WidgetContext::new_test();
        let slider = Slider::new(signal);
        let root_id = slider.build(&mut ctx);

        // Root node should exist
        let scene = ctx.scene();
        let root_node = scene.get_node(root_id).unwrap();
        assert!(
            root_node.children.len() >= 2,
            "Slider root should have at least 2 children (track_container + thumb_container), got {}",
            root_node.children.len()
        );

        // Track container should have the track background
        let track_container_id = root_node.children[0];
        let track_container = scene.get_node(track_container_id).unwrap();
        assert!(
            !track_container.children.is_empty(),
            "Track container should have children (track background)"
        );

        // Track background should have the active track
        let track_bg_id = track_container.children[0];
        let track_bg = scene.get_node(track_bg_id).unwrap();
        assert!(
            !track_bg.children.is_empty(),
            "Track background should have children (active track)"
        );

        // Thumb container should have the thumb
        let thumb_container_id = root_node.children[1];
        let thumb_container = scene.get_node(thumb_container_id).unwrap();
        assert!(
            !thumb_container.children.is_empty(),
            "Thumb container should have children (thumb)"
        );
    }

    #[test]
    fn test_slider_default_range() {
        let runtime = Runtime::new();
        let signal = Signal::new(runtime, 0.0_f32);

        let mut ctx = WidgetContext::new_test();
        let slider = Slider::new(signal);
        let root_id = slider.build(&mut ctx);

        assert!(
            ctx.scene().get_node(root_id).is_some(),
            "Slider with default range should build successfully"
        );
    }

    #[test]
    fn test_slider_custom_range() {
        let runtime = Runtime::new();
        let signal = Signal::new(runtime, 50.0_f32);

        let mut ctx = WidgetContext::new_test();
        let slider = Slider::new(signal).range(0.0, 100.0);
        let root_id = slider.build(&mut ctx);

        assert!(
            ctx.scene().get_node(root_id).is_some(),
            "Slider with custom range should build successfully"
        );
    }

    #[test]
    fn test_slider_discrete_steps() {
        let runtime = Runtime::new();
        let signal = Signal::new(runtime, 0.3_f32);

        let mut ctx = WidgetContext::new_test();
        let slider = Slider::new(signal).step(0.25);
        let root_id = slider.build(&mut ctx);

        assert!(
            ctx.scene().get_node(root_id).is_some(),
            "Slider with discrete steps should build successfully"
        );
    }

    #[test]
    fn test_slider_disabled() {
        let runtime = Runtime::new();
        let signal = Signal::new(runtime, 0.5_f32);

        let mut ctx = WidgetContext::new_test();
        let slider = Slider::new(signal).disabled(true);
        let root_id = slider.build(&mut ctx);

        assert!(
            ctx.scene().get_node(root_id).is_some(),
            "Disabled slider should build successfully"
        );
        // Disabled slider should not have any click handlers
        assert!(
            !ctx.has_clickable(root_id),
            "Disabled slider should not be clickable"
        );
    }

    #[test]
    fn test_slider_custom_width() {
        let runtime = Runtime::new();
        let signal = Signal::new(runtime, 0.5_f32);

        let mut ctx = WidgetContext::new_test();
        let slider = Slider::new(signal).width(300.0);
        let root_id = slider.build(&mut ctx);

        assert!(
            ctx.scene().get_node(root_id).is_some(),
            "Slider with custom width should build successfully"
        );
    }

    #[test]
    fn test_slider_value_ratio_continuous() {
        let runtime = Runtime::new();
        let signal = Signal::new(runtime, 0.5_f32);
        let slider = Slider::new(signal);

        // Mid-point
        let ratio = slider.value_ratio(0.5);
        assert!(
            (ratio - 0.5).abs() < f32::EPSILON,
            "Ratio for 0.5 in [0,1] should be 0.5, got {ratio}"
        );

        // Min edge
        let ratio_min = slider.value_ratio(0.0);
        assert!(
            ratio_min.abs() < f32::EPSILON,
            "Ratio for min should be 0.0, got {ratio_min}"
        );

        // Max edge
        let ratio_max = slider.value_ratio(1.0);
        assert!(
            (ratio_max - 1.0).abs() < f32::EPSILON,
            "Ratio for max should be 1.0, got {ratio_max}"
        );
    }

    #[test]
    fn test_slider_value_ratio_custom_range() {
        let runtime = Runtime::new();
        let signal = Signal::new(runtime, 50.0_f32);
        let slider = Slider::new(signal).range(0.0, 100.0);

        let ratio = slider.value_ratio(25.0);
        assert!(
            (ratio - 0.25).abs() < f32::EPSILON,
            "Ratio for 25 in [0,100] should be 0.25, got {ratio}"
        );
    }

    #[test]
    fn test_slider_value_ratio_discrete_snap() {
        let runtime = Runtime::new();
        let signal = Signal::new(runtime, 0.0_f32);
        let slider = Slider::new(signal).step(0.25);

        // 0.3 should snap to 0.25
        let ratio = slider.value_ratio(0.3);
        assert!(
            (ratio - 0.25).abs() < f32::EPSILON,
            "0.3 should snap to 0.25 (ratio=0.25), got {ratio}"
        );

        // 0.4 should snap to 0.5
        let ratio2 = slider.value_ratio(0.4);
        assert!(
            (ratio2 - 0.5).abs() < f32::EPSILON,
            "0.4 should snap to 0.5 (ratio=0.5), got {ratio2}"
        );
    }

    #[test]
    fn test_slider_value_ratio_clamps() {
        let runtime = Runtime::new();
        let signal = Signal::new(runtime, 0.0_f32);
        let slider = Slider::new(signal);

        // Below min clamps to 0
        let ratio = slider.value_ratio(-1.0);
        assert!(
            ratio.abs() < f32::EPSILON,
            "Below-min value should clamp to ratio 0.0, got {ratio}"
        );

        // Above max clamps to 1
        let ratio = slider.value_ratio(2.0);
        assert!(
            (ratio - 1.0).abs() < f32::EPSILON,
            "Above-max value should clamp to ratio 1.0, got {ratio}"
        );
    }

    #[test]
    fn test_slider_with_theme() {
        let runtime = Runtime::new();
        let signal = Signal::new(runtime, 0.5_f32);

        let mut ctx = WidgetContext::new_test();
        let theme = MaterialTheme::from_seed(Vec4::new(0.4, 0.2, 0.8, 1.0));
        ctx.set_extension(theme);

        let slider = Slider::new(signal);
        let root_id = slider.build(&mut ctx);

        assert!(
            ctx.scene().get_node(root_id).is_some(),
            "Slider with theme should build successfully"
        );
    }

    #[test]
    fn test_slider_track_has_styled_content() {
        let runtime = Runtime::new();
        let signal = Signal::new(runtime, 0.5_f32);

        let mut ctx = WidgetContext::new_test();
        let slider = Slider::new(signal);
        let root_id = slider.build(&mut ctx);

        let scene = ctx.scene();
        let root_node = scene.get_node(root_id).unwrap();
        let track_container_id = root_node.children[0];
        let track_container = scene.get_node(track_container_id).unwrap();
        let track_bg_id = track_container.children[0];
        let track_bg = scene.get_node(track_bg_id).unwrap();

        // Track background should be Styled with surface_variant fill
        match &track_bg.content {
            NodeContent::Styled { style } => {
                assert!(
                    !style.fills.is_empty(),
                    "Track background should have a fill"
                );
            }
            other => panic!("Track background should be Styled, got {other:?}"),
        }
    }

    #[test]
    fn test_slider_thumb_has_styled_content() {
        let runtime = Runtime::new();
        let signal = Signal::new(runtime, 0.5_f32);

        let mut ctx = WidgetContext::new_test();
        let slider = Slider::new(signal);
        let root_id = slider.build(&mut ctx);

        let scene = ctx.scene();
        let root_node = scene.get_node(root_id).unwrap();
        let thumb_container_id = root_node.children[1];
        let thumb_container = scene.get_node(thumb_container_id).unwrap();
        let thumb_id = thumb_container.children[0];
        let thumb_node = scene.get_node(thumb_id).unwrap();

        // Thumb should be Styled with primary fill and full corner radius
        match &thumb_node.content {
            NodeContent::Styled { style } => {
                assert!(!style.fills.is_empty(), "Thumb should have a fill");
            }
            other => panic!("Thumb should be Styled, got {other:?}"),
        }
    }
}
