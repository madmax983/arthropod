use bevy_ecs::prelude::*;
use plat_core::Rect;
use render_engine::{NodeId, Scene};
use std::collections::HashMap;

/// A spatial index for fast 2D querying of scene nodes.
///
/// This provides a supplementary index to the `Scene` to quickly find nodes
/// by area or point without traversing the entire scene graph.
#[derive(Resource, Default)]
pub struct SpatialIndex {
    // For now, a simple flat list/map. Could be upgraded to an R-Tree later.
    nodes: HashMap<NodeId, Rect>,
}

impl SpatialIndex {
    /// Create a new empty spatial index.
    pub fn new() -> Self {
        Self::default()
    }

    /// Insert or update a node's bounds in the index.
    pub fn insert(&mut self, node_id: NodeId, bounds: Rect) {
        self.nodes.insert(node_id, bounds);
    }

    /// Remove a node from the index.
    pub fn remove(&mut self, node_id: NodeId) {
        self.nodes.remove(&node_id);
    }

    /// Query all nodes that contain the given point.
    pub fn query_point(&self, x: f32, y: f32) -> Vec<NodeId> {
        self.nodes
            .iter()
            .filter_map(|(id, bounds)| {
                if bounds.contains(x, y) {
                    Some(*id)
                } else {
                    None
                }
            })
            .collect()
    }

    /// Query all nodes that intersect with the given rectangle.
    pub fn query_rect(&self, rect: Rect) -> Vec<NodeId> {
        self.nodes
            .iter()
            .filter_map(|(id, bounds)| {
                // Intersect rects logic
                if rect.x < bounds.x + bounds.width
                    && rect.x + rect.width > bounds.x
                    && rect.y < bounds.y + bounds.height
                    && rect.y + rect.height > bounds.y
                {
                    Some(*id)
                } else {
                    None
                }
            })
            .collect()
    }
}

/// System to keep the SpatialIndex synchronized with the Scene.
///
/// This should be added to the update loop.
pub fn update_spatial_index(scene: Res<Scene>, mut spatial_index: ResMut<SpatialIndex>) {
    // Brute force sync for now. Future optimization: use scene.dirty_nodes
    spatial_index.nodes.clear();
    for (id, node) in scene.nodes() {
        spatial_index.insert(id, node.bounds);
    }
}

/// Register the spatial query feature.
pub fn register_spatial_query(app: &mut crate::App) {
    app.world_mut().insert_resource(SpatialIndex::default());
    app.add_update_system(update_spatial_index);
}

#[cfg(test)]
mod tests {
    use super::*;
    use render_engine::{Color, NodeContent, SceneNode};

    #[test]
    fn test_spatial_index_insert_query_point() {
        let mut index = SpatialIndex::new();

        // Node 1: Contains point (50, 50)
        let id1 = NodeId(1);
        index.insert(id1, Rect::new(0.0, 0.0, 100.0, 100.0));

        // Node 2: Contains point (150, 150)
        let id2 = NodeId(2);
        index.insert(id2, Rect::new(100.0, 100.0, 100.0, 100.0));

        let hit1 = index.query_point(50.0, 50.0);
        assert_eq!(hit1.len(), 1);
        assert_eq!(hit1[0], id1);

        let hit2 = index.query_point(150.0, 150.0);
        assert_eq!(hit2.len(), 1);
        assert_eq!(hit2[0], id2);

        // 100, 100 is exclusive for Node 1 depending on Rect::contains implementation,
        // let's check it at 99.9, 99.9 or change test to check overlap area
        index.insert(id2, Rect::new(50.0, 50.0, 100.0, 100.0));
        let hit_both = index.query_point(75.0, 75.0);
        assert_eq!(hit_both.len(), 2);
    }

    #[test]
    fn test_spatial_index_query_rect() {
        let mut index = SpatialIndex::new();

        let id1 = NodeId(1);
        index.insert(id1, Rect::new(0.0, 0.0, 10.0, 10.0));

        let id2 = NodeId(2);
        index.insert(id2, Rect::new(20.0, 20.0, 10.0, 10.0));

        // Intersects id1
        let rect1 = Rect::new(-5.0, -5.0, 10.0, 10.0);
        let hits = index.query_rect(rect1);
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0], id1);

        // Intersects both
        let rect_both = Rect::new(5.0, 5.0, 20.0, 20.0);
        let hits_both = index.query_rect(rect_both);
        assert_eq!(hits_both.len(), 2);
    }

    #[test]
    fn test_update_spatial_index() {
        let mut world = World::new();
        let mut scene = Scene::new();

        let root = scene.root();
        let mut node = SceneNode::new(NodeContent::SolidColor { color: Color::RED });
        node.bounds = Rect::new(0.0, 0.0, 50.0, 50.0);
        let node_id = scene.add_node(root, node);

        world.insert_resource(scene);
        world.insert_resource(SpatialIndex::new());

        let mut schedule = Schedule::default();
        schedule.add_systems(update_spatial_index);

        schedule.run(&mut world);

        let index = world.resource::<SpatialIndex>();
        let hits = index.query_point(25.0, 25.0);
        assert!(hits.contains(&node_id));
    }
}
