import sys

def process(content):
    search = """pub(crate) fn collect_style_batches_for_bounds(
    ctx: &mut BatchCollectionContext,
    style: &style_engine::VisualStyle,
    effective_opacity: f32,
    render_bounds: plat_core::Rect,
    node_transform: Transform2D,
) -> (Vec<PrimitiveInstance>, Vec<PathBatch>) {
    let mut instances = Vec::new();
    let mut path_batches = Vec::new();"""

    replace = """pub(crate) fn collect_style_batches_for_bounds(
    ctx: &mut BatchCollectionContext,
    style: &style_engine::VisualStyle,
    effective_opacity: f32,
    render_bounds: plat_core::Rect,
    node_transform: Transform2D,
    instances: &mut Vec<PrimitiveInstance>,
    path_batches: &mut Vec<PathBatch>,
) {
    instances.clear();
    path_batches.clear();"""

    if search not in content:
        print("Search string not found")
        sys.exit(1)

    content = content.replace(search, replace)

    search_end = """    }

    apply_node_transform_to_instances(&mut instances, node_transform);

    (instances, path_batches)
}"""

    replace_end = """    }

    apply_node_transform_to_instances(instances, node_transform);
}"""

    if search_end not in content:
        # try another variant if the indentations are different
        search_end = """    apply_node_transform_to_instances(&mut instances, node_transform);

    (instances, path_batches)
}"""
        replace_end = """    apply_node_transform_to_instances(instances, node_transform);
}"""

    content = content.replace(search_end, replace_end)
    return content

with open("crates/render-engine/src/backend/wgpu/instance_collector.rs", "r") as f:
    content = f.read()

new_content = process(content)

with open("crates/render-engine/src/backend/wgpu/instance_collector.rs", "w") as f:
    f.write(new_content)
