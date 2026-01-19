//! Widget build context
//!
//! Provides API for widgets to build scene nodes and configure components.

use render_engine::{Scene, NodeId, NodeContent, SceneNode};
use layout_engine::{FlexStyle, FlexDirection};
use arthropod_ecs::FrameworkContext;
use std::collections::HashMap;

/// Widget building context
///
/// Provides access to:
/// - Scene graph for hierarchy
/// - ECS context for components
/// - Layout engine for flexbox
/// - Text engine for text shaping
pub struct WidgetContext {
    scene: Scene,
    #[allow(dead_code)]
    ecs_context: Option<FrameworkContext>,
    layout_styles: HashMap<NodeId, FlexStyle>,
}

impl WidgetContext {
    /// Create a new widget context (for testing)
    pub fn new_test() -> Self {
        Self {
            scene: Scene::new(),
            ecs_context: None,
            layout_styles: HashMap::new(),
        }
    }

    /// Get the scene
    pub fn scene(&self) -> &Scene {
        &self.scene
    }

    /// Get mutable scene
    pub fn scene_mut(&mut self) -> &mut Scene {
        &mut self.scene
    }

    /// Create a new scene node
    pub fn create_node(&mut self, parent: NodeId, content: NodeContent) -> NodeId {
        let node = SceneNode::new(content);
        self.scene.add_node(parent, node)
    }

    /// Get the root node ID
    pub fn root(&self) -> NodeId {
        self.scene.root()
    }

    /// Check if node has layout component
    pub fn has_layout_node(&self, _node_id: NodeId) -> bool {
        // TODO: Query ECS for LayoutNode component
        true // For now, assume all nodes have layout
    }

    /// Get layout style for node
    pub fn get_layout_style(&self, node_id: NodeId) -> Option<FlexStyle> {
        self.layout_styles.get(&node_id).cloned()
    }

    /// Check if node is a text node
    pub fn is_text_node(&self, node_id: NodeId) -> bool {
        if let Some(node) = self.scene.get_node(node_id) {
            matches!(node.content, NodeContent::Text { .. })
        } else {
            false
        }
    }

    /// Check if node has reactive text component
    pub fn has_reactive_text(&self, _node_id: NodeId) -> bool {
        // TODO: Query ECS for ReactiveText component
        true // For now, assume reactive nodes exist
    }

    /// Set layout style for a node
    pub fn set_layout_style(&mut self, node_id: NodeId, style: FlexStyle) {
        self.layout_styles.insert(node_id, style);
        // TODO: Add LayoutNode component to ECS entity
    }
}

/// Helper to check if a FlexStyle is a row layout
pub fn is_row_layout(style: &FlexStyle) -> bool {
    style.direction == FlexDirection::Row
}
