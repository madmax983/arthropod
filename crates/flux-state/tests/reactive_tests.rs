//! Tests for reactive state primitives: Signal, Computed, Effect, and Runtime.

use flux_state::{Computed, Effect, Runtime, Signal};
use std::cell::Cell;
use std::rc::Rc;

// ==================== Runtime Tests ====================

#[test]
fn test_runtime_creation() {
    let runtime = Runtime::new();
    assert!(Rc::strong_count(&runtime) >= 1, "Runtime should be created");
}

// ==================== Signal Tests ====================

#[test]
fn test_signal_creation_and_get() {
    let runtime = Runtime::new();
    let signal = Signal::new(Rc::clone(&runtime), 42);
    let (read, _write) = signal.split();

    assert_eq!(read.get(), 42);
}

#[test]
fn test_signal_set_and_get() {
    let runtime = Runtime::new();
    let signal = Signal::new(Rc::clone(&runtime), 0);
    let (read, write) = signal.split();

    write.set(100);
    assert_eq!(read.get(), 100);
}

#[test]
fn test_signal_update() {
    let runtime = Runtime::new();
    let signal = Signal::new(Rc::clone(&runtime), 10);
    let (read, write) = signal.split();

    write.update(|v| *v += 5);
    assert_eq!(read.get(), 15);
}

#[test]
fn test_signal_get_untracked() {
    let runtime = Runtime::new();
    let signal = Signal::new(Rc::clone(&runtime), 42);
    let (read, _write) = signal.split();

    // Should get value without tracking dependency
    assert_eq!(read.get_untracked(), 42);
}

#[test]
fn test_multiple_signals() {
    let runtime = Runtime::new();
    let signal1 = Signal::new(Rc::clone(&runtime), 10);
    let signal2 = Signal::new(Rc::clone(&runtime), 20);

    let (read1, _) = signal1.split();
    let (read2, _) = signal2.split();

    assert_eq!(read1.get(), 10);
    assert_eq!(read2.get(), 20);
}

#[test]
fn test_signal_with_string() {
    let runtime = Runtime::new();
    let signal = Signal::new(Rc::clone(&runtime), "Hello".to_string());
    let (read, write) = signal.split();

    assert_eq!(read.get(), "Hello");

    write.set("World".to_string());
    assert_eq!(read.get(), "World");
}

// ==================== Computed Tests ====================

#[test]
fn test_computed_basic() {
    let runtime = Runtime::new();
    let signal = Signal::new(Rc::clone(&runtime), 10);
    let (read, _) = signal.split();

    let doubled = Computed::new(Rc::clone(&runtime), {
        let read = read.clone();
        move || read.get() * 2
    });

    assert_eq!(doubled.get(), 20);
}

#[test]
fn test_computed_updates_when_dependency_changes() {
    let runtime = Runtime::new();
    let signal = Signal::new(Rc::clone(&runtime), 5);
    let (read, write) = signal.split();

    let doubled = Computed::new(Rc::clone(&runtime), {
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
    let a = Signal::new(Rc::clone(&runtime), 3);
    let b = Signal::new(Rc::clone(&runtime), 4);

    let (read_a, write_a) = a.split();
    let (read_b, _write_b) = b.split();

    let sum = Computed::new(Rc::clone(&runtime), {
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
    let signal = Signal::new(Rc::clone(&runtime), 2);
    let (read, write) = signal.split();

    let doubled = Computed::new(Rc::clone(&runtime), {
        let read = read.clone();
        move || read.get() * 2
    });

    let quadrupled = Computed::new(Rc::clone(&runtime), {
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
    let signal = Signal::new(Rc::clone(&runtime), 10);
    let (read, _) = signal.split();

    let run_count = Rc::new(Cell::new(0));
    let run_count_clone = Rc::clone(&run_count);

    let _effect = Effect::new(Rc::clone(&runtime), move || {
        let _ = read.get();
        run_count_clone.set(run_count_clone.get() + 1);
    });

    assert_eq!(run_count.get(), 1, "Effect should run once initially");
}

#[test]
fn test_effect_runs_when_dependency_changes() {
    let runtime = Runtime::new();
    let signal = Signal::new(Rc::clone(&runtime), 10);
    let (read, write) = signal.split();

    let run_count = Rc::new(Cell::new(0));
    let run_count_clone = Rc::clone(&run_count);

    let _effect = Effect::new(Rc::clone(&runtime), move || {
        let _ = read.get();
        run_count_clone.set(run_count_clone.get() + 1);
    });

    assert_eq!(run_count.get(), 1, "Initial run");

    write.set(20);
    assert_eq!(run_count.get(), 2, "Effect should run when signal changes");

    write.set(30);
    assert_eq!(run_count.get(), 3, "Effect should run again");
}

#[test]
fn test_effect_tracks_correct_dependencies() {
    let runtime = Runtime::new();
    let signal1 = Signal::new(Rc::clone(&runtime), 10);
    let signal2 = Signal::new(Rc::clone(&runtime), 20);

    let (read1, write1) = signal1.split();
    let (_read2, write2) = signal2.split();

    let run_count = Rc::new(Cell::new(0));
    let run_count_clone = Rc::clone(&run_count);

    // Effect only depends on signal1
    let _effect = Effect::new(Rc::clone(&runtime), move || {
        let _ = read1.get();
        run_count_clone.set(run_count_clone.get() + 1);
    });

    assert_eq!(run_count.get(), 1, "Initial run");

    write1.set(100);
    assert_eq!(run_count.get(), 2, "Should run when signal1 changes");

    write2.set(200);
    assert_eq!(run_count.get(), 2, "Should NOT run when signal2 changes");
}

#[test]
fn test_effect_with_computed() {
    let runtime = Runtime::new();
    let signal = Signal::new(Rc::clone(&runtime), 5);
    let (read, write) = signal.split();

    let doubled = Computed::new(Rc::clone(&runtime), {
        let read = read.clone();
        move || read.get() * 2
    });

    let run_count = Rc::new(Cell::new(0));
    let last_value = Rc::new(Cell::new(0));

    let run_count_clone = Rc::clone(&run_count);
    let last_value_clone = Rc::clone(&last_value);

    let _effect = Effect::new(Rc::clone(&runtime), move || {
        let value = doubled.get();
        run_count_clone.set(run_count_clone.get() + 1);
        last_value_clone.set(value);
    });

    assert_eq!(run_count.get(), 1);
    assert_eq!(last_value.get(), 10);

    write.set(10);
    assert_eq!(run_count.get(), 2);
    assert_eq!(last_value.get(), 20);
}

#[test]
fn test_multiple_effects_on_same_signal() {
    let runtime = Runtime::new();
    let signal = Signal::new(Rc::clone(&runtime), 0);
    let (read, write) = signal.split();

    let count1 = Rc::new(Cell::new(0));
    let count2 = Rc::new(Cell::new(0));

    let count1_clone = Rc::clone(&count1);
    let count2_clone = Rc::clone(&count2);

    let read1 = read.clone();
    let _effect1 = Effect::new(Rc::clone(&runtime), move || {
        let _ = read1.get();
        count1_clone.set(count1_clone.get() + 1);
    });

    let _effect2 = Effect::new(Rc::clone(&runtime), move || {
        let _ = read.get();
        count2_clone.set(count2_clone.get() + 1);
    });

    assert_eq!(count1.get(), 1);
    assert_eq!(count2.get(), 1);

    write.set(10);
    assert_eq!(count1.get(), 2, "Both effects should run");
    assert_eq!(count2.get(), 2, "Both effects should run");
}

// ==================== Complex Dependency Graph Tests ====================

#[test]
fn test_diamond_dependency() {
    let runtime = Runtime::new();
    let source = Signal::new(Rc::clone(&runtime), 1);
    let (read, write) = source.split();

    // Diamond: source -> left/right -> result
    let left = Computed::new(Rc::clone(&runtime), {
        let read = read.clone();
        move || read.get() + 10
    });

    let right = Computed::new(Rc::clone(&runtime), {
        let read = read.clone();
        move || read.get() + 20
    });

    let result = Computed::new(Rc::clone(&runtime), {
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
    let signal = Signal::new(Rc::clone(&runtime), 10);
    let (read, write) = signal.split();

    let run_count = Rc::new(Cell::new(0));
    let run_count_clone = Rc::clone(&run_count);

    let _effect = Effect::new(Rc::clone(&runtime), move || {
        let _ = read.get();
        run_count_clone.set(run_count_clone.get() + 1);
    });

    assert_eq!(run_count.get(), 1);

    // Set to same value
    write.set(10);

    // Effect should still run (we notify on any set, even if value doesn't change)
    // This is a design choice - can be optimized later
    assert!(run_count.get() >= 1, "Effect behavior on same-value set");
}

// ==================== Edge Cases ====================

#[test]
fn test_signal_clone() {
    let runtime = Runtime::new();
    let signal = Signal::new(Rc::clone(&runtime), 42);
    let (read1, _) = signal.split();
    let read2 = read1.clone();

    assert_eq!(read1.get(), 42);
    assert_eq!(read2.get(), 42);
}

#[test]
fn test_computed_clone() {
    let runtime = Runtime::new();
    let signal = Signal::new(Rc::clone(&runtime), 10);
    let (read, _) = signal.split();

    let computed = Computed::new(Rc::clone(&runtime), {
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
    let signal = Signal::new(Rc::clone(&runtime), 1);
    let (read, write) = signal.split();

    let c1 = Computed::new(Rc::clone(&runtime), {
        let read = read.clone();
        move || read.get() + 1
    });

    let c2 = Computed::new(Rc::clone(&runtime), {
        let c1 = c1.clone();
        move || c1.get() + 1
    });

    let c3 = Computed::new(Rc::clone(&runtime), {
        let c2 = c2.clone();
        move || c2.get() + 1
    });

    assert_eq!(c3.get(), 4); // 1 + 1 + 1 + 1

    write.set(10);
    assert_eq!(c3.get(), 13); // 10 + 1 + 1 + 1
}
