use a11y_engine::A11yTree;
use a11y_engine::platform::accesskit_bridge::AccessKitBridge;
use std::sync::{Arc, Mutex};
use std::thread;

#[test]
fn test_havoc_a11y_tree_poison() {
    let tree = Arc::new(Mutex::new(A11yTree::new()));
    let bridge = AccessKitBridge::new(tree.clone());

    let tree_clone = tree.clone();
    let _ = thread::spawn(move || {
        let _lock = tree_clone.lock().unwrap();
        panic!("Die!");
    })
    .join();

    let update = bridge.create_tree_update(&[]);
    assert_eq!(update.nodes.len(), 0);
}
