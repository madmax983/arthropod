#![allow(missing_docs)]
//! Full MCP server implementation using official rmcp SDK.

use crate::context::McpFrameworkContext;
use crate::live::ConnectedApp;
use crate::tools::Tool as ArthropodTool;
use rmcp::handler::server::tool::ToolRouter;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{
    CallToolResult, Content, Implementation, ProtocolVersion, ServerCapabilities, ServerInfo,
};
use rmcp::{ErrorData as McpError, ServerHandler, tool, tool_handler, tool_router};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::{Arc, Mutex, RwLock};

// ============================================================================
// Parameter Types
// ============================================================================

/// Parameters for scene_list_nodes
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
pub struct ListNodesParams {
    #[serde(default)]
    pub visible_only: bool,
    pub content_type: Option<String>,
    pub parent_id: Option<u64>,
}

/// Parameters for scene_get_node
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GetNodeParams {
    pub node_id: u64,
}

/// Parameters for scene_query_hierarchy
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
pub struct QueryHierarchyParams {
    pub root_id: Option<u64>,
    pub max_depth: Option<usize>,
}

/// Parameters for scene_find_nodes_at_position
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct FindNodesAtPositionParams {
    pub x: f32,
    pub y: f32,
}

/// Parameters for scene_update_node
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct UpdateNodeParams {
    pub node_id: u64,
    pub visible: Option<bool>,
    pub opacity: Option<f32>,
    pub bounds: Option<BoundsUpdate>,
    pub color: Option<[f32; 4]>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct BoundsUpdate {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

/// Parameters for scene_mark_dirty
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MarkDirtyParams {
    pub node_id: u64,
}

/// Parameters for ecs_query_entities
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
pub struct QueryEntitiesParams {
    #[serde(default)]
    pub components: Vec<String>,
    pub limit: Option<usize>,
}

/// Parameters for ecs_get_entity
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GetEntityParams {
    pub entity_id: u64,
}

/// Parameters for ecs_count_entities
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
pub struct CountEntitiesParams {
    pub components: Option<Vec<String>>,
}

/// Parameters for state_register_signal
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RegisterSignalParams {
    pub name: String,
    pub signal_type: String,
    pub initial_value: Value,
}

/// Parameters for state_set_signal
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SetSignalParams {
    pub name: String,
    pub value: Value,
}

/// Parameters for state_get_signal
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GetSignalParams {
    pub name: String,
}

/// Parameters for test_create_scene
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CreateSceneParams {
    pub nodes: Value,
}

/// Parameters for test_assert_node_state
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AssertNodeStateParams {
    pub node_name: String,
    pub expected: Value,
}

/// Parameters for test_verify_render_output
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct VerifyRenderOutputParams {
    pub expected: Value,
}

/// Parameters for test_setup_reactive_chain
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SetupReactiveChainParams {
    pub node_name: String,
    pub signal_name: String,
    pub signal_type: String,
    pub initial_value: Value,
}

// ============================================================================
// Server Implementation
// ============================================================================

#[derive(Clone)]
pub struct ArthropodServer {
    context: Arc<Mutex<McpFrameworkContext>>,
    connected_app: Arc<RwLock<Option<ConnectedApp>>>,
    tool_router: ToolRouter<Self>,
}

impl Default for ArthropodServer {
    fn default() -> Self {
        Self::new()
    }
}

#[tool_router]
impl ArthropodServer {
    pub fn new() -> Self {
        Self {
            context: Arc::new(Mutex::new(McpFrameworkContext::new())),
            connected_app: Arc::new(RwLock::new(None)),
            tool_router: Self::tool_router(),
        }
    }

    pub fn with_context(context: McpFrameworkContext) -> Self {
        Self {
            context: Arc::new(Mutex::new(context)),
            connected_app: Arc::new(RwLock::new(None)),
            tool_router: Self::tool_router(),
        }
    }

    pub fn with_live_app(
        context: McpFrameworkContext,
        connected_app: Arc<RwLock<Option<ConnectedApp>>>,
    ) -> Self {
        Self {
            context: Arc::new(Mutex::new(context)),
            connected_app,
            tool_router: Self::tool_router(),
        }
    }

    pub fn context(&self) -> std::sync::MutexGuard<'_, McpFrameworkContext> {
        self.context
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    /// Check if a live app is connected and return its info
    fn get_connected_app_info(&self) -> Option<(String, u32)> {
        let app_guard = self
            .connected_app
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        app_guard.as_ref().map(|app| (app.name.clone(), app.pid))
    }

    /// Generate a banner indicating whether using live app or test harness
    fn get_source_banner(&self) -> String {
        if let Some((name, pid)) = self.get_connected_app_info() {
            format!("🔴 LIVE APP: {} (PID: {})\n\n", name, pid)
        } else {
            "⚙️  Using test harness (no live app connected)\n\n".to_string()
        }
    }

    /// Execute a tool and return the result as a pretty-printed JSON string.
    ///
    /// This helper reduces boilerplate by handling context locking, parameter
    /// serialization, execution, and result formatting.
    fn execute_tool_core<T, P>(&self, tool: T, params: P) -> Result<String, McpError>
    where
        T: ArthropodTool,
        P: Serialize,
    {
        let mut ctx = self
            .context
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);

        let json_params = serde_json::to_value(params)
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        let result = ArthropodTool::execute(&tool, json_params, &mut ctx)
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        Ok(serde_json::to_string_pretty(&result).unwrap())
    }

    // ========================================================================
    // Scene Manipulation Tools
    // ========================================================================

    #[tool(description = "List all scene nodes with optional filtering")]
    async fn scene_list_nodes(
        &self,
        params: Parameters<ListNodesParams>,
    ) -> Result<CallToolResult, McpError> {
        let mut output = self.get_source_banner();

        // Check if we have live scene data
        if let Some(app) = self
            .connected_app
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .as_ref()
            && let Some(scene_json) = &app.scene
        {
            // Use live scene data
            if let Some(nodes) = scene_json.get("nodes") {
                let filtered_nodes = nodes
                    .as_array()
                    .unwrap_or(&vec![])
                    .iter()
                    .filter(|node| {
                        if params.0.visible_only {
                            node.get("visible")
                                .and_then(|v| v.as_bool())
                                .unwrap_or(true)
                        } else {
                            true
                        }
                    })
                    .cloned()
                    .collect::<Vec<_>>();

                output.push_str(
                    &serde_json::to_string_pretty(&serde_json::json!({
                        "nodes": filtered_nodes,
                        "count": filtered_nodes.len(),
                    }))
                    .unwrap(),
                );
                return Ok(CallToolResult::success(vec![Content::text(output)]));
            }
        }

        // Fallback to test harness
        let result = self.execute_tool_core(crate::tools::scene::ListNodesTool, params.0)?;
        output.push_str(&result);

        Ok(CallToolResult::success(vec![Content::text(output)]))
    }

    #[tool(description = "Get detailed information about a specific scene node")]
    async fn scene_get_node(
        &self,
        params: Parameters<GetNodeParams>,
    ) -> Result<CallToolResult, McpError> {
        let result = self.execute_tool_core(crate::tools::scene::GetNodeTool, params.0)?;

        let mut output = self.get_source_banner();
        output.push_str(&result);

        Ok(CallToolResult::success(vec![Content::text(output)]))
    }

    #[tool(description = "Query the scene hierarchy tree")]
    async fn scene_query_hierarchy(
        &self,
        params: Parameters<QueryHierarchyParams>,
    ) -> Result<CallToolResult, McpError> {
        let result = self.execute_tool_core(crate::tools::scene::QueryHierarchyTool, params.0)?;
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    #[tool(description = "Find all nodes at a specific screen position")]
    async fn scene_find_nodes_at_position(
        &self,
        params: Parameters<FindNodesAtPositionParams>,
    ) -> Result<CallToolResult, McpError> {
        let result =
            self.execute_tool_core(crate::tools::scene::FindNodesAtPositionTool, params.0)?;
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    #[tool(description = "Update properties of a scene node")]
    async fn scene_update_node(
        &self,
        params: Parameters<UpdateNodeParams>,
    ) -> Result<CallToolResult, McpError> {
        let result = self.execute_tool_core(crate::tools::scene::UpdateNodeTool, params.0)?;
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    #[tool(description = "Mark a node as dirty to trigger re-render")]
    async fn scene_mark_dirty(
        &self,
        params: Parameters<MarkDirtyParams>,
    ) -> Result<CallToolResult, McpError> {
        let result = self.execute_tool_core(crate::tools::scene::MarkDirtyTool, params.0)?;
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    // ========================================================================
    // ECS Query Tools
    // ========================================================================

    #[tool(description = "Query entities by components")]
    async fn ecs_query_entities(
        &self,
        params: Parameters<QueryEntitiesParams>,
    ) -> Result<CallToolResult, McpError> {
        let result = self.execute_tool_core(crate::tools::ecs::QueryEntitiesTool, params.0)?;

        let mut output = self.get_source_banner();
        output.push_str(&result);

        Ok(CallToolResult::success(vec![Content::text(output)]))
    }

    #[tool(description = "Get detailed entity information")]
    async fn ecs_get_entity(
        &self,
        params: Parameters<GetEntityParams>,
    ) -> Result<CallToolResult, McpError> {
        let result = self.execute_tool_core(crate::tools::ecs::GetEntityTool, params.0)?;
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    #[tool(description = "Count entities matching filter")]
    async fn ecs_count_entities(
        &self,
        params: Parameters<CountEntitiesParams>,
    ) -> Result<CallToolResult, McpError> {
        let result = self.execute_tool_core(crate::tools::ecs::CountEntitiesTool, params.0)?;
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    #[tool(description = "List all archetypes (component combinations) in the ECS world")]
    async fn ecs_list_archetypes(&self) -> Result<CallToolResult, McpError> {
        let result = self.execute_tool_core(crate::tools::ecs::ListArchetypesTool, Value::Null)?;
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    #[tool(description = "Verify that Scene nodes are properly linked to ECS entities")]
    async fn ecs_verify_linkage(&self) -> Result<CallToolResult, McpError> {
        let result = self.execute_tool_core(crate::tools::ecs::VerifyLinkageTool, Value::Null)?;
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    // ========================================================================
    // Reactive State Tools
    // ========================================================================

    // TEMPORARILY DISABLED: JSON Schema validation fails for serde_json::Value parameters
    // TODO: Fix schema generation for flexible JSON parameters
    // #[tool(description = "Register a new reactive signal")]
    // async fn state_register_signal(&self, params: Parameters<RegisterSignalParams>) -> Result<CallToolResult, McpError> {
    //     let mut ctx = self.context.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    //     let tool = crate::tools::state::RegisterSignalTool;
    //
    //     let json_params = serde_json::to_value(&params.0)
    //         .map_err(|e| McpError::internal_error(e.to_string(), None))?;
    //
    //     let result = ArthropodTool::execute(&tool, json_params, &mut ctx)
    //         .map_err(|e| McpError::internal_error(e.to_string(), None))?;
    //
    //     Ok(CallToolResult::success(vec![Content::text(
    //         serde_json::to_string_pretty(&result).unwrap(),
    //     )]))
    // }

    // #[tool(description = "Set a signal value")]
    // async fn state_set_signal(&self, params: Parameters<SetSignalParams>) -> Result<CallToolResult, McpError> {
    //     let mut ctx = self.context.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    //     let tool = crate::tools::state::SetSignalTool;
    //
    //     let json_params = serde_json::to_value(&params.0)
    //         .map_err(|e| McpError::internal_error(e.to_string(), None))?;
    //
    //     let result = ArthropodTool::execute(&tool, json_params, &mut ctx)
    //         .map_err(|e| McpError::internal_error(e.to_string(), None))?;
    //
    //     Ok(CallToolResult::success(vec![Content::text(
    //         serde_json::to_string_pretty(&result).unwrap(),
    //     )]))
    // }

    #[tool(description = "Get current signal value")]
    async fn state_get_signal(
        &self,
        params: Parameters<GetSignalParams>,
    ) -> Result<CallToolResult, McpError> {
        let result = self.execute_tool_core(crate::tools::state::GetSignalTool, params.0)?;
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    #[tool(description = "Trigger update cycle to propagate reactive state changes")]
    async fn state_trigger_update(&self) -> Result<CallToolResult, McpError> {
        let result = self.execute_tool_core(crate::tools::state::TriggerUpdateTool, Value::Null)?;
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    #[tool(description = "Trigger render and get render instances")]
    async fn state_trigger_render(&self) -> Result<CallToolResult, McpError> {
        let result = self.execute_tool_core(crate::tools::state::TriggerRenderTool, Value::Null)?;
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    // ========================================================================
    // Testing Utility Tools
    // ========================================================================

    // TEMPORARILY DISABLED: JSON Schema validation fails for serde_json::Value parameters
    // TODO: Fix schema generation for flexible JSON parameters
    // #[tool(description = "Create a test scene with nodes")]
    // async fn test_create_scene(&self, params: Parameters<CreateSceneParams>) -> Result<CallToolResult, McpError> {
    //     let mut ctx = self.context.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    //     let tool = crate::tools::test::CreateSceneTool;
    //
    //     let json_params = serde_json::to_value(&params.0)
    //         .map_err(|e| McpError::internal_error(e.to_string(), None))?;
    //
    //     let result = ArthropodTool::execute(&tool, json_params, &mut ctx)
    //         .map_err(|e| McpError::internal_error(e.to_string(), None))?;
    //
    //     Ok(CallToolResult::success(vec![Content::text(
    //         serde_json::to_string_pretty(&result).unwrap(),
    //     )]))
    // }

    // #[tool(description = "Assert node state matches expected")]
    // async fn test_assert_node_state(&self, params: Parameters<AssertNodeStateParams>) -> Result<CallToolResult, McpError> {
    //     let mut ctx = self.context.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    //     let tool = crate::tools::test::AssertNodeStateTool;
    //
    //     let json_params = serde_json::to_value(&params.0)
    //         .map_err(|e| McpError::internal_error(e.to_string(), None))?;
    //
    //     let result = ArthropodTool::execute(&tool, json_params, &mut ctx)
    //         .map_err(|e| McpError::internal_error(e.to_string(), None))?;
    //
    //     Ok(CallToolResult::success(vec![Content::text(
    //         serde_json::to_string_pretty(&result).unwrap(),
    //     )]))
    // }

    // #[tool(description = "Verify render output")]
    // async fn test_verify_render_output(&self, params: Parameters<VerifyRenderOutputParams>) -> Result<CallToolResult, McpError> {
    //     let mut ctx = self.context.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    //     let tool = crate::tools::test::VerifyRenderOutputTool;
    //
    //     let json_params = serde_json::to_value(&params.0)
    //         .map_err(|e| McpError::internal_error(e.to_string(), None))?;
    //
    //     let result = ArthropodTool::execute(&tool, json_params, &mut ctx)
    //         .map_err(|e| McpError::internal_error(e.to_string(), None))?;
    //
    //     Ok(CallToolResult::success(vec![Content::text(
    //         serde_json::to_string_pretty(&result).unwrap(),
    //     )]))
    // }

    // #[tool(description = "Setup a reactive chain for testing")]
    // async fn test_setup_reactive_chain(&self, params: Parameters<SetupReactiveChainParams>) -> Result<CallToolResult, McpError> {
    //     let mut ctx = self.context.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    //     let tool = crate::tools::test::SetupReactiveChainTool;
    //
    //     let json_params = serde_json::to_value(&params.0)
    //         .map_err(|e| McpError::internal_error(e.to_string(), None))?;
    //
    //     let result = ArthropodTool::execute(&tool, json_params, &mut ctx)
    //         .map_err(|e| McpError::internal_error(e.to_string(), None))?;
    //
    //     Ok(CallToolResult::success(vec![Content::text(
    //         serde_json::to_string_pretty(&result).unwrap(),
    //     )]))
    // }
}

#[tool_handler]
impl ServerHandler for ArthropodServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation::from_build_env(),
            instructions: Some(
                "Arthropod MCP server - GUI framework testing and automation".to_string(),
            ),
        }
    }
}
