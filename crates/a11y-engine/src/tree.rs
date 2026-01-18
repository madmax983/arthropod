//! Accessibility tree implementation
//!
//! Manages the accessibility node hierarchy and synchronization with platform bridges.

use bevy_ecs::prelude::Resource;
use crate::node::{A11yId, A11yNode, Role};
use std::collections::{HashMap, HashSet};

/// Accessibility tree - manages accessible nodes and platform synchronization
///
/// Separate from Scene tree to allow platform-specific optimizations (hide decorative elements, etc.)
///
/// # Performance
///
/// - Add node: O(1) HashMap insert
/// - Update node: O(1) HashMap lookup + mark dirty
/// - Remove node: O(children) recursive removal
/// - Query by role: O(n) linear scan (acceptable for testing/debugging)
/// - Get dirty: O(1) HashSet clone
#[derive(Debug, Resource)]
pub struct A11yTree {
    /// All accessible nodes (O(1) lookup)
    nodes: HashMap<A11yId, A11yNode>,

    /// Root node ID
    root: A11yId,

    /// Nodes marked dirty (need platform sync)
    dirty_nodes: HashSet<A11yId>,
}

impl A11yTree {
    /// Create a new accessibility tree with root node
    pub fn new() -> Self {
        let root_id = A11yId::new();
        let root_node = A11yNode {
            parent: None,
            ..Default::default()
        };

        let mut nodes = HashMap::new();
        nodes.insert(root_id, root_node);

        Self {
            nodes,
            root: root_id,
            dirty_nodes: HashSet::new(),
        }
    }

    /// Get the root node ID
    pub fn root(&self) -> A11yId {
        self.root
    }

    /// Add accessible node as child of parent
    ///
    /// Returns the new node's ID. Marks node as dirty for platform sync.
    ///
    /// # Example
    ///
    /// ```
    /// use a11y_engine::{A11yTree, A11yNode, Role, AccessibleName};
    ///
    /// let mut tree = A11yTree::new();
    /// let button = A11yNode {
    ///     role: Role::Button,
    ///     name: AccessibleName::Text("Click me".into()),
    ///     ..Default::default()
    /// };
    ///
    /// let button_id = tree.add_node(tree.root(), button);
    /// assert!(tree.get_node(button_id).is_some());
    /// ```
    pub fn add_node(&mut self, parent_id: A11yId, mut node: A11yNode) -> A11yId {
        let node_id = A11yId::new();

        // Set parent relationship
        node.parent = Some(parent_id);

        // Add to tree
        self.nodes.insert(node_id, node);

        // Update parent's children
        if let Some(parent) = self.nodes.get_mut(&parent_id) {
            parent.children.push(node_id);
        }

        // Mark dirty for platform sync
        self.dirty_nodes.insert(node_id);

        node_id
    }

    /// Get node by ID (immutable)
    pub fn get_node(&self, id: A11yId) -> Option<&A11yNode> {
        self.nodes.get(&id)
    }

    /// Get node by ID (mutable)
    pub fn get_node_mut(&mut self, id: A11yId) -> Option<&mut A11yNode> {
        self.nodes.get_mut(&id)
    }

    /// Update node properties (marks dirty)
    ///
    /// # Example
    ///
    /// ```
    /// use a11y_engine::{A11yTree, A11yNode, Role, AccessibleName};
    ///
    /// let mut tree = A11yTree::new();
    /// let node_id = tree.add_node(tree.root(), A11yNode::default());
    ///
    /// tree.update_node(node_id, |node| {
    ///     node.name = AccessibleName::Text("Updated".into());
    ///     node.state.disabled = true;
    /// });
    /// ```
    pub fn update_node(&mut self, id: A11yId, update: impl FnOnce(&mut A11yNode)) {
        if let Some(node) = self.nodes.get_mut(&id) {
            update(node);
            self.dirty_nodes.insert(id);
        }
    }

    /// Remove node and all children recursively
    ///
    /// Updates parent's children list and removes from dirty set.
    pub fn remove_node(&mut self, id: A11yId) {
        // Don't allow removing root
        if id == self.root {
            return;
        }

        // Get node to find parent and children
        let node = match self.nodes.get(&id) {
            Some(n) => n.clone(), // Clone to avoid borrow issues
            None => return,
        };

        // Remove from parent's children
        if let Some(parent_id) = node.parent {
            if let Some(parent) = self.nodes.get_mut(&parent_id) {
                parent.children.retain(|child_id| *child_id != id);
            }
        }

        // Recursively remove children
        for child_id in &node.children {
            self.remove_node(*child_id);
        }

        // Remove node itself
        self.nodes.remove(&id);
        self.dirty_nodes.remove(&id);
    }

    /// Query nodes by role (for testing/debugging)
    ///
    /// O(n) linear scan - acceptable for testing.
    pub fn query_by_role(&self, role: Role) -> Vec<A11yId> {
        self.nodes
            .iter()
            .filter(|(_, node)| node.role == role)
            .map(|(id, _)| *id)
            .collect()
    }

    /// Check if node is marked dirty
    pub fn is_dirty(&self, id: A11yId) -> bool {
        self.dirty_nodes.contains(&id)
    }

    /// Get all dirty nodes
    ///
    /// Returns clone of dirty set. Used by platform bridges for incremental sync.
    pub fn get_dirty_nodes(&self) -> HashSet<A11yId> {
        self.dirty_nodes.clone()
    }

    /// Clear all dirty flags
    ///
    /// Call after successful platform sync.
    pub fn clear_dirty(&mut self) {
        self.dirty_nodes.clear();
    }

    /// Get total node count (including root)
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }
}

impl Default for A11yTree {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::node::AccessibleName;

    #[test]
    fn test_tree_creation() {
        let tree = A11yTree::new();
        assert_eq!(tree.node_count(), 1); // Root only
        assert!(tree.get_node(tree.root()).is_some());
    }

    #[test]
    fn test_add_and_get_node() {
        let mut tree = A11yTree::new();
        let root = tree.root();

        let button = A11yNode {
            role: Role::Button,
            name: AccessibleName::Text("Test".into()),
            ..Default::default()
        };

        let button_id = tree.add_node(root, button);

        assert!(tree.get_node(button_id).is_some());
        assert_eq!(tree.node_count(), 2);
    }
}

