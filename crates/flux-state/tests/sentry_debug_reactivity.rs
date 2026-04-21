#[cfg(test)]
mod tests {
    use flux_state::{Computed, Effect, Runtime, Signal};
    use std::sync::{Arc, Mutex};

    #[test]
    fn test_computed_debug_reactivity() {
        let runtime = Runtime::new();
        let signal = Signal::new(runtime.clone(), 10);
        let (read, write) = signal.split();

        let computed = Computed::new(runtime.clone(), move || read.get() * 2);

        let run_count = Arc::new(Mutex::new(0));
        let run_count_clone = run_count.clone();

        let computed_clone = computed.clone();
        let _effect = Effect::new(runtime.clone(), move || {
            *run_count_clone
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner) += 1;
            // Accessing via Debug formatting
            // This SHOULD trigger tracking, but currently doesn't.
            let _ = format!("{:?}", computed_clone);
        });

        // Initially 1 run
        assert_eq!(
            *run_count
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner),
            1
        );

        // Update signal -> computed becomes stale
        write.set(20);

        // Should re-run
        assert_eq!(
            *run_count
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner),
            2,
            "Effect should re-run when computed dependency updates, even if accessed via Debug"
        );
    }

    #[test]
    fn test_signal_debug_reactivity() {
        let runtime = Runtime::new();
        let signal = Signal::new(runtime.clone(), "Hello");
        let (_read, write) = signal.clone().split();

        let run_count = Arc::new(Mutex::new(0));
        let run_count_clone = run_count.clone();

        let signal_clone = signal.clone();
        let _effect = Effect::new(runtime.clone(), move || {
            *run_count_clone
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner) += 1;
            // Accessing via Debug formatting
            let _ = format!("{:?}", signal_clone);
        });

        assert_eq!(
            *run_count
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner),
            1
        );

        write.set("World");

        assert_eq!(
            *run_count
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner),
            2,
            "Effect should re-run when signal updates, even if accessed via Debug"
        );
    }

    #[test]
    fn test_read_signal_debug_reactivity() {
        let runtime = Runtime::new();
        let signal = Signal::new(runtime.clone(), 100);
        let (read, write) = signal.clone().split();

        let run_count = Arc::new(Mutex::new(0));
        let run_count_clone = run_count.clone();

        let read_clone = read.clone();
        let _effect = Effect::new(runtime.clone(), move || {
            *run_count_clone
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner) += 1;
            // Accessing via Debug formatting
            let _ = format!("{:?}", read_clone);
        });

        assert_eq!(
            *run_count
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner),
            1
        );

        write.set(200);

        assert_eq!(
            *run_count
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner),
            2,
            "Effect should re-run when read signal updates, even if accessed via Debug"
        );
    }

    #[test]
    fn test_write_signal_debug_reactivity() {
        let runtime = Runtime::new();
        let signal = Signal::new(runtime.clone(), 42);
        let (_read, write) = signal.clone().split();

        let run_count = Arc::new(Mutex::new(0));
        let run_count_clone = run_count.clone();

        let write_clone = write.clone();
        let _effect = Effect::new(runtime.clone(), move || {
            *run_count_clone
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner) += 1;
            // Accessing via Debug formatting on WriteSignal (which reads the value!)
            let _ = format!("{:?}", write_clone);
        });

        assert_eq!(
            *run_count
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner),
            1
        );

        write.set(84);

        assert_eq!(
            *run_count
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner),
            2,
            "Effect should re-run when write signal updates, even if accessed via Debug"
        );
    }
}
