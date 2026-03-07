//! Magneto: Magnetic constraints system for nodes.
//!
//! Adds `MagneticNode` to link nodes using spring physics.

use arthropod_ecs::components::SceneNodeRef;
use bevy_ecs::prelude::*;
use render_engine::{NodeId, Scene};

#[derive(Component, Debug, Clone)]
pub struct MagneticNode {
    pub target_id: NodeId,
    pub target_distance: f32,
    pub stiffness: f32,
    pub damping: f32,
}

pub fn update_magnetic_links(
    mut scene: ResMut<Scene>,
    query: Query<(&MagneticNode, &SceneNodeRef)>,
) {
    // 1. Collect updates to avoid borrowing scene mutably while reading
    let mut updates = Vec::new();

    for (magnetic, node_ref) in query.iter() {
        let node1_id = magnetic.target_id;
        let node2_id = node_ref.0;

        if let (Some(node1), Some(node2)) = (scene.get_node(node1_id), scene.get_node(node2_id)) {
            let pos1 = node1.transform.translation();
            let pos2 = node2.transform.translation();

            let dist = pos1.distance(pos2);
            if dist > magnetic.target_distance {
                // Simplified Hooke's Law for spring force
                // F = -k * x
                let displacement = dist - magnetic.target_distance;
                let force = magnetic.stiffness * displacement;

                // Direction from node2 to node1
                let dir = (pos1 - pos2).normalize_or_zero();

                // Apply a simple position update based on force
                // We assume dt=1/60 for simplicity here since it's just a position update frame-by-frame
                // For a real physics system we would track velocity, but here we just interpolate position
                // based on damping and stiffness for visual effect.

                // Simplified integration:
                // Just move node2 closer to node1 by a fraction
                // This acts like a critically damped spring that just closes the gap smoothly
                let pull = dir * force * 0.016; // arbitrary dt

                // In reality we should also consider mass and damping, but for an experimental visual
                // effect, simple proportional pulling works.
                let new_pos = pos2 + pull;

                updates.push((node2_id, new_pos));
            }
        }
    }

    // 2. Apply updates
    for (node_id, new_pos) in updates {
        if let Some(node) = scene.get_node_mut(node_id) {
            node.transform = render_engine::Transform2D::translate(new_pos.x, new_pos.y);
        }
    }
}

pub fn register_magneto(app: &mut crate::App) {
    app.add_update_system(update_magnetic_links);
}

#[cfg(test)]
mod tests {
    use super::*;
    use render_engine::{Color, NodeContent, Rect, SceneNode, Transform2D, Vec2};

    #[test]
    fn test_magnetic_pull() {
        let mut scene = Scene::new();

        // Node 1
        let node1 = scene.add_node(
            scene.root(),
            SceneNode {
                content: NodeContent::Empty,
                transform: Transform2D::translate(0.0, 0.0),
                bounds: Rect::default(),
                children: vec![],
                parent: None,
                visible: true,
                opacity: 1.0,
            },
        );

        // Node 2 (Should be pulled to Node 1)
        let node2 = scene.add_node(
            scene.root(),
            SceneNode {
                content: NodeContent::Empty,
                transform: Transform2D::translate(100.0, 0.0),
                bounds: Rect::default(),
                children: vec![],
                parent: None,
                visible: true,
                opacity: 1.0,
            },
        );

        let mut world = World::new();
        world.insert_resource(scene);

        world
            .spawn(MagneticNode {
                target_id: node1,
                target_distance: 50.0,
                stiffness: 100.0,
                damping: 10.0,
            })
            .insert(arthropod_ecs::components::SceneNodeRef(node2));

        let mut schedule = Schedule::default();
        schedule.add_systems(update_magnetic_links);

        // Run a few ticks
        for _ in 0..10 {
            schedule.run(&mut world);
        }

        let scene = world.resource::<Scene>();
        let pos1 = scene.get_node(node1).unwrap().transform.translation();
        let pos2 = scene.get_node(node2).unwrap().transform.translation();

        // They shouldn't be 100 apart anymore
        let dist = pos1.distance(pos2);
        assert!(
            dist < 99.0,
            "Nodes should have moved closer, dist is {}",
            dist
        );
    }
}
