use crate::context::McpFrameworkContext;
use crate::tools::{Tool, ToolSchema};
use anyhow::Result;
use serde::Deserialize;
use serde_json::{Value, json};

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

fn default_tolerance() -> f32 {
    0.001
}

impl VerifyRenderOutputTool {
    fn check_color(
        idx: usize,
        expected: &[f32; 4],
        actual: &[f32; 4],
        tolerance: f32,
        failures: &mut Vec<String>,
    ) {
        for (i, comp) in ["r", "g", "b", "a"].iter().enumerate() {
            if !(actual[i] - expected[i]).abs().le(&tolerance) {
                failures.push(format!(
                    "instance[{}].color.{}: expected {}, got {} (tolerance: {})",
                    idx, comp, expected[i], actual[i], tolerance
                ));
            }
        }
    }

    fn check_position(
        idx: usize,
        expected: &PositionSpec,
        actual: &[f32; 2],
        tolerance: f32,
        failures: &mut Vec<String>,
    ) {
        if !(actual[0] - expected.x).abs().le(&tolerance) {
            failures.push(format!(
                "instance[{}].position.x: expected {}, got {} (tolerance: {})",
                idx, expected.x, actual[0], tolerance
            ));
        }
        if !(actual[1] - expected.y).abs().le(&tolerance) {
            failures.push(format!(
                "instance[{}].position.y: expected {}, got {} (tolerance: {})",
                idx, expected.y, actual[1], tolerance
            ));
        }
    }

    fn check_size(
        idx: usize,
        expected: &SizeSpec,
        actual: &[f32; 2],
        tolerance: f32,
        failures: &mut Vec<String>,
    ) {
        if !(actual[0] - expected.width).abs().le(&tolerance) {
            failures.push(format!(
                "instance[{}].size.width: expected {}, got {} (tolerance: {})",
                idx, expected.width, actual[0], tolerance
            ));
        }
        if !(actual[1] - expected.height).abs().le(&tolerance) {
            failures.push(format!(
                "instance[{}].size.height: expected {}, got {} (tolerance: {})",
                idx, expected.height, actual[1], tolerance
            ));
        }
    }
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

        // Run update (which includes render collection) then extract instances
        ctx.update();
        let (instances, _duration) = ctx.render();

        let mut failures = Vec::new();

        // Check expected count
        if params
            .expected_count
            .is_some_and(|count| instances.len() != count)
        {
            failures.push(format!(
                "instance_count: expected {}, got {}",
                params.expected_count.unwrap(),
                instances.len()
            ));
        }

        // Check individual instances
        for (idx, expected) in params.expected_instances.iter().enumerate() {
            if idx >= instances.len() {
                failures.push(format!("Missing instance at index {}", idx));
                continue;
            }

            let actual = &instances[idx];

            if let Some(expected_color) = &expected.color {
                Self::check_color(
                    idx,
                    expected_color,
                    &actual.color,
                    params.tolerance,
                    &mut failures,
                );
            }

            if let Some(expected_pos) = &expected.position {
                Self::check_position(
                    idx,
                    expected_pos,
                    &actual.pos,
                    params.tolerance,
                    &mut failures,
                );
            }

            if let Some(expected_size) = &expected.size {
                Self::check_size(
                    idx,
                    expected_size,
                    &actual.size,
                    params.tolerance,
                    &mut failures,
                );
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::McpFrameworkContext;
    use crate::tools::test::create_scene::CreateSceneTool;
    use arthropod_ecs::Renderable;

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

        assert!(result.get("passed").unwrap().as_bool().unwrap());
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

        assert!(result.get("passed").unwrap().as_bool().unwrap());
        assert_eq!(result.get("failures").unwrap().as_array().unwrap().len(), 0);
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

        assert!(!result.get("passed").unwrap().as_bool().unwrap());
        let failures = result.get("failures").unwrap().as_array().unwrap();
        assert!(!failures.is_empty());
    }

    #[test]
    fn test_check_color_nan() {
        let mut failures = Vec::new();
        VerifyRenderOutputTool::check_color(
            0,
            &[1.0, 0.0, 0.0, 1.0],
            &[std::f32::NAN, 0.0, 0.0, 1.0],
            0.001,
            &mut failures,
        );
        assert!(!failures.is_empty(), "NaN should cause a failure");
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

        assert!(result.get("passed").unwrap().as_bool().unwrap());
        assert_eq!(result.get("actual_count").unwrap().as_u64().unwrap(), 2);
    }
}
