#![cfg(feature = "nova")]

use arthropod::experimental::noise::{NoiseSignal, NoiseSignal2D, perlin_1d, perlin_2d};
use arthropod::prelude::*;

#[test]
fn test_perlin_1d_bounds() {
    // Check range [-1.0, 1.0]
    let mut non_zero_count = 0;
    for i in 0..1000 {
        let x = i as f32 * 0.1;
        let val = perlin_1d(x);
        assert!(
            val >= -1.0 && val <= 1.0,
            "Value {} at {} is out of bounds",
            val,
            x
        );
        if val.abs() > 1e-6 {
            non_zero_count += 1;
        }
    }
    assert!(non_zero_count > 0, "Perlin 1D produced all zeros!");
}

#[test]
fn test_perlin_2d_bounds() {
    // Check range [-1.0, 1.0]
    for i in 0..100 {
        for j in 0..100 {
            let x = i as f32 * 0.1;
            let y = j as f32 * 0.1;
            let val = perlin_2d(x, y);
            assert!(
                val >= -1.0 && val <= 1.0,
                "Value {} at ({}, {}) is out of bounds",
                val,
                x,
                y
            );
        }
    }
}

#[test]
fn test_perlin_determinism() {
    let x = 123.456;
    let val1 = perlin_1d(x);
    let val2 = perlin_1d(x);
    assert_eq!(val1, val2, "Noise should be deterministic");

    let y = 789.012;
    let val1 = perlin_2d(x, y);
    let val2 = perlin_2d(x, y);
    assert_eq!(val1, val2, "2D Noise should be deterministic");
}

#[test]
fn test_noise_signal_reactive() {
    let runtime = Runtime::new();
    let t_sig = Signal::new(runtime.clone(), 0.0);
    let (t_read, t_write) = t_sig.split();

    // Create NoiseSignal
    let noise_sig = NoiseSignal::new(runtime.clone(), t_read);

    // Initial value
    let val0 = noise_sig.get();

    // Update time
    t_write.set(1.123);
    let val1 = noise_sig.get();

    assert_ne!(val0, val1, "Noise should change when input changes");

    // Reset time -> should get same value (determinism)
    t_write.set(0.0);
    let val2 = noise_sig.get();
    assert_eq!(val0, val2, "Noise signal should be deterministic");
}

#[test]
fn test_noise_signal_2d_reactive() {
    let runtime = Runtime::new();
    let pos_sig = Signal::new(runtime.clone(), Vec2::ZERO);
    let (pos_read, pos_write) = pos_sig.split();

    let noise_sig = NoiseSignal2D::new(runtime.clone(), pos_read);

    let val0 = noise_sig.get();

    pos_write.set(Vec2::new(10.1, 10.2));
    let val1 = noise_sig.get();

    assert_ne!(val0, val1, "2D Noise should change when position changes");
}
