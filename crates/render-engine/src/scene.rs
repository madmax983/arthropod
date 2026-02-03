//! Scene graph data structures.
//!
//! The scene graph is a retained-mode structure that manages visual nodes
//! in a flattened hierarchy. It uses `NodeId` handles to reference nodes,
//! allowing for efficient O(1) lookups and updates.
//!
//! # Architecture
//!
//! - **Flattened Storage**: Nodes are stored in a `HashMap<NodeId, SceneNode>`,
//!   avoiding deep recursion for lookups.
//! - **Parent Pointers**: Each node stores its parent's ID, enabling O(1)
//!   upward traversal.
//! - **ECS Integration**: `Scene` is designed to be used as a Resource in
//!   `bevy_ecs`.

use crate::SceneNode;
use bevy_ecs::prelude::*;
use hashbrown::HashMap;
use serde::{Deserialize, Serialize};

/// Unique identifier for scene nodes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeId(pub u64);

/// The scene graph - owns all nodes.
///
/// Scene can now be stored as an ECS Resource, eliminating the need for
/// unsafe pointer juggling. Systems access Scene via `Res<Scene>` and `ResMut<Scene>`.
#[derive(Resource)]
pub struct Scene {
    // We use hashbrown::HashMap (AHash) instead of std::HashMap (SipHash)
    // for significantly faster integer key lookups (~60% speedup).
    nodes: HashMap<NodeId, SceneNode>,
    root: NodeId,
    next_id: u64,
    dirty_nodes: Vec<NodeId>,
}

impl Default for Scene {
    fn default() -> Self {
        Self::new()
    }
}

impl Scene {
    pub fn new() -> Self {
        let root_id = NodeId(0);
        let mut nodes = HashMap::new();
        nodes.insert(root_id, SceneNode::new_root());

        Self {
            nodes,
            root: root_id,
            next_id: 1,
            dirty_nodes: Vec::new(),
        }
    }

    pub fn root(&self) -> NodeId {
        self.root
    }

    /// Add a node to the scene, returns its ID.
    ///
    /// Sets the node's parent field automatically for O(1) parent lookup.
    ///
    /// # Example
    ///
    /// ```
    /// use render_engine::{Scene, SceneNode, NodeContent, Color};
    ///
    /// let mut scene = Scene::new();
    /// let root = scene.root();
    ///
    /// let child = scene.add_node(
    ///     root,
    ///     SceneNode::new(NodeContent::Rect { color: Color::RED })
    /// );
    /// ```
    pub fn add_node(&mut self, parent: NodeId, mut node: SceneNode) -> NodeId {
        let id = NodeId(self.next_id);
        self.next_id += 1;

        // Set the parent field for O(1) lookup
        node.parent = Some(parent);

        self.nodes.insert(id, node);
        if let Some(parent_node) = self.nodes.get_mut(&parent) {
            parent_node.children.push(id);
        }

        self.mark_dirty(id);
        id
    }

    /// Get a node by ID.
    pub fn get_node(&self, id: NodeId) -> Option<&SceneNode> {
        self.nodes.get(&id)
    }

    /// Get a mutable node by ID.
    pub fn get_node_mut(&mut self, id: NodeId) -> Option<&mut SceneNode> {
        self.nodes.get_mut(&id)
    }

    /// Get a node by ID (alias for get_node, for ECS compatibility).
    pub fn get(&self, id: NodeId) -> Option<&SceneNode> {
        self.get_node(id)
    }

    /// Get a mutable node by ID (alias for get_node_mut, for ECS compatibility).
    pub fn get_mut(&mut self, id: NodeId) -> Option<&mut SceneNode> {
        self.get_node_mut(id)
    }

    /// Get the parent of a node (O(1) lookup).
    ///
    /// Returns `None` if the node doesn't exist or is the root node.
    pub fn parent(&self, child_id: NodeId) -> Option<NodeId> {
        self.nodes.get(&child_id).and_then(|node| node.parent)
    }

    /// Find the parent of a node (O(1) lookup via parent field).
    ///
    /// This is now O(1) thanks to the parent field in SceneNode.
    /// For backwards compatibility, this returns the same result as `parent()`.
    pub fn find_parent(&self, child_id: NodeId) -> Option<NodeId> {
        self.parent(child_id)
    }

    /// Re-parent a node from old parent to new parent.
    ///
    /// Updates the child's parent field for O(1) lookup.
    ///
    /// # Example
    ///
    /// ```
    /// use render_engine::{Scene, SceneNode, NodeContent, Color};
    ///
    /// let mut scene = Scene::new();
    /// let root = scene.root();
    ///
    /// // Create container
    /// let container = scene.add_node(root, SceneNode::new(NodeContent::Empty));
    ///
    /// // Create child attached to root
    /// let child = scene.add_node(root, SceneNode::new(NodeContent::Empty));
    ///
    /// // Move child to container
    /// scene.reparent_node(child, root, container);
    ///
    /// assert!(scene.get_node(container).unwrap().children.contains(&child));
    /// ```
    pub fn reparent_node(&mut self, child_id: NodeId, old_parent: NodeId, new_parent: NodeId) {
        // Remove child from old parent's children list
        if let Some(old_parent_node) = self.nodes.get_mut(&old_parent) {
            old_parent_node.children.retain(|&id| id != child_id);
        }

        // Add child to new parent's children list
        if let Some(new_parent_node) = self.nodes.get_mut(&new_parent) {
            new_parent_node.children.push(child_id);
        }

        // Update child's parent field for O(1) lookup
        if let Some(child_node) = self.nodes.get_mut(&child_id) {
            child_node.parent = Some(new_parent);
        }

        // Mark both parents as dirty
        self.mark_dirty(old_parent);
        self.mark_dirty(new_parent);
    }

    /// Mark a node as needing redraw.
    pub fn mark_dirty(&mut self, id: NodeId) {
        if !self.dirty_nodes.contains(&id) {
            self.dirty_nodes.push(id);
        }
    }

    /// Remove a node from the scene.
    ///
    /// Removes the node from its parent's children list and deletes the node.
    /// Note: This does not recursively remove children. Orphaned children will remain in the map
    /// but have no parent, potentially leaking if not handled.
    ///
    /// # Example
    ///
    /// ```
    /// # use render_engine::{Scene, SceneNode, NodeContent, Color};
    /// # let mut scene = Scene::new();
    /// # let root = scene.root();
    /// let node = scene.add_node(root, SceneNode::new(NodeContent::Rect { color: Color::RED }));
    /// scene.remove_node(node);
    /// assert!(scene.get_node(node).is_none());
    /// ```
    #[allow(clippy::collapsible_if)]
    pub fn remove_node(&mut self, id: NodeId) {
        if let Some(node) = self.nodes.remove(&id) {
            // Remove from parent's children list
            if let Some(parent_id) = node.parent {
                if let Some(parent) = self.nodes.get_mut(&parent_id) {
                    parent.children.retain(|&child_id| child_id != id);
                    self.mark_dirty(parent_id);
                }
            }
        }
    }

    /// Get dirty nodes and clear the list.
    pub fn take_dirty(&mut self) -> Vec<NodeId> {
        std::mem::take(&mut self.dirty_nodes)
    }

    /// Iterate over all nodes in the scene.
    pub fn nodes(&self) -> impl Iterator<Item = (NodeId, &SceneNode)> {
        self.nodes.iter().map(|(id, node)| (*id, node))
    }

    /// Find a visible node at the given screen position.
    ///
    /// # Z-Order Warning
    ///
    /// **Behavior for overlapping nodes is undefined.**
    ///
    /// This method iterates through the internal node storage (hash map) and returns
    /// *a* node that contains the point. It does *not* strictly respect Z-order
    /// or hierarchy depth for overlapping siblings.
    ///
    /// # Arguments
    ///
    /// * `x` - X coordinate in screen space
    /// * `y` - Y coordinate in screen space
    ///
    /// # Returns
    ///
    /// `Some(NodeId)` of a node at this position, or `None` if no node found.
    ///
    /// # Example
    ///
    /// ```
    /// use render_engine::{Scene, SceneNode, NodeContent, Color};
    /// use plat_core::Rect;
    ///
    /// let mut scene = Scene::new();
    /// let root = scene.root();
    ///
    /// let mut node = SceneNode::new(NodeContent::Rect { color: Color::RED });
    /// node.bounds = Rect::new(10.0, 10.0, 100.0, 100.0);
    /// let node_id = scene.add_node(root, node);
    ///
    /// assert_eq!(scene.hit_test(50.0, 50.0), Some(node_id));
    /// assert_eq!(scene.hit_test(200.0, 200.0), None);
    /// ```
    pub fn hit_test(&self, x: f32, y: f32) -> Option<NodeId> {
        let mut result = None;

        for (id, node) in &self.nodes {
            if node.visible && node.bounds.contains(x, y) {
                result = Some(*id);
            }
        }

        result
    }

    /// Serialize the scene to JSON for MCP debugging
    pub fn serialize_to_json(&self) -> Result<serde_json::Value, serde_json::Error> {
        use serde_json::json;

        let nodes: Vec<_> = self
            .nodes
            .iter()
            .map(|(id, node)| {
                json!({
                    "id": id.0,
                    "content": node.content,
                    "transform": node.transform,
                    "bounds": node.bounds,
                    "children": node.children,
                    "visible": node.visible,
                    "opacity": node.opacity,
                })
            })
            .collect();

        Ok(json!({
            "root": self.root.0,
            "nodes": nodes,
            "node_count": self.nodes.len(),
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Color, NodeContent};
    use plat_core::Rect;

    #[test]
    fn test_hit_test_empty_scene_returns_none() {
        let scene = Scene::new();
        assert_eq!(scene.hit_test(100.0, 100.0), None);
    }

    #[test]
    fn test_hit_test_finds_node_at_position() {
        let mut scene = Scene::new();
        let root = scene.root();

        let mut node = SceneNode::new(NodeContent::Rect { color: Color::RED });
        node.bounds = Rect::new(10.0, 10.0, 100.0, 100.0);
        let node_id = scene.add_node(root, node);

        // Inside bounds
        assert_eq!(scene.hit_test(50.0, 50.0), Some(node_id));
        assert_eq!(scene.hit_test(10.0, 10.0), Some(node_id)); // Top-left corner
        assert_eq!(scene.hit_test(109.0, 109.0), Some(node_id)); // Near bottom-right
    }

    #[test]
    fn test_hit_test_misses_outside_bounds() {
        let mut scene = Scene::new();
        let root = scene.root();

        let mut node = SceneNode::new(NodeContent::Rect { color: Color::RED });
        node.bounds = Rect::new(10.0, 10.0, 100.0, 100.0);
        scene.add_node(root, node);

        // Outside bounds
        assert_eq!(scene.hit_test(0.0, 0.0), None);
        assert_eq!(scene.hit_test(5.0, 50.0), None); // Left of bounds
        assert_eq!(scene.hit_test(50.0, 5.0), None); // Above bounds
        assert_eq!(scene.hit_test(200.0, 50.0), None); // Right of bounds
        assert_eq!(scene.hit_test(50.0, 200.0), None); // Below bounds
    }

    #[test]
    fn test_hit_test_returns_overlapping_node() {
        let mut scene = Scene::new();
        let root = scene.root();

        // Add two overlapping nodes
        let mut node1 = SceneNode::new(NodeContent::Rect { color: Color::RED });
        node1.bounds = Rect::new(0.0, 0.0, 100.0, 100.0);
        let id1 = scene.add_node(root, node1);

        let mut node2 = SceneNode::new(NodeContent::Rect { color: Color::BLUE });
        node2.bounds = Rect::new(50.0, 50.0, 100.0, 100.0);
        let id2 = scene.add_node(root, node2);

        // In overlap region, should return one of the overlapping nodes
        // (HashMap iteration order is not guaranteed)
        let hit = scene.hit_test(75.0, 75.0);
        assert!(
            hit == Some(id1) || hit == Some(id2),
            "Expected id1 or id2, got {:?}",
            hit
        );

        // Only in first node
        assert_eq!(scene.hit_test(25.0, 25.0), Some(id1));

        // Only in second node
        assert_eq!(scene.hit_test(125.0, 125.0), Some(id2));
    }

    #[test]
    fn test_hit_test_ignores_invisible_nodes() {
        let mut scene = Scene::new();
        let root = scene.root();

        let mut node = SceneNode::new(NodeContent::Rect { color: Color::RED });
        node.bounds = Rect::new(10.0, 10.0, 100.0, 100.0);
        node.visible = false;
        scene.add_node(root, node);

        // Should not find invisible node
        assert_eq!(scene.hit_test(50.0, 50.0), None);
    }

    #[test]
    fn test_hit_test_with_nested_nodes() {
        let mut scene = Scene::new();
        let root = scene.root();

        // Parent container
        let mut parent = SceneNode::new(NodeContent::Rect {
            color: Color::WHITE,
        });
        parent.bounds = Rect::new(0.0, 0.0, 200.0, 200.0);
        let parent_id = scene.add_node(root, parent);

        // Child inside parent
        let mut child = SceneNode::new(NodeContent::Rect { color: Color::RED });
        child.bounds = Rect::new(50.0, 50.0, 50.0, 50.0);
        let child_id = scene.add_node(parent_id, child);

        // In overlap region, should return one of parent or child
        // (HashMap iteration order is not guaranteed)
        let hit = scene.hit_test(75.0, 75.0);
        assert!(
            hit == Some(parent_id) || hit == Some(child_id),
            "Expected parent_id or child_id, got {:?}",
            hit
        );

        // Hit parent only (outside child bounds)
        assert_eq!(scene.hit_test(25.0, 25.0), Some(parent_id));
    }

    #[test]
    fn test_hit_test_boundary_conditions() {
        let mut scene = Scene::new();
        let root = scene.root();

        let mut node = SceneNode::new(NodeContent::Rect { color: Color::RED });
        node.bounds = Rect::new(10.0, 10.0, 100.0, 100.0);
        let node_id = scene.add_node(root, node);

        // Exactly at boundaries
        assert_eq!(scene.hit_test(10.0, 10.0), Some(node_id)); // Top-left: inclusive
        assert_eq!(scene.hit_test(110.0, 110.0), None); // Bottom-right: exclusive
        assert_eq!(scene.hit_test(109.99, 109.99), Some(node_id)); // Just inside
    }

    #[test]
    fn test_remove_node_updates_parent_children_list() {
        let mut scene = Scene::new();
        let root = scene.root();

        // Create a child node
        let mut child = SceneNode::new(NodeContent::Rect { color: Color::RED });
        child.bounds = Rect::new(10.0, 10.0, 100.0, 100.0);
        let child_id = scene.add_node(root, child);

        // Verify child is in parent's list
        assert!(scene.get_node(root).unwrap().children.contains(&child_id));

        // Remove child
        scene.remove_node(child_id);

        // Verify child is gone
        assert!(scene.get_node(child_id).is_none());

        // Verify child is removed from parent's list
        assert!(!scene.get_node(root).unwrap().children.contains(&child_id));
    }

    #[test]
    fn test_remove_node_with_children_orphans_them() {
        let mut scene = Scene::new();
        let root = scene.root();

        // Create parent
        let parent_id = scene.add_node(
            root,
            SceneNode::new(NodeContent::Rect { color: Color::RED }),
        );

        // Create child
        let child_id = scene.add_node(
            parent_id,
            SceneNode::new(NodeContent::Rect { color: Color::BLUE }),
        );

        // Remove parent
        scene.remove_node(parent_id);

        // Parent is gone
        assert!(scene.get_node(parent_id).is_none());

        // Child still exists (orphaned)
        assert!(scene.get_node(child_id).is_some());

        // Child still points to deleted parent (as per current simplistic implementation)
        // This confirms the behavior described in doc comment
        assert_eq!(scene.get_node(child_id).unwrap().parent, Some(parent_id));
    }

    #[test]
    fn test_remove_root_node() {
        let mut scene = Scene::new();
        let root = scene.root();

        // Attempt to remove root
        scene.remove_node(root);

        // Root should be gone
        assert!(scene.get_node(root).is_none());

        // Scene state is technically invalid now (no root), but API allows it.
        // This test ensures no panic occurs.
    }
}
