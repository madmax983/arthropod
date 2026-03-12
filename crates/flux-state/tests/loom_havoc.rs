use loom::sync::Mutex as LoomMutex;

#[test]
#[should_panic(expected = "Race condition: stale flag was overwritten!")]
fn test_loom_stale_flag_model() {
    // Model the exact problem in flux-state where recompute clears the stale flag unconditionally
    loom::model(|| {
        let stale = loom::sync::Arc::new(LoomMutex::new(true));

        let stale_t1 = stale.clone();
        let t1 = loom::thread::spawn(move || {
            // Worker thread simulating `recompute` in Runtime
            // Let's say it finishes computation and clears the flag
            let mut guard = stale_t1.lock().unwrap();
            *guard = false;
        });

        let stale_t2 = stale.clone();
        let t2 = loom::thread::spawn(move || {
            // Mutator thread simulating `notify` setting stale back to true
            let mut guard = stale_t2.lock().unwrap();
            *guard = true;
        });

        t1.join().unwrap();
        t2.join().unwrap();

        // After both threads finish, the system expects the data to be stale if t2 ran last.
        // However, because there's no atomic swap or guard, t1 might overwrite t2's 'true'.
        let final_stale = *stale.lock().unwrap();

        // This will panic when loom explores the permutation where T2 runs, then T1 runs and clears it.
        assert!(final_stale, "Race condition: stale flag was overwritten!");
    });
}
