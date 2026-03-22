use flux_state::{Runtime, Signal};
use loom::sync::{Arc, Mutex};
use loom::thread;

#[test]
fn test_loom_signal_read_write_race() {
    let mut builder = loom::model::Builder::new();
    builder.preemption_bound = Some(2);

    builder.check(move || {
        let runtime = Runtime::new();
        let signal = Signal::new(runtime.clone(), 0);
        let (read, write) = signal.split();

        let w_clone = write.clone();
        let t1 = thread::spawn(move || {
            w_clone.update(|v| *v += 1);
        });

        let r_clone = read.clone();
        let log = Arc::new(Mutex::new(0));
        let log_clone = log.clone();
        let t2 = thread::spawn(move || {
            let val = r_clone.get();
            *log_clone.lock().unwrap() = val;
        });

        t1.join().unwrap();
        t2.join().unwrap();

        let final_log = *log.lock().unwrap();
        // Since threads are racing, we either read the value before write (0) or after write (1)
        assert!(final_log == 0 || final_log == 1);
    });
}
