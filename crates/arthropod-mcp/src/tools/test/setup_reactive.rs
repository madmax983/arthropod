use crate::context::McpFrameworkContext;
use crate::tools::{Tool, ToolSchema};
use anyhow::{Result, anyhow};
use arthropod_ecs::{ReactiveColor, Renderable};
use flux_state::Signal;
use render_engine::Color;
use serde::Deserialize;
use serde_json::{Value, json};

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
                let color = Color::rgba(
                    color_array[0],
                    color_array[1],
                    color_array[2],
                    color_array[3],
                );

                let runtime = flux_state::Runtime::new();
                let signal = Signal::new(runtime, color);
                let (read, write) = signal.split();

                // Spawn entity with ReactiveColor component
                ctx.inner_mut()
                    .spawn(node_id)
                    .insert(Renderable)
                    .insert(ReactiveColor::new(read.clone()));

                // Register signal for remote control
                ctx.signal_registry_mut().register_color(
                    params.signal_name.clone(),
                    read,
                    write,
                )?;

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
                    .register_f32(params.signal_name.clone(), read, write)?;

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
                    .register_bool(params.signal_name.clone(), read, write)?;

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
    use crate::tools::test::create_scene::CreateSceneTool;
    use render_engine::NodeContent;

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

        if let NodeContent::Styled { style } = &node.content {
            if let Some(render_engine::Paint::Solid(color)) = style.fills.first() {
                // Color should be blue now
                assert!((color.z - 1.0).abs() < 0.001);
                assert!((color.x - 0.0).abs() < 0.001);
            } else {
                panic!("Expected solid fill");
            }
        } else {
            panic!("Expected Styled node");
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
