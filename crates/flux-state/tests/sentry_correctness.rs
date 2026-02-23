use flux_state::{Computed, Effect, Runtime, Signal};
use std::panic;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

/// 🛡️ Sentry Test: Glitch Freedom for Computed Values
///
/// Ensures that a diamond dependency graph (A->B, A->C, B+C->D) results in
/// exactly one re-computation of D when A changes.
///
/// Graph:
///      A
///     / \
///    B   C
///     \ /
///      D
#[test]
fn test_glitch_freedom_computed() {
    let runtime = Runtime::new();

    // Source signal A
    let a = Signal::new(runtime.clone(), 1);
    let (read_a, write_a) = a.split();

    // Intermediate B = A + 10
    let read_a_b = read_a.clone();
    let b = Computed::new(runtime.clone(), move || read_a_b.get() + 10);

    // Intermediate C = A + 20
    let read_a_c = read_a.clone();
    let c = Computed::new(runtime.clone(), move || read_a_c.get() + 20);

    // Final D = B + C
    let b_clone = b.clone();
    let c_clone = c.clone();

    let compute_count = Arc::new(AtomicUsize::new(0));
    let count_clone = compute_count.clone();

    let d = Computed::new(runtime.clone(), move || {
        count_clone.fetch_add(1, Ordering::SeqCst);
        b_clone.get() + c_clone.get()
    });

    // Initial computation should happen immediately
    assert_eq!(d.get(), 32); // (1+10) + (1+20) = 11 + 21 = 32
    assert_eq!(
        compute_count.load(Ordering::SeqCst),
        1,
        "Initial computation"
    );

    // Update A
    // This invalidates B and C.
    // D depends on B and C.
    // D should recompute exactly ONCE.
    write_a.set(2);

    assert_eq!(d.get(), 34); // (2+10) + (2+20) = 12 + 22 = 34

    // VERIFICATION:
    // If glitchy, D might update when B updates (using stale C), then again when C updates.
    // Or it might update once.
    assert_eq!(
        compute_count.load(Ordering::SeqCst),
        2,
        "D should have recomputed exactly once after update"
    );
}

/// 🛡️ Sentry Test: Glitch Freedom for Effects
///
/// Ensures that an Effect dependent on a diamond graph runs exactly once.
#[test]
fn test_glitch_freedom_effect() {
    let runtime = Runtime::new();

    // Source A
    let a = Signal::new(runtime.clone(), 1);
    let (read_a, write_a) = a.split();

    // B = A * 2
    let read_a_b = read_a.clone();
    let b = Computed::new(runtime.clone(), move || read_a_b.get() * 2);

    // C = A * 3
    let read_a_c = read_a.clone();
    let c = Computed::new(runtime.clone(), move || read_a_c.get() * 3);

    // Effect reads B and C
    let b_clone = b.clone();
    let c_clone = c.clone();

    let run_count = Arc::new(AtomicUsize::new(0));
    let count_clone = run_count.clone();

    let _effect = Effect::new(runtime.clone(), move || {
        count_clone.fetch_add(1, Ordering::SeqCst);
        let val_b = b_clone.get();
        let val_c = c_clone.get();

        // Elenchus: Verify consistency between B and C.
        // B = A*2, C = A*3. Thus B/2 = A = C/3 => 3*B == 2*C.
        // This ensures we aren't seeing a "glitch" where B is from new A and C is from old A.
        assert_eq!(
            val_b * 3,
            val_c * 2,
            "Glitch detected! Inconsistent state: B={}, C={} (Expected 3B==2C)",
            val_b,
            val_c
        );
    });

    // Initial run
    assert_eq!(run_count.load(Ordering::SeqCst), 1, "Initial effect run");

    // Update A
    write_a.set(2);

    // VERIFICATION:
    // Effect should run exactly once, seeing the consistent state of B and C.
    assert_eq!(
        run_count.load(Ordering::SeqCst),
        2,
        "Effect should have run exactly once after update"
    );
}

/// 🛡️ Sentry Test: Dynamic Dependency Pruning (Computed)
///
/// Verifies that a computed value stops tracking dependencies it no longer uses.
/// A -> Switch -> (B or C).
/// If Switch selects B, updating C should NOT trigger A.
#[test]
fn test_dynamic_dependency_pruning() {
    let runtime = Runtime::new();

    let switch = Signal::new(runtime.clone(), true); // true = use B
    let (read_switch, write_switch) = switch.split();

    let b = Signal::new(runtime.clone(), 10);
    let (read_b, write_b) = b.split();

    let c = Signal::new(runtime.clone(), 20);
    let (read_c, write_c) = c.split();

    let compute_count = Arc::new(AtomicUsize::new(0));
    let count_clone = compute_count.clone();

    let read_switch_c = read_switch.clone();
    let read_b_c = read_b.clone();
    let read_c_c = read_c.clone();

    let output = Computed::new(runtime.clone(), move || {
        count_clone.fetch_add(1, Ordering::SeqCst);
        if read_switch_c.get() {
            read_b_c.get()
        } else {
            read_c_c.get()
        }
    });

    // Initial: switch=true, uses B=10
    assert_eq!(output.get(), 10);
    assert_eq!(compute_count.load(Ordering::SeqCst), 1);

    // Update C. Should NOT trigger recompute because we are using B.
    write_c.set(200);
    assert_eq!(output.get(), 10);
    assert_eq!(
        compute_count.load(Ordering::SeqCst),
        1,
        "Updating unused dependency C should not trigger recompute"
    );

    // Update B. Should trigger recompute.
    write_b.set(100);
    assert_eq!(output.get(), 100);
    assert_eq!(compute_count.load(Ordering::SeqCst), 2);

    // Switch to C.
    write_switch.set(false);
    assert_eq!(output.get(), 200); // Now reading C (which is 200)
    assert_eq!(compute_count.load(Ordering::SeqCst), 3);

    // Update B. Should NOT trigger recompute because we are using C.
    write_b.set(1000);
    assert_eq!(output.get(), 200);
    assert_eq!(
        compute_count.load(Ordering::SeqCst),
        3,
        "Updating unused dependency B should not run effect"
    );
}

/// 🛡️ Sentry Test: Dynamic Dependency Pruning (Effect)
///
/// Verifies that an effect stops tracking dependencies it no longer uses.
#[test]
fn test_effect_dynamic_dependency_cleanup() {
    let runtime = Runtime::new();

    let switch = Signal::new(runtime.clone(), true); // true = use B
    let (read_switch, write_switch) = switch.split();

    let b = Signal::new(runtime.clone(), 10);
    let (read_b, write_b) = b.split();

    let c = Signal::new(runtime.clone(), 20);
    let (read_c, write_c) = c.split();

    let run_count = Arc::new(AtomicUsize::new(0));
    let count_clone = run_count.clone();

    let read_switch_c = read_switch.clone();
    let read_b_c = read_b.clone();
    let read_c_c = read_c.clone();

    let _effect = Effect::new(runtime.clone(), move || {
        count_clone.fetch_add(1, Ordering::SeqCst);
        if read_switch_c.get() {
            let _ = read_b_c.get();
        } else {
            let _ = read_c_c.get();
        }
    });

    // Initial: switch=true, uses B
    assert_eq!(run_count.load(Ordering::SeqCst), 1);

    // Update C. Should NOT run effect.
    write_c.set(200);
    assert_eq!(
        run_count.load(Ordering::SeqCst),
        1,
        "Updating unused dependency C should not run effect"
    );

    // Update B. Should run effect.
    write_b.set(100);
    assert_eq!(run_count.load(Ordering::SeqCst), 2);

    // Switch to C.
    write_switch.set(false);
    assert_eq!(run_count.load(Ordering::SeqCst), 3);

    // Update B. Should NOT run effect.
    write_b.set(1000);
    assert_eq!(
        run_count.load(Ordering::SeqCst),
        3,
        "Updating unused dependency B should not run effect"
    );
}

#[test]
fn test_panic_state_consistency() {
    let runtime = Runtime::new();

    let x_sig = Signal::new(runtime.clone(), 1);
    let (x, set_x) = x_sig.split();

    let y_sig = Signal::new(runtime.clone(), 10);
    let (y, set_y) = y_sig.split();

    // c = if x == 0 { panic } else { y }
    // We intentionally make x depend on y only in the non-panic path
    // to simulate "lost dependency" if panic happens before y is tracked.
    let x_clone = x.clone();
    let y_clone = y.clone();
    let c = Computed::new(runtime.clone(), move || {
        let val_x = x_clone.get();
        if val_x == 0 {
            panic!("Boom");
        }
        y_clone.get()
    });

    // 1. Initial stable state
    assert_eq!(c.get(), 10);

    // 2. Put x in panic state
    set_x.set(0);

    // Verify it panics
    let result = panic::catch_unwind(panic::AssertUnwindSafe(|| c.get()));
    assert!(result.is_err());

    // 3. Update y. This should mark c as stale IF it was listening to y.
    // But due to the panic in step 2 (before reaching y.get()), c lost dependency on y.
    // If the bug exists, c is NOT marked stale.
    set_y.set(20);

    // 4. Query c again.
    // EXPECTATION: It should still panic (because x is 0).
    // REALITY (Hypothesis): It returns the old cached value (10) because it thinks it's not stale.
    let result = panic::catch_unwind(panic::AssertUnwindSafe(|| c.get()));

    if let Ok(val) = result {
        panic!(
            "Computed value should panic because x is 0. Instead, it returned stale value: {}",
            val
        );
    }
}
