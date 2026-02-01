use flux_state::{Runtime, Signal, Computed};
use std::sync::{Arc, Barrier};
use std::sync::atomic::{AtomicUsize, AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

#[test]
fn test_concurrent_dependency_tracking() {
    let runtime = Runtime::new();

    let s1 = Signal::new(runtime.clone(), 10);
    let (r_s1, w_s1) = s1.split();

    let s2 = Signal::new(runtime.clone(), 20);
    let (r_s2, w_s2) = s2.split();

    let c1_count = Arc::new(AtomicUsize::new(0));
    let c1_count_clone = c1_count.clone();

    // Barrier to ensure both threads are inside their computation at the same time
    let barrier = Arc::new(Barrier::new(2));
    let barrier_c1 = barrier.clone();
    let barrier_c2 = barrier.clone();

    // Control flag to only block during the concurrent phase
    let should_block = Arc::new(AtomicBool::new(false));
    let should_block_c1 = should_block.clone();
    let should_block_c2 = should_block.clone();

    // C1 depends on S1
    let r_s1_c = r_s1.clone();
    let c1 = Computed::new(runtime.clone(), move || {
        c1_count_clone.fetch_add(1, Ordering::SeqCst);
        if should_block_c1.load(Ordering::SeqCst) {
            // Wait for other thread to be ready
            barrier_c1.wait();
            // Sleep to ensure overlap
            thread::sleep(Duration::from_millis(50));
        }
        r_s1_c.get()
    });

    let c2_count = Arc::new(AtomicUsize::new(0));
    let c2_count_clone = c2_count.clone();

    // C2 depends on S2
    let r_s2_c = r_s2.clone();
    let c2 = Computed::new(runtime.clone(), move || {
        c2_count_clone.fetch_add(1, Ordering::SeqCst);
        if should_block_c2.load(Ordering::SeqCst) {
            // Wait for other thread to be ready
            barrier_c2.wait();
            // Sleep to ensure overlap
            thread::sleep(Duration::from_millis(50));
        }
        r_s2_c.get()
    });

    // Initial computation done during creation
    assert_eq!(c1.get(), 10);
    assert_eq!(c2.get(), 20);
    assert_eq!(c1_count.load(Ordering::SeqCst), 1);
    assert_eq!(c2_count.load(Ordering::SeqCst), 1);

    // Enable blocking and mark stale
    should_block.store(true, Ordering::SeqCst);
    w_s1.set(11); // Marks c1 stale
    w_s2.set(21); // Marks c2 stale

    // Spawn threads to access them concurrently
    let c1_thread = c1.clone();
    let t1 = thread::spawn(move || {
        c1_thread.get()
    });

    let c2_thread = c2.clone();
    let t2 = thread::spawn(move || {
        c2_thread.get()
    });

    let v1 = t1.join().unwrap();
    let v2 = t2.join().unwrap();

    assert_eq!(v1, 11);
    assert_eq!(v2, 21);

    // Recomputed once more
    assert_eq!(c1_count.load(Ordering::SeqCst), 2);
    assert_eq!(c2_count.load(Ordering::SeqCst), 2);

    println!("Concurrent verification passed. Testing dependencies...");

    // Disable blocking for subsequent checks
    should_block.store(false, Ordering::SeqCst);

    // 1. Update S1. Should trigger C1 recompute next time it's read. Should NOT trigger C2.
    w_s1.set(12);

    // Check C1 (should verify S1 dependency)
    assert_eq!(c1.get(), 12, "C1 should have updated when S1 changed");
    assert_eq!(c1_count.load(Ordering::SeqCst), 3, "C1 should have recomputed");

    // Check C2 (should verify S1 is NOT a dependency)
    assert_eq!(c2.get(), 21, "C2 should not change when S1 changes");
    assert_eq!(c2_count.load(Ordering::SeqCst), 2, "C2 should NOT have recomputed when S1 changed");

    // 2. Update S2. Should trigger C2. Should NOT trigger C1.
    w_s2.set(22);

    // Check C2 (should verify S2 dependency)
    assert_eq!(c2.get(), 22, "C2 should have updated when S2 changed");
    assert_eq!(c2_count.load(Ordering::SeqCst), 3, "C2 should have recomputed");

    // Check C1 (should verify S2 is NOT a dependency)
    assert_eq!(c1.get(), 12, "C1 should not change when S2 changes");
    assert_eq!(c1_count.load(Ordering::SeqCst), 3, "C1 should NOT have recomputed when S2 changed");
}
