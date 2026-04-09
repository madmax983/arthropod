use flux_state::{Computed, Runtime, Signal};

#[test]
fn test_value_inspection() {
    let runtime = Runtime::new();

    let count = Signal::new(runtime.clone(), 42).with_label("count");
    let (read_count, write_count) = count.split();

    let double = Computed::new(runtime.clone(), move || read_count.get() * 2).with_label("double");

    // Evaluate computed
    assert_eq!(double.get(), 84);

    let snap = runtime.inspect_graph();

    let count_node = snap.nodes.iter().find(|n| n.label == "count").unwrap();
    assert_eq!(count_node.value, Some("42".to_string()));

    let double_node = snap.nodes.iter().find(|n| n.label == "double").unwrap();
    assert_eq!(double_node.value, Some("84".to_string()));

    // Change value
    write_count.set(100);
    assert_eq!(double.get(), 200);

    let snap2 = runtime.inspect_graph();
    let count_node2 = snap2.nodes.iter().find(|n| n.label == "count").unwrap();
    assert_eq!(count_node2.value, Some("100".to_string()));

    let double_node2 = snap2.nodes.iter().find(|n| n.label == "double").unwrap();
    assert_eq!(double_node2.value, Some("200".to_string()));
}
