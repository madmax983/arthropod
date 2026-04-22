#![allow(missing_docs)]
//! MCP tool implementations.
//!
//! Tools are organized by category:
//! - `scene`: Scene graph inspection and manipulation
//! - `ecs`: ECS entity and component queries
//! - `state`: Reactive state manipulation
//! - `test`: Testing utilities
//! - `visual`: Visual testing and frame capture
//! - `perf`: Performance monitoring

use crate::context::McpFrameworkContext;
use anyhow::Result;
use hashbrown::HashMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub mod ecs;
pub mod scene;
pub mod state;
pub mod test;

/// Tool trait for MCP operations
pub trait Tool: Send + Sync {
    /// Get the tool name (e.g., "scene.list_nodes")
    fn name(&self) -> &str;

    /// Get the tool's JSON schema for parameter validation
    fn schema(&self) -> ToolSchema;

    /// Execute the tool with given parameters
    fn execute(&self, params: Value, ctx: &mut McpFrameworkContext) -> Result<Value>;
}

/// Tool schema for MCP discovery
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSchema {
    /// Tool name
    pub name: String,

    /// Tool description
    pub description: String,

    /// Parameter schema (JSON Schema)
    pub parameters: Value,
}

/// Registry of available tools
pub struct ToolRegistry {
    tools: HashMap<String, Box<dyn Tool>>,
}

impl ToolRegistry {
    /// Create a new tool registry
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
        }
    }

    /// Register a tool
    pub fn register(&mut self, tool: Box<dyn Tool>) {
        let name = tool.name().to_string();
        self.tools.insert(name, tool);
    }

    /// Get a tool by name
    pub fn get(&self, name: &str) -> Option<&dyn Tool> {
        self.tools.get(name).map(|t| t.as_ref())
    }

    /// List all tool names
    pub fn list_tools(&self) -> Vec<String> {
        self.tools.keys().cloned().collect()
    }

    /// Get all tool schemas
    pub fn schemas(&self) -> Vec<ToolSchema> {
        self.tools.values().map(|t| t.schema()).collect()
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        let mut registry = Self::new();

        // Register scene tools
        registry.register(Box::new(scene::ListNodesTool));
        registry.register(Box::new(scene::GetNodeTool));
        registry.register(Box::new(scene::QueryHierarchyTool));
        registry.register(Box::new(scene::FindNodesAtPositionTool));
        registry.register(Box::new(scene::UpdateNodeTool));
        registry.register(Box::new(scene::MarkDirtyTool));

        // Register ECS tools
        registry.register(Box::new(ecs::QueryEntitiesTool));
        registry.register(Box::new(ecs::GetEntityTool));
        registry.register(Box::new(ecs::CountEntitiesTool));
        registry.register(Box::new(ecs::ListArchetypesTool));
        registry.register(Box::new(ecs::VerifyLinkageTool));

        // Register state tools
        registry.register(Box::new(state::RegisterSignalTool));
        registry.register(Box::new(state::SetSignalTool));
        registry.register(Box::new(state::GetSignalTool));
        registry.register(Box::new(state::TriggerUpdateTool));
        registry.register(Box::new(state::TriggerRenderTool));

        // Register test tools
        registry.register(Box::new(test::CreateSceneTool));
        registry.register(Box::new(test::AssertNodeStateTool));
        registry.register(Box::new(test::VerifyRenderOutputTool));
        registry.register(Box::new(test::SetupReactiveChainTool));

        registry
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_registry_default() {
        let registry = ToolRegistry::default();
        let tools = registry.list_tools();

        // Scene tools
        assert!(tools.contains(&"scene.list_nodes".to_string()));
        assert!(tools.contains(&"scene.get_node".to_string()));

        // ECS tools
        assert!(tools.contains(&"ecs.query_entities".to_string()));
        assert!(tools.contains(&"ecs.get_entity".to_string()));

        // State tools
        assert!(tools.contains(&"state.register_signal".to_string()));
        assert!(tools.contains(&"state.set_signal".to_string()));
        assert!(tools.contains(&"state.trigger_update".to_string()));

        // Test tools
        assert!(tools.contains(&"test.create_scene".to_string()));
        assert!(tools.contains(&"test.assert_node_state".to_string()));
        assert!(tools.contains(&"test.verify_render_output".to_string()));
        assert!(tools.contains(&"test.setup_reactive_chain".to_string()));

        assert!(tools.len() >= 20); // 6 scene + 5 ECS + 5 state + 4 test tools
    }

    #[test]
    fn test_tool_registration() {
        let mut registry = ToolRegistry::new();
        let tool = Box::new(scene::ListNodesTool);

        registry.register(tool);

        assert!(registry.get("scene.list_nodes").is_some());
        assert!(registry.get("nonexistent").is_none());
    }

    #[test]
    fn test_tool_schemas() {
        let registry = ToolRegistry::default();
        let schemas = registry.schemas();

        assert!(!schemas.is_empty());
        assert!(schemas.iter().any(|s| s.name == "scene.list_nodes"));
    }
}
