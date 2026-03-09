use arbitrary::Arbitrary;
use flux_state::{Runtime, Signal};

#[derive(Arbitrary, Debug)]
struct FuzzData {
    operations: Vec<u8>,
}

#[test]
#[should_panic(expected = "Reactive recursion limit exceeded")]
fn test_fuzz_operations_trigger_limit() {
    let raw_bytes = &[0x41, 0xFF, 0x00, 0x01, 0x2A, 0x8F, 0xCC, 0xDD, 0xEE, 0xFF];
    let mut u = arbitrary::Unstructured::new(raw_bytes);

    // We intentionally create a circular dependency to trigger the system's own panic
    if let Ok(data) = FuzzData::arbitrary(&mut u) {
        let runtime = Runtime::new();
        let s = Signal::new(runtime.clone(), 0);
        let (r, w) = s.split();

        let w_clone = w.clone();
        let _e = flux_state::Effect::new(runtime.clone(), move || {
            // Read and write in the same effect causes an infinite loop
            // and trigger the recursion limit panic naturally!
            w_clone.set(r.get() + data.operations.len() as i32);
        });
    }
}
