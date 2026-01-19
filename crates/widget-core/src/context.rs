//! Widget build context
//!
//! Provides API for widgets to build scene nodes and configure components.

use render_engine::{Scene, NodeId, NodeContent, SceneNode};
use layout_engine::{FlexStyle, FlexDirection};
use arthropod_ecs::FrameworkContext;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use glam::Vec4;

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
    hover_states: HashSet<NodeId>,
    clickables: HashMap<NodeId, Arc<dyn Fn() + Send + Sync>>,
    background_colors: HashMap<NodeId, Vec4>,
}

impl WidgetContext {
    /// Create a new widget context (for testing)
    pub fn new_test() -> Self {
        Self {
            scene: Scene::new(),
            ecs_context: None,
            layout_styles: HashMap::new(),
            hover_states: HashSet::new(),
            clickables: HashMap::new(),
            background_colors: HashMap::new(),
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

    /// Re-parent a node from old parent to new parent
    pub fn reparent_node(&mut self, child_id: NodeId, old_parent: NodeId, new_parent: NodeId) {
        self.scene.reparent_node(child_id, old_parent, new_parent);
    }

    /// Add hover state tracking to a node
    pub fn add_hover_state(&mut self, node_id: NodeId) {
        self.hover_states.insert(node_id);
    }

    /// Check if node has hover state
    pub fn has_hover_state(&self, node_id: NodeId) -> bool {
        self.hover_states.contains(&node_id)
    }

    /// Add clickable component to a node
    pub fn add_clickable(&mut self, node_id: NodeId, callback: Arc<dyn Fn() + Send + Sync>) {
        self.clickables.insert(node_id, callback);
    }

    /// Check if node is clickable
    pub fn has_clickable(&self, node_id: NodeId) -> bool {
        self.clickables.contains_key(&node_id)
    }

    /// Set background color for a node
    pub fn set_background_color(&mut self, node_id: NodeId, color: Vec4) {
        self.background_colors.insert(node_id, color);
    }

    /// Check if node has background color
    pub fn has_background_color(&self, node_id: NodeId) -> bool {
        self.background_colors.contains_key(&node_id)
    }

    /// Get background color for a node
    pub fn get_background_color(&self, node_id: NodeId) -> Option<Vec4> {
        self.background_colors.get(&node_id).copied()
    }

    /// Simulate click on a node (for testing)
    pub fn trigger_click(&mut self, node_id: NodeId) {
        if let Some(callback) = self.clickables.get(&node_id) {
            callback();
        }
    }

    /// Simulate hover on a node (for testing)
    pub fn trigger_hover(&mut self, node_id: NodeId, hovered: bool) {
        if hovered {
            self.hover_states.insert(node_id);
        } else {
            self.hover_states.remove(&node_id);
        }
    }
}

/// Helper to check if a FlexStyle is a row layout
pub fn is_row_layout(style: &FlexStyle) -> bool {
    style.direction == FlexDirection::Row
}
