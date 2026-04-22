use a11y_engine::A11yId;
use a11y_engine::platform::action_handler::ArthropodActionHandler;
use accesskit::{ActionHandler, ActionRequest, NodeId as AccessKitNodeId};
use std::panic;
use std::sync::{Arc, Mutex};

#[test]
fn test_focus_handler_recovers_from_poison() {
    let mut handler = ArthropodActionHandler::new();
    let node1 = A11yId::new();
    let node2 = A11yId::new();

    let recovered = Arc::new(Mutex::new(false));
    let recovered_clone = recovered.clone();

    handler.set_focus_handler(move |id| {
        if id == node1 {
            panic!("Intentional panic to poison focus handler");
        }
        if id == node2 {
            *recovered_clone.lock().unwrap() = true;
        }
    });

    let _ = panic::catch_unwind(panic::AssertUnwindSafe(|| {
        handler.do_action(ActionRequest {
            action: accesskit::Action::Focus,
            target_node: AccessKitNodeId(node1.raw()),
            target_tree: accesskit::TreeId(accesskit::Uuid::nil()),
            data: None,
        });
    }));

    // Now send action to node2. If the handler handles poison correctly, it will execute.
    // If it ignores poison by returning None (like .ok()), it will fail silently.
    handler.do_action(ActionRequest {
        action: accesskit::Action::Focus,
        target_node: AccessKitNodeId(node2.raw()),
        target_tree: accesskit::TreeId(accesskit::Uuid::nil()),
        data: None,
    });

    assert!(
        *recovered.lock().unwrap(),
        "Focus handler did not recover from poison"
    );
}
