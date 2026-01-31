use render_engine::NodeId;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

/// Context for managing widget interactions (hover, click)
pub struct InteractionContext {
    pub hover_states: HashSet<NodeId>,
    pub clickables: HashMap<NodeId, Arc<dyn Fn() + Send + Sync>>,
}

impl InteractionContext {
    pub fn new() -> Self {
        Self {
            hover_states: HashSet::new(),
            clickables: HashMap::new(),
        }
    }

    pub fn add_hover_state(&mut self, node_id: NodeId) {
        self.hover_states.insert(node_id);
    }

    pub fn has_hover_state(&self, node_id: NodeId) -> bool {
        self.hover_states.contains(&node_id)
    }

    pub fn trigger_hover(&mut self, node_id: NodeId, hovered: bool) {
        if hovered {
            self.hover_states.insert(node_id);
        } else {
            self.hover_states.remove(&node_id);
        }
    }

    pub fn add_clickable(&mut self, node_id: NodeId, callback: Arc<dyn Fn() + Send + Sync>) {
        self.clickables.insert(node_id, callback);
    }

    pub fn has_clickable(&self, node_id: NodeId) -> bool {
        self.clickables.contains_key(&node_id)
    }

    pub fn get_clickables(&self) -> &HashMap<NodeId, Arc<dyn Fn() + Send + Sync>> {
        &self.clickables
    }

    pub fn trigger_click(&mut self, node_id: NodeId) {
        if let Some(callback) = self.clickables.get(&node_id) {
            callback();
        }
    }
}
