use arthropod::experimental::chronos::{RetroSignal, Timeline};
use flux_state::Runtime;
use std::thread;

#[test]
#[should_panic(expected = "called `Result::unwrap()` on an `Err` value")]
fn test_chronos_poison() {
    let runtime = Runtime::new();
    let timeline = Timeline::new(runtime.clone());

    let timeline_clone = timeline.clone();
    let t = thread::spawn(move || {
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let mut _tl = timeline_clone.lock().unwrap();
            panic!("Poisoning chronos timeline lock!");
        }));
    });
    let _ = t.join();

    let signal = RetroSignal::new(runtime, timeline.clone(), "test", 0);
    // this calls `self.timeline.lock().unwrap().record(...)` inside `set`
    signal.set(42);
}
