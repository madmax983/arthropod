//! Scene graph data structures.

use crate::SceneNode;
use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Unique identifier for scene nodes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeId(pub u64);

/// The scene graph - owns all nodes.
///
/// Scene can now be stored as an ECS Resource, eliminating the need for
/// unsafe pointer juggling. Systems access Scene via Res<Scene> and ResMut<Scene>.
#[derive(Resource)]
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

    /// Get a node by ID (alias for get_node, for ECS compatibility).
    pub fn get(&self, id: NodeId) -> Option<&SceneNode> {
        self.get_node(id)
    }

    /// Get a mutable node by ID (alias for get_node_mut, for ECS compatibility).
    pub fn get_mut(&mut self, id: NodeId) -> Option<&mut SceneNode> {
        self.get_node_mut(id)
    }

    /// Re-parent a node from old parent to new parent.
    pub fn reparent_node(&mut self, child_id: NodeId, old_parent: NodeId, new_parent: NodeId) {
        // Remove child from old parent
        if let Some(old_parent_node) = self.nodes.get_mut(&old_parent) {
            old_parent_node.children.retain(|&id| id != child_id);
        }

        // Add child to new parent
        if let Some(new_parent_node) = self.nodes.get_mut(&new_parent) {
            new_parent_node.children.push(child_id);
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

    /// Get dirty nodes and clear the list.
    pub fn take_dirty(&mut self) -> Vec<NodeId> {
        std::mem::take(&mut self.dirty_nodes)
    }

    /// Iterate over all nodes in the scene.
    pub fn nodes(&self) -> impl Iterator<Item = (NodeId, &SceneNode)> {
        self.nodes.iter().map(|(id, node)| (*id, node))
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
