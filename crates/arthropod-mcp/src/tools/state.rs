//! State manipulation and signal control tools.

use crate::context::McpFrameworkContext;
use crate::tools::{Tool, ToolSchema};
use anyhow::{anyhow, Result};
use flux_state::Signal;
use render_engine::Color;
use serde::Deserialize;
use serde_json::{json, Value};

// ============================================================================
// state.register_signal
// ============================================================================

/// Register a signal for remote control
pub struct RegisterSignalTool;

#[derive(Debug, Deserialize)]
struct RegisterSignalParams {
    name: String,
    signal_type: String,
    initial_value: Value,
}

impl Tool for RegisterSignalTool {
    fn name(&self) -> &str {
        "state.register_signal"
    }

    fn schema(&self) -> ToolSchema {
        ToolSchema {
            name: self.name().to_string(),
            description: "Register a new signal for remote control".to_string(),
            parameters: json!({
                "type": "object",
                "required": ["name", "signal_type", "initial_value"],
                "properties": {
                    "name": {
                        "type": "string",
                        "description": "Unique name for the signal"
                    },
                    "signal_type": {
                        "type": "string",
                        "enum": ["color", "f32", "bool"],
                        "description": "Type of signal to create"
                    },
                    "initial_value": {
                        "description": "Initial value for the signal"
                    }
                }
            }),
        }
    }

    fn execute(&self, params: Value, ctx: &mut McpFrameworkContext) -> Result<Value> {
        let params: RegisterSignalParams = serde_json::from_value(params)?;

        // Check if signal already exists
        if ctx.signal_registry().contains(&params.name) {
            return Err(anyhow!("Signal '{}' already exists", params.name));
        }

        match params.signal_type.as_str() {
            "color" => {
                let color_array: [f32; 4] = serde_json::from_value(params.initial_value)?;
                let color = Color::rgba(color_array[0], color_array[1], color_array[2], color_array[3]);

                let runtime = flux_state::Runtime::new();
                let signal = Signal::new(runtime, color);
                let (read, write) = signal.split();

                ctx.signal_registry_mut().register_color(params.name.clone(), read, write);
            }
            "f32" => {
                let value: f32 = serde_json::from_value(params.initial_value)?;

                let runtime = flux_state::Runtime::new();
                let signal = Signal::new(runtime, value);
                let (read, write) = signal.split();

                ctx.signal_registry_mut().register_f32(params.name.clone(), read, write);
            }
            "bool" => {
                let value: bool = serde_json::from_value(params.initial_value)?;

                let runtime = flux_state::Runtime::new();
                let signal = Signal::new(runtime, value);
                let (read, write) = signal.split();

                ctx.signal_registry_mut().register_bool(params.name.clone(), read, write);
            }
            _ => return Err(anyhow!("Unknown signal type: {}", params.signal_type)),
        }

        Ok(json!({
            "name": params.name,
            "type": params.signal_type,
            "status": "registered"
        }))
    }
}

// ============================================================================
// state.set_signal
// ============================================================================

/// Update a signal value
pub struct SetSignalTool;

#[derive(Debug, Deserialize)]
struct SetSignalParams {
    name: String,
    value: Value,
}

impl Tool for SetSignalTool {
    fn name(&self) -> &str {
        "state.set_signal"
    }

    fn schema(&self) -> ToolSchema {
        ToolSchema {
            name: self.name().to_string(),
            description: "Update a registered signal value".to_string(),
            parameters: json!({
                "type": "object",
                "required": ["name", "value"],
                "properties": {
                    "name": {
                        "type": "string",
                        "description": "Signal name"
                    },
                    "value": {
                        "description": "New value (type must match signal type)"
                    }
                }
            }),
        }
    }

    fn execute(&self, params: Value, ctx: &mut McpFrameworkContext) -> Result<Value> {
        let params: SetSignalParams = serde_json::from_value(params)?;

        let signal_type = ctx
            .signal_registry()
            .get_type(&params.name)
            .ok_or_else(|| anyhow!("Signal '{}' not found", params.name))?;

        let new_value = params.value.clone();
        let old_value = match signal_type {
            "color" => {
                let color_array: [f32; 4] = serde_json::from_value(params.value)?;
                let color = Color::rgba(color_array[0], color_array[1], color_array[2], color_array[3]);
                let old = ctx.signal_registry_mut().set_color(&params.name, color)?;
                json!([old.r(), old.g(), old.b(), old.a()])
            }
            "f32" => {
                let value: f32 = serde_json::from_value(params.value)?;
                let old = ctx.signal_registry_mut().set_f32(&params.name, value)?;
                json!(old)
            }
            "bool" => {
                let value: bool = serde_json::from_value(params.value)?;
                let old = ctx.signal_registry_mut().set_bool(&params.name, value)?;
                json!(old)
            }
            _ => return Err(anyhow!("Unknown signal type: {}", signal_type)),
        };

        Ok(json!({
            "name": params.name,
            "old_value": old_value,
            "new_value": new_value
        }))
    }
}

// ============================================================================
// state.get_signal
// ============================================================================

/// Read a signal value (non-reactive)
pub struct GetSignalTool;

#[derive(Debug, Deserialize)]
struct GetSignalParams {
    name: String,
}

impl Tool for GetSignalTool {
    fn name(&self) -> &str {
        "state.get_signal"
    }

    fn schema(&self) -> ToolSchema {
        ToolSchema {
            name: self.name().to_string(),
            description: "Read current signal value (non-reactive)".to_string(),
            parameters: json!({
                "type": "object",
                "required": ["name"],
                "properties": {
                    "name": {
                        "type": "string",
                        "description": "Signal name"
                    }
                }
            }),
        }
    }

    fn execute(&self, params: Value, ctx: &mut McpFrameworkContext) -> Result<Value> {
        let params: GetSignalParams = serde_json::from_value(params)?;

        let signal_type = ctx
            .signal_registry()
            .get_type(&params.name)
            .ok_or_else(|| anyhow!("Signal '{}' not found", params.name))?;

        let value = match signal_type {
            "color" => {
                let color = ctx.signal_registry().get_color(&params.name)?;
                json!([color.r(), color.g(), color.b(), color.a()])
            }
            "f32" => {
                let value = ctx.signal_registry().get_f32(&params.name)?;
                json!(value)
            }
            "bool" => {
                let value = ctx.signal_registry().get_bool(&params.name)?;
                json!(value)
            }
            _ => return Err(anyhow!("Unknown signal type: {}", signal_type)),
        };

        Ok(json!({
            "name": params.name,
            "type": signal_type,
            "value": value
        }))
    }
}

// ============================================================================
// state.trigger_update
// ============================================================================

/// Run ECS update systems
pub struct TriggerUpdateTool;

impl Tool for TriggerUpdateTool {
    fn name(&self) -> &str {
        "state.trigger_update"
    }

    fn schema(&self) -> ToolSchema {
        ToolSchema {
            name: self.name().to_string(),
            description: "Run ECS update systems (processes reactive signals)".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {}
            }),
        }
    }

    fn execute(&self, _params: Value, ctx: &mut McpFrameworkContext) -> Result<Value> {
        let duration = ctx.update();

        Ok(json!({
            "duration_us": duration.as_micros(),
            "duration_ms": duration.as_secs_f64() * 1000.0
        }))
    }
}

// ============================================================================
// state.trigger_render
// ============================================================================

/// Run render systems
pub struct TriggerRenderTool;

impl Tool for TriggerRenderTool {
    fn name(&self) -> &str {
        "state.trigger_render"
    }

    fn schema(&self) -> ToolSchema {
        ToolSchema {
            name: self.name().to_string(),
            description: "Run render systems and generate instances".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {}
            }),
        }
    }

    fn execute(&self, _params: Value, ctx: &mut McpFrameworkContext) -> Result<Value> {
        let (instances, duration) = ctx.render();

        Ok(json!({
            "instance_count": instances.len(),
            "duration_us": duration.as_micros(),
            "duration_ms": duration.as_secs_f64() * 1000.0
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::McpFrameworkContext;

    #[test]
    fn test_register_color_signal() {
        let mut ctx = McpFrameworkContext::new();
        let tool = RegisterSignalTool;

        let result = tool
            .execute(
                json!({
                    "name": "test_color",
                    "signal_type": "color",
                    "initial_value": [1.0, 0.0, 0.0, 1.0]
                }),
                &mut ctx,
            )
            .unwrap();

        assert_eq!(result.get("status").unwrap().as_str().unwrap(), "registered");
        assert!(ctx.signal_registry().contains("test_color"));
    }

    #[test]
    fn test_register_f32_signal() {
        let mut ctx = McpFrameworkContext::new();
        let tool = RegisterSignalTool;

        let result = tool
            .execute(
                json!({
                    "name": "opacity",
                    "signal_type": "f32",
                    "initial_value": 0.5
                }),
                &mut ctx,
            )
            .unwrap();

        assert_eq!(result.get("status").unwrap().as_str().unwrap(), "registered");
        assert!(ctx.signal_registry().contains("opacity"));
    }

    #[test]
    fn test_register_signal_duplicate_error() {
        let mut ctx = McpFrameworkContext::new();
        let tool = RegisterSignalTool;

        // Register first time
        tool.execute(
            json!({
                "name": "test",
                "signal_type": "f32",
                "initial_value": 1.0
            }),
            &mut ctx,
        )
        .unwrap();

        // Try to register again
        let result = tool.execute(
            json!({
                "name": "test",
                "signal_type": "f32",
                "initial_value": 2.0
            }),
            &mut ctx,
        );

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("already exists"));
    }

    #[test]
    fn test_set_signal() {
        let mut ctx = McpFrameworkContext::new();

        // Register signal
        let register_tool = RegisterSignalTool;
        register_tool
            .execute(
                json!({
                    "name": "test_color",
                    "signal_type": "color",
                    "initial_value": [1.0, 0.0, 0.0, 1.0]
                }),
                &mut ctx,
            )
            .unwrap();

        // Set new value
        let set_tool = SetSignalTool;
        let result = set_tool
            .execute(
                json!({
                    "name": "test_color",
                    "value": [0.0, 1.0, 0.0, 1.0]
                }),
                &mut ctx,
            )
            .unwrap();

        let old_value = result.get("old_value").unwrap().as_array().unwrap();
        assert_eq!(old_value[0].as_f64().unwrap(), 1.0);
        assert_eq!(old_value[1].as_f64().unwrap(), 0.0);
    }

    #[test]
    fn test_get_signal() {
        let mut ctx = McpFrameworkContext::new();

        // Register signal
        let register_tool = RegisterSignalTool;
        register_tool
            .execute(
                json!({
                    "name": "opacity",
                    "signal_type": "f32",
                    "initial_value": 0.75
                }),
                &mut ctx,
            )
            .unwrap();

        // Get value
        let get_tool = GetSignalTool;
        let result = get_tool
            .execute(json!({"name": "opacity"}), &mut ctx)
            .unwrap();

        assert_eq!(result.get("value").unwrap().as_f64().unwrap(), 0.75);
        assert_eq!(result.get("type").unwrap().as_str().unwrap(), "f32");
    }

    #[test]
    fn test_get_signal_not_found() {
        let mut ctx = McpFrameworkContext::new();
        let tool = GetSignalTool;

        let result = tool.execute(json!({"name": "nonexistent"}), &mut ctx);
        assert!(result.is_err());
    }

    #[test]
    fn test_trigger_update() {
        let mut ctx = McpFrameworkContext::new();
        let tool = TriggerUpdateTool;

        let result = tool.execute(json!({}), &mut ctx).unwrap();

        assert!(result.get("duration_us").is_some());
        assert!(result.get("duration_ms").is_some());
    }

    #[test]
    fn test_trigger_render() {
        let mut ctx = McpFrameworkContext::new();
        let tool = TriggerRenderTool;

        let result = tool.execute(json!({}), &mut ctx).unwrap();

        assert!(result.get("instance_count").is_some());
        assert!(result.get("duration_us").is_some());
    }

    #[test]
    fn test_signal_workflow() {
        let mut ctx = McpFrameworkContext::new();

        // 1. Register signal
        RegisterSignalTool
            .execute(
                json!({
                    "name": "workflow_test",
                    "signal_type": "f32",
                    "initial_value": 1.0
                }),
                &mut ctx,
            )
            .unwrap();

        // 2. Get initial value
        let result = GetSignalTool
            .execute(json!({"name": "workflow_test"}), &mut ctx)
            .unwrap();
        assert_eq!(result.get("value").unwrap().as_f64().unwrap(), 1.0);

        // 3. Set new value
        SetSignalTool
            .execute(
                json!({
                    "name": "workflow_test",
                    "value": 2.5
                }),
                &mut ctx,
            )
            .unwrap();

        // 4. Verify new value
        let result = GetSignalTool
            .execute(json!({"name": "workflow_test"}), &mut ctx)
            .unwrap();
        assert_eq!(result.get("value").unwrap().as_f64().unwrap(), 2.5);

        // 5. Trigger update
        let result = TriggerUpdateTool.execute(json!({}), &mut ctx).unwrap();
        assert!(result.get("duration_us").unwrap().as_u64().unwrap() < 10000);
    }
}
