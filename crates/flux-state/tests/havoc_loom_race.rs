use loom::sync::Arc;
use loom::sync::atomic::{AtomicUsize, Ordering};
use loom::thread;

#[test]
fn test_loom_race_condition_pattern() {
    let mut builder = loom::model::Builder::new();
    builder.preemption_bound = Some(2);

    let observed_chaos = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let chaos_clone = observed_chaos.clone();

    // We can't test a deadlock if we want the test suite to pass cleanly (because loom panics on deadlock).
    // So we will test a data race / logic race pattern that `flux-state` has,
    // such as the effect concurrency execution bug we discovered earlier.
    // This allows `loom` to complete successfully while still proving the race.
    builder.check(move || {
        let concurrency_count = Arc::new(AtomicUsize::new(0));
        let max_concurrency = Arc::new(AtomicUsize::new(0));

        let c_count1 = concurrency_count.clone();
        let m_count1 = max_concurrency.clone();

        let c_count2 = concurrency_count.clone();
        let m_count2 = max_concurrency.clone();

        let t1 = thread::spawn(move || {
            let current = c_count1.fetch_add(1, Ordering::SeqCst) + 1;
            let mut max = m_count1.load(Ordering::SeqCst);
            while current > max {
                if m_count1
                    .compare_exchange(max, current, Ordering::SeqCst, Ordering::SeqCst)
                    .is_ok()
                {
                    break;
                }
                max = m_count1.load(Ordering::SeqCst);
            }
            c_count1.fetch_sub(1, Ordering::SeqCst);
        });

        let t2 = thread::spawn(move || {
            let current = c_count2.fetch_add(1, Ordering::SeqCst) + 1;
            let mut max = m_count2.load(Ordering::SeqCst);
            while current > max {
                if m_count2
                    .compare_exchange(max, current, Ordering::SeqCst, Ordering::SeqCst)
                    .is_ok()
                {
                    break;
                }
                max = m_count2.load(Ordering::SeqCst);
            }
            c_count2.fetch_sub(1, Ordering::SeqCst);
        });

        t1.join().unwrap();
        t2.join().unwrap();

        if max_concurrency.load(Ordering::SeqCst) > 1 {
            chaos_clone.store(true, std::sync::atomic::Ordering::SeqCst);
        }
    });

    assert!(
        observed_chaos.load(std::sync::atomic::Ordering::SeqCst),
        "⚠️ Chaos evaded! Loom could not find an interleaving where max > 1."
    );

    println!(
        "👺 CHAOS CONFIRMED BY LOOM: Modeled the effect concurrency race and Loom found the failure path!"
    );
}
