//! Tests for animation primitives and interpolation.

use anim_graph::{Animatable, Animation, Easing};
use render_engine::Color;
use std::time::Duration;

// ==================== Animatable Trait Tests ====================

#[test]
fn test_f32_interpolation() {
    let start = 0.0_f32;
    let end = 100.0_f32;

    assert_eq!(start.interpolate(&end, 0.0), 0.0);
    assert_eq!(start.interpolate(&end, 0.5), 50.0);
    assert_eq!(start.interpolate(&end, 1.0), 100.0);
}

#[test]
fn test_f32_interpolation_negative() {
    let start = -50.0_f32;
    let end = 50.0_f32;

    assert_eq!(start.interpolate(&end, 0.0), -50.0);
    assert_eq!(start.interpolate(&end, 0.5), 0.0);
    assert_eq!(start.interpolate(&end, 1.0), 50.0);
}

#[test]
fn test_color_interpolation() {
    let red = Color::RED;
    let blue = Color::BLUE;

    let purple = red.interpolate(&blue, 0.5);
    assert_eq!(purple.r, 0.5);
    assert_eq!(purple.g, 0.0);
    assert_eq!(purple.b, 0.5);
    assert_eq!(purple.a, 1.0);
}

#[test]
fn test_color_interpolation_with_alpha() {
    let opaque = Color::rgba(1.0, 0.0, 0.0, 1.0);
    let transparent = Color::rgba(1.0, 0.0, 0.0, 0.0);

    let half = opaque.interpolate(&transparent, 0.5);
    assert_eq!(half.a, 0.5);
}

// ==================== Easing Function Tests ====================

#[test]
fn test_easing_linear() {
    let easing = Easing::Linear;

    assert_eq!(easing.apply(0.0), 0.0);
    assert_eq!(easing.apply(0.25), 0.25);
    assert_eq!(easing.apply(0.5), 0.5);
    assert_eq!(easing.apply(0.75), 0.75);
    assert_eq!(easing.apply(1.0), 1.0);
}

#[test]
fn test_easing_clamps_input() {
    let easing = Easing::Linear;

    assert_eq!(easing.apply(-0.5), 0.0, "Should clamp negative to 0");
    assert_eq!(easing.apply(1.5), 1.0, "Should clamp > 1 to 1");
}

#[test]
fn test_easing_ease_in() {
    let easing = Easing::EaseIn;

    // EaseIn should start slow (t^2)
    let result_at_half = easing.apply(0.5);
    assert!(
        result_at_half < 0.5,
        "EaseIn at 0.5 should be < 0.5 (slow start)"
    );

    assert_eq!(easing.apply(0.0), 0.0);
    assert_eq!(easing.apply(1.0), 1.0);
}

#[test]
fn test_easing_ease_out() {
    let easing = Easing::EaseOut;

    // EaseOut should start fast
    let result_at_half = easing.apply(0.5);
    assert!(
        result_at_half > 0.5,
        "EaseOut at 0.5 should be > 0.5 (fast start)"
    );

    assert_eq!(easing.apply(0.0), 0.0);
    assert_eq!(easing.apply(1.0), 1.0);
}

#[test]
fn test_easing_ease_in_out() {
    let easing = Easing::EaseInOut;

    assert_eq!(easing.apply(0.0), 0.0);
    assert_eq!(easing.apply(1.0), 1.0);

    // Should be symmetric around 0.5
    let quarter = easing.apply(0.25);
    let three_quarters = easing.apply(0.75);

    assert!(quarter < 0.25, "Should start slow");
    assert!(three_quarters > 0.75, "Should end slow");
}

// ==================== Tween Animation Tests ====================

#[test]
fn test_tween_creation() {
    let anim = Animation::tween(0.0_f32, 100.0_f32, Duration::from_secs(1), Easing::Linear);

    match anim {
        Animation::Tween {
            from, to, duration, ..
        } => {
            assert_eq!(from, 0.0);
            assert_eq!(to, 100.0);
            assert_eq!(duration, Duration::from_secs(1));
        }
        _ => panic!("Expected Tween variant"),
    }
}

#[test]
fn test_tween_at_start() {
    let mut anim = Animation::tween(0.0_f32, 100.0_f32, Duration::from_secs(1), Easing::Linear);

    let value = anim.tick(Duration::ZERO);
    assert_eq!(value, 0.0, "Should be at start value");
}

#[test]
fn test_tween_at_midpoint() {
    let mut anim = Animation::tween(0.0_f32, 100.0_f32, Duration::from_secs(1), Easing::Linear);

    let value = anim.tick(Duration::from_millis(500));
    assert_eq!(value, 50.0, "Should be halfway through");
}

#[test]
fn test_tween_at_end() {
    let mut anim = Animation::tween(0.0_f32, 100.0_f32, Duration::from_secs(1), Easing::Linear);

    let value = anim.tick(Duration::from_secs(1));
    assert_eq!(value, 100.0, "Should be at end value");
}

#[test]
fn test_tween_clamps_at_end() {
    let mut anim = Animation::tween(0.0_f32, 100.0_f32, Duration::from_secs(1), Easing::Linear);

    let value = anim.tick(Duration::from_secs(2));
    assert_eq!(value, 100.0, "Should clamp to end value");
}

#[test]
fn test_tween_with_ease_in() {
    let mut anim = Animation::tween(0.0_f32, 100.0_f32, Duration::from_secs(1), Easing::EaseIn);

    let value = anim.tick(Duration::from_millis(500));
    assert!(
        value < 50.0,
        "EaseIn should be slow at start, value: {}",
        value
    );
}

#[test]
fn test_tween_incremental_ticks() {
    let mut anim = Animation::tween(0.0_f32, 100.0_f32, Duration::from_secs(1), Easing::Linear);

    // Tick in small increments
    let v1 = anim.tick(Duration::from_millis(250));
    let v2 = anim.tick(Duration::from_millis(250));
    let v3 = anim.tick(Duration::from_millis(500));

    assert_eq!(v1, 25.0);
    assert_eq!(v2, 50.0);
    assert_eq!(v3, 100.0, "Should accumulate time");
}

#[test]
fn test_tween_color_animation() {
    let mut anim = Animation::tween(
        Color::RED,
        Color::BLUE,
        Duration::from_secs(1),
        Easing::Linear,
    );

    let color = anim.tick(Duration::from_millis(500));

    // Should be purple at midpoint
    assert_eq!(color.r, 0.5);
    assert_eq!(color.g, 0.0);
    assert_eq!(color.b, 0.5);
}

// ==================== Animation Completion Tests ====================

#[test]
fn test_tween_not_complete_initially() {
    let anim = Animation::tween(0.0_f32, 100.0_f32, Duration::from_secs(1), Easing::Linear);

    assert!(
        !anim.is_complete(),
        "Tween should not be complete initially"
    );
}

#[test]
fn test_tween_not_complete_during() {
    let mut anim = Animation::tween(0.0_f32, 100.0_f32, Duration::from_secs(1), Easing::Linear);

    anim.tick(Duration::from_millis(500));
    assert!(
        !anim.is_complete(),
        "Tween should not be complete at midpoint"
    );
}

#[test]
fn test_tween_complete_at_end() {
    let mut anim = Animation::tween(0.0_f32, 100.0_f32, Duration::from_secs(1), Easing::Linear);

    anim.tick(Duration::from_secs(1));
    assert!(anim.is_complete(), "Tween should be complete at end");
}

#[test]
fn test_tween_complete_after_end() {
    let mut anim = Animation::tween(0.0_f32, 100.0_f32, Duration::from_secs(1), Easing::Linear);

    anim.tick(Duration::from_secs(2));
    assert!(anim.is_complete(), "Tween should be complete after end");
}

// ==================== Spring Animation Tests ====================

#[test]
fn test_spring_creation() {
    let anim = Animation::spring(0.0_f32, 100.0_f32, 200.0, 20.0);

    match anim {
        Animation::Spring {
            current,
            target,
            stiffness,
            damping,
            ..
        } => {
            assert_eq!(current, 0.0);
            assert_eq!(target, 100.0);
            assert_eq!(stiffness, 200.0);
            assert_eq!(damping, 20.0);
        }
        _ => panic!("Expected Spring variant"),
    }
}

#[test]
fn test_spring_starts_at_initial_value() {
    let mut anim = Animation::spring(0.0_f32, 100.0_f32, 200.0, 20.0);

    let value = anim.tick(Duration::ZERO);
    assert_eq!(value, 0.0, "Spring should start at initial value");
}

// Note: Spring physics implementation will be tested more thoroughly
// once we implement the actual spring simulation
