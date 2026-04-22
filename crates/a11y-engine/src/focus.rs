//! Focus management for accessibility
//!
//! Tracks which node currently has accessibility focus and handles focus changes.

use crate::node::A11yId;
use bevy_ecs::system::Resource;

/// Tracks the currently focused accessible node
///
/// Screen readers use this to determine which element to announce.
/// Focus changes trigger events that are sent to the platform.
#[derive(Resource, Default, Clone)]
pub struct FocusManager {
    focused_node: Option<A11yId>,
    previous_node: Option<A11yId>,
}

impl FocusManager {
    /// Creates a new, empty focus manager.
    ///
    /// By default, no node is focused.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use a11y_engine::FocusManager;
    ///
    /// let manager = FocusManager::new();
    /// assert_eq!(manager.focused_node(), None);
    /// ```
    pub fn new() -> Self {
        Self {
            focused_node: None,
            previous_node: None,
        }
    }

    /// Get the currently focused node
    pub fn focused_node(&self) -> Option<A11yId> {
        self.focused_node
    }

    /// Get the previously focused node
    pub fn previous_node(&self) -> Option<A11yId> {
        self.previous_node
    }

    /// Set focus to a specific node
    ///
    /// Returns true if focus changed, false if the node was already focused.
    pub fn set_focus(&mut self, node_id: A11yId) -> bool {
        if self.focused_node == Some(node_id) {
            return false; // Already focused
        }

        self.previous_node = self.focused_node;
        self.focused_node = Some(node_id);
        true
    }

    /// Clear focus (no node focused)
    ///
    /// Returns true if there was a focused node, false if already unfocused.
    pub fn clear_focus(&mut self) -> bool {
        if self.focused_node.is_none() {
            return false;
        }

        self.previous_node = self.focused_node;
        self.focused_node = None;
        true
    }

    /// Check if a specific node has focus
    pub fn has_focus(&self, node_id: A11yId) -> bool {
        self.focused_node == Some(node_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_focus_manager_initial_state() {
        let manager = FocusManager::new();
        assert_eq!(manager.focused_node(), None);
        assert_eq!(manager.previous_node(), None);
    }

    #[test]
    fn test_set_focus() {
        let mut manager = FocusManager::new();
        let node1 = A11yId::new();

        let changed = manager.set_focus(node1);
        assert!(changed, "Focus should have changed");
        assert_eq!(manager.focused_node(), Some(node1));
        assert_eq!(manager.previous_node(), None);
    }

    #[test]
    fn test_set_focus_tracks_previous() {
        let mut manager = FocusManager::new();
        let node1 = A11yId::new();
        let node2 = A11yId::new();

        manager.set_focus(node1);
        manager.set_focus(node2);

        assert_eq!(manager.focused_node(), Some(node2));
        assert_eq!(manager.previous_node(), Some(node1));
    }

    #[test]
    fn test_set_focus_no_change_when_already_focused() {
        let mut manager = FocusManager::new();
        let node1 = A11yId::new();

        manager.set_focus(node1);
        let changed = manager.set_focus(node1);

        assert!(!changed, "Focus should not have changed");
        assert_eq!(manager.focused_node(), Some(node1));
    }

    #[test]
    fn test_clear_focus() {
        let mut manager = FocusManager::new();
        let node1 = A11yId::new();

        manager.set_focus(node1);
        let changed = manager.clear_focus();

        assert!(changed, "Focus should have been cleared");
        assert_eq!(manager.focused_node(), None);
        assert_eq!(manager.previous_node(), Some(node1));
    }

    #[test]
    fn test_clear_focus_when_already_unfocused() {
        let mut manager = FocusManager::new();
        let changed = manager.clear_focus();

        assert!(!changed, "Should not change when already unfocused");
        assert_eq!(manager.focused_node(), None);
    }

    #[test]
    fn test_has_focus() {
        let mut manager = FocusManager::new();
        let node1 = A11yId::new();
        let node2 = A11yId::new();

        manager.set_focus(node1);

        assert!(manager.has_focus(node1));
        assert!(!manager.has_focus(node2));
    }
}
