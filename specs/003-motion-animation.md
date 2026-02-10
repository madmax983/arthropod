# 🔭 Vantage: Spec for Motion (Reactive Animation System)

**Status**: Draft
**Owner**: Vantage
**Target Release**: Arthropod Core (Feature Promotion)

## 1. Context & Problem
Currently, the `crates/anim-graph` crate provides raw physics and tweening primitives, but they are disconnected from the application state (`flux-state`).
Developers (User Persona "Bard") must manually pump animation loops and bridge signals to values.
This friction leads to static, lifeless UIs because adding motion is "too hard."

## 2. User Story
**As a** UI Developer / Designer,
**I want to** declare that a value (like opacity or position) should "spring" to its new target,
**So that** the interface feels responsive and organic without me writing complex frame loops.

## 3. The "So What?" (Business Value)
*   **Perceived Performance**: Smooth transitions mask loading times and latency.
*   **Context**: Motion conveys *directionality* (e.g., a menu sliding in from the left implies "back" is to the right).
*   **Delight**: "Juice" transforms a utility into a product.

## 4. Success Metrics
*   **Ease of Use**: Adding motion to a signal should require < 3 lines of code.
*   **Performance**: 1000 concurrent animations must run at 60 FPS (overhead < 1ms per frame).
*   **Fluidity**: Zero "popping" artifacts when retargeting an animation mid-flight.

## 5. Acceptance Criteria (MVP -> Production)

### 5.1 The `MotionSignal` Primitive
*   The system **must** provide a wrapper around `ReadSignal<T>` called `MotionSignal<T>` (or similar).
*   When the source signal changes, the `MotionSignal` **must** interpolate from the current value to the new target over time.
*   It **must** support both:
    *   **Springs**: Physics-based (Mass, Stiffness, Damping) - *Default for interactions*.
    *   **Tweens**: Time-based (Duration, Easing) - *Default for sequences*.

### 5.2 Interruptibility (Retargeting)
*   If the source signal changes *while* an animation is in progress, the system **must** seamlessly transition to the new target.
*   **Springs**: Must preserve current velocity (momentum) to avoid "hiccups."
*   **Tweens**: Must blend or restart gracefully (policy TBD, but must not snap).

### 5.3 Type Support
*   The system **must** support animating the following types out of the box:
    *   `f32` (Opacity, Width)
    *   `render_engine::Color` (Backgrounds)
    *   `glam::Vec2` / `glam::Vec3` (Position, Scale)

### 5.4 Automatic Cleanup
*   When the `MotionSignal` is dropped or the parent component unmounts, the animation loop **must** stop automatically to prevent resource leaks (Zombie Animations).

## 6. Technical Notes (For Engineering Context)
*   **Integration**: This likely involves a `create_motion(cx, source, config)` function in `flux-state` or a new `flux-motion` crate.
*   **Scheduler**: Animations should probably run in a dedicated system or the `Update` schedule, separate from the main logic loop if possible (or just before layout).
*   **`anim-graph`**: Leverage the existing crate; do not rewrite the physics solver.

## 7. Out of Scope (Phase 1)
*   **Keyframe Sequences**: Complex multi-stage animations (Timeline).
*   **Skeletal Animation**: Bone-based deformation.
*   **Layout Animation**: Automatically animating layout changes (FLIP technique) - Phase 2.
