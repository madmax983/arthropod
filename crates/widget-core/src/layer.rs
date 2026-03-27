//! Scene Graph Layer Management
//!
//! Provides a z-ordering management system (`LayerManager`) for the render engine scene graph.
//! Layers ensure that UI elements like tooltips and dropdowns naturally render on top
//! of base content without needing manual z-index bookkeeping on every node.

use render_engine::{NodeContent, NodeId, Scene, SceneNode};
use std::collections::HashMap;

/// Named layers with guaranteed z-ordering in the scene tree.
/// Inspired by Flash/Flex's SystemManager display list architecture.
///
/// Layers are created as direct children of the scene root, in order.
/// The renderer traverses children in order, so layers naturally z-stack.
///
/// # Z-Order
///
/// | Layer | Z-Index | Purpose |
/// |-------|---------|---------|
/// | `Content` | 0 | Normal widget tree (default) |
/// | `Dropdown` | 1 | Select, Menu, Autocomplete popups |
/// | `Dialog` | 2 | Modal, Dialog, Drawer overlay |
/// | `Notification` | 3 | Snackbar, Toast |
/// | `Tooltip` | 4 | Tooltips (always on top of UI content) |
/// | `Cursor` | 5 | Drag previews, custom cursors |
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Layer {
    /// z=0 — Normal widget tree (default)
    Content,
    /// z=1 — Select, Menu, Autocomplete popups
    Dropdown,
    /// z=2 — Modal, Dialog, Drawer overlay
    Dialog,
    /// z=3 — Snackbar, Toast
    Notification,
    /// z=4 — Tooltips (always on top of UI content)
    Tooltip,
    /// z=5 — Drag previews, custom cursors
    Cursor,
}

impl Layer {
    /// All layers in z-order (lowest to highest).
    pub const ALL: [Layer; 6] = [
        Layer::Content,
        Layer::Dropdown,
        Layer::Dialog,
        Layer::Notification,
        Layer::Tooltip,
        Layer::Cursor,
    ];
}

/// Manages the mapping from [`Layer`] enum to scene [`NodeId`]s.
/// Created once during `WidgetContext` initialization.
///
/// Each layer is a direct child of the scene root. Because the renderer
/// traverses children in insertion order, layers added later render on top.
///
/// # Example
///
/// ```no_run
/// use render_engine::Scene;
/// use widget_core::layer::{Layer, LayerManager};
///
/// let mut scene = Scene::new();
/// let layers = LayerManager::new(&mut scene);
///
/// // Get the root node for the Content layer
/// let content_root = layers.get(Layer::Content);
/// ```
pub struct LayerManager {
    layers: HashMap<Layer, NodeId>,
}

impl LayerManager {
    /// Initialize all layer nodes as children of the scene root.
    /// Must be called once during `WidgetContext` construction.
    ///
    /// Creates one `NodeContent::Empty` child per layer variant, in z-order.
    pub fn new(scene: &mut Scene) -> Self {
        let root = scene.root();
        let mut layers = HashMap::new();

        for layer in Layer::ALL {
            let node = SceneNode::new(NodeContent::Empty);
            let node_id = scene.add_node(root, node);
            layers.insert(layer, node_id);
        }

        Self { layers }
    }

    /// Get the root [`NodeId`] for a given layer.
    ///
    /// # Panics
    ///
    /// Panics if the layer doesn't exist. This should never happen after
    /// initialization since all variants are created in [`LayerManager::new`].
    pub fn get(&self, layer: Layer) -> NodeId {
        self.layers[&layer]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layer_manager_creates_all_layers() {
        let mut scene = Scene::new();
        let manager = LayerManager::new(&mut scene);

        // Every layer variant should have a valid NodeId
        for layer in Layer::ALL {
            let node_id = manager.get(layer);
            assert!(
                scene.get_node(node_id).is_some(),
                "Layer {layer:?} should have a valid scene node"
            );
        }
    }

    #[test]
    fn test_layer_nodes_are_children_of_root() {
        let mut scene = Scene::new();
        let manager = LayerManager::new(&mut scene);

        let root = scene.root();
        let root_node = scene.get_node(root).unwrap();

        for layer in Layer::ALL {
            let node_id = manager.get(layer);
            assert!(
                root_node.children.contains(&node_id),
                "Layer {layer:?} node should be a child of the scene root"
            );
        }
    }

    #[test]
    fn test_layer_z_order_matches_insertion_order() {
        let mut scene = Scene::new();
        let manager = LayerManager::new(&mut scene);

        let root = scene.root();
        let root_node = scene.get_node(root).unwrap();
        let children = &root_node.children;

        // Layers should appear in ALL order (Content first, Cursor last)
        for (i, layer) in Layer::ALL.iter().enumerate() {
            assert_eq!(
                children[i],
                manager.get(*layer),
                "Layer {layer:?} should be at index {i} in root's children"
            );
        }
    }

    #[test]
    fn test_all_layer_nodes_are_empty() {
        let mut scene = Scene::new();
        let manager = LayerManager::new(&mut scene);

        for layer in Layer::ALL {
            let node_id = manager.get(layer);
            let node = scene.get_node(node_id).unwrap();
            assert!(
                matches!(node.content, NodeContent::Empty),
                "Layer {layer:?} node should have NodeContent::Empty"
            );
        }
    }

    #[test]
    fn test_each_layer_has_unique_node_id() {
        let mut scene = Scene::new();
        let manager = LayerManager::new(&mut scene);

        let ids: Vec<NodeId> = Layer::ALL.iter().map(|l| manager.get(*l)).collect();
        for (i, id) in ids.iter().enumerate() {
            for (j, other) in ids.iter().enumerate() {
                if i != j {
                    assert_ne!(id, other, "Layer node IDs must be unique");
                }
            }
        }
    }

    #[test]
    fn test_layer_all_has_correct_count() {
        assert_eq!(Layer::ALL.len(), 6);
    }

    #[test]
    fn test_layer_equality() {
        assert_eq!(Layer::Content, Layer::Content);
        assert_ne!(Layer::Content, Layer::Tooltip);
    }
}
