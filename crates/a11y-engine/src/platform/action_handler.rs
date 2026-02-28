//! Action handler for accessibility interactions
//!
//! Handles actions triggered by screen readers (Click, Focus, etc.) and
//! dispatches them to application callbacks.

use crate::node::A11yId;
use accesskit::{ActionHandler, ActionRequest, NodeId as AccessKitNodeId};
use bevy_ecs::system::Resource;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Callback type for action handlers
pub type ActionCallback = Arc<dyn Fn() + Send + Sync>;

/// Callback type for focus handler
pub type FocusHandler = Arc<Mutex<dyn FnMut(A11yId) + Send>>;

/// Arthropod implementation of AccessKit ActionHandler
///
/// Routes actions from screen readers to application callbacks.
#[derive(Resource)]
pub struct ArthropodActionHandler {
    click_handlers: HashMap<A11yId, ActionCallback>,
    focus_handler: Option<FocusHandler>,
}

impl ArthropodActionHandler {
    pub fn new() -> Self {
        Self {
            click_handlers: HashMap::new(),
            focus_handler: None,
        }
    }

    /// Register a click callback for a specific node
    pub fn register_click<F>(&mut self, node_id: A11yId, callback: F)
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.click_handlers.insert(node_id, Arc::new(callback));
    }

    /// Unregister a click callback for a specific node
    pub fn unregister_click(&mut self, node_id: A11yId) {
        self.click_handlers.remove(&node_id);
    }

    /// Register a focus change handler
    pub fn set_focus_handler<F>(&mut self, handler: F)
    where
        F: FnMut(A11yId) + Send + 'static,
    {
        self.focus_handler = Some(Arc::new(Mutex::new(handler)));
    }

    /// Convert AccessKit NodeId to A11yId
    fn to_a11y_id(&self, ak_node_id: AccessKitNodeId) -> A11yId {
        // AccessKit NodeId wraps our A11yId.raw()
        // We need to reconstruct A11yId from the raw value
        // This is safe because we control both sides of the conversion
        A11yId::from_raw(ak_node_id.0)
    }
}

impl ActionHandler for ArthropodActionHandler {
    fn do_action(&mut self, request: ActionRequest) {
        let node_id = self.to_a11y_id(request.target_node);

        match request.action {
            accesskit::Action::Click => {
                // Invoke click callback if registered
                if let Some(callback) = self.click_handlers.get(&node_id) {
                    callback();
                }
            }
            accesskit::Action::Focus => {
                // Invoke focus handler if registered
                #[allow(clippy::collapsible_if)]
                if let Some(ref handler) = self.focus_handler {
                    #[allow(clippy::collapsible_if)]
                    if let Ok(mut h) = handler.lock() {
                        h(node_id);
                    }
                }
            }
            accesskit::Action::Increment
            | accesskit::Action::Decrement
            | accesskit::Action::ShowContextMenu
            | accesskit::Action::ScrollIntoView
            | accesskit::Action::ScrollUp
            | accesskit::Action::ScrollDown
            | accesskit::Action::ScrollLeft
            | accesskit::Action::ScrollRight => {
                log::debug!(
                    "Action {:?} received for node {:?} - handler not yet registered",
                    request.action,
                    node_id
                );
            }
            _ => {
                log::debug!(
                    "Unhandled action: {:?} for node {:?}",
                    request.action,
                    node_id
                );
            }
        }
    }
}

impl Default for ArthropodActionHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_action_handler_click_invokes_callback() {
        let clicked = Arc::new(Mutex::new(false));
        let clicked_clone = clicked.clone();

        let mut handler = ArthropodActionHandler::new();
        let node_id = A11yId::new();

        handler.register_click(node_id, move || {
            *clicked_clone.lock().unwrap() = true;
        });

        // Simulate AccessKit sending a click action
        let request = ActionRequest {
            action: accesskit::Action::Click,
            target_node: AccessKitNodeId(node_id.raw()),
            target_tree: accesskit::TreeId(accesskit::Uuid::nil()), // Dummy
            data: None,
        };

        handler.do_action(request);

        assert!(
            *clicked.lock().unwrap(),
            "Click callback should have been invoked"
        );
    }

    #[test]
    fn test_action_handler_focus_invokes_handler() {
        let focused_id = Arc::new(Mutex::new(None));
        let focused_clone = focused_id.clone();

        let mut handler = ArthropodActionHandler::new();
        let node_id = A11yId::new();

        handler.set_focus_handler(move |id| {
            *focused_clone.lock().unwrap() = Some(id);
        });

        // Simulate AccessKit sending a focus action
        let request = ActionRequest {
            action: accesskit::Action::Focus,
            target_node: AccessKitNodeId(node_id.raw()),
            target_tree: accesskit::TreeId(accesskit::Uuid::nil()),
            data: None,
        };

        handler.do_action(request);

        assert_eq!(
            *focused_id.lock().unwrap(),
            Some(node_id),
            "Focus handler should have been invoked with correct node ID"
        );
    }

    #[test]
    fn test_action_handler_multiple_clicks() {
        let click_count = Arc::new(Mutex::new(0));
        let count_clone1 = click_count.clone();
        let count_clone2 = click_count.clone();

        let mut handler = ArthropodActionHandler::new();
        let node1 = A11yId::new();
        let node2 = A11yId::new();

        handler.register_click(node1, move || {
            *count_clone1.lock().unwrap() += 1;
        });

        handler.register_click(node2, move || {
            *count_clone2.lock().unwrap() += 10;
        });

        // Click node1 twice
        handler.do_action(ActionRequest {
            action: accesskit::Action::Click,
            target_node: AccessKitNodeId(node1.raw()),
            target_tree: accesskit::TreeId(accesskit::Uuid::nil()),
            data: None,
        });
        handler.do_action(ActionRequest {
            action: accesskit::Action::Click,
            target_node: AccessKitNodeId(node1.raw()),
            target_tree: accesskit::TreeId(accesskit::Uuid::nil()),
            data: None,
        });

        // Click node2 once
        handler.do_action(ActionRequest {
            action: accesskit::Action::Click,
            target_node: AccessKitNodeId(node2.raw()),
            target_tree: accesskit::TreeId(accesskit::Uuid::nil()),
            data: None,
        });

        assert_eq!(*click_count.lock().unwrap(), 12); // 1+1+10 = 12
    }

    #[test]
    fn test_action_handler_unregistered_node() {
        let clicked = Arc::new(Mutex::new(false));
        let clicked_clone = clicked.clone();

        let mut handler = ArthropodActionHandler::new();
        let registered_node = A11yId::new();
        let unregistered_node = A11yId::new();

        handler.register_click(registered_node, move || {
            *clicked_clone.lock().unwrap() = true;
        });

        // Try to click unregistered node - should not panic
        handler.do_action(ActionRequest {
            action: accesskit::Action::Click,
            target_node: AccessKitNodeId(unregistered_node.raw()),
            target_tree: accesskit::TreeId(accesskit::Uuid::nil()),
            data: None,
        });

        assert!(
            !*clicked.lock().unwrap(),
            "Unregistered node should not trigger callback"
        );
    }
}
