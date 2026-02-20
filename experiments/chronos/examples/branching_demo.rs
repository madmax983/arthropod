use chronos::{RetroSignal, Timeline};
use flux_state::Runtime;

fn main() {
    let runtime = Runtime::new();
    let timeline = Timeline::new();
    let signal = RetroSignal::new(runtime.clone(), timeline.clone(), "MySignal", 0);

    println!("Initial State: {}", signal.get());

    // Change 1
    println!("Setting to 10...");
    signal.set(10);
    println!("State: {}", signal.get());

    // Change 2
    println!("Setting to 20...");
    signal.set(20);
    println!("State: {}", signal.get());

    // Undo 20 -> 10
    println!("Undoing...");
    timeline.lock().unwrap().undo();
    println!("State: {}", signal.get());

    // Branching: Set to 30 instead of 20
    println!("Setting to 30 (Branching)...");
    signal.set(30);
    println!("State: {}", signal.get());

    // Print Tree
    println!("\nHistory Tree:");
    timeline.lock().unwrap().print_tree();

    // Jump back to the "20" branch (Node 2)
    // Node 0: Root
    // Node 1: Set 10
    // Node 2: Set 20 (The one we undid)
    // Node 3: Set 30 (The new branch)

    println!("\nJumping to Node 2 (Set 20)...");
    timeline.lock().unwrap().jump_to(2);
    println!("State: {}", signal.get());

    // Jump back to Node 3 (Set 30)
    println!("\nJumping to Node 3 (Set 30)...");
    timeline.lock().unwrap().jump_to(3);
    println!("State: {}", signal.get());
}
