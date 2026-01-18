# ADR 0006: Animation System Architecture (anim-graph)

**Status:** Accepted

**Date:** 2026-01-17

**Deciders:** Architecture Team

## Context

Modern GUI applications require smooth, native-feeling animations for:
- Micro-interactions (button hover, focus rings)
- Transitions (page navigation, modal appearance)
- Gesture-driven animations (swipe-to-dismiss, pull-to-refresh)
- Data visualization (charts, progress indicators)
- Loading states and skeleton screens

Key requirements:
1. **Native Feel**: Physics-based motion that feels natural
2. **Interruptible**: Can stop mid-animation and change direction
3. **Gesture Binding**: Connect touch/drag directly to animation progress
4. **Accessibility**: Respect prefers-reduced-motion
5. **Performance**: 60fps on typical hardware, minimal CPU overhead
6. **Composable**: Sequences, parallel, staggered animations
7. **Declarative API**: Simple cases should be trivial
8. **Integration**: Works with Scene graph and ECS

## Decision

We implement `anim-graph` as a **dedicated animation subsystem** separate from but integrated with the scene graph and ECS.

### Architecture

```
┌──────────────────────────────────────────┐
│  Widget Framework (widget-core)         │
│  - Declarative animation API             │
│  - .animation(Spring::default())         │
└────────────────┬─────────────────────────┘
                 ↓
┌──────────────────────────────────────────┐
│  Animation Graph (anim-graph)            │
│  ┌────────────────────────────────────┐  │
│  │ Animation Primitives               │  │
│  │ - Tween (linear interpolation)     │  │
│  │ - Spring (physics-based)           │  │
│  │ - Keyframes (multi-step)           │  │
│  └────────────────────────────────────┘  │
│  ┌────────────────────────────────────┐  │
│  │ Animation Controller               │  │
│  │ - tick(delta_time)                 │  │
│  │ - updated_properties()             │  │
│  │ - respects reduced motion          │  │
│  └────────────────────────────────────┘  │
└────────────────┬─────────────────────────┘
                 ↓
┌──────────────────────────────────────────┐
│  Scene Graph + ECS                       │
│  - Consumes property updates             │
│  - scene.set_property(node, prop, val)   │
└──────────────────────────────────────────┘
```

### Animation Primitives

```rust
pub enum Animation<T: Animatable> {
    // Simple interpolation
    Tween {
        from: T,
        to: T,
        duration: Duration,
        easing: Easing,
        elapsed: Duration,
    },

    // Physics-based (interruptible, natural feel)
    Spring {
        current: T,
        target: T,
        velocity: T,
        stiffness: f32,  // Higher = snappier
        damping: f32,    // Higher = less oscillation
        mass: f32,
    },

    // Complex multi-step
    Keyframes {
        frames: Vec<Keyframe<T>>,
        timing: KeyframeTiming,
        elapsed: Duration,
    },

    // Composition
    Sequence(Vec<Animation<T>>),
    Parallel(Vec<Animation<T>>),
}

pub trait Animatable: Copy + 'static {
    fn lerp(from: Self, to: Self, t: f32) -> Self;
    fn add(self, rhs: Self) -> Self;
    fn scale(self, scalar: f32) -> Self;
}

// Impl for f32, f64, Color, Transform2D, Vec2, etc.
```

### Platform-Native Easing

```rust
pub enum Easing {
    PlatformDefault,              // Uses native curve per-platform
    IosSpring,                    // Critically damped spring (iOS feel)
    MaterialStandard,             // Material Design standard curve
    MaterialEmphasized,           // Material Design emphasized curve
    CubicBezier(f32, f32, f32, f32),  // Custom bezier curve
}
```

### Declarative API (Simple Cases)

```rust
// SwiftUI-style implicit animations
fn build(&self, cx: Scope) -> impl Element {
    let is_expanded = cx.signal(false);

    container()
        .padding(if is_expanded.get() { 32.0 } else { 16.0 })
        .background(if is_expanded.get() { Color::BLUE } else { Color::GRAY })
        // Any property change animates automatically
        .animation(Spring::default())
        .on_click(move || is_expanded.toggle())
}
```

### Gesture Binding (Advanced)

```rust
gesture_detector(|g| {
    g.on_pan(|pan| {
        // Map drag distance to animation progress (0.0 - 1.0)
        let progress = (pan.translation.y / 300.0).clamp(0.0, 1.0);
        card_animation.seek(progress);
    })
    .on_pan_end(|pan| {
        if pan.velocity.y > 500.0 {
            // Fling to dismiss - continues from current progress
            card_animation.animate_to_end(Spring::from_velocity(pan.velocity));
        } else {
            // Snap back
            card_animation.animate_to_start(Spring::bouncy());
        }
    })
})
```

### Reduced Motion Support

Built into the system, not an afterthought. When `prefers-reduced-motion` is enabled:

| Animation Type | Behavior |
|----------------|----------|
| Spring         | Becomes instant (no animation) |
| Tween          | Reduced to 1ms duration |
| Keyframes      | Jumps to final frame |
| Gesture-bound  | Maintains interactivity, no inertia |

```rust
impl AnimationController {
    pub fn tick(&mut self, delta_time: Duration, reduced_motion: bool) {
        if reduced_motion {
            // Skip to end of all animations
            self.skip_to_end();
        } else {
            // Normal animation update
            self.update(delta_time);
        }
    }
}
```

## Rationale

### Why Dedicated Animation System?

1. **Separation of Concerns**:
   - Scene graph: "what exists and where"
   - Animation graph: "how things change over time"
   - Clear interface between them

2. **Performance**:
   - Batch property updates
   - Only update animated properties
   - Independent tick rate (can run at 120fps while scene renders at 60fps)

3. **Testability**:
   - Animation system can be tested independently
   - Time control for deterministic tests
   - No need to render frames to test animation logic

4. **Flexibility**:
   - Can animate any `Animatable` type
   - Not tied to scene graph representation
   - Could animate non-visual properties (audio, haptics)

### Why Spring-Based Physics?

Springs provide **natural, interruptible motion**:

```rust
// Spring animation from iOS/SwiftUI
Spring {
    stiffness: 300.0,   // How quickly it responds
    damping: 30.0,      // How much it oscillates
    mass: 1.0,          // Inertia
}

// Benefits:
// - Feels natural (real physics)
// - Interruptible (can change target mid-animation)
// - Velocity-aware (respects fling gestures)
// - No duration (adapts to distance)
```

**Comparison**:
- **Tween**: Fixed duration, not interruptible, feels mechanical
- **Spring**: Adaptive duration, interruptible, feels natural
- **Keyframes**: Complex control, good for sequences

**Decision**: Springs for UI interactions, tweens for precise timing (loading bars), keyframes for complex sequences.

### Why Separate from ECS?

**Alternative Considered**: Animate ECS components directly

**Rejected Because**:
- ECS is for bulk operations, not time-based interpolation
- Animation state is transient (doesn't need entity lifetime)
- Springs require velocity tracking (extra component overhead)
- Reduced motion would require entity queries (slow)

**Chosen Approach**: Animation system outputs property updates, ECS consumes them.

## Consequences

### Positive

- **Natural Motion**: Physics-based springs feel native
- **Interruptible**: Can change animation mid-flight
- **Gesture Integration**: Direct binding to touch/drag
- **Accessibility**: Reduced motion built-in
- **Performance**: Batch updates, independent tick
- **Composable**: Sequences, parallel, staggered
- **Testable**: Time control, no rendering required
- **Declarative API**: Simple cases are trivial

### Negative

- **Complexity**: Separate subsystem to maintain
- **State Synchronization**: Must keep animation and scene in sync
- **Learning Curve**: Developers must understand springs vs tweens
- **Memory**: Animation state per animated property
- **API Surface**: More concepts (springs, tweens, keyframes, gestures)

### Mitigations

- **Documentation**: Comprehensive examples, cookbook
- **Defaults**: Smart defaults (Spring::default() works for most cases)
- **Integration**: Clean integration with ECS and scene graph
- **Testing**: Built-in test harness with time control

## Implementation Status

### Phase 1 (Basic Implementation Complete)
- ✅ Tween animation (simple interpolation)
- ✅ Easing curves (cubic bezier)
- ✅ `Animatable` trait for f32, f64

### Phase 2 (Planned)
- 🚧 Spring physics implementation
- 🚧 Keyframes and sequences
- 🚧 Animation controller
- 🚧 Scene graph integration
- 🚧 Reduced motion support

### Phase 3 (Future)
- 📅 Gesture binding
- 📅 Animation state machine
- 📅 Declarative API (.animation())
- 📅 Staggered list animations
- 📅 Testing integration

## Performance Characteristics

**Target**: 60fps (16.67ms frame budget)

**Measured** (estimates based on similar systems):
- Spring tick: ~100ns per animated property
- Tween tick: ~50ns per animated property
- Batch update: ~1μs for 100 animated properties
- Scene integration: ~1μs overhead

**Scalability**:
- 100 animated properties: ~10μs per frame (0.06% of budget)
- 1,000 animated properties: ~100μs per frame (0.6% of budget)
- 10,000 animated properties: ~1ms per frame (6% of budget)

**Bottleneck**: Scene graph property updates, not animation calculation.

## Alternatives Considered

### 1. CSS Transitions/Animations

**Pros:**
- Familiar to web developers
- Declarative
- Well-understood

**Cons:**
- Not interruptible
- Fixed durations (not physics-based)
- Web-specific paradigm
- Doesn't map well to native GUI

**Rejected because:** Too web-centric, not natural for gesture-driven UIs.

### 2. Direct ECS Animation

**Pros:**
- Reuse ECS infrastructure
- No separate subsystem

**Cons:**
- Springs need velocity (extra components)
- Time-based updates awkward in ECS
- Reduced motion requires entity queries
- Animation state pollutes entity data

**Rejected because:** ECS is for bulk operations, not time-based interpolation.

### 3. Immediate Mode (Recompute Each Frame)

**Pros:**
- Simple (no retained state)
- No synchronization issues

**Cons:**
- Expensive (recompute all animations)
- Can't be interruptible (no velocity state)
- No reduced motion (always animating)

**Rejected because:** Performance inadequate, no gesture binding.

### 4. Lottie/Rive (JSON-based)

**Pros:**
- Designer-friendly
- Complex animations from tools
- Battle-tested

**Cons:**
- Not interruptible
- File-based (slower iteration)
- Harder to bind to code state
- Large runtime

**Rejected because:** Focused on pre-baked animations, not UI interactions. Could be added later as opt-in.

## Testing Strategy

```rust
#[test]
fn spring_reaches_target() {
    let mut spring = Animation::Spring {
        current: 0.0,
        target: 100.0,
        velocity: 0.0,
        stiffness: 300.0,
        damping: 30.0,
        mass: 1.0,
    };

    // Simulate 1 second
    for _ in 0..60 {
        spring.tick(Duration::from_millis(16));
    }

    assert!((spring.current() - 100.0).abs() < 0.1);
}

#[test]
fn respects_reduced_motion() {
    let mut controller = AnimationController::new();
    controller.add_animation(node_id, Property::Opacity,
        Animation::Spring { ... });

    // With reduced motion, should complete instantly
    controller.tick(Duration::from_millis(1), true);

    assert_eq!(controller.get_value(node_id, Property::Opacity), 1.0);
}
```

## Future Enhancements

- **GPU-Accelerated Springs**: Compute shaders for massive parallelism
- **Sound Integration**: Sync animations with haptics/audio
- **Lottie Support**: Import designer animations (opt-in)
- **Animation Profiler**: Visual timeline of active animations
- **AI-Assisted**: ML-based easing curve suggestions
- **Shared Element Transitions**: Hero animations between screens

## References

- SwiftUI Animations: https://developer.apple.com/documentation/swiftui/animation
- Material Motion: https://material.io/design/motion/
- iOS UIViewPropertyAnimator: https://developer.apple.com/documentation/uikit/uiviewpropertyanimator
- Flutter Animations: https://docs.flutter.dev/development/ui/animations
- React Spring: https://www.react-spring.dev/
- [Arthropod Design Doc](../design/arthropod-design-doc.md)

## Conclusion

A dedicated animation system with physics-based springs, gesture binding, and reduced motion support provides the foundation for native-feeling, accessible animations. The architecture separates concerns while integrating cleanly with the scene graph and ECS, delivering performance and flexibility for modern GUI applications.
