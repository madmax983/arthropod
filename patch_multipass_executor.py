import sys

with open("crates/render-engine/src/backend/wgpu/multipass_executor.rs", "r") as f:
    content = f.read()

search1 = """    pub(crate) fn render_multipass_effect_nodes(
        &mut self,
        scene: &Scene,
        multipass_node_ids: &[crate::NodeId],
        surface_texture: &wgpu::Texture,
        surface_view: &wgpu::TextureView,
    ) {
        let frame_key =
            RenderTargetKey::new(self.context.config.width, self.context.config.height, false);"""

replace1 = """    pub(crate) fn render_multipass_effect_nodes(
        &mut self,
        scene: &Scene,
        multipass_node_ids: &[crate::NodeId],
        surface_texture: &wgpu::Texture,
        surface_view: &wgpu::TextureView,
    ) {
        let frame_key =
            RenderTargetKey::new(self.context.config.width, self.context.config.height, false);

        let mut instances_buffer = Vec::new();
        let mut path_batches_buffer = Vec::new();"""

content = content.replace(search1, replace1)


search2 = """            let mut batch_ctx = BatchCollectionContext {
                pipeline: self.primitive_pipeline,
                tessellation_cache: self.tessellation_cache,
                path_interner: self.path_interner,
                text_renderer: self.text_renderer,
                glyph_texture: self.glyph_texture,
                queue: &self.context.queue,
            };

            let (node_instances, node_path_batches) = collect_style_batches_for_bounds(
                &mut batch_ctx,
                style,
                effective_opacity,
                render_bounds,
                node.transform,
            );"""

replace2 = """            let mut batch_ctx = BatchCollectionContext {
                pipeline: self.primitive_pipeline,
                tessellation_cache: self.tessellation_cache,
                path_interner: self.path_interner,
                text_renderer: self.text_renderer,
                glyph_texture: self.glyph_texture,
                queue: &self.context.queue,
            };

            collect_style_batches_for_bounds(
                &mut batch_ctx,
                style,
                effective_opacity,
                render_bounds,
                node.transform,
                &mut instances_buffer,
                &mut path_batches_buffer,
            );"""

content = content.replace(search2, replace2)

search3 = """            if background_blur_radius.is_none() {
                self.draw_batches_to_view(
                    &src_view,
                    wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                    &node_instances,
                    &node_path_batches,
                    None,
                );
            } else {"""

replace3 = """            if background_blur_radius.is_none() {
                self.draw_batches_to_view(
                    &src_view,
                    wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                    &instances_buffer,
                    &path_batches_buffer,
                    None,
                );
            } else {"""

content = content.replace(search3, replace3)


search4 = """                    wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                    &node_instances,
                    &node_path_batches,
                    None,
                );"""

replace4 = """                    wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                    &instances_buffer,
                    &path_batches_buffer,
                    None,
                );"""

content = content.replace(search4, replace4)


search5 = """    fn render_direct_node(
        &mut self,
        scene: &Scene,
        node_id: crate::NodeId,
        surface_view: &wgpu::TextureView,
    ) {
        let Some(node) = scene.get_node(node_id) else {
            return;
        };"""

replace5 = """    fn render_direct_node(
        &mut self,
        scene: &Scene,
        node_id: crate::NodeId,
        surface_view: &wgpu::TextureView,
    ) {
        let mut instances_buffer = Vec::new();
        let mut path_batches_buffer = Vec::new();

        let Some(node) = scene.get_node(node_id) else {
            return;
        };"""

content = content.replace(search5, replace5)


search6 = """                let mut batch_ctx = BatchCollectionContext {
                    pipeline: self.primitive_pipeline,
                    tessellation_cache: self.tessellation_cache,
                    path_interner: self.path_interner,
                    text_renderer: self.text_renderer,
                    glyph_texture: self.glyph_texture,
                    queue: &self.context.queue,
                };
                let (instances, path_batches) = collect_style_batches_for_bounds(
                    &mut batch_ctx,
                    style,
                    effective_opacity,
                    render_bounds,
                    node.transform,
                );
                self.draw_batches_to_view(
                    surface_view,
                    wgpu::LoadOp::Load,
                    &instances,
                    &path_batches,
                    scissor,
                );"""

replace6 = """                let mut batch_ctx = BatchCollectionContext {
                    pipeline: self.primitive_pipeline,
                    tessellation_cache: self.tessellation_cache,
                    path_interner: self.path_interner,
                    text_renderer: self.text_renderer,
                    glyph_texture: self.glyph_texture,
                    queue: &self.context.queue,
                };
                collect_style_batches_for_bounds(
                    &mut batch_ctx,
                    style,
                    effective_opacity,
                    render_bounds,
                    node.transform,
                    &mut instances_buffer,
                    &mut path_batches_buffer,
                );
                self.draw_batches_to_view(
                    surface_view,
                    wgpu::LoadOp::Load,
                    &instances_buffer,
                    &path_batches_buffer,
                    scissor,
                );"""

content = content.replace(search6, replace6)


with open("crates/render-engine/src/backend/wgpu/multipass_executor.rs", "w") as f:
    f.write(content)
