pub mod models;
pub(crate) mod schema;

pub use models::*;

use render_engine::{NodeContent, Scene, SceneNode};
use schema::*;
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum FigmaImportError {
    #[error("failed to parse figma json: {0}")]
    Parse(#[from] serde_json::Error),
    #[error(
        "invalid figma json shape: expected an array of nodes or an object with a `nodes` array"
    )]
    InvalidDocumentShape,
}

pub fn import_figma_document(json: &str) -> Result<ImportedFigmaDocument, FigmaImportError> {
    let document = parse_figma_document(json)?;

    let mut scene = Scene::new();
    let root = scene.root();
    let mut figma_to_scene = HashMap::new();
    let mut layout_styles = HashMap::new();
    let mut constraints_map = HashMap::new();
    let mut components = HashMap::new();
    let mut instances = HashMap::new();
    let mut variant_properties = HashMap::new();
    let mut component_property_definitions = HashMap::new();
    let mut instance_property_overrides = HashMap::new();
    let mut pending: Vec<usize> = (0..document.nodes.len()).collect();

    while !pending.is_empty() {
        let mut progressed = false;
        let mut unresolved = Vec::new();

        for index in pending {
            let node = &document.nodes[index];
            let parent = match &node.parent_id {
                Some(parent_id) => figma_to_scene.get(&parent_id.as_key()).copied(),
                None => Some(root),
            };

            if let Some(parent_id) = parent {
                let bounds = node.resolved_bounds();
                let mut scene_node = SceneNode::new(NodeContent::Styled {
                    style: Box::new(node.to_visual_style()),
                });
                scene_node.bounds = bounds;
                scene_node.opacity = node.to_node_opacity();
                if let Some(transform) = node.to_node_transform(bounds) {
                    scene_node.transform = transform;
                }
                let scene_id = scene.add_node(parent_id, scene_node);

                figma_to_scene.insert(node.id.as_key(), scene_id);
                layout_styles.insert(scene_id, node.to_flex_style(bounds));
                constraints_map.insert(scene_id, node.to_constraints());
                if let Some(component) = node.to_component_node() {
                    components.insert(scene_id, component);
                }
                if let Some(instance) = node.to_instance_node() {
                    instances.insert(scene_id, instance);
                }
                let variants = node.to_variant_properties();
                if !variants.is_empty() {
                    variant_properties.insert(scene_id, variants);
                }
                let definitions = node.to_component_property_definitions();
                if !definitions.is_empty() {
                    component_property_definitions.insert(scene_id, definitions);
                }
                let overrides = node.to_instance_property_overrides();
                if !overrides.is_empty() {
                    instance_property_overrides.insert(scene_id, overrides);
                }
                progressed = true;
            } else {
                unresolved.push(index);
            }
        }

        if !progressed {
            for index in unresolved {
                let node = &document.nodes[index];
                let bounds = node.resolved_bounds();
                let mut scene_node = SceneNode::new(NodeContent::Styled {
                    style: Box::new(node.to_visual_style()),
                });
                scene_node.bounds = bounds;
                scene_node.opacity = node.to_node_opacity();
                if let Some(transform) = node.to_node_transform(bounds) {
                    scene_node.transform = transform;
                }
                let scene_id = scene.add_node(root, scene_node);

                figma_to_scene.insert(node.id.as_key(), scene_id);
                layout_styles.insert(scene_id, node.to_flex_style(bounds));
                constraints_map.insert(scene_id, node.to_constraints());
                if let Some(component) = node.to_component_node() {
                    components.insert(scene_id, component);
                }
                if let Some(instance) = node.to_instance_node() {
                    instances.insert(scene_id, instance);
                }
                let variants = node.to_variant_properties();
                if !variants.is_empty() {
                    variant_properties.insert(scene_id, variants);
                }
                let definitions = node.to_component_property_definitions();
                if !definitions.is_empty() {
                    component_property_definitions.insert(scene_id, definitions);
                }
                let overrides = node.to_instance_property_overrides();
                if !overrides.is_empty() {
                    instance_property_overrides.insert(scene_id, overrides);
                }
            }
            break;
        }

        pending = unresolved;
    }

    let mut prototype_edges = Vec::new();
    for node in &document.nodes {
        let Some(from) = figma_to_scene.get(&node.id.as_key()).copied() else {
            continue;
        };
        for interaction in &node.prototype_interactions {
            let Some((trigger, trigger_timeout_ms)) = interaction.trigger_spec() else {
                continue;
            };

            let inherited_transition = interaction.transition_details();
            let inherited_preserve_scroll = interaction.preserve_scroll_position.unwrap_or(false);

            if interaction.actions.is_empty() {
                if let Some(edge) = interaction.to_legacy_edge(
                    from,
                    trigger,
                    trigger_timeout_ms,
                    inherited_transition,
                    inherited_preserve_scroll,
                ) {
                    prototype_edges.push(edge);
                }
                continue;
            }

            for action in &interaction.actions {
                if let Some(edge) = action.to_edge(
                    from,
                    trigger,
                    trigger_timeout_ms,
                    inherited_transition.clone(),
                    inherited_preserve_scroll,
                ) {
                    prototype_edges.push(edge);
                }
            }
        }
    }

    let mut resolved_instance_properties = HashMap::new();
    let mut component_base_cache = HashMap::new();

    for (instance_node_id, instance_meta) in &instances {
        if let Some(component_scene_id) = instance_meta
            .component_id
            .as_ref()
            .and_then(|figma_id| figma_to_scene.get(figma_id))
            .copied()
        {
            let base = component_base_cache
                .entry(component_scene_id)
                .or_insert_with(|| {
                    let mut resolved = HashMap::new();
                    if let Some(definitions) =
                        component_property_definitions.get(&component_scene_id)
                    {
                        for (name, definition) in definitions {
                            if let Some(default_value) = definition.default_value.clone() {
                                resolved.insert(name.clone(), default_value);
                            }
                        }
                    }

                    if let Some(component_variants) = variant_properties.get(&component_scene_id) {
                        for (name, value) in component_variants {
                            resolved.entry(name.clone()).or_insert_with(|| {
                                ImportedComponentPropertyValue::Text(value.clone())
                            });
                        }
                    }
                    resolved
                });

            if !base.is_empty() {
                resolved_instance_properties.insert(*instance_node_id, base.clone());
            }
        }
    }

    for (node_id, overrides) in &instance_property_overrides {
        if instances.contains_key(node_id) && !overrides.is_empty() {
            let resolved = resolved_instance_properties
                .entry(*node_id)
                .or_insert_with(HashMap::new);
            for (name, override_entry) in overrides {
                resolved.insert(name.clone(), override_entry.value.clone());
            }
        }
    }

    for (node_id, instance_variants) in &variant_properties {
        if instances.contains_key(node_id) && !instance_variants.is_empty() {
            let resolved = resolved_instance_properties
                .entry(*node_id)
                .or_insert_with(HashMap::new);
            for (name, value) in instance_variants {
                resolved.insert(
                    name.clone(),
                    ImportedComponentPropertyValue::Text(value.clone()),
                );
            }
        }
    }

    Ok(ImportedFigmaDocument {
        scene,
        layout_styles,
        constraints: constraints_map,
        prototype_graph: PrototypeGraph {
            edges: prototype_edges,
        },
        figma_to_scene,
        components,
        instances,
        variant_properties,
        component_property_definitions,
        instance_property_overrides,
        resolved_instance_properties,
    })
}
