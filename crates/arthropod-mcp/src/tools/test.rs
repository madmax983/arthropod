//! High-level testing utilities for declarative scene creation and assertions.

use crate::context::McpFrameworkContext;
use crate::tools::{Tool, ToolSchema};
use anyhow::{anyhow, Result};
use arthropod_ecs::{ReactiveColor, Renderable};
use flux_state::Signal;
use render_engine::{Color, NodeContent, NodeId, Scene, SceneNode};
use serde::Deserialize;
use serde_json::{json, Value};
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
                NodeContentSpec::RoundedRect { color, corner_radius } => NodeContent::RoundedRect {
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

// ============================================================================
// test.assert_node_state
// ============================================================================

/// Assert node properties match expected values
pub struct AssertNodeStateTool;

#[derive(Debug, Deserialize)]
struct AssertNodeStateParams {
    node_name: String,

    #[serde(default)]
    expected: ExpectedNodeState,

    #[serde(default = "default_tolerance")]
    tolerance: f32,
}

#[derive(Debug, Deserialize, Default)]
struct ExpectedNodeState {
    #[serde(default)]
    visible: Option<bool>,

    #[serde(default)]
    opacity: Option<f32>,

    #[serde(default)]
    bounds: Option<BoundsSpec>,

    #[serde(default)]
    color: Option<[f32; 4]>,

    #[serde(default)]
    corner_radius: Option<f32>,
}

fn default_tolerance() -> f32 {
    0.001
}

impl Tool for AssertNodeStateTool {
    fn name(&self) -> &str {
        "test.assert_node_state"
    }

    fn schema(&self) -> ToolSchema {
        ToolSchema {
            name: self.name().to_string(),
            description: "Assert that a node's properties match expected values".to_string(),
            parameters: json!({
                "type": "object",
                "required": ["node_name"],
                "properties": {
                    "node_name": {
                        "type": "string",
                        "description": "Name of the node to check"
                    },
                    "expected": {
                        "type": "object",
                        "description": "Expected node properties",
                        "properties": {
                            "visible": { "type": "boolean" },
                            "opacity": { "type": "number" },
                            "bounds": {
                                "type": "object",
                                "properties": {
                                    "x": { "type": "number" },
                                    "y": { "type": "number" },
                                    "width": { "type": "number" },
                                    "height": { "type": "number" }
                                }
                            },
                            "color": {
                                "type": "array",
                                "items": { "type": "number" },
                                "minItems": 4,
                                "maxItems": 4
                            },
                            "corner_radius": { "type": "number" }
                        }
                    },
                    "tolerance": {
                        "type": "number",
                        "default": 0.001,
                        "description": "Tolerance for float comparisons"
                    }
                }
            }),
        }
    }

    fn execute(&self, params: Value, ctx: &mut McpFrameworkContext) -> Result<Value> {
        let params: AssertNodeStateParams = serde_json::from_value(params)?;

        // Get the node
        let node_id = ctx
            .get_named_node(&params.node_name)
            .ok_or_else(|| anyhow!("Node '{}' not found", params.node_name))?;

        let scene = ctx.scene();
        let node = scene
            .get_node(node_id)
            .ok_or_else(|| anyhow!("Node {} not in scene", params.node_name))?;

        let mut failures = Vec::new();

        // Check visibility
        if let Some(expected_visible) = params.expected.visible {
            if node.visible != expected_visible {
                failures.push(format!(
                    "visible: expected {}, got {}",
                    expected_visible, node.visible
                ));
            }
        }

        // Check opacity
        if let Some(expected_opacity) = params.expected.opacity {
            if (node.opacity - expected_opacity).abs() > params.tolerance {
                failures.push(format!(
                    "opacity: expected {}, got {} (tolerance: {})",
                    expected_opacity, node.opacity, params.tolerance
                ));
            }
        }

        // Check bounds
        if let Some(expected_bounds) = &params.expected.bounds {
            if (node.bounds.x - expected_bounds.x).abs() > params.tolerance {
                failures.push(format!(
                    "bounds.x: expected {}, got {} (tolerance: {})",
                    expected_bounds.x, node.bounds.x, params.tolerance
                ));
            }
            if (node.bounds.y - expected_bounds.y).abs() > params.tolerance {
                failures.push(format!(
                    "bounds.y: expected {}, got {} (tolerance: {})",
                    expected_bounds.y, node.bounds.y, params.tolerance
                ));
            }
            if (node.bounds.width - expected_bounds.width).abs() > params.tolerance {
                failures.push(format!(
                    "bounds.width: expected {}, got {} (tolerance: {})",
                    expected_bounds.width, node.bounds.width, params.tolerance
                ));
            }
            if (node.bounds.height - expected_bounds.height).abs() > params.tolerance {
                failures.push(format!(
                    "bounds.height: expected {}, got {} (tolerance: {})",
                    expected_bounds.height, node.bounds.height, params.tolerance
                ));
            }
        }

        // Check color (extract from NodeContent)
        if let Some(expected_color) = &params.expected.color {
            let actual_color = match &node.content {
                NodeContent::Rect { color } | NodeContent::RoundedRect { color, .. } => color,
                _ => {
                    failures.push("Node does not have a color property".to_string());
                    &Color::rgba(0.0, 0.0, 0.0, 0.0)
                }
            };

            let expected = Color::rgba(
                expected_color[0],
                expected_color[1],
                expected_color[2],
                expected_color[3],
            );

            if (actual_color.r() - expected.r()).abs() > params.tolerance {
                failures.push(format!(
                    "color.r: expected {}, got {} (tolerance: {})",
                    expected.r(),
                    actual_color.r(),
                    params.tolerance
                ));
            }
            if (actual_color.g() - expected.g()).abs() > params.tolerance {
                failures.push(format!(
                    "color.g: expected {}, got {} (tolerance: {})",
                    expected.g(),
                    actual_color.g(),
                    params.tolerance
                ));
            }
            if (actual_color.b() - expected.b()).abs() > params.tolerance {
                failures.push(format!(
                    "color.b: expected {}, got {} (tolerance: {})",
                    expected.b(),
                    actual_color.b(),
                    params.tolerance
                ));
            }
            if (actual_color.a() - expected.a()).abs() > params.tolerance {
                failures.push(format!(
                    "color.a: expected {}, got {} (tolerance: {})",
                    expected.a(),
                    actual_color.a(),
                    params.tolerance
                ));
            }
        }

        // Check corner_radius
        if let Some(expected_radius) = params.expected.corner_radius {
            match &node.content {
                NodeContent::RoundedRect { corner_radius, .. } => {
                    if (corner_radius - expected_radius).abs() > params.tolerance {
                        failures.push(format!(
                            "corner_radius: expected {}, got {} (tolerance: {})",
                            expected_radius, corner_radius, params.tolerance
                        ));
                    }
                }
                _ => {
                    failures.push("Node is not a RoundedRect".to_string());
                }
            }
        }

        let passed = failures.is_empty();

        Ok(json!({
            "passed": passed,
            "node_name": params.node_name,
            "failures": failures
        }))
    }
}

// ============================================================================
// test.verify_render_output
// ============================================================================

/// Verify render instances match expected output
pub struct VerifyRenderOutputTool;

#[derive(Debug, Deserialize)]
struct VerifyRenderOutputParams {
    #[serde(default)]
    expected_count: Option<usize>,

    #[serde(default)]
    expected_instances: Vec<ExpectedInstance>,

    #[serde(default = "default_tolerance")]
    tolerance: f32,
}

#[derive(Debug, Deserialize)]
struct ExpectedInstance {
    #[serde(default)]
    color: Option<[f32; 4]>,

    #[serde(default)]
    position: Option<PositionSpec>,

    #[serde(default)]
    size: Option<SizeSpec>,
}

#[derive(Debug, Deserialize)]
struct PositionSpec {
    x: f32,
    y: f32,
}

#[derive(Debug, Deserialize)]
struct SizeSpec {
    width: f32,
    height: f32,
}

impl Tool for VerifyRenderOutputTool {
    fn name(&self) -> &str {
        "test.verify_render_output"
    }

    fn schema(&self) -> ToolSchema {
        ToolSchema {
            name: self.name().to_string(),
            description: "Verify that render output matches expected instances".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "expected_count": {
                        "type": "number",
                        "description": "Expected number of render instances"
                    },
                    "expected_instances": {
                        "type": "array",
                        "description": "Expected instance specifications",
                        "items": {
                            "type": "object",
                            "properties": {
                                "color": {
                                    "type": "array",
                                    "items": { "type": "number" },
                                    "minItems": 4,
                                    "maxItems": 4
                                },
                                "position": {
                                    "type": "object",
                                    "properties": {
                                        "x": { "type": "number" },
                                        "y": { "type": "number" }
                                    }
                                },
                                "size": {
                                    "type": "object",
                                    "properties": {
                                        "width": { "type": "number" },
                                        "height": { "type": "number" }
                                    }
                                }
                            }
                        }
                    },
                    "tolerance": {
                        "type": "number",
                        "default": 0.001,
                        "description": "Tolerance for float comparisons"
                    }
                }
            }),
        }
    }

    fn execute(&self, params: Value, ctx: &mut McpFrameworkContext) -> Result<Value> {
        let params: VerifyRenderOutputParams = serde_json::from_value(params)?;

        // Run render to get actual instances
        let (instances, _duration) = ctx.render();

        let mut failures = Vec::new();

        // Check expected count
        if let Some(expected_count) = params.expected_count {
            if instances.len() != expected_count {
                failures.push(format!(
                    "instance_count: expected {}, got {}",
                    expected_count,
                    instances.len()
                ));
            }
        }

        // Check individual instances
        for (idx, expected) in params.expected_instances.iter().enumerate() {
            if idx >= instances.len() {
                failures.push(format!("Missing instance at index {}", idx));
                continue;
            }

            let actual = &instances[idx];

            // Check color (color is [f32; 4])
            if let Some(expected_color) = &expected.color {
                if (actual.color[0] - expected_color[0]).abs() > params.tolerance {
                    failures.push(format!(
                        "instance[{}].color.r: expected {}, got {} (tolerance: {})",
                        idx, expected_color[0], actual.color[0], params.tolerance
                    ));
                }
                if (actual.color[1] - expected_color[1]).abs() > params.tolerance {
                    failures.push(format!(
                        "instance[{}].color.g: expected {}, got {} (tolerance: {})",
                        idx, expected_color[1], actual.color[1], params.tolerance
                    ));
                }
                if (actual.color[2] - expected_color[2]).abs() > params.tolerance {
                    failures.push(format!(
                        "instance[{}].color.b: expected {}, got {} (tolerance: {})",
                        idx, expected_color[2], actual.color[2], params.tolerance
                    ));
                }
                if (actual.color[3] - expected_color[3]).abs() > params.tolerance {
                    failures.push(format!(
                        "instance[{}].color.a: expected {}, got {} (tolerance: {})",
                        idx, expected_color[3], actual.color[3], params.tolerance
                    ));
                }
            }

            // Check position (pos is [f32; 2])
            if let Some(expected_pos) = &expected.position {
                if (actual.pos[0] - expected_pos.x).abs() > params.tolerance {
                    failures.push(format!(
                        "instance[{}].position.x: expected {}, got {} (tolerance: {})",
                        idx, expected_pos.x, actual.pos[0], params.tolerance
                    ));
                }
                if (actual.pos[1] - expected_pos.y).abs() > params.tolerance {
                    failures.push(format!(
                        "instance[{}].position.y: expected {}, got {} (tolerance: {})",
                        idx, expected_pos.y, actual.pos[1], params.tolerance
                    ));
                }
            }

            // Check size (size is [f32; 2])
            if let Some(expected_size) = &expected.size {
                if (actual.size[0] - expected_size.width).abs() > params.tolerance {
                    failures.push(format!(
                        "instance[{}].size.width: expected {}, got {} (tolerance: {})",
                        idx, expected_size.width, actual.size[0], params.tolerance
                    ));
                }
                if (actual.size[1] - expected_size.height).abs() > params.tolerance {
                    failures.push(format!(
                        "instance[{}].size.height: expected {}, got {} (tolerance: {})",
                        idx, expected_size.height, actual.size[1], params.tolerance
                    ));
                }
            }
        }

        let passed = failures.is_empty();

        Ok(json!({
            "passed": passed,
            "actual_count": instances.len(),
            "failures": failures
        }))
    }
}

// ============================================================================
// test.setup_reactive_chain
// ============================================================================

/// Setup a reactive chain for testing (signal + component + registration)
pub struct SetupReactiveChainTool;

#[derive(Debug, Deserialize)]
struct SetupReactiveChainParams {
    node_name: String,
    signal_name: String,
    signal_type: String,
    initial_value: Value,
}

impl Tool for SetupReactiveChainTool {
    fn name(&self) -> &str {
        "test.setup_reactive_chain"
    }

    fn schema(&self) -> ToolSchema {
        ToolSchema {
            name: self.name().to_string(),
            description: "Setup a complete reactive chain (signal + component + registration)"
                .to_string(),
            parameters: json!({
                "type": "object",
                "required": ["node_name", "signal_name", "signal_type", "initial_value"],
                "properties": {
                    "node_name": {
                        "type": "string",
                        "description": "Name of the node to attach reactive component to"
                    },
                    "signal_name": {
                        "type": "string",
                        "description": "Name to register the signal under"
                    },
                    "signal_type": {
                        "type": "string",
                        "enum": ["color", "f32", "bool"],
                        "description": "Type of reactive signal to create"
                    },
                    "initial_value": {
                        "description": "Initial value for the signal"
                    }
                }
            }),
        }
    }

    fn execute(&self, params: Value, ctx: &mut McpFrameworkContext) -> Result<Value> {
        let params: SetupReactiveChainParams = serde_json::from_value(params)?;

        // Get the node ID
        let node_id = ctx
            .get_named_node(&params.node_name)
            .ok_or_else(|| anyhow!("Node '{}' not found", params.node_name))?;

        // Create signal and register based on type
        match params.signal_type.as_str() {
            "color" => {
                let color_array: [f32; 4] = serde_json::from_value(params.initial_value)?;
                let color = Color::rgba(color_array[0], color_array[1], color_array[2], color_array[3]);

                let runtime = flux_state::Runtime::new();
                let signal = Signal::new(runtime, color);
                let (read, write) = signal.split();

                // Spawn entity with ReactiveColor component
                ctx.inner_mut()
                    .spawn(node_id)
                    .insert(Renderable)
                    .insert(ReactiveColor::new(read.clone()));

                // Register signal for remote control
                ctx.signal_registry_mut()
                    .register_color(params.signal_name.clone(), read, write);

                Ok(json!({
                    "node_name": params.node_name,
                    "signal_name": params.signal_name,
                    "signal_type": "color",
                    "status": "ready"
                }))
            }
            "f32" => {
                // F32 signals (e.g., opacity) not yet implemented in reactive components
                // For now, just register the signal
                let value: f32 = serde_json::from_value(params.initial_value)?;

                let runtime = flux_state::Runtime::new();
                let signal = Signal::new(runtime, value);
                let (read, write) = signal.split();

                // Register signal for remote control
                ctx.signal_registry_mut()
                    .register_f32(params.signal_name.clone(), read, write);

                Ok(json!({
                    "node_name": params.node_name,
                    "signal_name": params.signal_name,
                    "signal_type": "f32",
                    "status": "signal_registered_only",
                    "note": "Reactive f32 components not yet implemented"
                }))
            }
            "bool" => {
                // Bool signals not yet implemented in reactive components
                let value: bool = serde_json::from_value(params.initial_value)?;

                let runtime = flux_state::Runtime::new();
                let signal = Signal::new(runtime, value);
                let (read, write) = signal.split();

                // Register signal for remote control
                ctx.signal_registry_mut()
                    .register_bool(params.signal_name.clone(), read, write);

                Ok(json!({
                    "node_name": params.node_name,
                    "signal_name": params.signal_name,
                    "signal_type": "bool",
                    "status": "signal_registered_only",
                    "note": "Reactive bool components not yet implemented"
                }))
            }
            _ => Err(anyhow!("Unknown signal type: {}", params.signal_type)),
        }
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

        assert_eq!(node.visible, false);
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

    #[test]
    fn test_assert_node_state_pass() {
        let mut ctx = McpFrameworkContext::new();

        // Create a node
        CreateSceneTool
            .execute(
                json!({
                    "nodes": [{
                        "name": "test_rect",
                        "content": { "type": "Rect", "color": [1.0, 0.0, 0.0, 1.0] },
                        "bounds": { "x": 100.0, "y": 200.0, "width": 300.0, "height": 400.0 },
                        "visible": true,
                        "opacity": 0.75
                    }]
                }),
                &mut ctx,
            )
            .unwrap();

        // Assert state matches
        let tool = AssertNodeStateTool;
        let result = tool
            .execute(
                json!({
                    "node_name": "test_rect",
                    "expected": {
                        "visible": true,
                        "opacity": 0.75,
                        "bounds": { "x": 100.0, "y": 200.0, "width": 300.0, "height": 400.0 },
                        "color": [1.0, 0.0, 0.0, 1.0]
                    }
                }),
                &mut ctx,
            )
            .unwrap();

        assert_eq!(result.get("passed").unwrap().as_bool().unwrap(), true);
        assert_eq!(
            result.get("failures").unwrap().as_array().unwrap().len(),
            0
        );
    }

    #[test]
    fn test_assert_node_state_fail() {
        let mut ctx = McpFrameworkContext::new();

        // Create a node
        CreateSceneTool
            .execute(
                json!({
                    "nodes": [{
                        "name": "test_rect",
                        "content": { "type": "Rect", "color": [1.0, 0.0, 0.0, 1.0] },
                        "bounds": { "x": 100.0, "y": 200.0, "width": 300.0, "height": 400.0 },
                        "visible": true,
                        "opacity": 0.75
                    }]
                }),
                &mut ctx,
            )
            .unwrap();

        // Assert state with wrong values
        let tool = AssertNodeStateTool;
        let result = tool
            .execute(
                json!({
                    "node_name": "test_rect",
                    "expected": {
                        "visible": false,
                        "opacity": 0.5,
                        "color": [0.0, 1.0, 0.0, 1.0]
                    }
                }),
                &mut ctx,
            )
            .unwrap();

        assert_eq!(result.get("passed").unwrap().as_bool().unwrap(), false);
        let failures = result.get("failures").unwrap().as_array().unwrap();
        assert!(failures.len() > 0);
    }

    #[test]
    fn test_assert_node_state_tolerance() {
        let mut ctx = McpFrameworkContext::new();

        // Create a node
        CreateSceneTool
            .execute(
                json!({
                    "nodes": [{
                        "name": "test_rect",
                        "content": { "type": "Rect", "color": [1.0, 0.0, 0.0, 1.0] },
                        "bounds": { "x": 100.0, "y": 200.0, "width": 300.0, "height": 400.0 }
                    }]
                }),
                &mut ctx,
            )
            .unwrap();

        // Assert with slightly different values but within tolerance
        let tool = AssertNodeStateTool;
        let result = tool
            .execute(
                json!({
                    "node_name": "test_rect",
                    "expected": {
                        "bounds": { "x": 100.0001, "y": 200.0001, "width": 300.0, "height": 400.0 }
                    },
                    "tolerance": 0.001
                }),
                &mut ctx,
            )
            .unwrap();

        assert_eq!(result.get("passed").unwrap().as_bool().unwrap(), true);
    }

    #[test]
    fn test_assert_node_state_rounded_rect() {
        let mut ctx = McpFrameworkContext::new();

        // Create a rounded rect
        CreateSceneTool
            .execute(
                json!({
                    "nodes": [{
                        "name": "rounded",
                        "content": {
                            "type": "RoundedRect",
                            "color": [0.0, 0.0, 1.0, 1.0],
                            "corner_radius": 8.0
                        },
                        "bounds": { "x": 0.0, "y": 0.0, "width": 100.0, "height": 100.0 }
                    }]
                }),
                &mut ctx,
            )
            .unwrap();

        // Assert corner radius
        let tool = AssertNodeStateTool;
        let result = tool
            .execute(
                json!({
                    "node_name": "rounded",
                    "expected": {
                        "corner_radius": 8.0,
                        "color": [0.0, 0.0, 1.0, 1.0]
                    }
                }),
                &mut ctx,
            )
            .unwrap();

        assert_eq!(result.get("passed").unwrap().as_bool().unwrap(), true);
    }

    #[test]
    fn test_assert_node_state_not_found() {
        let mut ctx = McpFrameworkContext::new();
        let tool = AssertNodeStateTool;

        let result = tool.execute(
            json!({
                "node_name": "nonexistent",
                "expected": { "visible": true }
            }),
            &mut ctx,
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_verify_render_output_count() {
        let mut ctx = McpFrameworkContext::new();

        // Create a node and spawn as renderable
        CreateSceneTool
            .execute(
                json!({
                    "nodes": [{
                        "name": "rect1",
                        "content": { "type": "Rect", "color": [1.0, 0.0, 0.0, 1.0] },
                        "bounds": { "x": 0.0, "y": 0.0, "width": 100.0, "height": 100.0 }
                    }]
                }),
                &mut ctx,
            )
            .unwrap();

        let node_id = ctx.get_named_node("rect1").unwrap();
        ctx.inner_mut().spawn(node_id).insert(Renderable);

        // Verify render output has 1 instance
        let tool = VerifyRenderOutputTool;
        let result = tool
            .execute(json!({ "expected_count": 1 }), &mut ctx)
            .unwrap();

        assert_eq!(result.get("passed").unwrap().as_bool().unwrap(), true);
        assert_eq!(result.get("actual_count").unwrap().as_u64().unwrap(), 1);
    }

    #[test]
    fn test_verify_render_output_properties() {
        let mut ctx = McpFrameworkContext::new();

        // Create a node
        CreateSceneTool
            .execute(
                json!({
                    "nodes": [{
                        "name": "rect1",
                        "content": { "type": "Rect", "color": [1.0, 0.0, 0.0, 1.0] },
                        "bounds": { "x": 100.0, "y": 200.0, "width": 300.0, "height": 400.0 }
                    }]
                }),
                &mut ctx,
            )
            .unwrap();

        let node_id = ctx.get_named_node("rect1").unwrap();
        ctx.inner_mut().spawn(node_id).insert(Renderable);

        // Verify render output properties
        let tool = VerifyRenderOutputTool;
        let result = tool
            .execute(
                json!({
                    "expected_instances": [{
                        "color": [1.0, 0.0, 0.0, 1.0],
                        "position": { "x": 100.0, "y": 200.0 },
                        "size": { "width": 300.0, "height": 400.0 }
                    }]
                }),
                &mut ctx,
            )
            .unwrap();

        assert_eq!(result.get("passed").unwrap().as_bool().unwrap(), true);
        assert_eq!(
            result.get("failures").unwrap().as_array().unwrap().len(),
            0
        );
    }

    #[test]
    fn test_verify_render_output_fail() {
        let mut ctx = McpFrameworkContext::new();

        // Create a node
        CreateSceneTool
            .execute(
                json!({
                    "nodes": [{
                        "name": "rect1",
                        "content": { "type": "Rect", "color": [1.0, 0.0, 0.0, 1.0] },
                        "bounds": { "x": 100.0, "y": 200.0, "width": 300.0, "height": 400.0 }
                    }]
                }),
                &mut ctx,
            )
            .unwrap();

        let node_id = ctx.get_named_node("rect1").unwrap();
        ctx.inner_mut().spawn(node_id).insert(Renderable);

        // Verify with wrong color
        let tool = VerifyRenderOutputTool;
        let result = tool
            .execute(
                json!({
                    "expected_instances": [{
                        "color": [0.0, 1.0, 0.0, 1.0]
                    }]
                }),
                &mut ctx,
            )
            .unwrap();

        assert_eq!(result.get("passed").unwrap().as_bool().unwrap(), false);
        let failures = result.get("failures").unwrap().as_array().unwrap();
        assert!(failures.len() > 0);
    }

    #[test]
    fn test_verify_render_output_multiple_instances() {
        let mut ctx = McpFrameworkContext::new();

        // Create two nodes
        CreateSceneTool
            .execute(
                json!({
                    "nodes": [
                        {
                            "name": "rect1",
                            "content": { "type": "Rect", "color": [1.0, 0.0, 0.0, 1.0] },
                            "bounds": { "x": 0.0, "y": 0.0, "width": 100.0, "height": 100.0 }
                        },
                        {
                            "name": "rect2",
                            "content": { "type": "Rect", "color": [0.0, 1.0, 0.0, 1.0] },
                            "bounds": { "x": 100.0, "y": 100.0, "width": 100.0, "height": 100.0 }
                        }
                    ]
                }),
                &mut ctx,
            )
            .unwrap();

        let node_id1 = ctx.get_named_node("rect1").unwrap();
        let node_id2 = ctx.get_named_node("rect2").unwrap();
        ctx.inner_mut().spawn(node_id1).insert(Renderable);
        ctx.inner_mut().spawn(node_id2).insert(Renderable);

        // Verify count
        let tool = VerifyRenderOutputTool;
        let result = tool
            .execute(json!({ "expected_count": 2 }), &mut ctx)
            .unwrap();

        assert_eq!(result.get("passed").unwrap().as_bool().unwrap(), true);
        assert_eq!(result.get("actual_count").unwrap().as_u64().unwrap(), 2);
    }

    #[test]
    fn test_setup_reactive_chain_color() {
        let mut ctx = McpFrameworkContext::new();

        // Create a node
        CreateSceneTool
            .execute(
                json!({
                    "nodes": [{
                        "name": "test_rect",
                        "content": { "type": "Rect", "color": [1.0, 0.0, 0.0, 1.0] },
                        "bounds": { "x": 0.0, "y": 0.0, "width": 100.0, "height": 100.0 }
                    }]
                }),
                &mut ctx,
            )
            .unwrap();

        // Setup reactive chain
        let tool = SetupReactiveChainTool;
        let result = tool
            .execute(
                json!({
                    "node_name": "test_rect",
                    "signal_name": "rect_color",
                    "signal_type": "color",
                    "initial_value": [0.0, 1.0, 0.0, 1.0]
                }),
                &mut ctx,
            )
            .unwrap();

        assert_eq!(result.get("status").unwrap().as_str().unwrap(), "ready");
        assert_eq!(
            result.get("signal_type").unwrap().as_str().unwrap(),
            "color"
        );

        // Verify signal was registered
        assert!(ctx.signal_registry().contains("rect_color"));
    }

    #[test]
    fn test_setup_reactive_chain_f32() {
        let mut ctx = McpFrameworkContext::new();

        // Create a node
        CreateSceneTool
            .execute(
                json!({
                    "nodes": [{
                        "name": "test_rect",
                        "content": { "type": "Rect", "color": [1.0, 0.0, 0.0, 1.0] },
                        "bounds": { "x": 0.0, "y": 0.0, "width": 100.0, "height": 100.0 }
                    }]
                }),
                &mut ctx,
            )
            .unwrap();

        // Setup reactive chain for f32
        let tool = SetupReactiveChainTool;
        let result = tool
            .execute(
                json!({
                    "node_name": "test_rect",
                    "signal_name": "opacity_signal",
                    "signal_type": "f32",
                    "initial_value": 0.5
                }),
                &mut ctx,
            )
            .unwrap();

        assert_eq!(
            result.get("status").unwrap().as_str().unwrap(),
            "signal_registered_only"
        );

        // Verify signal was registered
        assert!(ctx.signal_registry().contains("opacity_signal"));
    }

    #[test]
    fn test_setup_reactive_chain_integration() {
        let mut ctx = McpFrameworkContext::new();

        // Create a node
        CreateSceneTool
            .execute(
                json!({
                    "nodes": [{
                        "name": "test_rect",
                        "content": { "type": "Rect", "color": [1.0, 0.0, 0.0, 1.0] },
                        "bounds": { "x": 0.0, "y": 0.0, "width": 100.0, "height": 100.0 }
                    }]
                }),
                &mut ctx,
            )
            .unwrap();

        // Setup reactive chain
        SetupReactiveChainTool
            .execute(
                json!({
                    "node_name": "test_rect",
                    "signal_name": "rect_color",
                    "signal_type": "color",
                    "initial_value": [0.0, 1.0, 0.0, 1.0]
                }),
                &mut ctx,
            )
            .unwrap();

        // Update the signal (via state.set_signal tool)
        use crate::tools::state::SetSignalTool;
        SetSignalTool
            .execute(
                json!({
                    "name": "rect_color",
                    "value": [0.0, 0.0, 1.0, 1.0]
                }),
                &mut ctx,
            )
            .unwrap();

        // Trigger update to propagate signal changes
        ctx.update();

        // Verify the scene node was updated
        let node_id = ctx.get_named_node("test_rect").unwrap();
        let scene = ctx.scene();
        let node = scene.get_node(node_id).unwrap();

        if let NodeContent::Rect { color } = &node.content {
            // Color should be blue now
            assert!((color.b() - 1.0).abs() < 0.001);
            assert!((color.r() - 0.0).abs() < 0.001);
        } else {
            panic!("Expected Rect node");
        }
    }

    #[test]
    fn test_setup_reactive_chain_node_not_found() {
        let mut ctx = McpFrameworkContext::new();
        let tool = SetupReactiveChainTool;

        let result = tool.execute(
            json!({
                "node_name": "nonexistent",
                "signal_name": "test_signal",
                "signal_type": "color",
                "initial_value": [1.0, 0.0, 0.0, 1.0]
            }),
            &mut ctx,
        );

        assert!(result.is_err());
    }
}
