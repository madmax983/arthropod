use flux_state::{Effect, Runtime, Signal};
use proptest::prelude::*;
use std::sync::Arc;
use std::thread;

proptest! {
    // We will test if running thousands of concurrent updates or creating a huge graph works properly
    #[test]
    fn test_concurrent_signal_updates(updates in 100..10000) {
        let runtime = Runtime::new();
        let signal = Signal::new(runtime.clone(), 0);
        let (read, write) = signal.split();

        // One thread rapidly updating
        let w_clone = write.clone();
        let writer1 = thread::spawn(move || {
            for _ in 0..updates {
                w_clone.update(|v| *v += 1);
            }
        });

        // Another thread also updating
        let w_clone2 = write.clone();
        let writer2 = thread::spawn(move || {
            for _ in 0..updates {
                w_clone2.update(|v| *v += 1);
            }
        });

        // Let's create an effect that continuously reads
        let log = Arc::new(std::sync::Mutex::new(0));
        let log_clone = log.clone();
        let r_clone = read.clone();

        let _effect = Effect::new(runtime.clone(), move || {
            let val = r_clone.get();
            *log_clone.lock().unwrap() = val;
        });

        writer1.join().unwrap();
        writer2.join().unwrap();

        // Wait a bit for pending effects to flush
        let val = read.get();
        assert_eq!(val, updates * 2);
    }
}
