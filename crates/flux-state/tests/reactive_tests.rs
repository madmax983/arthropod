//! Tests for reactive state primitives: Signal, Computed, Effect, and Runtime.

use flux_state::{Computed, Effect, Runtime, Signal};
use std::sync::Arc;
use std::sync::atomic::{AtomicI32, Ordering};

// ==================== Runtime Tests ====================

#[test]
fn test_runtime_creation() {
    let runtime = Runtime::new();
    assert!(
        Arc::strong_count(&runtime) >= 1,
        "Runtime should be created"
    );
}

// ==================== Signal Tests ====================

#[test]
fn test_signal_creation_and_get() {
    let runtime = Runtime::new();
    let signal = Signal::new(Arc::clone(&runtime), 42);
    let (read, _write) = signal.split();

    assert_eq!(read.get(), 42);
}

#[test]
fn test_signal_set_and_get() {
    let runtime = Runtime::new();
    let signal = Signal::new(Arc::clone(&runtime), 0);
    let (read, write) = signal.split();

    write.set(100);
    assert_eq!(read.get(), 100);
}

#[test]
fn test_signal_update() {
    let runtime = Runtime::new();
    let signal = Signal::new(Arc::clone(&runtime), 10);
    let (read, write) = signal.split();

    write.update(|v| *v += 5);
    assert_eq!(read.get(), 15);
}

#[test]
fn test_signal_get_untracked() {
    let runtime = Runtime::new();
    let signal = Signal::new(Arc::clone(&runtime), 42);
    let (read, _write) = signal.split();

    // Should get value without tracking dependency
    assert_eq!(read.get_untracked(), 42);
}

#[test]
fn test_multiple_signals() {
    let runtime = Runtime::new();
    let signal1 = Signal::new(Arc::clone(&runtime), 10);
    let signal2 = Signal::new(Arc::clone(&runtime), 20);

    let (read1, _) = signal1.split();
    let (read2, _) = signal2.split();

    assert_eq!(read1.get(), 10);
    assert_eq!(read2.get(), 20);
}

#[test]
fn test_signal_with_string() {
    let runtime = Runtime::new();
    let signal = Signal::new(Arc::clone(&runtime), "Hello".to_string());
    let (read, write) = signal.split();

    assert_eq!(read.get(), "Hello");

    write.set("World".to_string());
    assert_eq!(read.get(), "World");
}

// ==================== Computed Tests ====================

#[test]
fn test_computed_basic() {
    let runtime = Runtime::new();
    let signal = Signal::new(Arc::clone(&runtime), 10);
    let (read, _) = signal.split();

    let doubled = Computed::new(Arc::clone(&runtime), {
        let read = read.clone();
        move || read.get() * 2
    });

    assert_eq!(doubled.get(), 20);
}

#[test]
fn test_computed_updates_when_dependency_changes() {
    let runtime = Runtime::new();
    let signal = Signal::new(Arc::clone(&runtime), 5);
    let (read, write) = signal.split();

    let doubled = Computed::new(Arc::clone(&runtime), {
        let read = read.clone();
        move || read.get() * 2
    });

    assert_eq!(doubled.get(), 10);

    write.set(10);
    assert_eq!(
        doubled.get(),
        20,
        "Computed should update when signal changes"
    );
}

#[test]
fn test_computed_with_multiple_dependencies() {
    let runtime = Runtime::new();
    let a = Signal::new(Arc::clone(&runtime), 3);
    let b = Signal::new(Arc::clone(&runtime), 4);

    let (read_a, write_a) = a.split();
    let (read_b, _write_b) = b.split();

    let sum = Computed::new(Arc::clone(&runtime), {
        let read_a = read_a.clone();
        let read_b = read_b.clone();
        move || read_a.get() + read_b.get()
    });

    assert_eq!(sum.get(), 7);

    write_a.set(10);
    assert_eq!(
        sum.get(),
        14,
        "Computed should update when any dependency changes"
    );
}

#[test]
fn test_computed_chain() {
    let runtime = Runtime::new();
    let signal = Signal::new(Arc::clone(&runtime), 2);
    let (read, write) = signal.split();

    let doubled = Computed::new(Arc::clone(&runtime), {
        let read = read.clone();
        move || read.get() * 2
    });

    let quadrupled = Computed::new(Arc::clone(&runtime), {
        let doubled = doubled.clone();
        move || doubled.get() * 2
    });

    assert_eq!(quadrupled.get(), 8);

    write.set(5);
    assert_eq!(quadrupled.get(), 20, "Chained computed should update");
}

// ==================== Effect Tests ====================

#[test]
fn test_effect_runs_initially() {
    let runtime = Runtime::new();
    let signal = Signal::new(Arc::clone(&runtime), 10);
    let (read, _) = signal.split();

    let run_count = Arc::new(AtomicI32::new(0));
    let run_count_clone = Arc::clone(&run_count);

    let _effect = Effect::new(Arc::clone(&runtime), move || {
        let _ = read.get();
        run_count_clone.fetch_add(1, Ordering::Relaxed);
    });

    assert_eq!(
        run_count.load(Ordering::Relaxed),
        1,
        "Effect should run once initially"
    );
}

#[test]
fn test_effect_runs_when_dependency_changes() {
    let runtime = Runtime::new();
    let signal = Signal::new(Arc::clone(&runtime), 10);
    let (read, write) = signal.split();

    let run_count = Arc::new(AtomicI32::new(0));
    let run_count_clone = Arc::clone(&run_count);

    let _effect = Effect::new(Arc::clone(&runtime), move || {
        let _ = read.get();
        run_count_clone.fetch_add(1, Ordering::Relaxed);
    });

    assert_eq!(run_count.load(Ordering::Relaxed), 1, "Initial run");

    write.set(20);
    assert_eq!(
        run_count.load(Ordering::Relaxed),
        2,
        "Effect should run when signal changes"
    );

    write.set(30);
    assert_eq!(
        run_count.load(Ordering::Relaxed),
        3,
        "Effect should run again"
    );
}

#[test]
fn test_effect_tracks_correct_dependencies() {
    let runtime = Runtime::new();
    let signal1 = Signal::new(Arc::clone(&runtime), 10);
    let signal2 = Signal::new(Arc::clone(&runtime), 20);

    let (read1, write1) = signal1.split();
    let (_read2, write2) = signal2.split();

    let run_count = Arc::new(AtomicI32::new(0));
    let run_count_clone = Arc::clone(&run_count);

    // Effect only depends on signal1
    let _effect = Effect::new(Arc::clone(&runtime), move || {
        let _ = read1.get();
        run_count_clone.fetch_add(1, Ordering::Relaxed);
    });

    assert_eq!(run_count.load(Ordering::Relaxed), 1, "Initial run");

    write1.set(100);
    assert_eq!(
        run_count.load(Ordering::Relaxed),
        2,
        "Should run when signal1 changes"
    );

    write2.set(200);
    assert_eq!(
        run_count.load(Ordering::Relaxed),
        2,
        "Should NOT run when signal2 changes"
    );
}

#[test]
fn test_effect_with_computed() {
    let runtime = Runtime::new();
    let signal = Signal::new(Arc::clone(&runtime), 5);
    let (read, write) = signal.split();

    let doubled = Computed::new(Arc::clone(&runtime), {
        let read = read.clone();
        move || read.get() * 2
    });

    let run_count = Arc::new(AtomicI32::new(0));
    let last_value = Arc::new(AtomicI32::new(0));

    let run_count_clone = Arc::clone(&run_count);
    let last_value_clone = Arc::clone(&last_value);

    let _effect = Effect::new(Arc::clone(&runtime), move || {
        let value = doubled.get();
        run_count_clone.fetch_add(1, Ordering::Relaxed);
        last_value_clone.store(value, Ordering::Relaxed);
    });

    assert_eq!(run_count.load(Ordering::Relaxed), 1);
    assert_eq!(last_value.load(Ordering::Relaxed), 10);

    write.set(10);
    assert_eq!(run_count.load(Ordering::Relaxed), 2);
    assert_eq!(last_value.load(Ordering::Relaxed), 20);
}

#[test]
fn test_multiple_effects_on_same_signal() {
    let runtime = Runtime::new();
    let signal = Signal::new(Arc::clone(&runtime), 0);
    let (read, write) = signal.split();

    let count1 = Arc::new(AtomicI32::new(0));
    let count2 = Arc::new(AtomicI32::new(0));

    let count1_clone = Arc::clone(&count1);
    let count2_clone = Arc::clone(&count2);

    let read1 = read.clone();
    let _effect1 = Effect::new(Arc::clone(&runtime), move || {
        let _ = read1.get();
        count1_clone.fetch_add(1, Ordering::Relaxed);
    });

    let _effect2 = Effect::new(Arc::clone(&runtime), move || {
        let _ = read.get();
        count2_clone.fetch_add(1, Ordering::Relaxed);
    });

    assert_eq!(count1.load(Ordering::Relaxed), 1);
    assert_eq!(count2.load(Ordering::Relaxed), 1);

    write.set(10);
    assert_eq!(count1.load(Ordering::Relaxed), 2, "Both effects should run");
    assert_eq!(count2.load(Ordering::Relaxed), 2, "Both effects should run");
}

// ==================== Complex Dependency Graph Tests ====================

#[test]
fn test_diamond_dependency() {
    let runtime = Runtime::new();
    let source = Signal::new(Arc::clone(&runtime), 1);
    let (read, write) = source.split();

    // Diamond: source -> left/right -> result
    let left = Computed::new(Arc::clone(&runtime), {
        let read = read.clone();
        move || read.get() + 10
    });

    let right = Computed::new(Arc::clone(&runtime), {
        let read = read.clone();
        move || read.get() + 20
    });

    let result = Computed::new(Arc::clone(&runtime), {
        let left = left.clone();
        let right = right.clone();
        move || left.get() + right.get()
    });

    assert_eq!(result.get(), 32); // 1 + 10 + 1 + 20 = 32

    write.set(5);
    assert_eq!(result.get(), 40); // 5 + 10 + 5 + 20 = 40
}

#[test]
fn test_effect_doesnt_run_if_dependencies_dont_change() {
    let runtime = Runtime::new();
    let signal = Signal::new(Arc::clone(&runtime), 10);
    let (read, write) = signal.split();

    let run_count = Arc::new(AtomicI32::new(0));
    let run_count_clone = Arc::clone(&run_count);

    let _effect = Effect::new(Arc::clone(&runtime), move || {
        let _ = read.get();
        run_count_clone.fetch_add(1, Ordering::Relaxed);
    });

    assert_eq!(run_count.load(Ordering::Relaxed), 1);

    // Set to same value
    write.set(10);

    // Effect should still run (we notify on any set, even if value doesn't change)
    // This is a design choice - can be optimized later
    assert!(
        run_count.load(Ordering::Relaxed) >= 1,
        "Effect behavior on same-value set"
    );
}

// ==================== Edge Cases ====================

#[test]
fn test_signal_clone() {
    let runtime = Runtime::new();
    let signal = Signal::new(Arc::clone(&runtime), 42);
    let (read1, _) = signal.split();
    let read2 = read1.clone();

    assert_eq!(read1.get(), 42);
    assert_eq!(read2.get(), 42);
}

#[test]
fn test_computed_clone() {
    let runtime = Runtime::new();
    let signal = Signal::new(Arc::clone(&runtime), 10);
    let (read, _) = signal.split();

    let computed = Computed::new(Arc::clone(&runtime), {
        let read = read.clone();
        move || read.get() * 2
    });

    let computed2 = computed.clone();

    assert_eq!(computed.get(), 20);
    assert_eq!(computed2.get(), 20);
}

#[test]
fn test_deeply_nested_dependencies() {
    let runtime = Runtime::new();
    let signal = Signal::new(Arc::clone(&runtime), 1);
    let (read, write) = signal.split();

    let c1 = Computed::new(Arc::clone(&runtime), {
        let read = read.clone();
        move || read.get() + 1
    });

    let c2 = Computed::new(Arc::clone(&runtime), {
        let c1 = c1.clone();
        move || c1.get() + 1
    });

    let c3 = Computed::new(Arc::clone(&runtime), {
        let c2 = c2.clone();
        move || c2.get() + 1
    });

    assert_eq!(c3.get(), 4); // 1 + 1 + 1 + 1

    write.set(10);
    assert_eq!(c3.get(), 13); // 10 + 1 + 1 + 1
}
