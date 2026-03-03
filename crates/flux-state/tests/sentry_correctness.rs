use flux_state::{Runtime, Signal};

#[test]
fn test_untested_update_logic() {
    let runtime = Runtime::new();
    let count: Signal<i32> = Signal::new(runtime, 0);
    let (read, write) = count.split();

    write.update(|val| *val += 5);
    assert_eq!(read.get(), 5);
}

#[test]
fn test_update_triggers_effects() {
    use flux_state::Effect;
    use std::sync::{Arc, Mutex};

    let runtime = Runtime::new();
    let count: Signal<i32> = Signal::new(runtime.clone(), 0);
    let (read, write) = count.split();

    let run_count = Arc::new(Mutex::new(0));
    let run_count_clone = run_count.clone();

    let read_clone = read.clone();
    let _effect = Effect::new(runtime.clone(), move || {
        read_clone.get();
        *run_count_clone.lock().unwrap() += 1;
    });

    assert_eq!(*run_count.lock().unwrap(), 1);

    write.update(|val| *val += 5);
    assert_eq!(*run_count.lock().unwrap(), 2);
    assert_eq!(read.get(), 5);
}
