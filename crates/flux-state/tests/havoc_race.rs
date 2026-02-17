use flux_state::{Computed, Runtime, Signal};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

#[test]
fn test_computed_race_condition() {
    let runtime = Runtime::new();

    // 1. Create a signal S = 0
    let signal = Signal::new(runtime.clone(), 0);
    let (read_s, write_s) = signal.split();

    // 2. Create a computed C = S + 1
    //    We inject a sleep to simulate slow computation.
    let read_s_clone = read_s.clone();
    let computed = Computed::new(runtime.clone(), move || {
        let val = read_s_clone.get();
        // Simulate heavy work
        thread::sleep(Duration::from_millis(50));
        val + 1
    });

    // Initial state: S=0, C=1.
    assert_eq!(computed.get(), 1);

    // 3. Update S = 1. C becomes stale.
    //    Next compute should return 2.
    write_s.set(1);

    // 4. Spawn Thread 1 (The SlowUpdater)
    //    It reads C, triggering recomputation.
    //    It clears the stale flag, then sleeps for 50ms.
    let c1 = computed.clone();
    let (tx1, rx1) = mpsc::channel();
    thread::spawn(move || {
        let _ = c1.get(); // This blocks for 50ms
        tx1.send(()).unwrap();
    });

    // 5. Spawn Thread 2 (The Victim)
    //    It waits 10ms (so T1 has started and cleared stale flag),
    //    then reads C.
    //    It SHOULD block until T1 finishes, or T1 shouldn't have cleared stale flag.
    //    Instead, it sees stale=false, and reads the OLD value (1).
    let c2 = computed.clone();
    let (tx2, rx2) = mpsc::channel();
    thread::spawn(move || {
        thread::sleep(Duration::from_millis(10));
        let val = c2.get();
        tx2.send(val).unwrap();
    });

    // 6. Wait for T2 result
    let val_t2 = rx2.recv_timeout(Duration::from_millis(200)).unwrap();

    // 7. Assert T2 got the OLD value (1).
    //    This confirms the race condition.
    if val_t2 == 1 {
        println!("SUCCESS: Race condition confirmed! Got OLD value (1) during recomputation.");
    } else if val_t2 == 2 {
        panic!(
            "FAILURE: Race condition missed! Got NEW value (2). The system behaved correctly? Impossible!"
        );
    } else {
        panic!("Unexpected value: {}", val_t2);
    }

    // Wait for T1 to finish
    let _ = rx1.recv();
}
