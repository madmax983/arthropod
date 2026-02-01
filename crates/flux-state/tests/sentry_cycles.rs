use flux_state::{Computed, Runtime, Signal};
use std::sync::{Arc, Mutex};

#[test]
fn test_computed_cycle_behavior() {
    let runtime = Runtime::new();

    // Create handles to store computeds before they are fully created
    let a_handle: Arc<Mutex<Option<Computed<i32>>>> = Arc::new(Mutex::new(None));
    let b_handle: Arc<Mutex<Option<Computed<i32>>>> = Arc::new(Mutex::new(None));

    let _a_handle_c = a_handle.clone();
    let b_handle_c = b_handle.clone();

    // Proxies to force staleness
    let proxy_a = Signal::new(runtime.clone(), 0);
    let (r_pa, w_pa) = proxy_a.split();

    let proxy_b = Signal::new(runtime.clone(), 0);
    let (r_pb, w_pb) = proxy_b.split();

    let r_pa_c = r_pa.clone();
    let a = Computed::new(runtime.clone(), move || {
        r_pa_c.get(); // Depend on ProxyA
        if let Some(ref b) = *b_handle_c.lock().unwrap() {
            // If B exists, read it.
            // Breaking cycle by limiting depth? No, let's see what happens.
            b.get() + 1
        } else {
            0
        }
    });

    let r_pb_c = r_pb.clone();
    let a_clone = a.clone();
    let b = Computed::new(runtime.clone(), move || {
        r_pb_c.get(); // Depend on ProxyB
        // Read A
        a_clone.get() + 1
    });

    *a_handle.lock().unwrap() = Some(a.clone());
    *b_handle.lock().unwrap() = Some(b.clone());

    // Initial state:
    // A was computed first. B was None. A = 0.
    // B was computed second. A = 0. B = 1.
    assert_eq!(a.get(), 0);
    assert_eq!(b.get(), 1);

    // Now trigger the cycle.
    // Force both to be stale.
    w_pa.set(1); // A stale.
    w_pb.set(1); // B stale.

    // Now A and B are both stale.
    // A depends on B. B depends on A.

    // Access A.
    // A recomputes. Calls B.
    // B recomputes. Calls A.
    // A should return old value (0) because it's currently computing?
    let val_a = a.get();

    // If it didn't crash:
    println!("Cycle survived! A = {}", val_a);

    // B should also be accessible
    let val_b = b.get();
    println!("Cycle survived! B = {}", val_b);

    // We don't enforce specific values as they depend on implementation details of cycle handling,
    // but we assert it doesn't panic or stack overflow.
}
