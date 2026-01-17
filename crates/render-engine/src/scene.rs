//! Scene graph data structures.

use crate::SceneNode;
use std::collections::HashMap;

/// Unique identifier for scene nodes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(pub u64);

/// The scene graph - owns all nodes.
pub struct Scene {
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
    pub fn add_node(&mut self, parent: NodeId, node: SceneNode) -> NodeId {
        let id = NodeId(self.next_id);
        self.next_id += 1;

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

    /// Mark a node as needing redraw.
    pub fn mark_dirty(&mut self, id: NodeId) {
        if !self.dirty_nodes.contains(&id) {
            self.dirty_nodes.push(id);
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
}
