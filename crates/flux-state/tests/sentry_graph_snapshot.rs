#![cfg(feature = "nova")]
use flux_state::{Computed, Runtime, Signal};

#[test]
fn test_inspect_graph_stability() {
    let runtime = Runtime::new();
    let sig1 = Signal::new(runtime.clone(), 1);
    let (rs1, ws1) = sig1.split();

    let sig2 = Signal::new(runtime.clone(), 2);
    let (rs2, _ws2) = sig2.split();

    // Setup an effect to force dependencies to register in the runtime graph
    let comp = Computed::new(runtime.clone(), move || rs1.get() + rs2.get());
    comp.get();

    // Force some nodes to be stale
    ws1.set(3);

    let snapshot1 = runtime.inspect_graph();

    // Verify properties of GraphSnapshot - especially the sorting guarantees
    // The sorting is defined by the memory:
    // "In the `flux-state` crate, the `inspect_graph` method in `Runtime` provides a stable
    // `GraphSnapshot` by explicitly sorting its fields (nodes by ID, dependencies by source/target ID,
    // and stale nodes by ID) before returning."

    // Check sorted nodes by ID
    let mut prev_node_id = None;
    for node in &snapshot1.nodes {
        if let Some(prev) = prev_node_id {
            assert!(
                node.id.0 > prev,
                "Nodes should be sorted by ID in GraphSnapshot"
            );
        }
        prev_node_id = Some(node.id.0);
    }

    // Check sorted dependencies by source then target
    let mut prev_dep = None;
    for dep in &snapshot1.dependencies {
        if let Some((prev_src, prev_tgt)) = prev_dep {
            assert!(
                dep.0.0 > prev_src || (dep.0.0 == prev_src && dep.1.0 > prev_tgt),
                "Dependencies should be sorted by source, then target in GraphSnapshot"
            );
        }
        prev_dep = Some((dep.0.0, dep.1.0));
    }

    // Check sorted stale nodes
    let mut prev_stale_id = None;
    for stale_id in &snapshot1.stale_nodes {
        if let Some(prev) = prev_stale_id {
            assert!(
                stale_id.0 > prev,
                "Stale nodes should be sorted by ID in GraphSnapshot"
            );
        }
        prev_stale_id = Some(stale_id.0);
    }

    // Second snapshot should be identical to first to prove stability
    let snapshot2 = runtime.inspect_graph();

    // Test for equality without PartialEq.
    // They are sorted, so we can check element-wise equality

    // Check nodes length
    assert_eq!(snapshot1.nodes.len(), snapshot2.nodes.len());
    for i in 0..snapshot1.nodes.len() {
        assert_eq!(snapshot1.nodes[i].id.0, snapshot2.nodes[i].id.0);
        assert_eq!(snapshot1.nodes[i].node_type, snapshot2.nodes[i].node_type);
        assert_eq!(snapshot1.nodes[i].label, snapshot2.nodes[i].label);
    }

    // Check dependencies length
    assert_eq!(snapshot1.dependencies.len(), snapshot2.dependencies.len());
    for i in 0..snapshot1.dependencies.len() {
        assert_eq!(snapshot1.dependencies[i].0.0, snapshot2.dependencies[i].0.0);
        assert_eq!(snapshot1.dependencies[i].1.0, snapshot2.dependencies[i].1.0);
    }

    // Check stale_nodes length
    assert_eq!(snapshot1.stale_nodes.len(), snapshot2.stale_nodes.len());
    for i in 0..snapshot1.stale_nodes.len() {
        assert_eq!(snapshot1.stale_nodes[i].0, snapshot2.stale_nodes[i].0);
    }
}
