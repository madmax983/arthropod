use layout_engine::FlexStyle;
use render_engine::NodeId;
use std::collections::HashMap;

/// Context for managing widget layout styles
pub struct LayoutContext {
    pub layout_styles: HashMap<NodeId, FlexStyle>,
}

impl LayoutContext {
    pub fn new() -> Self {
        Self {
            layout_styles: HashMap::new(),
        }
    }

    pub fn set_style(&mut self, node_id: NodeId, style: FlexStyle) {
        self.layout_styles.insert(node_id, style);
    }

    pub fn get_style(&self, node_id: NodeId) -> Option<FlexStyle> {
        self.layout_styles.get(&node_id).cloned()
    }

    pub fn get_all_styles(&self) -> &HashMap<NodeId, FlexStyle> {
        &self.layout_styles
    }
}
