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
use std::cell::RefCell;

thread_local! {
    /// Reusable stack for hit_test to avoid allocations.
    /// Stores (NodeId, next_child_index_to_visit).
    static HIT_TEST_STACK: RefCell<Vec<(NodeId, usize)>> = RefCell::new(Vec::with_capacity(64));
}

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
    /// use style_engine::VisualStyle;
    ///
    /// let mut scene = Scene::new();
    /// let root = scene.root();
    ///
    /// let child = scene.add_node(
    ///     root,
    ///     SceneNode::new(NodeContent::Styled { style: Box::new(VisualStyle::new().solid_fill(Color::RED.as_vec4())) })
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
        // Prevent reparenting the root
        if child_id == self.root {
            return;
        }

        // Prevent immediate cycle
        if child_id == new_parent {
            return;
        }

        // Prevent cycle: check if child_id is an ancestor of new_parent
        let mut ancestor = new_parent;
        loop {
            if ancestor == child_id {
                // Cycle detected: child is an ancestor of new_parent!
                // Abort reparenting.
                return;
            }
            if let Some(p) = self.parent(ancestor) {
                ancestor = p;
            } else {
                // Reached a root (or orphan), no cycle detected in this path
                break;
            }
        }

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
    ///
    /// # ⚠️ Memory Leak Warning
    ///
    /// This method does **not** recursively remove children. If the removed node
    /// has children, they will become "orphaned" (they remain in the internal
    /// map but are no longer reachable via traversal). This causes a memory leak.
    ///
    /// **Recommendation:** If you need to remove a subtree, you must manually
    /// collect and remove all descendants first, or use a helper that does so.
    ///
    /// # Example
    ///
    /// ```
    /// # use render_engine::{Scene, SceneNode, NodeContent, Color};
    /// # use style_engine::VisualStyle;
    /// # let mut scene = Scene::new();
    /// # let root = scene.root();
    /// let node = scene.add_node(root, SceneNode::new(NodeContent::Styled { style: Box::new(VisualStyle::new().solid_fill(Color::RED.as_vec4())) }));
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

    /// Iterate over visual nodes in depth-first (Painter's Algorithm) order.
    ///
    /// This iterator traverses the scene graph starting from the root, yielding
    /// nodes in the order they should be drawn:
    /// 1. Parent
    /// 2. Children (first to last)
    ///
    /// This ensures correct Z-ordering for 2D rendering.
    pub fn iter_visuals(&self) -> VisualIterator<'_> {
        VisualIterator {
            scene: self,
            stack: vec![self.root],
        }
    }

    /// Find a visible node at the given screen position.
    ///
    /// This method performs an iterative tree traversal starting from the root,
    /// checking children in **reverse order** (top-most first). This guarantees
    /// that if multiple nodes overlap at the given point, the one that was
    /// added last (and thus rendered on top) is returned.
    ///
    /// # Performance
    ///
    /// This is an O(log N) operation for typical balanced trees, but can be
    /// O(N) in the worst case (deeply nested hierarchies). It is significantly
    /// faster than a global linear scan and correctly handles Z-ordering.
    ///
    /// # Safety
    ///
    /// This method uses an iterative approach with a heap-allocated stack to
    /// prevent stack overflows on deeply nested scene graphs (e.g. >10k nodes deep).
    ///
    /// # Z-Order Guarantee
    ///
    /// The Z-order is determined by the order of children in the parent's list.
    /// `add_node` appends to this list. `hit_test` checks this list in reverse.
    /// Therefore, the last added sibling is the "top" node.
    ///
    /// # Example
    ///
    /// ```
    /// use render_engine::{Scene, SceneNode, NodeContent, Color};
    /// use style_engine::VisualStyle;
    /// use plat_core::Rect;
    ///
    /// let mut scene = Scene::new();
    /// let root = scene.root();
    ///
    /// // Bottom node (added first)
    /// let mut node1 = SceneNode::new(NodeContent::Styled { style: Box::new(VisualStyle::new().solid_fill(Color::RED.as_vec4())) });
    /// node1.bounds = Rect::new(0.0, 0.0, 100.0, 100.0);
    /// let id1 = scene.add_node(root, node1);
    ///
    /// // Top node (added second)
    /// let mut node2 = SceneNode::new(NodeContent::Styled { style: Box::new(VisualStyle::new().solid_fill(Color::BLUE.as_vec4())) });
    /// node2.bounds = Rect::new(50.0, 50.0, 100.0, 100.0);
    /// let id2 = scene.add_node(root, node2);
    ///
    /// // Hit test in overlap region returns the top node
    /// assert_eq!(scene.hit_test(75.0, 75.0), Some(id2));
    /// ```
    pub fn hit_test(&self, x: f32, y: f32) -> Option<NodeId> {
        // Use a thread-local stack for iterative traversal to avoid recursion limits
        // and allocations.
        HIT_TEST_STACK.with(|stack_cell| {
            let mut stack = stack_cell.borrow_mut();
            stack.clear();

            // Start with root
            if let Some(root_node) = self.nodes.get(&self.root) {
                stack.push((self.root, root_node.children.len()));
            } else {
                return None;
            }

            while !stack.is_empty() {
                // Peek at the current node state
                let (node_id, child_index) = *stack.last().unwrap();

                let node = match self.nodes.get(&node_id) {
                    Some(n) => n,
                    None => {
                        // Should not happen in a valid scene, but handle safely
                        stack.pop();
                        continue;
                    }
                };

                if !node.visible {
                    stack.pop();
                    continue;
                }

                if child_index > 0 {
                    // Visit next child. We iterate in reverse order (back to front).
                    let next_child_idx = child_index - 1;

                    // Update the state on the stack to mark this child as visited
                    stack.last_mut().unwrap().1 = next_child_idx;

                    let child_id = node.children[next_child_idx];

                    // Push child to stack
                    if let Some(child_node) = self.nodes.get(&child_id) {
                        stack.push((child_id, child_node.children.len()));
                    }
                } else {
                    // All children visited (and none returned a hit).
                    // Check self.
                    stack.pop();

                    if node.bounds.contains(x, y) {
                        return Some(node_id);
                    }
                }
            }

            None
        })
    }

    /// Serialize the scene to JSON for debugging
    ///
    /// Useful for inspecting the scene structure via MCP or logging.
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

/// Iterator for visual nodes in depth-first (Painter's Algorithm) order.
pub struct VisualIterator<'a> {
    scene: &'a Scene,
    stack: Vec<NodeId>,
}

impl<'a> Iterator for VisualIterator<'a> {
    type Item = (NodeId, &'a SceneNode);

    fn next(&mut self) -> Option<Self::Item> {
        let id = self.stack.pop()?;
        let node = self.scene.get_node(id)?;

        // Push children in reverse order so they are processed in forward order
        // (stack is LIFO, so pushing [1, 2] means popping 2 then 1, visiting 1 then 2)
        for &child_id in node.children.iter().rev() {
            self.stack.push(child_id);
        }

        Some((id, node))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Color, NodeContent, VisualStyle};
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

        let mut node = SceneNode::new(NodeContent::Styled {
            style: Box::new(VisualStyle::new().solid_fill(Color::RED.as_vec4())),
        });
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

        let mut node = SceneNode::new(NodeContent::Styled {
            style: Box::new(VisualStyle::new().solid_fill(Color::RED.as_vec4())),
        });
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
        let mut node1 = SceneNode::new(NodeContent::Styled {
            style: Box::new(VisualStyle::new().solid_fill(Color::RED.as_vec4())),
        });
        node1.bounds = Rect::new(0.0, 0.0, 100.0, 100.0);
        let id1 = scene.add_node(root, node1);

        let mut node2 = SceneNode::new(NodeContent::Styled {
            style: Box::new(VisualStyle::new().solid_fill(Color::BLUE.as_vec4())),
        });
        node2.bounds = Rect::new(50.0, 50.0, 100.0, 100.0);
        let id2 = scene.add_node(root, node2);

        // In overlap region, hit_test should return the top-most node (last added)
        // This is guaranteed by the reverse iteration order in hit_test.
        let hit = scene.hit_test(75.0, 75.0);
        assert_eq!(hit, Some(id2), "Should hit the top-most node (id2)");

        // Only in first node
        assert_eq!(scene.hit_test(25.0, 25.0), Some(id1));

        // Only in second node
        assert_eq!(scene.hit_test(125.0, 125.0), Some(id2));
    }

    #[test]
    fn test_hit_test_ignores_invisible_nodes() {
        let mut scene = Scene::new();
        let root = scene.root();

        let mut node = SceneNode::new(NodeContent::Styled {
            style: Box::new(VisualStyle::new().solid_fill(Color::RED.as_vec4())),
        });
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
        let mut parent = SceneNode::new(NodeContent::Styled {
            style: Box::new(VisualStyle::new().solid_fill(Color::WHITE.as_vec4())),
        });
        parent.bounds = Rect::new(0.0, 0.0, 200.0, 200.0);
        let parent_id = scene.add_node(root, parent);

        // Child inside parent
        let mut child = SceneNode::new(NodeContent::Styled {
            style: Box::new(VisualStyle::new().solid_fill(Color::RED.as_vec4())),
        });
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

        let mut node = SceneNode::new(NodeContent::Styled {
            style: Box::new(VisualStyle::new().solid_fill(Color::RED.as_vec4())),
        });
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
        let mut child = SceneNode::new(NodeContent::Styled {
            style: Box::new(VisualStyle::new().solid_fill(Color::RED.as_vec4())),
        });
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
            SceneNode::new(NodeContent::Styled {
                style: Box::new(VisualStyle::new().solid_fill(Color::RED.as_vec4())),
            }),
        );

        // Create child
        let child_id = scene.add_node(
            parent_id,
            SceneNode::new(NodeContent::Styled {
                style: Box::new(VisualStyle::new().solid_fill(Color::BLUE.as_vec4())),
            }),
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

    #[test]
    fn test_hit_test_respects_z_order() {
        let mut scene = Scene::new();
        let root = scene.root();

        // Add two overlapping nodes
        let mut node1 = SceneNode::new(NodeContent::Styled {
            style: Box::new(VisualStyle::new().solid_fill(Color::RED.as_vec4())),
        });
        node1.bounds = Rect::new(0.0, 0.0, 100.0, 100.0);
        let id1 = scene.add_node(root, node1);

        let mut node2 = SceneNode::new(NodeContent::Styled {
            style: Box::new(VisualStyle::new().solid_fill(Color::BLUE.as_vec4())),
        });
        node2.bounds = Rect::new(50.0, 50.0, 100.0, 100.0);
        let id2 = scene.add_node(root, node2);

        // Add a third node that is smaller and inside both
        let mut node3 = SceneNode::new(NodeContent::Styled {
            style: Box::new(VisualStyle::new().solid_fill(Color::GREEN.as_vec4())),
        });
        node3.bounds = Rect::new(60.0, 60.0, 10.0, 10.0);
        let id3 = scene.add_node(root, node3);

        // id3 is added last, so it should be on top of id2, which is on top of id1
        assert_eq!(
            scene.hit_test(65.0, 65.0),
            Some(id3),
            "Should hit the top-most node (id3)"
        );

        // At 55,55 (overlap of id1 and id2), id2 should be on top
        assert_eq!(
            scene.hit_test(55.0, 55.0),
            Some(id2),
            "Should hit the middle node (id2) over bottom node (id1)"
        );

        // At 10,10 (only id1), id1 should be hit
        assert_eq!(scene.hit_test(10.0, 10.0), Some(id1));
    }

    #[test]
    fn test_iter_visuals_order() {
        let mut scene = Scene::new();
        let root = scene.root();

        // Tree structure:
        // Root
        //  |-> Child1
        //       |-> Grandchild1
        //  |-> Child2

        let child1 = scene.add_node(root, SceneNode::new(NodeContent::Empty));
        let grandchild1 = scene.add_node(child1, SceneNode::new(NodeContent::Empty));
        let child2 = scene.add_node(root, SceneNode::new(NodeContent::Empty));

        let traversal: Vec<NodeId> = scene.iter_visuals().map(|(id, _)| id).collect();

        // Expected order: Root, Child1, Grandchild1, Child2
        assert_eq!(traversal, vec![root, child1, grandchild1, child2]);
    }
}
