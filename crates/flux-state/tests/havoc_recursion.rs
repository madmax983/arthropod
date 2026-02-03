use flux_state::{Effect, Runtime, Signal};

#[test]
#[ignore] // Crashes CI with stack overflow (as designed)
fn test_infinite_recursion_crash() {
    // 👺 HAVOC: This test is designed to crash the process with a stack overflow.
    // It sets up two effects that ping-pong updates to each other.
    // Since flux-state executes effects synchronously in `notify()`, this recurses.

    let runtime = Runtime::new();

    let sig_a = Signal::new(runtime.clone(), 0);
    let (read_a, write_a) = sig_a.split();

    let sig_b = Signal::new(runtime.clone(), 0);
    let (read_b, write_b) = sig_b.split();

    // Effect 1: When A changes, set B = A + 1
    let write_b_clone = write_b.clone();
    let _effect1 = Effect::new(runtime.clone(), move || {
        let val = read_a.get();
        if val % 1000 == 0 {
             println!("Depth: {}", val);
        }
        if val < 10_000_000 {
             write_b_clone.set(val + 1);
        }
    });

    // Effect 2: When B changes, set A = B + 1
    let write_a_clone = write_a.clone();
    let _effect2 = Effect::new(runtime.clone(), move || {
        let val = read_b.get();
        if val < 10_000_000 {
            write_a_clone.set(val + 1);
        }
    });

    // Trigger the cycle
    println!("👺 Detonating infinite recursion...");
    write_a.set(1);

    // If we reach here, the system is robust (iterative flushing).
    // If we crash, Havoc wins.
}
