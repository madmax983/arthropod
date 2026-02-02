use super::core::App;
use arthropod_ecs::Renderable;
use render_engine::{NodeId, Scene, SceneNode};
use std::collections::HashMap;
use widget_core::WidgetContext;

/// Integrate widget scene into app scene.
pub fn integrate_widget_scene(
    app: &mut App,
    widget_ctx: &WidgetContext,
    widget_root: NodeId,
) -> (NodeId, HashMap<NodeId, NodeId>) {
    let widget_scene = widget_ctx.scene();
    let mut node_id_map = HashMap::new();

    fn copy_recursive(
        src: &Scene,
        dst: &mut Scene,
        src_id: NodeId,
        dst_parent: NodeId,
        map: &mut HashMap<NodeId, NodeId>,
    ) -> NodeId {
        let src_node = src.get_node(src_id).unwrap();

        let mut new_node = SceneNode::new(src_node.content.clone());
        new_node.bounds = src_node.bounds;
        new_node.visible = src_node.visible;
        new_node.opacity = src_node.opacity;

        let dst_id = dst.add_node(dst_parent, new_node);
        map.insert(src_id, dst_id);

        let children = src_node.children.clone();
        for child_id in children {
            copy_recursive(src, dst, child_id, dst_id, map);
        }

        dst_id
    }

    let app_root = {
        let mut scene = app.world_mut().resource_mut::<Scene>();
        let root = scene.root();
        copy_recursive(
            widget_scene,
            &mut scene,
            widget_root,
            root,
            &mut node_id_map,
        )
    };

    // Spawn Renderable entities
    for &app_node in node_id_map.values() {
        app.spawn(app_node).insert(Renderable);
    }

    (app_root, node_id_map)
}
