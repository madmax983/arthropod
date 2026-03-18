use crate::context::McpFrameworkContext;
use crate::tools::{Tool, ToolSchema};
use anyhow::{Result, anyhow};
use render_engine::{Color, NodeContent};
use serde::Deserialize;
use serde_json::{Value, json};

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

#[derive(Debug, Deserialize)]
struct BoundsSpec {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}

fn default_tolerance() -> f32 {
    0.001
}

impl AssertNodeStateTool {
    fn check_visibility(expected: Option<bool>, actual: bool, failures: &mut Vec<String>) {
        if let Some(expected_visible) = expected
            && actual != expected_visible
        {
            failures.push(format!(
                "visible: expected {}, got {}",
                expected_visible, actual
            ));
        }
    }

    fn check_opacity(
        expected: Option<f32>,
        actual: f32,
        tolerance: f32,
        failures: &mut Vec<String>,
    ) {
        if let Some(expected_opacity) = expected
            && (actual - expected_opacity).abs() > tolerance
        {
            failures.push(format!(
                "opacity: expected {}, got {} (tolerance: {})",
                expected_opacity, actual, tolerance
            ));
        }
    }

    fn check_bounds(
        expected: &Option<BoundsSpec>,
        bounds: &plat_core::Rect,
        tolerance: f32,
        failures: &mut Vec<String>,
    ) {
        if let Some(expected_bounds) = expected {
            if (bounds.x - expected_bounds.x).abs() > tolerance {
                failures.push(format!(
                    "bounds.x: expected {}, got {} (tolerance: {})",
                    expected_bounds.x, bounds.x, tolerance
                ));
            }
            if (bounds.y - expected_bounds.y).abs() > tolerance {
                failures.push(format!(
                    "bounds.y: expected {}, got {} (tolerance: {})",
                    expected_bounds.y, bounds.y, tolerance
                ));
            }
            if (bounds.width - expected_bounds.width).abs() > tolerance {
                failures.push(format!(
                    "bounds.width: expected {}, got {} (tolerance: {})",
                    expected_bounds.width, bounds.width, tolerance
                ));
            }
            if (bounds.height - expected_bounds.height).abs() > tolerance {
                failures.push(format!(
                    "bounds.height: expected {}, got {} (tolerance: {})",
                    expected_bounds.height, bounds.height, tolerance
                ));
            }
        }
    }

    fn check_color(
        expected: &Option<[f32; 4]>,
        content: &NodeContent,
        tolerance: f32,
        failures: &mut Vec<String>,
    ) {
        if let Some(expected_color) = expected {
            let actual_color_vec4 = match content {
                NodeContent::Styled { style } => {
                    if let Some(render_engine::Paint::Solid(c)) = style.fills.first() {
                        *c
                    } else {
                        failures.push("Node does not have a solid fill color".to_string());
                        return;
                    }
                }
                _ => {
                    failures.push("Node does not have a color property".to_string());
                    return;
                }
            };

            let actual_color = Color::rgba(
                actual_color_vec4.x,
                actual_color_vec4.y,
                actual_color_vec4.z,
                actual_color_vec4.w,
            );
            let expected = Color::rgba(
                expected_color[0],
                expected_color[1],
                expected_color[2],
                expected_color[3],
            );

            let mut check_channel = |name: &str, expected_val: f32, actual_val: f32| {
                if (actual_val - expected_val).abs() > tolerance {
                    failures.push(format!(
                        "color.{}: expected {}, got {} (tolerance: {})",
                        name, expected_val, actual_val, tolerance
                    ));
                }
            };

            check_channel("r", expected.r(), actual_color.r());
            check_channel("g", expected.g(), actual_color.g());
            check_channel("b", expected.b(), actual_color.b());
            check_channel("a", expected.a(), actual_color.a());
        }
    }

    fn check_corner_radius(
        expected: Option<f32>,
        content: &NodeContent,
        tolerance: f32,
        failures: &mut Vec<String>,
    ) {
        if let Some(expected_radius) = expected {
            match content {
                NodeContent::Styled { style } => {
                    let actual_radius = style.corner_radii.top_left; // Use top_left as representative
                    if (actual_radius - expected_radius).abs() > tolerance {
                        failures.push(format!(
                            "corner_radius: expected {}, got {} (tolerance: {})",
                            expected_radius, actual_radius, tolerance
                        ));
                    }
                }
                _ => {
                    failures.push("Node is not a Styled node".to_string());
                }
            }
        }
    }
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
        Self::check_visibility(params.expected.visible, node.visible, &mut failures);

        // Check opacity
        Self::check_opacity(
            params.expected.opacity,
            node.opacity,
            params.tolerance,
            &mut failures,
        );

        // Check bounds
        Self::check_bounds(
            &params.expected.bounds,
            &node.bounds,
            params.tolerance,
            &mut failures,
        );

        // Check color (extract from NodeContent)
        Self::check_color(
            &params.expected.color,
            &node.content,
            params.tolerance,
            &mut failures,
        );

        // Check corner_radius
        Self::check_corner_radius(
            params.expected.corner_radius,
            &node.content,
            params.tolerance,
            &mut failures,
        );
        let passed = failures.is_empty();

        Ok(json!({
            "passed": passed,
            "node_name": params.node_name,
            "failures": failures
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::McpFrameworkContext;
    // We need CreateSceneTool for setup
    use crate::tools::test::create_scene::CreateSceneTool;

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

        assert!(result.get("passed").unwrap().as_bool().unwrap());
        assert_eq!(result.get("failures").unwrap().as_array().unwrap().len(), 0);
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

        assert!(!result.get("passed").unwrap().as_bool().unwrap());
        let failures = result.get("failures").unwrap().as_array().unwrap();
        assert!(!failures.is_empty());
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

        assert!(result.get("passed").unwrap().as_bool().unwrap());
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

        assert!(result.get("passed").unwrap().as_bool().unwrap());
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
}
