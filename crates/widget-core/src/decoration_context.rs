use glam::Vec4;
use render_engine::NodeId;
use std::collections::HashMap;

/// Context for managing widget decorations (backgrounds, etc.)
pub struct DecorationContext {
    pub background_colors: HashMap<NodeId, Vec4>,
}

impl DecorationContext {
    pub fn new() -> Self {
        Self {
            background_colors: HashMap::new(),
        }
    }

    pub fn set_background_color(&mut self, node_id: NodeId, color: Vec4) {
        self.background_colors.insert(node_id, color);
    }

    pub fn get_background_color(&self, node_id: NodeId) -> Option<Vec4> {
        self.background_colors.get(&node_id).copied()
    }

    pub fn has_background_color(&self, node_id: NodeId) -> bool {
        self.background_colors.contains_key(&node_id)
    }

    pub fn get_background_colors(&self) -> &HashMap<NodeId, Vec4> {
        &self.background_colors
    }
}
