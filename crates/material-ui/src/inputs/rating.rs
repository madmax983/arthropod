//! MD3 Rating widget -- star-based rating input (1-5 by default).
//!
//! Stars are represented as filled/unfilled squares with slight rounding
//! until icon rendering is available. Each star is clickable and sets the
//! signal to that star's index value.

use crate::theme::MaterialTheme;
use flux_state::{Effect, ReadSignal, Signal, WriteSignal};
use glam::Vec4;
use layout_engine::{FlexAlign, FlexDirection, FlexStyle};
use render_engine::node::NodeContent;
use render_engine::{Color, NodeId};
use std::sync::Arc;
use widget_core::WidgetContext;
use widget_core::widget_trait::Widget;

// ---------------------------------------------------------------------------
// MD3 fallback colors (used when no MaterialTheme is provided)
// ---------------------------------------------------------------------------

/// MD3 primary (#6750A4)
const FALLBACK_PRIMARY: Vec4 = Vec4::new(0.404, 0.314, 0.643, 1.0);

/// MD3 outline (#79747E)
const FALLBACK_OUTLINE: Vec4 = Vec4::new(0.475, 0.455, 0.494, 1.0);

/// Default number of stars.
const DEFAULT_MAX: u32 = 5;

/// Default star size in dp.
const DEFAULT_SIZE: f32 = 24.0;

/// MD3 Rating widget.
///
/// A star-based rating input that displays filled/unfilled indicators
/// based on the current value. Stars are rendered as slightly-rounded
/// squares until icon rendering is available.
///
/// # Example
///
/// ```rust,no_run
/// use flux_state::{Runtime, Signal};
/// use material_ui::inputs::Rating;
///
/// let runtime = Runtime::new();
/// let rating = Signal::new(runtime, 3.0_f32);
///
/// let widget = Rating::new(rating)
///     .max(5)
///     .size(32.0);
/// ```
pub struct Rating {
    read_signal: ReadSignal<f32>,
    write_signal: WriteSignal<f32>,
    max: u32,
    disabled: bool,
    size: f32,
}

impl Rating {
    /// Create a new rating widget wired to a reactive `f32` signal.
    ///
    /// The signal is split internally into read/write halves. Clicking
    /// a star sets the write side to that star's index (1-based, as `f32`).
    pub fn new(signal: Signal<f32>) -> Self {
        let (read_signal, write_signal) = signal.split();
        Self {
            read_signal,
            write_signal,
            max: DEFAULT_MAX,
            disabled: false,
            size: DEFAULT_SIZE,
        }
    }

    /// Set the maximum number of stars (default 5).
    pub fn max(mut self, max: u32) -> Self {
        self.max = max;
        self
    }

    /// Disable interaction. The rating still reflects signal changes visually,
    /// but clicks are ignored.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Set the star size in dp (default 24.0). Each star is a square of this
    /// dimension.
    pub fn size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }
}

impl Widget for Rating {
    fn build(&self, ctx: &mut WidgetContext) -> NodeId {
        let runtime = self.read_signal.runtime().clone();
        let read = self.read_signal.clone();
        let write = self.write_signal.clone();

        // -- Resolve theme colors --
        let theme = ctx.get_extension::<MaterialTheme>().cloned();
        let primary = theme
            .as_ref()
            .map(|t| t.color.primary)
            .unwrap_or(FALLBACK_PRIMARY);
        let outline = theme
            .as_ref()
            .map(|t| t.color.outline)
            .unwrap_or(FALLBACK_OUTLINE);

        let star_size = self.size;
        let corner_radius = star_size * 0.1;
        let max = self.max;

        // -- Get initial value for initial star colors --
        let initial_value = read.get_untracked();

        // -- Build scene nodes --

        // 1. Root row container
        let root = ctx.create_node(ctx.root(), NodeContent::Empty);
        ctx.set_layout_style(
            root,
            FlexStyle {
                direction: FlexDirection::Row,
                gap: 4.0,
                align_items: FlexAlign::Center,
                ..Default::default()
            },
        );

        // 2. Create star nodes with reactive color signals
        let mut star_color_writes = Vec::with_capacity(max as usize);

        for i in 1..=max {
            let is_filled = (i as f32) <= initial_value.floor();
            let initial_color = if is_filled { primary } else { outline };

            // Create a reactive color signal for this star
            let star_color_signal = Signal::new(
                runtime.clone(),
                Color::rgba(
                    initial_color.x,
                    initial_color.y,
                    initial_color.z,
                    initial_color.w,
                ),
            );
            let (star_color_read, star_color_write) = star_color_signal.split();
            star_color_writes.push(star_color_write);

            // Create the star node
            let star = ctx.create_node(
                root,
                NodeContent::Styled {
                    style: Box::new(
                        render_engine::VisualStyle::new()
                            .solid_fill(initial_color)
                            .corner_radius(corner_radius),
                    ),
                },
            );
            ctx.set_layout_style(
                star,
                FlexStyle {
                    width: Some(star_size),
                    height: Some(star_size),
                    ..Default::default()
                },
            );
            ctx.add_reactive_color_state(star, star_color_read);

            // Click handler: set signal to this star's index
            if !self.disabled {
                let write_clone = write.clone();
                let star_value = i as f32;
                ctx.add_clickable(
                    star,
                    Arc::new(move || {
                        write_clone.set(star_value);
                    }),
                );
            }
        }

        // -- Effect: sync signal value -> star colors --
        let read_for_effect = read.clone();
        let effect = Effect::new(runtime.clone(), move || {
            let value = read_for_effect.get();
            let filled_count = value.floor() as u32;
            for (idx, color_write) in star_color_writes.iter().enumerate() {
                let star_index = (idx as u32) + 1;
                let color = if star_index <= filled_count {
                    primary
                } else {
                    outline
                };
                color_write.set(Color::rgba(color.x, color.y, color.z, color.w));
            }
        });
        ctx.store_effect(effect);

        root
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use flux_state::Runtime;
    use render_engine::Paint;

    #[test]
    fn test_rating_builds_correct_star_count() {
        let runtime = Runtime::new();
        let signal = Signal::new(runtime, 3.0_f32);

        let mut ctx = WidgetContext::new_test();
        let rating = Rating::new(signal);
        let root_id = rating.build(&mut ctx);

        // Default 5 stars -- root should have 5 children
        let scene = ctx.scene();
        let root_node = scene.get_node(root_id).unwrap();
        assert_eq!(
            root_node.children.len(),
            5,
            "Default rating should have 5 star children, got {}",
            root_node.children.len()
        );
    }

    #[test]
    fn test_rating_default_5_stars() {
        let runtime = Runtime::new();
        let signal = Signal::new(runtime, 0.0_f32);

        let mut ctx = WidgetContext::new_test();
        let rating = Rating::new(signal);

        // Verify max defaults to 5 by building and checking children
        let root_id = rating.build(&mut ctx);
        let scene = ctx.scene();
        let root_node = scene.get_node(root_id).unwrap();
        assert_eq!(
            root_node.children.len(),
            5,
            "Rating max should default to 5"
        );
    }

    #[test]
    fn test_rating_custom_max() {
        let runtime = Runtime::new();
        let signal = Signal::new(runtime, 0.0_f32);

        let mut ctx = WidgetContext::new_test();
        let rating = Rating::new(signal).max(10);
        let root_id = rating.build(&mut ctx);

        let scene = ctx.scene();
        let root_node = scene.get_node(root_id).unwrap();
        assert_eq!(
            root_node.children.len(),
            10,
            "Rating with .max(10) should have 10 star children, got {}",
            root_node.children.len()
        );
    }

    #[test]
    fn test_rating_click_sets_value() {
        let runtime = Runtime::new();
        let signal = Signal::new(runtime, 0.0_f32);
        let observer = signal.clone();
        let (obs_read, _) = observer.split();

        let mut ctx = WidgetContext::new_test();
        let rating = Rating::new(signal);
        let root_id = rating.build(&mut ctx);

        // Click the 3rd star (index 2)
        let scene = ctx.scene();
        let root_node = scene.get_node(root_id).unwrap();
        let third_star = root_node.children[2];

        ctx.trigger_click(third_star);
        assert!(
            (obs_read.get_untracked() - 3.0).abs() < f32::EPSILON,
            "Clicking 3rd star should set signal to 3.0, got {}",
            obs_read.get_untracked()
        );
    }

    #[test]
    fn test_rating_disabled_not_clickable() {
        let runtime = Runtime::new();
        let signal = Signal::new(runtime, 0.0_f32);

        let mut ctx = WidgetContext::new_test();
        let rating = Rating::new(signal).disabled(true);
        let root_id = rating.build(&mut ctx);

        // Each star should not be clickable
        let scene = ctx.scene();
        let root_node = scene.get_node(root_id).unwrap();
        for (i, &star_id) in root_node.children.iter().enumerate() {
            assert!(
                !ctx.has_clickable(star_id),
                "Disabled rating star {} should not be clickable",
                i + 1
            );
        }
    }

    #[test]
    fn test_rating_filled_stars_have_primary_color() {
        let runtime = Runtime::new();
        let signal = Signal::new(runtime, 3.0_f32);

        let mut ctx = WidgetContext::new_test();
        let rating = Rating::new(signal);
        let root_id = rating.build(&mut ctx);

        let scene = ctx.scene();
        let root_node = scene.get_node(root_id).unwrap();

        // First 3 stars should have primary fill, last 2 should have outline fill
        for (i, &star_id) in root_node.children.iter().enumerate() {
            let star_node = scene.get_node(star_id).unwrap();
            if let NodeContent::Styled { ref style } = star_node.content {
                assert!(!style.fills.is_empty(), "Star {} should have a fill", i + 1);
                if let Paint::Solid(color) = &style.fills[0] {
                    if i < 3 {
                        // Filled stars should have primary color
                        assert!(
                            (*color - FALLBACK_PRIMARY).length() < 0.01,
                            "Star {} should have primary fill color, got {:?}",
                            i + 1,
                            color
                        );
                    } else {
                        // Unfilled stars should have outline color
                        assert!(
                            (*color - FALLBACK_OUTLINE).length() < 0.01,
                            "Star {} should have outline fill color, got {:?}",
                            i + 1,
                            color
                        );
                    }
                } else {
                    panic!("Star {} fill should be Solid", i + 1);
                }
            } else {
                panic!("Star {} should be Styled content", i + 1);
            }
        }
    }

    #[test]
    fn test_rating_enabled_stars_are_clickable() {
        let runtime = Runtime::new();
        let signal = Signal::new(runtime, 0.0_f32);

        let mut ctx = WidgetContext::new_test();
        let rating = Rating::new(signal);
        let root_id = rating.build(&mut ctx);

        let scene = ctx.scene();
        let root_node = scene.get_node(root_id).unwrap();
        for (i, &star_id) in root_node.children.iter().enumerate() {
            assert!(
                ctx.has_clickable(star_id),
                "Enabled rating star {} should be clickable",
                i + 1
            );
        }
    }

    #[test]
    fn test_rating_with_theme() {
        let runtime = Runtime::new();
        let signal = Signal::new(runtime, 2.0_f32);

        let mut ctx = WidgetContext::new_test();
        let theme = MaterialTheme::from_seed(Vec4::new(0.4, 0.2, 0.8, 1.0));
        ctx.set_extension(theme);

        let rating = Rating::new(signal);
        let root_id = rating.build(&mut ctx);

        assert!(
            ctx.scene().get_node(root_id).is_some(),
            "Rating with theme should build successfully"
        );
    }

    #[test]
    fn test_rating_custom_size() {
        let runtime = Runtime::new();
        let signal = Signal::new(runtime, 0.0_f32);

        let mut ctx = WidgetContext::new_test();
        let rating = Rating::new(signal).size(48.0);
        let root_id = rating.build(&mut ctx);

        assert!(
            ctx.scene().get_node(root_id).is_some(),
            "Rating with custom size should build successfully"
        );
    }
}
