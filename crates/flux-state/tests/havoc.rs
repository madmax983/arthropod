use flux_state::{Computed, Runtime, Signal};

#[test]
fn test_stale_flag_race_condition() {
    let runtime = Runtime::new();

    let s = Signal::new(runtime.clone(), 10);
    let (r_s, w_s) = s.split();

    let r_s_clone = r_s.clone();
    let x = Computed::new(runtime.clone(), move || {
        let val = r_s_clone.get();
        if val == 11 {
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
        val * 2
    });

    assert_eq!(x.get(), 20);

    w_s.set(11);

    let x_clone = x.clone();
    let t1 = std::thread::spawn(move || x_clone.get());

    std::thread::sleep(std::time::Duration::from_millis(10));
    w_s.set(20);

    let val1 = t1.join().unwrap();
    assert_eq!(val1, 22);

    assert_eq!(
        x.get(),
        40,
        "Race condition detected! X returned stale data."
    );
}
