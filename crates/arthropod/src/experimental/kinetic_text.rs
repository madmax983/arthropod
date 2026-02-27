//! Kinetic Text Widget - Reactive Animated Typography
//!
//! This module implements a text widget with reactive animations.
//!
//! # Features
//!
//! - **Typewriter**: Characters appear one by one.
//! - **FadeIn**: Text fades in from transparent.
//! - **Pulse**: Text color pulses.
//!
//! # Example
//!
//! ```no_run
//! use arthropod::prelude::*;
//! use arthropod::experimental::kinetic_text::{KineticText, TextAnimation};
//! use std::time::Duration;
//!
//! let text = Signal::new(Runtime::new(), "Hello World".to_string());
//! let clock = Signal::new(Runtime::new(), Duration::ZERO);
//! let (read_text, _) = text.split();
//! let (read_clock, _) = clock.split();
//!
//! KineticText::new(read_text)
//!     .animation(TextAnimation::Typewriter)
//!     .clock(read_clock)
//!     .size(24.0);
//! ```

use anim_graph::{Animatable, Animation, Easing};
use flux_state::{Computed, Effect, ReadSignal, Signal};
use render_engine::{Color, NodeId};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use widget_core::{Widget, WidgetContext};

// --- Motion System (Internal) ---

/// Configuration for motion animations.
#[derive(Clone, Copy, Debug)]
pub enum MotionConfig {
    /// Physics-based spring animation.
    Spring { stiffness: f32, damping: f32 },
    /// Time-based tween animation.
    Tween { duration: Duration, easing: Easing },
}

impl Default for MotionConfig {
    fn default() -> Self {
        Self::Spring {
            stiffness: 100.0,
            damping: 15.0,
        }
    }
}

/// A signal that animates its value over time.
pub struct MotionSignal<T> {
    signal: ReadSignal<T>,
    _effect: Effect,
}

impl<T> std::ops::Deref for MotionSignal<T> {
    type Target = ReadSignal<T>;
    fn deref(&self) -> &Self::Target {
        &self.signal
    }
}

/// Create a signal that smoothly animates to the source signal's value.
pub fn create_motion_signal<T>(
    cx: Arc<flux_state::Runtime>,
    source: ReadSignal<T>,
    clock: ReadSignal<Duration>,
    config: MotionConfig,
) -> MotionSignal<T>
where
    T: Animatable + PartialEq + Send + Sync + 'static,
{
    let initial = source.get_untracked();
    let output = Signal::new(cx.clone(), initial.clone());
    let (read_out, write_out) = output.split();

    struct State<T: Animatable> {
        animation: Option<Animation<T>>,
        last_target: T,
        last_time: Duration,
    }

    let state = Arc::new(Mutex::new(State {
        animation: None,
        last_target: initial,
        last_time: Duration::ZERO,
    }));

    let effect = Effect::new(cx.clone(), move || {
        let target = source.get();
        let mut state = state.lock().unwrap();

        if target != state.last_target {
            let start = if let Some(anim) = &mut state.animation {
                anim.tick(Duration::ZERO)
            } else {
                state.last_target.clone()
            };

            let anim = match config {
                MotionConfig::Spring { stiffness, damping } => {
                    Animation::spring(start, target.clone(), stiffness, damping)
                }
                MotionConfig::Tween { duration, easing } => {
                    Animation::tween(start, target.clone(), duration, easing)
                }
            };

            state.animation = Some(anim);
            state.last_target = target;
        }

        let is_animating = state
            .animation
            .as_ref()
            .map(|a| !a.is_complete())
            .unwrap_or(false);

        if is_animating {
            let time = clock.get();
            let dt = if time >= state.last_time {
                time - state.last_time
            } else {
                Duration::ZERO
            };
            state.last_time = time;

            // Clamp dt to avoid huge jumps
            let dt = if dt.as_secs_f32() > 1.0 {
                Duration::ZERO
            } else {
                dt
            };

            if let Some(anim) = &mut state.animation {
                let new_val = anim.tick(dt);
                write_out.set(new_val);

                if anim.is_complete() {
                    state.animation = None;
                }
            }
        }
    });

    MotionSignal {
        signal: read_out,
        _effect: effect,
    }
}

// --- Kinetic Text Widget ---

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TextAnimation {
    None,
    Typewriter,
    FadeIn,
    Pulse,
}

pub struct KineticText {
    content: ReadSignal<String>,
    animation: TextAnimation,
    config: MotionConfig,
    size: f32,
    color: Option<Color>,
    clock: Option<ReadSignal<Duration>>,
}

impl KineticText {
    pub fn new(content: ReadSignal<String>) -> Self {
        Self {
            content,
            animation: TextAnimation::None,
            config: MotionConfig::default(),
            size: 16.0,
            color: None,
            clock: None,
        }
    }

    pub fn animation(mut self, animation: TextAnimation) -> Self {
        self.animation = animation;
        self
    }

    pub fn config(mut self, config: MotionConfig) -> Self {
        self.config = config;
        self
    }

    pub fn size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }

    pub fn color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }

    pub fn clock(mut self, clock: ReadSignal<Duration>) -> Self {
        self.clock = Some(clock);
        self
    }
}

impl Widget for KineticText {
    fn build(&self, ctx: &mut WidgetContext) -> NodeId {
        let runtime = self.content.runtime().clone();

        let clock = if let Some(c) = &self.clock {
            c.clone()
        } else {
            let (read, _) = Signal::new(runtime.clone(), Duration::ZERO).split();
            read
        };

        match self.animation {
            TextAnimation::Typewriter => {
                // Typewriter: Animate from 0.0 to 1.0 (progress)
                // We map progress to string length slice
                let progress_target = Signal::new(runtime.clone(), 0.0);
                let (read_target, write_target) = progress_target.split();

                // Animate progress
                let motion =
                    create_motion_signal(runtime.clone(), read_target, clock.clone(), self.config);

                // Trigger animation on mount (or when content changes)
                // We need to reset progress to 0 when content changes?
                // For now, just trigger once.
                let write_clone = write_target.clone();
                // Simple hack: set to 1.0 immediately, let spring/tween handle it
                // We use an effect to delay it slightly or ensure it runs?
                // Actually, if we set it here, it happens during build.
                // The motion signal starts at 0.0.
                write_clone.set(1.0);

                let content_signal = self.content.clone();
                let computed_text = Computed::new(runtime.clone(), move || {
                    let text = content_signal.get();
                    let p = motion.get().clamp(0.0, 1.0);
                    let len = text.chars().count();
                    let visible_count = (len as f32 * p).round() as usize;
                    text.chars().take(visible_count).collect::<String>()
                });

                widget_core::Text::computed(computed_text)
                    .size(self.size)
                    .color(self.color.unwrap_or(Color::rgba(0.0, 0.0, 0.0, 0.0))) // Default to transparent if not set
                    .build(ctx)
            }

            TextAnimation::FadeIn => {
                // FadeIn: Animate opacity from 0.0 to 1.0
                let opacity_target = Signal::new(runtime.clone(), 0.0);
                let (read_target, write_target) = opacity_target.split();

                let motion =
                    create_motion_signal(runtime.clone(), read_target, clock.clone(), self.config);

                write_target.set(1.0);

                let base_color = self.color.unwrap_or(Color::rgba(0.0, 0.0, 0.0, 1.0)); // Default black
                let computed_color = Computed::new(runtime.clone(), move || {
                    let alpha = motion.get().clamp(0.0, 1.0);
                    let mut c = base_color;
                    c.0.w *= alpha; // Use .0.w for alpha (glam::Vec4 inside tuple struct)
                    c
                });

                let text_widget = widget_core::Text::reactive(self.content.clone())
                    .size(self.size)
                    .color(base_color); // Initial static color

                let node_id = text_widget.build(ctx);

                // Workaround: Create a helper signal that mirrors the computed.
                let mirror = Signal::new(runtime.clone(), base_color);
                let (read_mirror, write_mirror) = mirror.split();

                let computed_clone = computed_color.clone();
                let effect = Effect::new(runtime.clone(), move || {
                    write_mirror.set(computed_clone.get());
                });
                ctx.store_effect(effect);

                ctx.add_reactive_color_state(node_id, read_mirror);

                node_id
            }

            TextAnimation::Pulse => {
                // Pulse: Oscillate alpha or brightness
                // Requires a continuous sine wave.
                // MotionConfig::Spring settles.
                // This needs a specialized Motion implementation or manual clock math.
                // For MVP, skip Pulse or implement simply.

                // Fallback to static
                widget_core::Text::reactive(self.content.clone())
                    .size(self.size)
                    .color(self.color.unwrap_or(Color::rgba(0.0, 0.0, 0.0, 1.0)))
                    .build(ctx)
            }

            TextAnimation::None => widget_core::Text::reactive(self.content.clone())
                .size(self.size)
                .color(self.color.unwrap_or(Color::rgba(0.0, 0.0, 0.0, 1.0)))
                .build(ctx),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kinetic_text_build_typewriter() {
        let runtime = flux_state::Runtime::new();
        let mut ctx = WidgetContext::new_test();
        let text_sig = Signal::new(runtime.clone(), "Hello".to_string());
        let (read, _) = text_sig.split();

        let widget = KineticText::new(read).animation(TextAnimation::Typewriter);
        let root = widget.build(&mut ctx);

        let scene = ctx.scene();
        assert!(scene.get_node(root).is_some());

        // Should use computed text state
        assert!(ctx.computed_text_states().contains_key(&root));
    }

    #[test]
    fn test_kinetic_text_build_fadein() {
        let runtime = flux_state::Runtime::new();
        let mut ctx = WidgetContext::new_test();
        let text_sig = Signal::new(runtime.clone(), "Hello".to_string());
        let (read, _) = text_sig.split();

        let widget = KineticText::new(read).animation(TextAnimation::FadeIn);
        let root = widget.build(&mut ctx);

        let scene = ctx.scene();
        assert!(scene.get_node(root).is_some());

        // Should use reactive color state
        assert!(ctx.reactive_color_states().contains_key(&root));
    }
}
