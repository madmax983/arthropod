//! MD3 Switch widget -- toggle switch with sliding thumb indicator.

use crate::theme::MaterialTheme;
use flux_state::{Effect, ReadSignal, Signal, WriteSignal};
use glam::Vec4;
use layout_engine::{FlexAlign, FlexDirection, FlexStyle};
use render_engine::node::NodeContent;
use render_engine::{Color, NodeId};
use std::sync::Arc;
use widget_core::Widget;
use widget_core::{Text, WidgetContext};

// ---------------------------------------------------------------------------
// MD3 fallback colors (used when no MaterialTheme is provided)
// ---------------------------------------------------------------------------

/// MD3 primary (#6750A4)
const FALLBACK_PRIMARY: Vec4 = Vec4::new(0.404, 0.314, 0.643, 1.0);

/// MD3 on_primary (white)
const FALLBACK_ON_PRIMARY: Vec4 = Vec4::new(1.0, 1.0, 1.0, 1.0);

/// MD3 surface_variant (#E7E0EC)
const FALLBACK_SURFACE_VARIANT: Vec4 = Vec4::new(0.906, 0.878, 0.925, 1.0);

/// MD3 outline (#79747E)
const FALLBACK_OUTLINE: Vec4 = Vec4::new(0.475, 0.455, 0.494, 1.0);

/// MD3 Switch toggle.
///
/// A track with a sliding thumb indicator that represents an on/off state.
/// Follows Material Design 3 guidelines for dimensions and color mapping.
///
/// # Example
///
/// ```rust,no_run
/// use flux_state::{Runtime, Signal};
/// use material_ui::inputs::Switch;
///
/// let runtime = Runtime::new();
/// let enabled = Signal::new(runtime, false);
///
/// let switch = Switch::new(enabled)
///     .label("Enable notifications");
/// ```
pub struct Switch {
    read_signal: ReadSignal<bool>,
    write_signal: WriteSignal<bool>,
    disabled: bool,
    label: Option<String>,
}

impl Switch {
    /// Create a new switch wired to a reactive boolean signal.
    ///
    /// The signal is split internally into read/write halves. Clicking
    /// the switch toggles the write side; the read side drives visual updates.
    pub fn new(signal: Signal<bool>) -> Self {
        let (read_signal, write_signal) = signal.split();
        Self {
            read_signal,
            write_signal,
            disabled: false,
            label: None,
        }
    }

    /// Disable interaction. The switch still reflects signal changes visually,
    /// but clicks are ignored.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Attach a text label displayed to the right of the switch track.
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }
}

impl Widget for Switch {
    fn build(&self, ctx: &mut WidgetContext) -> NodeId {
        let runtime = self.read_signal.runtime().clone();
        let read = self.read_signal.clone();
        let write = self.write_signal.clone();

        // -- Resolve theme colors --
        let theme = ctx.get_extension::<MaterialTheme>().cloned();
        let track_on_color = theme
            .as_ref()
            .map(|t| t.color.primary)
            .unwrap_or(FALLBACK_PRIMARY);
        let track_off_color = theme
            .as_ref()
            .map(|t| t.color.surface_variant)
            .unwrap_or(FALLBACK_SURFACE_VARIANT);
        let thumb_on_color = theme
            .as_ref()
            .map(|t| t.color.on_primary)
            .unwrap_or(FALLBACK_ON_PRIMARY);
        let thumb_off_color = theme
            .as_ref()
            .map(|t| t.color.outline)
            .unwrap_or(FALLBACK_OUTLINE);

        // -- Reactive color signals for track and thumb --
        let initial_on = read.get_untracked();

        let initial_track_color = if initial_on {
            track_on_color
        } else {
            track_off_color
        };
        let initial_thumb_color = if initial_on {
            thumb_on_color
        } else {
            thumb_off_color
        };

        let track_color_signal = Signal::new(
            runtime.clone(),
            Color::rgba(
                initial_track_color.x,
                initial_track_color.y,
                initial_track_color.z,
                initial_track_color.w,
            ),
        );
        let (track_color_read, track_color_write) = track_color_signal.split();

        let thumb_color_signal = Signal::new(
            runtime.clone(),
            Color::rgba(
                initial_thumb_color.x,
                initial_thumb_color.y,
                initial_thumb_color.z,
                initial_thumb_color.w,
            ),
        );
        let (thumb_color_read, thumb_color_write) = thumb_color_signal.split();

        // -- Effect: sync boolean state -> track/thumb colors --
        let read_for_effect = read.clone();
        let effect = Effect::new(runtime.clone(), move || {
            let is_on = read_for_effect.get();
            if is_on {
                track_color_write.set(Color::rgba(
                    track_on_color.x,
                    track_on_color.y,
                    track_on_color.z,
                    track_on_color.w,
                ));
                thumb_color_write.set(Color::rgba(
                    thumb_on_color.x,
                    thumb_on_color.y,
                    thumb_on_color.z,
                    thumb_on_color.w,
                ));
            } else {
                track_color_write.set(Color::rgba(
                    track_off_color.x,
                    track_off_color.y,
                    track_off_color.z,
                    track_off_color.w,
                ));
                thumb_color_write.set(Color::rgba(
                    thumb_off_color.x,
                    thumb_off_color.y,
                    thumb_off_color.z,
                    thumb_off_color.w,
                ));
            }
        });
        ctx.store_effect(effect);

        // -- Build scene nodes --

        // 1. Root row container
        let root = ctx.create_node(ctx.root(), NodeContent::Empty);
        ctx.set_layout_style(
            root,
            FlexStyle {
                direction: FlexDirection::Row,
                gap: 12.0,
                align_items: FlexAlign::Center,
                ..Default::default()
            },
        );

        // 2. Track (52x32, full corner radius = 16)
        //    padding_left positions the thumb: OFF=4, ON=24
        let track_padding_left = if initial_on { 24.0 } else { 4.0 };
        let track = ctx.create_node(
            root,
            NodeContent::Styled {
                style: Box::new(
                    render_engine::VisualStyle::new()
                        .solid_fill(initial_track_color)
                        .corner_radius(16.0),
                ),
            },
        );
        ctx.set_layout_style(
            track,
            FlexStyle {
                width: Some(52.0),
                height: Some(32.0),
                padding_left: track_padding_left,
                padding_top: 4.0,
                align_items: FlexAlign::Center,
                ..Default::default()
            },
        );
        ctx.add_reactive_color_state(track, track_color_read);

        // 3. Thumb (24x24, full corner radius = 12)
        let thumb = ctx.create_node(
            track,
            NodeContent::Styled {
                style: Box::new(
                    render_engine::VisualStyle::new()
                        .solid_fill(initial_thumb_color)
                        .corner_radius(12.0),
                ),
            },
        );
        ctx.set_layout_style(
            thumb,
            FlexStyle {
                width: Some(24.0),
                height: Some(24.0),
                ..Default::default()
            },
        );
        ctx.add_reactive_color_state(thumb, thumb_color_read);

        // 4. Click handler on root (toggles signal)
        if !self.disabled {
            let write_clone = write.clone();
            ctx.add_clickable(
                root,
                Arc::new(move || {
                    write_clone.update(|b| *b = !*b);
                }),
            );
        }

        // 5. Optional label
        if let Some(ref label_text) = self.label {
            let label_widget = Text::new(label_text.clone());
            let label_id = label_widget.build(ctx);
            ctx.reparent_to(label_id, root);

            // Also make label clickable if enabled
            if !self.disabled {
                let write_clone = write.clone();
                ctx.add_clickable(
                    label_id,
                    Arc::new(move || {
                        write_clone.update(|b| *b = !*b);
                    }),
                );
            }
        }

        root
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use flux_state::Runtime;

    #[test]
    fn test_switch_builds_successfully() {
        let runtime = Runtime::new();
        let signal = Signal::new(runtime, false);

        let mut ctx = WidgetContext::new_test();
        let switch = Switch::new(signal);
        let node_id = switch.build(&mut ctx);

        // Node should exist in the scene
        assert!(
            ctx.scene().get_node(node_id).is_some(),
            "Switch root node should exist in scene"
        );
    }

    #[test]
    fn test_switch_has_track_and_thumb_nodes() {
        let runtime = Runtime::new();
        let signal = Signal::new(runtime, false);

        let mut ctx = WidgetContext::new_test();
        let switch = Switch::new(signal);
        let root_id = switch.build(&mut ctx);

        let scene = ctx.scene();
        let root_node = scene.get_node(root_id).unwrap();

        // Root should have at least one child (the track)
        assert!(
            !root_node.children.is_empty(),
            "Switch root should have children (track)"
        );

        // Track should have at least one child (the thumb)
        let track_id = root_node.children[0];
        let track_node = scene.get_node(track_id).unwrap();
        assert!(
            !track_node.children.is_empty(),
            "Track should have children (thumb)"
        );
    }

    #[test]
    fn test_switch_with_label() {
        let runtime = Runtime::new();
        let signal = Signal::new(runtime, false);

        let mut ctx = WidgetContext::new_test();
        let switch = Switch::new(signal).label("Dark mode");
        let root_id = switch.build(&mut ctx);

        let scene = ctx.scene();
        let root_node = scene.get_node(root_id).unwrap();

        // Root should have track + label = 2 children
        assert!(
            root_node.children.len() >= 2,
            "Switch with label should have at least 2 children (track + label), got {}",
            root_node.children.len()
        );
    }

    #[test]
    fn test_switch_disabled_not_clickable() {
        let runtime = Runtime::new();
        let signal = Signal::new(runtime, false);

        let mut ctx = WidgetContext::new_test();
        let switch = Switch::new(signal).disabled(true);
        let root_id = switch.build(&mut ctx);

        assert!(
            !ctx.has_clickable(root_id),
            "Disabled switch root should not be clickable"
        );
    }

    #[test]
    fn test_switch_enabled_is_clickable() {
        let runtime = Runtime::new();
        let signal = Signal::new(runtime, false);

        let mut ctx = WidgetContext::new_test();
        let switch = Switch::new(signal);
        let root_id = switch.build(&mut ctx);

        assert!(
            ctx.has_clickable(root_id),
            "Enabled switch root should be clickable"
        );
    }

    #[test]
    fn test_switch_click_toggles_state() {
        let runtime = Runtime::new();
        let signal = Signal::new(runtime, false);
        let observer = signal.clone();
        let (read, _) = observer.split();

        let mut ctx = WidgetContext::new_test();
        let switch = Switch::new(signal);
        let root_id = switch.build(&mut ctx);

        // Initially off
        assert!(!read.get_untracked(), "Switch should start OFF");

        // Toggle via click
        ctx.trigger_click(root_id);
        assert!(read.get_untracked(), "Switch should be ON after click");

        // Toggle back
        ctx.trigger_click(root_id);
        assert!(
            !read.get_untracked(),
            "Switch should be OFF after second click"
        );
    }

    #[test]
    fn test_switch_with_theme() {
        let runtime = Runtime::new();
        let signal = Signal::new(runtime, true);

        let mut ctx = WidgetContext::new_test();
        let theme = MaterialTheme::from_seed(Vec4::new(0.4, 0.2, 0.8, 1.0));
        ctx.set_extension(theme);

        let switch = Switch::new(signal);
        let root_id = switch.build(&mut ctx);

        // Should build without error with a theme present
        assert!(
            ctx.scene().get_node(root_id).is_some(),
            "Switch with theme should build successfully"
        );
    }

    #[test]
    fn test_switch_disabled_with_label_not_clickable() {
        let runtime = Runtime::new();
        let signal = Signal::new(runtime, false);

        let mut ctx = WidgetContext::new_test();
        let switch = Switch::new(signal).label("Option").disabled(true);
        let root_id = switch.build(&mut ctx);

        let scene = ctx.scene();
        let root_node = scene.get_node(root_id).unwrap();

        // Root not clickable
        assert!(
            !ctx.has_clickable(root_id),
            "Disabled switch root should not be clickable"
        );

        // Label not clickable either (second child)
        if root_node.children.len() >= 2 {
            let label_id = root_node.children[1];
            assert!(
                !ctx.has_clickable(label_id),
                "Disabled switch label should not be clickable"
            );
        }
    }
}
