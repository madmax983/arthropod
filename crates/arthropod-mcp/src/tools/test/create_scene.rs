use crate::context::McpFrameworkContext;
use crate::tools::{Tool, ToolSchema};
use anyhow::{Result, anyhow};
use render_engine::{Color, NodeContent, NodeId, Scene, SceneNode};
use serde::Deserialize;
use serde_json::{Value, json};
use std::collections::HashMap;

// ============================================================================
// test.create_scene
// ============================================================================

/// Declarative scene builder from JSON specification
pub struct CreateSceneTool;

#[derive(Debug, Deserialize)]
struct CreateSceneParams {
    nodes: Vec<NodeSpec>,
}

#[derive(Debug, Deserialize)]
struct NodeSpec {
    name: String,
    content: NodeContentSpec,
    bounds: BoundsSpec,
    #[serde(default)]
    parent: Option<String>,
    #[serde(default = "default_visible")]
    visible: bool,
    #[serde(default = "default_opacity")]
    opacity: f32,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
enum NodeContentSpec {
    Rect { color: [f32; 4] },
    RoundedRect { color: [f32; 4], corner_radius: f32 },
}

#[derive(Debug, Deserialize)]
struct BoundsSpec {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}

fn default_visible() -> bool {
    true
}

fn default_opacity() -> f32 {
    1.0
}

impl Tool for CreateSceneTool {
    fn name(&self) -> &str {
        "test.create_scene"
    }

    fn schema(&self) -> ToolSchema {
        ToolSchema {
            name: self.name().to_string(),
            description: "Create scene declaratively from JSON specification".to_string(),
            parameters: json!({
                "type": "object",
                "required": ["nodes"],
                "properties": {
                    "nodes": {
                        "type": "array",
                        "description": "Array of node specifications",
                        "items": {
                            "type": "object",
                            "required": ["name", "content", "bounds"],
                            "properties": {
                                "name": {
                                    "type": "string",
                                    "description": "Unique name for the node"
                                },
                                "content": {
                                    "type": "object",
                                    "description": "Node content specification",
                                    "required": ["type"],
                                    "properties": {
                                        "type": {
                                            "type": "string",
                                            "enum": ["Rect", "RoundedRect"]
                                        },
                                        "color": {
                                            "type": "array",
                                            "items": { "type": "number" },
                                            "minItems": 4,
                                            "maxItems": 4,
                                            "description": "RGBA color [r, g, b, a]"
                                        },
                                        "corner_radius": {
                                            "type": "number",
                                            "description": "Corner radius (for RoundedRect)"
                                        }
                                    }
                                },
                                "bounds": {
                                    "type": "object",
                                    "required": ["x", "y", "width", "height"],
                                    "properties": {
                                        "x": { "type": "number" },
                                        "y": { "type": "number" },
                                        "width": { "type": "number" },
                                        "height": { "type": "number" }
                                    }
                                },
                                "parent": {
                                    "type": "string",
                                    "description": "Name of parent node (optional)"
                                },
                                "visible": {
                                    "type": "boolean",
                                    "default": true
                                },
                                "opacity": {
                                    "type": "number",
                                    "default": 1.0
                                }
                            }
                        }
                    }
                }
            }),
        }
    }

    fn execute(&self, params: Value, ctx: &mut McpFrameworkContext) -> Result<Value> {
        let params: CreateSceneParams = serde_json::from_value(params)?;

        let mut name_to_id = HashMap::new();
        let scene = ctx.scene_mut();

        // Build a map of node specs by name for easy lookup
        let node_specs: HashMap<String, &NodeSpec> = params
            .nodes
            .iter()
            .map(|spec| (spec.name.clone(), spec))
            .collect();

        // Track which nodes have been created
        let mut created = std::collections::HashSet::new();

        // Helper function to recursively create a node and its dependencies
        fn create_node_recursive(
            spec_name: &str,
            node_specs: &HashMap<String, &NodeSpec>,
            scene: &mut Scene,
            name_to_id: &mut HashMap<String, NodeId>,
            created: &mut std::collections::HashSet<String>,
        ) -> Result<NodeId> {
            // If already created, return the existing ID
            if let Some(node_id) = name_to_id.get(spec_name) {
                return Ok(*node_id);
            }

            // Get the spec
            let spec = node_specs
                .get(spec_name)
                .ok_or_else(|| anyhow!("Node '{}' not found", spec_name))?;

            // Check for circular dependency
            if created.contains(spec_name) {
                return Err(anyhow!("Circular dependency detected: {}", spec_name));
            }
            created.insert(spec_name.to_string());

            // Determine the parent ID
            let parent_id = if let Some(parent_name) = &spec.parent {
                // Recursively create parent first
                create_node_recursive(parent_name, node_specs, scene, name_to_id, created)?
            } else {
                // Use scene root if no parent specified
                scene.root()
            };

            // Create the node content
            let content = match &spec.content {
                NodeContentSpec::Rect { color } => NodeContent::Rect {
                    color: Color::rgba(color[0], color[1], color[2], color[3]),
                },
                NodeContentSpec::RoundedRect {
                    color,
                    corner_radius,
                } => NodeContent::RoundedRect {
                    color: Color::rgba(color[0], color[1], color[2], color[3]),
                    corner_radius: *corner_radius,
                },
            };

            let mut node = SceneNode::new(content);
            node.bounds = plat_core::Rect::new(
                spec.bounds.x,
                spec.bounds.y,
                spec.bounds.width,
                spec.bounds.height,
            );
            node.visible = spec.visible;
            node.opacity = spec.opacity;

            // Add node to scene with correct parent
            let node_id = scene.add_node(parent_id, node);
            name_to_id.insert(spec_name.to_string(), node_id);

            Ok(node_id)
        }

        // Create all nodes (dependencies are resolved recursively)
        let node_names: Vec<String> = params.nodes.iter().map(|s| s.name.clone()).collect();
        for name in &node_names {
            create_node_recursive(name, &node_specs, scene, &mut name_to_id, &mut created)?;
        }

        // Register all named nodes in context
        for (name, node_id) in &name_to_id {
            ctx.register_node(name.clone(), *node_id);
        }

        // Convert NodeId map to u64 map for JSON serialization
        let id_map: HashMap<String, u64> = name_to_id
            .iter()
            .map(|(name, node_id)| (name.clone(), node_id.0))
            .collect();

        Ok(json!({
            "nodes_created": params.nodes.len(),
            "node_ids": id_map
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::McpFrameworkContext;

    #[test]
    fn test_create_scene_simple() {
        let mut ctx = McpFrameworkContext::new();
        let tool = CreateSceneTool;

        let result = tool
            .execute(
                json!({
                    "nodes": [
                        {
                            "name": "background",
                            "content": { "type": "Rect", "color": [0.1, 0.1, 0.1, 1.0] },
                            "bounds": { "x": 0.0, "y": 0.0, "width": 800.0, "height": 600.0 }
                        }
                    ]
                }),
                &mut ctx,
            )
            .unwrap();

        assert_eq!(result.get("nodes_created").unwrap().as_u64().unwrap(), 1);
        assert!(ctx.get_named_node("background").is_some());
    }

    #[test]
    fn test_create_scene_with_hierarchy() {
        let mut ctx = McpFrameworkContext::new();
        let tool = CreateSceneTool;

        let result = tool
            .execute(
                json!({
                    "nodes": [
                        {
                            "name": "background",
                            "content": { "type": "Rect", "color": [0.1, 0.1, 0.1, 1.0] },
                            "bounds": { "x": 0.0, "y": 0.0, "width": 800.0, "height": 600.0 }
                        },
                        {
                            "name": "button",
                            "content": {
                                "type": "RoundedRect",
                                "color": [0.2, 0.4, 0.8, 1.0],
                                "corner_radius": 8.0
                            },
                            "bounds": { "x": 300.0, "y": 250.0, "width": 200.0, "height": 100.0 },
                            "parent": "background"
                        }
                    ]
                }),
                &mut ctx,
            )
            .unwrap();

        assert_eq!(result.get("nodes_created").unwrap().as_u64().unwrap(), 2);

        let bg_id = ctx.get_named_node("background").unwrap();
        let button_id = ctx.get_named_node("button").unwrap();

        let scene = ctx.scene();
        let bg_node = scene.get_node(bg_id).unwrap();

        // Verify button is child of background
        assert!(bg_node.children.contains(&button_id));
    }

    #[test]
    fn test_create_scene_with_properties() {
        let mut ctx = McpFrameworkContext::new();
        let tool = CreateSceneTool;

        tool.execute(
            json!({
                "nodes": [
                    {
                        "name": "hidden",
                        "content": { "type": "Rect", "color": [1.0, 0.0, 0.0, 1.0] },
                        "bounds": { "x": 0.0, "y": 0.0, "width": 100.0, "height": 100.0 },
                        "visible": false,
                        "opacity": 0.5
                    }
                ]
            }),
            &mut ctx,
        )
        .unwrap();

        let node_id = ctx.get_named_node("hidden").unwrap();
        let node = ctx.scene().get_node(node_id).unwrap();

        assert!(!node.visible);
        assert_eq!(node.opacity, 0.5);
    }

    #[test]
    fn test_create_scene_invalid_parent() {
        let mut ctx = McpFrameworkContext::new();
        let tool = CreateSceneTool;

        let result = tool.execute(
            json!({
                "nodes": [
                    {
                        "name": "orphan",
                        "content": { "type": "Rect", "color": [1.0, 0.0, 0.0, 1.0] },
                        "bounds": { "x": 0.0, "y": 0.0, "width": 100.0, "height": 100.0 },
                        "parent": "nonexistent"
                    }
                ]
            }),
            &mut ctx,
        );

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }
}
