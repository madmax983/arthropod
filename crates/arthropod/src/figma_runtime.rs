use std::collections::{HashMap, HashSet};

use layout_engine::FlexStyle;
use render_engine::{
    NodeId, Scene,
    backend::{PrimitiveInstance, wgpu::create_node_instances},
};
use thiserror::Error;

use crate::{
    figma::{FigmaImportError, ImportedFigmaDocument, import_figma_document},
    layout::auto_layout,
    prototype_runtime::{
        PrototypeRuntime, PrototypeRuntimeEffect, PrototypeRuntimeError, PrototypeRuntimeEvent,
    },
};

#[derive(Debug, Error)]
pub enum FigmaRuntimeError {
    #[error("figma import failed: {0}")]
    Import(#[from] FigmaImportError),
    #[error("prototype runtime initialization failed: {0}")]
    Prototype(#[from] PrototypeRuntimeError),
}

/// Runtime bridge for imported Figma documents.
///
/// Responsibilities:
/// - Own imported scene + layout metadata
/// - Run layout on demand for viewport updates
/// - Dispatch prototype events and apply visibility/navigation semantics
/// - Collect render instances for rendering backends or test harnesses
pub struct FigmaRuntime {
    scene: Scene,
    layout_styles: HashMap<NodeId, FlexStyle>,
    figma_to_scene: HashMap<String, NodeId>,
    top_level_screens: Vec<NodeId>,
    visible_overlays: HashSet<NodeId>,
    prototype_runtime: Option<PrototypeRuntime>,
}

impl FigmaRuntime {
    pub fn from_figma_json(json: &str) -> Result<Self, FigmaRuntimeError> {
        let imported = import_figma_document(json)?;
        Self::from_imported(imported)
    }

    pub fn from_imported(imported: ImportedFigmaDocument) -> Result<Self, FigmaRuntimeError> {
        let mut runtime = if imported.figma_to_scene.is_empty() {
            None
        } else {
            Some(PrototypeRuntime::new(&imported)?)
        };

        let root_children = imported
            .scene
            .get_node(imported.scene.root())
            .map(|node| node.children.clone())
            .unwrap_or_default();

        let mut state = Self {
            scene: imported.scene,
            layout_styles: imported.layout_styles,
            figma_to_scene: imported.figma_to_scene,
            top_level_screens: root_children,
            visible_overlays: HashSet::new(),
            prototype_runtime: runtime.take(),
        };

        let current_screen = state
            .prototype_runtime
            .as_ref()
            .and_then(PrototypeRuntime::current_screen)
            .or_else(|| state.top_level_screens.first().copied());
        if let Some(screen) = current_screen {
            state.show_screen(screen);
        }
        state.sync_overlay_visibility_from_runtime();

        Ok(state)
    }

    pub fn scene(&self) -> &Scene {
        &self.scene
    }

    pub fn scene_mut(&mut self) -> &mut Scene {
        &mut self.scene
    }

    pub fn figma_to_scene(&self) -> &HashMap<String, NodeId> {
        &self.figma_to_scene
    }

    pub fn node_for_figma_id(&self, figma_id: &str) -> Option<NodeId> {
        self.figma_to_scene.get(figma_id).copied()
    }

    pub fn current_screen(&self) -> Option<NodeId> {
        self.prototype_runtime
            .as_ref()
            .and_then(PrototypeRuntime::current_screen)
            .or_else(|| self.top_level_screens.first().copied())
    }

    pub fn set_current_screen(&mut self, node: NodeId) {
        if let Some(runtime) = &mut self.prototype_runtime {
            runtime.set_current_screen(node);
        }
        self.show_screen(node);
    }

    pub fn is_visible(&self, node: NodeId) -> bool {
        self.scene
            .get_node(node)
            .map(|entry| entry.visible)
            .unwrap_or(false)
    }

    pub fn apply_layout(&mut self, viewport_width: f32, viewport_height: f32) {
        let root = self.scene.root();
        auto_layout(
            &mut self.scene,
            root,
            viewport_width,
            viewport_height,
            &self.layout_styles,
        );
    }

    pub fn dispatch(&mut self, event: PrototypeRuntimeEvent) -> Vec<PrototypeRuntimeEffect> {
        let Some(runtime) = &mut self.prototype_runtime else {
            return Vec::new();
        };

        let effects = runtime.dispatch(event);
        self.apply_runtime_effects(&effects);
        effects
    }

    pub fn collect_render_instances(&self) -> Vec<PrimitiveInstance> {
        self.scene
            .iter_visuals()
            .flat_map(|(_, node)| create_node_instances(node))
            .collect()
    }

    fn apply_runtime_effects(&mut self, effects: &[PrototypeRuntimeEffect]) {
        for effect in effects {
            match effect {
                PrototypeRuntimeEffect::Navigate { to, .. } => {
                    self.show_screen(*to);
                }
                PrototypeRuntimeEffect::Back { to: Some(node), .. } => {
                    self.show_screen(*node);
                }
                PrototypeRuntimeEffect::Back { to: None, .. }
                | PrototypeRuntimeEffect::ScrollTo { .. }
                | PrototypeRuntimeEffect::OpenOverlay { .. }
                | PrototypeRuntimeEffect::SwapOverlay { .. }
                | PrototypeRuntimeEffect::CloseOverlay { .. }
                | PrototypeRuntimeEffect::OpenUrl { .. } => {}
            }
        }
        self.sync_overlay_visibility_from_runtime();
    }

    fn show_screen(&mut self, target: NodeId) {
        for node_id in &self.top_level_screens {
            if let Some(node) = self.scene.get_node_mut(*node_id) {
                node.visible = *node_id == target;
            }
        }
    }

    fn sync_overlay_visibility_from_runtime(&mut self) {
        let Some(runtime) = self.prototype_runtime.as_ref() else {
            return;
        };

        let desired: HashSet<NodeId> = runtime
            .overlay_stack()
            .iter()
            .map(|item| item.node)
            .collect();
        let to_hide: Vec<NodeId> = self
            .visible_overlays
            .iter()
            .copied()
            .filter(|node| !desired.contains(node))
            .collect();
        for node in to_hide {
            if let Some(entry) = self.scene.get_node_mut(node) {
                entry.visible = false;
            }
            self.visible_overlays.remove(&node);
        }
        for node in desired {
            if let Some(entry) = self.scene.get_node_mut(node) {
                entry.visible = true;
            }
            self.visible_overlays.insert(node);
        }
    }
}
