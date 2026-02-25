//! Scene inspection and manipulation tools.

use crate::context::McpFrameworkContext;
use crate::tools::{Tool, ToolSchema};
use crate::validation::{validate_finite, validate_range};
use anyhow::{Result, anyhow};
use render_engine::{NodeContent, NodeId};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::HashSet;

// ============================================================================
// scene.list_nodes
// ============================================================================

/// List all nodes with optional filtering
pub struct ListNodesTool;

#[derive(Debug, Deserialize)]
struct ListNodesParams {
    #[serde(default)]
    filter: ListNodesFilter,
}

#[derive(Debug, Default, Deserialize)]
struct ListNodesFilter {
    #[serde(default)]
    visible_only: bool,

    content_type: Option<String>,

    parent_id: Option<u64>,
}

#[derive(Debug, Serialize)]
struct NodeSummary {
    id: u64,
    content_type: String,
    visible: bool,
    opacity: f32,
    bounds: BoundsData,
}

#[derive(Debug, Serialize)]
struct BoundsData {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}

impl Tool for ListNodesTool {
    fn name(&self) -> &str {
        "scene.list_nodes"
    }

    fn schema(&self) -> ToolSchema {
        ToolSchema {
            name: self.name().to_string(),
            description: "List all scene nodes with optional filtering".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "filter": {
                        "type": "object",
                        "properties": {
                            "visible_only": { "type": "boolean" },
                            "content_type": { "type": "string" },
                            "parent_id": { "type": "number" }
                        }
                    }
                }
            }),
        }
    }

    fn execute(&self, params: Value, ctx: &mut McpFrameworkContext) -> Result<Value> {
        let params: ListNodesParams =
            serde_json::from_value(params).map_err(|e| anyhow!("Invalid parameters: {}", e))?;

        let scene = ctx.scene();
        let mut nodes = Vec::new();

        for (node_id, node) in scene.nodes() {
            // Apply filters
            if params.filter.visible_only && !node.visible {
                continue;
            }

            if let Some(ref content_type) = params.filter.content_type {
                let node_type = match &node.content {
                    NodeContent::Styled { style } => {
                        if style.text.is_some() {
                            "Text"
                        } else if !style.corner_radii.is_zero() {
                            "RoundedRect"
                        } else if !style.fills.is_empty() {
                            "Rect"
                        } else {
                            "Empty"
                        }
                    }
                    NodeContent::SolidColor { .. } => "Rect",
                    NodeContent::Empty => "Empty",
                };
                if node_type != content_type {
                    continue;
                }
            }

            if let Some(parent_id) = params.filter.parent_id {
                // Find parent by checking which node has this as a child
                let parent = scene
                    .nodes()
                    .find(|(_, n)| n.children.contains(&node_id))
                    .map(|(id, _)| id);
                match parent {
                    Some(pid) if pid.0 == parent_id => {}
                    _ => continue,
                }
            }

            let content_type = match &node.content {
                NodeContent::Styled { style } => {
                    if style.text.is_some() {
                        "Text"
                    } else if !style.corner_radii.is_zero() {
                        "RoundedRect"
                    } else if !style.fills.is_empty() {
                        "Rect"
                    } else {
                        "Empty"
                    }
                }
                NodeContent::SolidColor { .. } => "Rect",
                NodeContent::Empty => "Empty",
            };

            nodes.push(NodeSummary {
                id: node_id.0,
                content_type: content_type.to_string(),
                visible: node.visible,
                opacity: node.opacity,
                bounds: BoundsData {
                    x: node.bounds.x,
                    y: node.bounds.y,
                    width: node.bounds.width,
                    height: node.bounds.height,
                },
            });
        }

        Ok(json!({ "nodes": nodes }))
    }
}

// ============================================================================
// scene.get_node
// ============================================================================

/// Get detailed information about a specific node
pub struct GetNodeTool;

#[derive(Debug, Deserialize)]
struct GetNodeParams {
    id: u64,
}

#[derive(Debug, Serialize)]
struct NodeDetail {
    id: u64,
    content: NodeContentData,
    bounds: BoundsData,
    visible: bool,
    opacity: f32,
    parent_id: Option<u64>,
    children: Vec<u64>,
}

#[derive(Debug, Serialize)]
#[serde(tag = "type")]
enum NodeContentData {
    Rect {
        color: [f32; 4],
    },
    RoundedRect {
        color: [f32; 4],
        corner_radius: f32,
    },
    Text {
        text: String,
        font_size: f32,
        color: [f32; 4],
    },
    Empty,
}

impl Tool for GetNodeTool {
    fn name(&self) -> &str {
        "scene.get_node"
    }

    fn schema(&self) -> ToolSchema {
        ToolSchema {
            name: self.name().to_string(),
            description: "Get detailed information about a specific node".to_string(),
            parameters: json!({
                "type": "object",
                "required": ["id"],
                "properties": {
                    "id": { "type": "number", "description": "Node ID" }
                }
            }),
        }
    }

    fn execute(&self, params: Value, ctx: &mut McpFrameworkContext) -> Result<Value> {
        let params: GetNodeParams = serde_json::from_value(params)?;
        let node_id = NodeId(params.id);

        let scene = ctx.scene();
        let node = scene
            .get_node(node_id)
            .ok_or_else(|| anyhow!("Node {} not found", params.id))?;

        let content = match &node.content {
            NodeContent::Styled { style } => {
                // Extract color from first fill if present
                let color = if let Some(render_engine::Paint::Solid(c)) = style.fills.first() {
                    [c.x, c.y, c.z, c.w]
                } else {
                    [0.0, 0.0, 0.0, 1.0]
                };

                if let Some(ref text_content) = style.text {
                    NodeContentData::Text {
                        text: text_content.text.clone(),
                        font_size: text_content.font_size,
                        color,
                    }
                } else if !style.corner_radii.is_zero() {
                    NodeContentData::RoundedRect {
                        color,
                        corner_radius: style.corner_radii.top_left, // Use top_left as representative
                    }
                } else if !style.fills.is_empty() {
                    NodeContentData::Rect { color }
                } else {
                    NodeContentData::Empty
                }
            }
            NodeContent::SolidColor { color } => NodeContentData::Rect {
                color: color.to_array(),
            },
            NodeContent::Empty => NodeContentData::Empty,
        };

        // Find parent by checking which node has this as a child
        let parent_id = scene
            .nodes()
            .find(|(_, n)| n.children.contains(&node_id))
            .map(|(id, _)| id.0);
        let children: Vec<u64> = node.children.iter().map(|c| c.0).collect();

        let detail = NodeDetail {
            id: params.id,
            content,
            bounds: BoundsData {
                x: node.bounds.x,
                y: node.bounds.y,
                width: node.bounds.width,
                height: node.bounds.height,
            },
            visible: node.visible,
            opacity: node.opacity,
            parent_id,
            children,
        };

        Ok(serde_json::to_value(detail)?)
    }
}

// ============================================================================
// scene.query_hierarchy
// ============================================================================

/// Query the scene hierarchy tree
pub struct QueryHierarchyTool;

#[derive(Debug, Deserialize)]
struct QueryHierarchyParams {
    root_id: Option<u64>,
    #[serde(default = "default_max_depth")]
    max_depth: usize,
}

fn default_max_depth() -> usize {
    10
}

const MAX_QUERY_DEPTH: usize = 32;
const MAX_RESPONSE_NODES: usize = 5000;

#[derive(Debug, Serialize)]
struct HierarchyNode {
    id: u64,
    content_type: String,
    visible: bool,
    children: Vec<HierarchyNode>,
}

impl Tool for QueryHierarchyTool {
    fn name(&self) -> &str {
        "scene.query_hierarchy"
    }

    fn schema(&self) -> ToolSchema {
        ToolSchema {
            name: self.name().to_string(),
            description: "Get hierarchical tree structure of scene nodes".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "root_id": { "type": "number", "description": "Root node ID (default: scene root)" },
                    "max_depth": { "type": "number", "description": "Maximum depth to traverse (default: 10)" }
                }
            }),
        }
    }

    fn execute(&self, params: Value, ctx: &mut McpFrameworkContext) -> Result<Value> {
        let params: QueryHierarchyParams = serde_json::from_value(params)?;

        if params.max_depth > MAX_QUERY_DEPTH {
            return Err(anyhow!(
                "Max depth cannot exceed {} (requested: {})",
                MAX_QUERY_DEPTH,
                params.max_depth
            ));
        }

        let scene = ctx.scene();
        let root_id = params.root_id.map(NodeId).unwrap_or_else(|| scene.root());

        let mut node_count = 0;
        let mut visited = HashSet::new();

        fn build_hierarchy(
            scene: &render_engine::Scene,
            node_id: NodeId,
            depth: usize,
            max_depth: usize,
            node_count: &mut usize,
            visited: &mut HashSet<NodeId>,
        ) -> Option<HierarchyNode> {
            if depth >= max_depth {
                return None;
            }

            if *node_count >= MAX_RESPONSE_NODES {
                return None;
            }

            if !visited.insert(node_id) {
                return None;
            }

            *node_count += 1;

            let node = scene.get_node(node_id)?;

            let content_type = match &node.content {
                NodeContent::Styled { style } => {
                    if style.text.is_some() {
                        "Text"
                    } else if !style.corner_radii.is_zero() {
                        "RoundedRect"
                    } else if !style.fills.is_empty() {
                        "Rect"
                    } else {
                        "Empty"
                    }
                }
                NodeContent::SolidColor { .. } => "Rect",
                NodeContent::Empty => "Empty",
            };

            let children: Vec<HierarchyNode> = node
                .children
                .iter()
                .filter_map(|child_id| {
                    build_hierarchy(scene, *child_id, depth + 1, max_depth, node_count, visited)
                })
                .collect();

            Some(HierarchyNode {
                id: node_id.0,
                content_type: content_type.to_string(),
                visible: node.visible,
                children,
            })
        }

        let hierarchy = build_hierarchy(
            scene,
            root_id,
            0,
            params.max_depth,
            &mut node_count,
            &mut visited,
        )
        .ok_or_else(|| anyhow!("Failed to build hierarchy"))?;

        Ok(serde_json::to_value(hierarchy)?)
    }
}

// ============================================================================
// scene.find_nodes_at_position
// ============================================================================

/// Find nodes at a specific screen position
pub struct FindNodesAtPositionTool;

#[derive(Debug, Deserialize)]
struct FindNodesAtPositionParams {
    x: f32,
    y: f32,
}

impl Tool for FindNodesAtPositionTool {
    fn name(&self) -> &str {
        "scene.find_nodes_at_position"
    }

    fn schema(&self) -> ToolSchema {
        ToolSchema {
            name: self.name().to_string(),
            description: "Find nodes intersecting a screen position (sorted by z-index)"
                .to_string(),
            parameters: json!({
                "type": "object",
                "required": ["x", "y"],
                "properties": {
                    "x": { "type": "number" },
                    "y": { "type": "number" }
                }
            }),
        }
    }

    fn execute(&self, params: Value, ctx: &mut McpFrameworkContext) -> Result<Value> {
        let params: FindNodesAtPositionParams = serde_json::from_value(params)?;

        let x = validate_finite(params.x, "x")?;
        let y = validate_finite(params.y, "y")?;

        let scene = ctx.scene();
        let mut nodes_at_pos: Vec<NodeId> = Vec::new();

        for (node_id, node) in scene.nodes() {
            if !node.visible {
                continue;
            }

            // Check if point is inside bounds
            let bounds = &node.bounds;
            if x >= bounds.x
                && x <= bounds.x + bounds.width
                && y >= bounds.y
                && y <= bounds.y + bounds.height
            {
                nodes_at_pos.push(node_id);
            }
        }

        let node_ids: Vec<u64> = nodes_at_pos.iter().map(|id| id.0).collect();

        Ok(json!({ "nodes": node_ids }))
    }
}

// ============================================================================
// scene.update_node
// ============================================================================

/// Update node properties
pub struct UpdateNodeTool;

#[derive(Debug, Deserialize)]
struct UpdateNodeParams {
    id: u64,
    #[serde(default)]
    updates: NodeUpdates,
}

#[derive(Debug, Default, Deserialize)]
struct NodeUpdates {
    visible: Option<bool>,
    opacity: Option<f32>,
    bounds: Option<BoundsUpdate>,
    color: Option<[f32; 4]>,
}

#[derive(Debug, Deserialize)]
struct BoundsUpdate {
    x: Option<f32>,
    y: Option<f32>,
    width: Option<f32>,
    height: Option<f32>,
}

impl Tool for UpdateNodeTool {
    fn name(&self) -> &str {
        "scene.update_node"
    }

    fn schema(&self) -> ToolSchema {
        ToolSchema {
            name: self.name().to_string(),
            description: "Update properties of a scene node".to_string(),
            parameters: json!({
                "type": "object",
                "required": ["id"],
                "properties": {
                    "id": { "type": "number" },
                    "updates": {
                        "type": "object",
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
                            }
                        }
                    }
                }
            }),
        }
    }

    fn execute(&self, params: Value, ctx: &mut McpFrameworkContext) -> Result<Value> {
        let params: UpdateNodeParams = serde_json::from_value(params)?;
        let node_id = NodeId(params.id);

        let scene = ctx.scene_mut();
        let node = scene
            .get_node_mut(node_id)
            .ok_or_else(|| anyhow!("Node {} not found", params.id))?;

        let mut updated_fields = Vec::new();

        if let Some(visible) = params.updates.visible {
            node.visible = visible;
            updated_fields.push("visible");
        }

        if let Some(opacity) = params.updates.opacity {
            node.opacity = validate_range(opacity, 0.0, 1.0, "opacity")?;
            updated_fields.push("opacity");
        }

        if let Some(bounds_update) = params.updates.bounds {
            let mut new_bounds = node.bounds;
            if let Some(x) = bounds_update.x {
                validate_finite(x, "bounds.x")?;
                new_bounds =
                    plat_core::Rect::new(x, new_bounds.y, new_bounds.width, new_bounds.height);
                updated_fields.push("bounds.x");
            }
            if let Some(y) = bounds_update.y {
                validate_finite(y, "bounds.y")?;
                new_bounds =
                    plat_core::Rect::new(new_bounds.x, y, new_bounds.width, new_bounds.height);
                updated_fields.push("bounds.y");
            }
            if let Some(width) = bounds_update.width {
                validate_finite(width, "bounds.width")?;
                new_bounds =
                    plat_core::Rect::new(new_bounds.x, new_bounds.y, width, new_bounds.height);
                updated_fields.push("bounds.width");
            }
            if let Some(height) = bounds_update.height {
                validate_finite(height, "bounds.height")?;
                new_bounds =
                    plat_core::Rect::new(new_bounds.x, new_bounds.y, new_bounds.width, height);
                updated_fields.push("bounds.height");
            }
            node.bounds = new_bounds;
        }

        if let Some(color_array) = params.updates.color {
            for (i, &c) in color_array.iter().enumerate() {
                validate_finite(c, &format!("color[{}]", i))?;
            }

            let color = render_engine::Color::rgba(
                color_array[0],
                color_array[1],
                color_array[2],
                color_array[3],
            );

            match &mut node.content {
                NodeContent::Styled { style } => {
                    if style.fills.is_empty() {
                        return Err(anyhow!("Cannot set color on node without fills"));
                    }
                    style.fills[0] = render_engine::Paint::Solid(color.as_vec4());
                    updated_fields.push("color");
                }
                NodeContent::SolidColor { color: node_color } => {
                    *node_color = color;
                    updated_fields.push("color");
                }
                NodeContent::Empty => {
                    return Err(anyhow!("Cannot set color on Empty node"));
                }
            }
        }

        Ok(json!({ "updated": updated_fields }))
    }
}

// ============================================================================
// scene.mark_dirty
// ============================================================================

/// Mark nodes as dirty for redraw
pub struct MarkDirtyTool;

#[derive(Debug, Deserialize)]
struct MarkDirtyParams {
    nodes: Vec<u64>,
}

impl Tool for MarkDirtyTool {
    fn name(&self) -> &str {
        "scene.mark_dirty"
    }

    fn schema(&self) -> ToolSchema {
        ToolSchema {
            name: self.name().to_string(),
            description: "Mark nodes as dirty for redraw (testing utility)".to_string(),
            parameters: json!({
                "type": "object",
                "required": ["nodes"],
                "properties": {
                    "nodes": {
                        "type": "array",
                        "items": { "type": "number" }
                    }
                }
            }),
        }
    }

    fn execute(&self, params: Value, _ctx: &mut McpFrameworkContext) -> Result<Value> {
        let params: MarkDirtyParams = serde_json::from_value(params)?;

        // For now, this is a no-op since we don't have dirty tracking yet
        // When dirty tracking is implemented, this will mark nodes for redraw

        Ok(json!({
            "marked": params.nodes.len(),
            "note": "Dirty tracking not yet implemented"
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::McpFrameworkContext;
    use render_engine::{Color, NodeContent, SceneNode, VisualStyle};

    fn create_test_scene(ctx: &mut McpFrameworkContext) -> Vec<NodeId> {
        let scene = ctx.scene_mut();

        let mut node1 = SceneNode::new(NodeContent::Styled {
            style: Box::new(VisualStyle::new().solid_fill(Color::RED.as_vec4())),
        });
        node1.bounds = plat_core::Rect::new(0.0, 0.0, 100.0, 100.0);
        node1.visible = true;
        node1.opacity = 1.0;
        let rect1 = scene.add_node(scene.root(), node1);

        let mut node2 = SceneNode::new(NodeContent::Styled {
            style: Box::new(VisualStyle::new().solid_fill(Color::BLUE.as_vec4())),
        });
        node2.bounds = plat_core::Rect::new(50.0, 50.0, 100.0, 100.0);
        node2.visible = false;
        node2.opacity = 0.5;
        let rect2 = scene.add_node(scene.root(), node2);

        vec![rect1, rect2]
    }

    #[test]
    fn test_list_nodes_all() {
        let mut ctx = McpFrameworkContext::new();
        create_test_scene(&mut ctx);

        let tool = ListNodesTool;
        let result = tool.execute(json!({}), &mut ctx).unwrap();

        let nodes = result.get("nodes").unwrap().as_array().unwrap();
        assert!(nodes.len() >= 2); // At least our 2 test nodes (+ root)
    }

    #[test]
    fn test_list_nodes_visible_only() {
        let mut ctx = McpFrameworkContext::new();
        create_test_scene(&mut ctx);

        let tool = ListNodesTool;
        let result = tool
            .execute(json!({"filter": {"visible_only": true}}), &mut ctx)
            .unwrap();

        let nodes = result.get("nodes").unwrap().as_array().unwrap();

        // Should not include the hidden rect2
        for node in nodes {
            assert!(node.get("visible").unwrap().as_bool().unwrap());
        }
    }

    #[test]
    fn test_get_node() {
        let mut ctx = McpFrameworkContext::new();
        let nodes = create_test_scene(&mut ctx);

        let tool = GetNodeTool;
        let result = tool.execute(json!({"id": nodes[0].0}), &mut ctx).unwrap();

        assert_eq!(result.get("id").unwrap().as_u64().unwrap(), nodes[0].0);
        assert!(result.get("content").is_some());
    }

    #[test]
    fn test_get_node_not_found() {
        let mut ctx = McpFrameworkContext::new();

        let tool = GetNodeTool;
        let result = tool.execute(json!({"id": 99999}), &mut ctx);

        assert!(result.is_err());
    }

    #[test]
    fn test_query_hierarchy() {
        let mut ctx = McpFrameworkContext::new();
        create_test_scene(&mut ctx);

        let tool = QueryHierarchyTool;
        let result = tool.execute(json!({}), &mut ctx).unwrap();

        assert!(result.get("id").is_some());
        assert!(result.get("children").unwrap().as_array().is_some());
    }

    #[test]
    fn test_find_nodes_at_position() {
        let mut ctx = McpFrameworkContext::new();
        create_test_scene(&mut ctx);

        let tool = FindNodesAtPositionTool;

        // Point inside first rect
        let result = tool
            .execute(json!({"x": 50.0, "y": 50.0}), &mut ctx)
            .unwrap();
        let nodes = result.get("nodes").unwrap().as_array().unwrap();

        // Should find at least one node (rect2 is hidden so not counted)
        assert!(!nodes.is_empty());
    }

    #[test]
    fn test_update_node_visibility() {
        let mut ctx = McpFrameworkContext::new();
        let nodes = create_test_scene(&mut ctx);

        let tool = UpdateNodeTool;
        let result = tool
            .execute(
                json!({"id": nodes[0].0, "updates": {"visible": false}}),
                &mut ctx,
            )
            .unwrap();

        let updated = result.get("updated").unwrap().as_array().unwrap();
        assert!(updated.contains(&json!("visible")));

        // Verify the change
        let node = ctx.scene().get_node(nodes[0]).unwrap();
        assert!(!node.visible);
    }

    #[test]
    fn test_update_node_color() {
        let mut ctx = McpFrameworkContext::new();
        let nodes = create_test_scene(&mut ctx);

        let tool = UpdateNodeTool;
        let result = tool
            .execute(
                json!({"id": nodes[0].0, "updates": {"color": [0.0, 1.0, 0.0, 1.0]}}),
                &mut ctx,
            )
            .unwrap();

        let updated = result.get("updated").unwrap().as_array().unwrap();
        assert!(updated.contains(&json!("color")));

        // Verify the change
        let node = ctx.scene().get_node(nodes[0]).unwrap();
        if let NodeContent::Styled { style } = &node.content {
            if let Some(render_engine::Paint::Solid(color)) = style.fills.first() {
                assert_eq!(color.y, 1.0);
            } else {
                panic!("Expected solid fill");
            }
        } else {
            panic!("Expected Styled content");
        }
    }

    #[test]
    fn test_mark_dirty() {
        let mut ctx = McpFrameworkContext::new();
        let nodes = create_test_scene(&mut ctx);

        let tool = MarkDirtyTool;
        let result = tool
            .execute(json!({"nodes": [nodes[0].0, nodes[1].0]}), &mut ctx)
            .unwrap();

        assert_eq!(result.get("marked").unwrap().as_u64().unwrap(), 2);
    }

    #[test]
    fn test_query_hierarchy_depth_limit() {
        let mut ctx = McpFrameworkContext::new();
        let tool = QueryHierarchyTool;

        let result = tool.execute(json!({ "max_depth": 101 }), &mut ctx);

        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Max depth cannot exceed")
        );
    }
}
