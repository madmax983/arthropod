//! ECS entity and component query tools.

use crate::context::McpFrameworkContext;
use crate::tools::{Tool, ToolSchema};
use anyhow::{anyhow, Result};
use arthropod_ecs::{Renderable, SceneNodeRef};
use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

// ============================================================================
// ecs.query_entities
// ============================================================================

/// Query entities by component composition
pub struct QueryEntitiesTool;

#[derive(Debug, Deserialize)]
struct QueryEntitiesParams {
    #[serde(default)]
    with_components: Vec<String>,

    #[serde(default)]
    without_components: Vec<String>,

    #[serde(default = "default_limit")]
    limit: usize,
}

fn default_limit() -> usize {
    100
}

#[derive(Debug, Serialize)]
struct EntityData {
    entity_id: u32,
    node_id: Option<u64>,
    components: Vec<String>,
}

impl Tool for QueryEntitiesTool {
    fn name(&self) -> &str {
        "ecs.query_entities"
    }

    fn schema(&self) -> ToolSchema {
        ToolSchema {
            name: self.name().to_string(),
            description: "Query entities by component composition".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "with_components": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "Components that must be present"
                    },
                    "without_components": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "Components that must NOT be present"
                    },
                    "limit": {
                        "type": "number",
                        "default": 100,
                        "description": "Maximum number of entities to return"
                    }
                }
            }),
        }
    }

    fn execute(&self, params: Value, ctx: &mut McpFrameworkContext) -> Result<Value> {
        let params: QueryEntitiesParams = serde_json::from_value(params)?;

        let world = ctx.inner_mut().world_mut();
        let mut entities = Vec::new();

        // Query all entities with optional components
        let mut query = world.query::<(Entity, Option<&SceneNodeRef>, Option<&Renderable>)>();

        for (entity, node_ref, renderable) in query.iter(world) {
            let mut components = Vec::new();

            if let Some(_) = node_ref {
                components.push("SceneNodeRef".to_string());
            }

            if renderable.is_some() {
                components.push("Renderable".to_string());
            }

            // Check filters
            if !params.with_components.is_empty() {
                if !params.with_components.iter().all(|c| components.contains(c)) {
                    continue;
                }
            }

            if !params.without_components.is_empty() {
                if params.without_components.iter().any(|c| components.contains(c)) {
                    continue;
                }
            }

            entities.push(EntityData {
                entity_id: entity.index(),
                node_id: node_ref.map(|n| n.0.0),
                components,
            });

            if entities.len() >= params.limit {
                break;
            }
        }

        Ok(json!({ "entities": entities, "count": entities.len() }))
    }
}

// ============================================================================
// ecs.get_entity
// ============================================================================

/// Get all components for a specific entity
pub struct GetEntityTool;

#[derive(Debug, Deserialize)]
struct GetEntityParams {
    entity_id: u32,
}

#[derive(Debug, Serialize)]
struct EntityDetail {
    entity_id: u32,
    components: EntityComponents,
}

#[derive(Debug, Serialize)]
struct EntityComponents {
    scene_node_ref: Option<u64>,
    renderable: bool,
}

impl Tool for GetEntityTool {
    fn name(&self) -> &str {
        "ecs.get_entity"
    }

    fn schema(&self) -> ToolSchema {
        ToolSchema {
            name: self.name().to_string(),
            description: "Get all components for a specific entity".to_string(),
            parameters: json!({
                "type": "object",
                "required": ["entity_id"],
                "properties": {
                    "entity_id": {
                        "type": "number",
                        "description": "Entity ID to query"
                    }
                }
            }),
        }
    }

    fn execute(&self, params: Value, ctx: &mut McpFrameworkContext) -> Result<Value> {
        let params: GetEntityParams = serde_json::from_value(params)?;

        let world = ctx.inner().world();
        let entity = Entity::from_raw(params.entity_id);

        // Check if entity exists
        if !world.entities().contains(entity) {
            return Err(anyhow!("Entity {} not found", params.entity_id));
        }

        let scene_node_ref = world.get::<SceneNodeRef>(entity).map(|n| n.0.0);
        let renderable = world.get::<Renderable>(entity).is_some();

        let detail = EntityDetail {
            entity_id: params.entity_id,
            components: EntityComponents {
                scene_node_ref,
                renderable,
            },
        };

        Ok(serde_json::to_value(detail)?)
    }
}

// ============================================================================
// ecs.count_entities
// ============================================================================

/// Count entities matching filters
pub struct CountEntitiesTool;

#[derive(Debug, Deserialize)]
struct CountEntitiesParams {
    #[serde(default)]
    with_components: Vec<String>,
}

impl Tool for CountEntitiesTool {
    fn name(&self) -> &str {
        "ecs.count_entities"
    }

    fn schema(&self) -> ToolSchema {
        ToolSchema {
            name: self.name().to_string(),
            description: "Count entities matching component filters".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "with_components": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "Components that must be present"
                    }
                }
            }),
        }
    }

    fn execute(&self, params: Value, ctx: &mut McpFrameworkContext) -> Result<Value> {
        let params: CountEntitiesParams = serde_json::from_value(params)?;

        let world = ctx.inner_mut().world_mut();
        let mut count = 0;

        let mut query = world.query::<(Entity, Option<&SceneNodeRef>, Option<&Renderable>)>();

        for (_entity, node_ref, renderable) in query.iter(world) {
            let mut components = Vec::new();

            if let Some(_) = node_ref {
                components.push("SceneNodeRef".to_string());
            }

            if renderable.is_some() {
                components.push("Renderable".to_string());
            }

            if params.with_components.is_empty()
                || params.with_components.iter().all(|c| components.contains(c))
            {
                count += 1;
            }
        }

        Ok(json!({ "count": count }))
    }
}

// ============================================================================
// ecs.list_archetypes
// ============================================================================

/// List all archetypes (component combinations)
pub struct ListArchetypesTool;

#[derive(Debug, Serialize)]
struct ArchetypeInfo {
    archetype_id: u32,
    component_count: usize,
    entity_count: usize,
}

impl Tool for ListArchetypesTool {
    fn name(&self) -> &str {
        "ecs.list_archetypes"
    }

    fn schema(&self) -> ToolSchema {
        ToolSchema {
            name: self.name().to_string(),
            description: "List all ECS archetypes with entity counts".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {}
            }),
        }
    }

    fn execute(&self, _params: Value, ctx: &mut McpFrameworkContext) -> Result<Value> {
        let world = ctx.inner().world();
        let mut archetypes = Vec::new();

        for archetype in world.archetypes().iter() {
            archetypes.push(ArchetypeInfo {
                archetype_id: archetype.id().index() as u32,
                component_count: archetype.component_count(),
                entity_count: archetype.len(),
            });
        }

        Ok(json!({ "archetypes": archetypes, "total": archetypes.len() }))
    }
}

// ============================================================================
// ecs.verify_linkage
// ============================================================================

/// Verify scene-entity linkage integrity
pub struct VerifyLinkageTool;

#[derive(Debug, Serialize)]
struct LinkageReport {
    total_entities: usize,
    linked_entities: usize,
    broken_links: Vec<BrokenLink>,
    orphaned_entities: Vec<u32>,
}

#[derive(Debug, Serialize)]
struct BrokenLink {
    entity_id: u32,
    node_id: u64,
    reason: String,
}

impl Tool for VerifyLinkageTool {
    fn name(&self) -> &str {
        "ecs.verify_linkage"
    }

    fn schema(&self) -> ToolSchema {
        ToolSchema {
            name: self.name().to_string(),
            description: "Verify integrity of scene-entity linkage".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {}
            }),
        }
    }

    fn execute(&self, _params: Value, ctx: &mut McpFrameworkContext) -> Result<Value> {
        // Collect entity data first
        let entity_data: Vec<(u32, Option<render_engine::NodeId>)> = {
            let world = ctx.inner_mut().world_mut();
            let mut query = world.query::<(Entity, Option<&SceneNodeRef>)>();

            query
                .iter(world)
                .map(|(entity, node_ref)| (entity.index(), node_ref.map(|n| n.0)))
                .collect()
        };

        // Now verify against scene
        let scene = ctx.scene();
        let mut total_entities = 0;
        let mut linked_entities = 0;
        let mut broken_links = Vec::new();
        let mut orphaned_entities = Vec::new();

        for (entity_id, node_id_opt) in entity_data {
            total_entities += 1;

            if let Some(node_id) = node_id_opt {
                linked_entities += 1;

                // Verify node exists in scene
                if scene.get_node(node_id).is_none() {
                    broken_links.push(BrokenLink {
                        entity_id,
                        node_id: node_id.0,
                        reason: "Node not found in scene".to_string(),
                    });
                }
            } else {
                orphaned_entities.push(entity_id);
            }
        }

        let report = LinkageReport {
            total_entities,
            linked_entities,
            broken_links,
            orphaned_entities,
        };

        Ok(serde_json::to_value(report)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::McpFrameworkContext;
    use render_engine::{Color, NodeContent, SceneNode};

    fn create_test_entities(ctx: &mut McpFrameworkContext) -> Vec<Entity> {
        let scene = ctx.scene_mut();

        // Create test nodes
        let mut node1 = SceneNode::new(NodeContent::Rect { color: Color::RED });
        node1.bounds = plat_core::Rect::new(0.0, 0.0, 100.0, 100.0);
        let node_id1 = scene.add_node(scene.root(), node1);

        let mut node2 = SceneNode::new(NodeContent::Rect { color: Color::BLUE });
        node2.bounds = plat_core::Rect::new(100.0, 100.0, 100.0, 100.0);
        let node_id2 = scene.add_node(scene.root(), node2);

        // Create entities with different component combinations
        let entity1 = ctx.inner_mut().spawn(node_id1).insert(Renderable).id();

        let entity2 = ctx.inner_mut().spawn(node_id2).id();

        vec![entity1, entity2]
    }

    #[test]
    fn test_query_entities_all() {
        let mut ctx = McpFrameworkContext::new();
        create_test_entities(&mut ctx);

        let tool = QueryEntitiesTool;
        let result = tool.execute(json!({}), &mut ctx).unwrap();

        let count = result.get("count").unwrap().as_u64().unwrap();
        assert!(count >= 2); // At least our 2 test entities
    }

    #[test]
    fn test_query_entities_with_renderable() {
        let mut ctx = McpFrameworkContext::new();
        create_test_entities(&mut ctx);

        let tool = QueryEntitiesTool;
        let result = tool
            .execute(json!({"with_components": ["Renderable"]}), &mut ctx)
            .unwrap();

        let entities = result.get("entities").unwrap().as_array().unwrap();
        assert!(entities.len() >= 1); // At least entity1 has Renderable
    }

    #[test]
    fn test_get_entity() {
        let mut ctx = McpFrameworkContext::new();
        let entities = create_test_entities(&mut ctx);

        let tool = GetEntityTool;
        let result = tool
            .execute(json!({"entity_id": entities[0].index()}), &mut ctx)
            .unwrap();

        assert_eq!(
            result.get("entity_id").unwrap().as_u64().unwrap(),
            entities[0].index() as u64
        );
        assert!(result.get("components").is_some());
    }

    #[test]
    fn test_get_entity_not_found() {
        let mut ctx = McpFrameworkContext::new();

        let tool = GetEntityTool;
        let result = tool.execute(json!({"entity_id": 99999}), &mut ctx);

        assert!(result.is_err());
    }

    #[test]
    fn test_count_entities() {
        let mut ctx = McpFrameworkContext::new();
        create_test_entities(&mut ctx);

        let tool = CountEntitiesTool;
        let result = tool.execute(json!({}), &mut ctx).unwrap();

        let count = result.get("count").unwrap().as_u64().unwrap();
        assert!(count >= 2);
    }

    #[test]
    fn test_count_entities_with_filter() {
        let mut ctx = McpFrameworkContext::new();
        create_test_entities(&mut ctx);

        let tool = CountEntitiesTool;
        let result = tool
            .execute(json!({"with_components": ["Renderable"]}), &mut ctx)
            .unwrap();

        let count = result.get("count").unwrap().as_u64().unwrap();
        assert!(count >= 1);
    }

    #[test]
    fn test_list_archetypes() {
        let mut ctx = McpFrameworkContext::new();
        create_test_entities(&mut ctx);

        let tool = ListArchetypesTool;
        let result = tool.execute(json!({}), &mut ctx).unwrap();

        let archetypes = result.get("archetypes").unwrap().as_array().unwrap();
        assert!(!archetypes.is_empty());
    }

    #[test]
    fn test_verify_linkage() {
        let mut ctx = McpFrameworkContext::new();
        create_test_entities(&mut ctx);

        let tool = VerifyLinkageTool;
        let result = tool.execute(json!({}), &mut ctx).unwrap();

        let total = result.get("total_entities").unwrap().as_u64().unwrap();
        let linked = result.get("linked_entities").unwrap().as_u64().unwrap();

        assert!(total >= 2);
        assert!(linked >= 2);
        assert_eq!(
            result.get("broken_links").unwrap().as_array().unwrap().len(),
            0
        );
    }
}
