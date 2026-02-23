use flux_state::{Effect, Runtime, Signal};
use std::panic;
use std::sync::{Arc, Mutex};

/// 🛡️ Sentry Test: Multi-Effect Resilience
///
/// Ensures that if one effect panics, other unrelated effects are RESTORED to the
/// pending queue and run on the NEXT update cycle.
///
/// This verifies that the runtime correctly handles panic unwinding and
/// prevents "lost" effects, even if it cannot continue the current flush cycle immediately.
#[test]
fn test_multi_effect_resilience() {
    let runtime = Runtime::new();

    // Source signal S
    let s = Signal::new(runtime.clone(), 0);
    let (read_s, write_s) = s.split();

    // Counters for verification
    let c1 = Arc::new(Mutex::new(0));
    let c2 = Arc::new(Mutex::new(0));
    let c3 = Arc::new(Mutex::new(0));

    let c1_clone = c1.clone();
    let c2_clone = c2.clone();
    let c3_clone = c3.clone();

    // E1: Safe effect
    let r_s1 = read_s.clone();
    let _e1 = Effect::new(runtime.clone(), move || {
        let _val = r_s1.get();
        *c1_clone.lock().unwrap() += 1;
    });

    // E2: Panics when S == 1
    let r_s2 = read_s.clone();
    let _e2 = Effect::new(runtime.clone(), move || {
        let val = r_s2.get();
        *c2_clone.lock().unwrap() += 1;
        if val == 1 {
            panic!("Effect 2 Panic!");
        }
    });

    // E3: Safe effect
    let r_s3 = read_s.clone();
    let _e3 = Effect::new(runtime.clone(), move || {
        let _val = r_s3.get();
        *c3_clone.lock().unwrap() += 1;
    });

    // Initial state: S=0. All effects run once.
    assert_eq!(*c1.lock().unwrap(), 1, "E1 initial run");
    assert_eq!(*c2.lock().unwrap(), 1, "E2 initial run");
    assert_eq!(*c3.lock().unwrap(), 1, "E3 initial run");

    // Update S to 1. This should trigger all 3 effects.
    // E2 will panic. The runtime aborts the current flush cycle.
    // Effects processed BEFORE panic run.
    // Effects AFTER panic are restored to pending queue but NOT run immediately.
    // Order is non-deterministic (HashSet iteration).
    println!("Updating S to 1 (Expect panic)...");
    let result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
        write_s.set(1);
    }));

    // Verify panic happened
    assert!(
        result.is_err(),
        "Update should have propagated the panic from E2"
    );

    // Verify E2 attempted to run
    assert_eq!(
        *c2.lock().unwrap(),
        2,
        "E2 should have started running for S=1"
    );

    // We cannot guarantee c1/c3 values because execution order is random.
    // But they should be >= 1.

    // Update S to 2. This should be safe.
    // Any pending effects from S=1 should run now.
    println!("Updating S to 2 (Expect recovery)...");
    let result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
        write_s.set(2);
    }));

    assert!(result.is_ok(), "Update to S=2 should succeed");

    // Verify Resilience: All effects should be alive and processing S=2.
    // If they missed S=1, they catch up now.
    // Minimum runs: Init (1) + S=2 (1) = 2.
    // Maximum runs: Init (1) + S=1 (1) + S=2 (1) = 3.

    let v1 = *c1.lock().unwrap();
    let v2 = *c2.lock().unwrap();
    let v3 = *c3.lock().unwrap();

    assert!(v1 >= 2, "E1 should have recovered (runs: {})", v1);
    assert!(v3 >= 2, "E3 should have recovered (runs: {})", v3);
    assert!(v2 >= 3, "E2 should have recovered (runs: {})", v2); // Init, Panic, Recovery
}
