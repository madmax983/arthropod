import sys

with open("crates/render-engine/src/backend/wgpu/multipass_executor.rs", "r") as f:
    content = f.read()

search = """                    &node_instances,
                    &node_path_batches,"""

replace = """                    &instances_buffer,
                    &path_batches_buffer,"""

content = content.replace(search, replace)

with open("crates/render-engine/src/backend/wgpu/multipass_executor.rs", "w") as f:
    f.write(content)


with open("crates/render-engine/src/backend/wgpu/instance_collector.rs", "r") as f:
    content = f.read()

search2 = """            &mut interner,
            &mut path_batches,"""

replace2 = """            &mut interner,
            path_batches,"""
content = content.replace(search2, replace2)

search3 = """            effective_opacity,
            &mut cache,
            &mut path_batches,"""

replace3 = """            effective_opacity,
            &mut cache,
            path_batches,"""
content = content.replace(search3, replace3)


with open("crates/render-engine/src/backend/wgpu/instance_collector.rs", "w") as f:
    f.write(content)
