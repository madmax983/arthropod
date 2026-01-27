//! Input Processing Benchmarks
//!
//! Benchmarks for keyboard input processing including key-to-character conversion.
//!
//! Performance targets:
//! - Key::to_char < 10ns per call (this is called on every keystroke)

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use plat_core::Key;

/// Benchmark Key::to_char for letter keys
fn bench_key_to_char_letters(c: &mut Criterion) {
    let mut group = c.benchmark_group("key_to_char_letters");

    let letters = [
        Key::A,
        Key::B,
        Key::C,
        Key::D,
        Key::E,
        Key::F,
        Key::G,
        Key::H,
        Key::I,
        Key::J,
        Key::K,
        Key::L,
        Key::M,
        Key::N,
        Key::O,
        Key::P,
        Key::Q,
        Key::R,
        Key::S,
        Key::T,
        Key::U,
        Key::V,
        Key::W,
        Key::X,
        Key::Y,
        Key::Z,
    ];

    group.bench_function("lowercase", |b| {
        b.iter(|| {
            for key in &letters {
                black_box(key.to_char(false));
            }
        });
    });

    group.bench_function("uppercase", |b| {
        b.iter(|| {
            for key in &letters {
                black_box(key.to_char(true));
            }
        });
    });

    group.bench_function("single_key", |b| {
        let key = Key::A;
        b.iter(|| {
            black_box(key.to_char(false));
        });
    });

    group.finish();
}

/// Benchmark Key::to_char for number keys
fn bench_key_to_char_numbers(c: &mut Criterion) {
    let mut group = c.benchmark_group("key_to_char_numbers");

    let numbers = [
        Key::Key0,
        Key::Key1,
        Key::Key2,
        Key::Key3,
        Key::Key4,
        Key::Key5,
        Key::Key6,
        Key::Key7,
        Key::Key8,
        Key::Key9,
    ];

    group.bench_function("digits", |b| {
        b.iter(|| {
            for key in &numbers {
                black_box(key.to_char(false));
            }
        });
    });

    group.bench_function("symbols", |b| {
        b.iter(|| {
            for key in &numbers {
                black_box(key.to_char(true)); // Shift produces symbols
            }
        });
    });

    group.finish();
}

/// Benchmark Key::to_char for punctuation keys
fn bench_key_to_char_punctuation(c: &mut Criterion) {
    let mut group = c.benchmark_group("key_to_char_punctuation");

    let punctuation = [
        Key::Period,
        Key::Comma,
        Key::Minus,
        Key::Equal,
        Key::Semicolon,
        Key::Quote,
        Key::Slash,
        Key::Backslash,
        Key::BracketLeft,
        Key::BracketRight,
        Key::Backtick,
    ];

    group.bench_function("no_shift", |b| {
        b.iter(|| {
            for key in &punctuation {
                black_box(key.to_char(false));
            }
        });
    });

    group.bench_function("with_shift", |b| {
        b.iter(|| {
            for key in &punctuation {
                black_box(key.to_char(true));
            }
        });
    });

    group.finish();
}

/// Benchmark Key::to_char for non-printable keys (should return None quickly)
fn bench_key_to_char_non_printable(c: &mut Criterion) {
    let mut group = c.benchmark_group("key_to_char_non_printable");

    let non_printable = [
        Key::Enter,
        Key::Tab,
        Key::Backspace,
        Key::Escape,
        Key::Shift,
        Key::Control,
        Key::Alt,
        Key::Meta,
        Key::Left,
        Key::Right,
        Key::Up,
        Key::Down,
        Key::F1,
        Key::F2,
        Key::F3,
        Key::F4,
    ];

    group.bench_function("all", |b| {
        b.iter(|| {
            for key in &non_printable {
                black_box(key.to_char(false));
            }
        });
    });

    group.bench_function("single_key", |b| {
        let key = Key::Enter;
        b.iter(|| {
            black_box(key.to_char(false));
        });
    });

    group.finish();
}

/// Benchmark mixed key input (simulating real typing)
fn bench_key_to_char_mixed(c: &mut Criterion) {
    let mut group = c.benchmark_group("key_to_char_mixed");

    // Simulate typing "Hello, World!"
    let keystrokes: Vec<(Key, bool)> = vec![
        (Key::H, true),      // H
        (Key::E, false),     // e
        (Key::L, false),     // l
        (Key::L, false),     // l
        (Key::O, false),     // o
        (Key::Comma, false), // ,
        (Key::Space, false), // space
        (Key::W, true),      // W
        (Key::O, false),     // o
        (Key::R, false),     // r
        (Key::L, false),     // l
        (Key::D, false),     // d
        (Key::Key1, true),   // !
    ];

    group.bench_function("typing_simulation", |b| {
        b.iter(|| {
            for (key, shift) in &keystrokes {
                black_box(key.to_char(*shift));
            }
        });
    });

    group.finish();
}

/// Benchmark Rect::contains (used in hit testing)
fn bench_rect_contains(c: &mut Criterion) {
    use plat_core::Rect;

    let mut group = c.benchmark_group("rect_contains");

    let rect = Rect::new(100.0, 100.0, 200.0, 150.0);

    group.bench_function("hit_inside", |b| {
        b.iter(|| {
            black_box(rect.contains(150.0, 150.0));
        });
    });

    group.bench_function("miss_outside", |b| {
        b.iter(|| {
            black_box(rect.contains(50.0, 50.0));
        });
    });

    group.bench_function("edge_cases", |b| {
        b.iter(|| {
            // Test all four edges
            black_box(rect.contains(100.0, 100.0)); // top-left (inside)
            black_box(rect.contains(299.9, 249.9)); // near bottom-right (inside)
            black_box(rect.contains(300.0, 250.0)); // exactly at bottom-right (outside)
            black_box(rect.contains(99.9, 150.0)); // just outside left
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_key_to_char_letters,
    bench_key_to_char_numbers,
    bench_key_to_char_punctuation,
    bench_key_to_char_non_printable,
    bench_key_to_char_mixed,
    bench_rect_contains,
);
criterion_main!(benches);
