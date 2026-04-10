use flux_state::{Runtime, Signal};

#[test]
fn test_rwlock_poison_crashes_get() {
    let runtime = Runtime::new();
    let signal = Signal::new(runtime.clone(), 0);
    let (read, write) = signal.split();

    let res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        write.update(|v| {
            *v = 1;
            panic!("Boom");
        });
    }));
    assert!(res.is_err());

    // Reading the signal should now panic with "poisoned lock"
    let res2 = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        read.get();
    }));
    assert!(res2.is_err());
}
