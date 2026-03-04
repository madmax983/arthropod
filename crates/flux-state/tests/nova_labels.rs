#[cfg(feature = "nova")]
use flux_state::{Computed, Effect, Runtime, Signal};

#[test]
#[cfg(feature = "nova")]
fn test_nova_labels_and_inspect_graph() {
    let runtime = Runtime::new();

    let signal = Signal::new(runtime.clone(), 10).with_label("MySignal");
    let (read, write) = signal.split();

    let read_clone = read.clone();
    let computed =
        Computed::new(runtime.clone(), move || read_clone.get() * 2).with_label("MyComputed");

    let computed_clone = computed.clone();
    let _effect = Effect::new(runtime.clone(), move || {
        let _ = computed_clone.get();
    })
    .with_label("MyEffect");

    // Inspect the graph
    let snapshot = runtime.inspect_graph();

    // Verify nodes and labels
    assert_eq!(snapshot.nodes.len(), 3);

    let mut has_signal = false;
    let mut has_computed = false;
    let mut has_effect = false;

    for node in snapshot.nodes {
        match node.node_type {
            flux_state::NodeType::Signal => {
                assert_eq!(node.label, "MySignal");
                has_signal = true;
            }
            flux_state::NodeType::Computed => {
                assert_eq!(node.label, "MyComputed");
                has_computed = true;
            }
            flux_state::NodeType::Effect => {
                assert_eq!(node.label, "MyEffect");
                has_effect = true;
            }
        }
    }

    assert!(has_signal);
    assert!(has_computed);
    assert!(has_effect);

    // Update signal
    write.set(20);
    assert_eq!(computed.get(), 40);
}
