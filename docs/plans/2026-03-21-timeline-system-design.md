# Timeline System Design

**Date**: 2026-03-21
**Status**: Approved
**Crate**: `anim-graph`

## Problem

The `anim-graph` crate has individual animation primitives (Tween, Spring) but no way to:
- Sequence animations (fade in, then slide, then scale)
- Choreograph parallel motion (staggered list entrances)
- Loop without floating-point drift
- Interrupt a running animation and hand off to a spring with velocity preservation
- Import Figma prototype animations as playable timelines

## Inspirations

**Flash timeline model**: A single primitive that scales from "fade button" to "30-second cinematic." Power comes from composition — timelines containing timelines.

**Orpheus pattern algebra** (`orpheus-pattern` crate): The `Pattern` trait's `query(span) -> Vec<Event<T>>` reframed for animation as `evaluate(phase) -> Sample<T>`. Composition operators (`fast`, `slow`, `stack`, `sequence`) map directly to animation combinators. Cycle-based time eliminates drift.

**AletheiaDB hybrid logical clock**: Two-component time `(wallclock: i64, logical: u32)` adapted as `(cycle: u64, phase: f32)`. Integer cycle count is exact; phase only accumulates within 0.0-1.0, then resets. Self-healing delta clamping handles tab backgrounding.

## Architecture

Four layers, each depending only on the one below:

```
┌──────────────────────────────────────────────┐
│  Integration: TimelineDriver<T> + ECS system │  ← writes to WriteSignal<T>
├──────────────────────────────────────────────┤
│  Timeline<T>: clock + evaluable + state      │  ← tick(), interrupt(), builder
├──────────────────────────────────────────────┤
│  Evaluable<T>: pure evaluation core          │  ← Keyframe, Hold, Spring, Sequence...
├──────────────────────────────────────────────┤
│  AnimationClock: cycle + phase time model    │  ← tick(), self-healing, playback modes
└──────────────────────────────────────────────┘
```

## Layer 1: Time Model

### AnimationClock

```rust
pub struct AnimationClock {
    cycle: u64,   // Completed loop iterations (exact integer, no drift)
    phase: f32,   // 0.0-1.0 within current cycle (resets each loop)
}
```

**Why two components**: `cycle` is an integer — no drift after hours of looping. `phase` stays in 0.0-1.0 so error never compounds across cycles. Total time recoverable: `(cycle as f64 * duration) + (phase as f64 * duration)`.

**Self-healing**: Delta clamped to 100ms max. If tab backgrounds for 30 seconds and comes back, the animation doesn't jump 30 seconds ahead — it advances one clamped tick and resumes smoothly. Matches AletheiaDB's "clamp to frontier" pattern.

**Playback modes**: Once, Loop, PingPong, Count(n). PingPong uses cycle parity — odd cycles evaluate at `1.0 - phase`.

### ClockEvent

```rust
pub enum ClockEvent {
    Normal,
    CycleBoundary { completed: u64 },
}
```

Returned from `tick()` to let the timeline decide what happens at loop boundaries.

## Layer 2: Evaluation Core

### Sample<T> — value + velocity envelope

```rust
pub struct Sample<T> {
    pub value: T,
    pub velocity: T,  // T per second
}
```

Every evaluable returns both value and velocity. This is the key insight that unifies springs and timelines — interruption is just reading the current sample and using its velocity to seed a spring. No special handoff protocol.

### Evaluable<T> trait

```rust
pub trait Evaluable<T: Animatable>: Send + Sync {
    fn evaluate(&self, phase: f32) -> Sample<T>;
    fn natural_duration(&self) -> f32;
}
```

Pure function from normalized time to value+velocity. `natural_duration()` is used by sequences to allocate proportional time.

### Concrete evaluables

**Keyframe<T>**: Interpolates from A to B with easing. Velocity is the analytical derivative of the easing curve scaled by `(to - from) / duration`.

**Hold<T>**: Constant value, zero velocity. Used for pauses in sequences.

**SpringSegment<T>**: Physics spring with a time budget. Estimates settling time from stiffness/damping, runs simulation proportional to phase. Can accept initial velocity for interruption chains.

### Composition operators

**Sequence<T>**: Chains evaluables end-to-end. Phase 0.0-1.0 maps proportionally across children based on `natural_duration()` ratios.

**TimeWarp<T>**: Wraps an evaluable and remaps its phase. Enables reverse (`1.0 - t`), speed multiplier (`2t`), custom curves.

**Stagger<T>**: Offsets children's start times within a shared phase window. The choreography primitive — staggered list entrances fall out naturally.

## Layer 3: Timeline

### Timeline<T> — the stateful wrapper

```rust
pub struct Timeline<T: Animatable> {
    root: Box<dyn Evaluable<T>>,
    clock: AnimationClock,
    duration: f32,
    playback: PlaybackMode,
    state: TimelineState<T>,
}

enum TimelineState<T> {
    Playing,
    Interrupted { spring, spring_clock, resume },
    Completed { final_sample },
}
```

**tick(delta) -> Sample<T>**: Advances clock, evaluates root, handles state transitions.

**interrupt(target, spring_config)**: Captures current sample's velocity, spawns a SpringSegment that starts from current value with current velocity toward new target. When settled, transitions to Completed (or resumes if configured).

### Builder API

```rust
// One-liners for the 90% case
let fade = Timeline::tween(1.0, 0.0, Duration::from_millis(300));
let spring = Timeline::spring(Color::RED, Color::BLUE).stiffness(300.0);

// Choreography
let entrance = Timeline::sequence()
    .then_tween(off_screen, on_screen, Duration::from_millis(400))
    .then_hold(Duration::from_millis(100))
    .then_tween(0.0, 1.0, Duration::from_millis(200))
    .build();

// Staggered list
let stagger = Timeline::stagger(Duration::from_millis(50))
    .each(items.iter().map(|_| Timeline::tween(0.0, 1.0, Duration::from_millis(300))))
    .build();
```

## Layer 4: ECS Integration

### TimelineDriver<T>

```rust
#[derive(Component)]
pub struct TimelineDriver<T: Animatable + Send + Sync> {
    pub timeline: Timeline<T>,
    pub target: WriteSignal<T>,
}
```

`timeline_system<T>` ticks all drivers each frame, writes `sample.value` to signals, removes completed drivers. Shares `TimeResource` with existing `animation_system`.

`AnimationDriver<T>` deprecated but kept working for migration period.

## Design Constraints (from reverse brainstorming)

1. **Pattern algebra is an implementation detail** — public API speaks animation language, not music language
2. **Drift-resistant loops** — cycle+phase time model, no float accumulation across cycles
3. **Explicit handoff contract** — Sample { value, velocity } is the interruption interface
4. **Inspectable** — Timeline can report what each track evaluates to at any time T
5. **Simple case stays simple** — `Timeline::tween()` is a one-liner, not a 15-line ceremony
6. **Pure core, effectful shell** — Evaluable is provable, integration layer is testable
