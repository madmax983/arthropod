# Forge's Journal

**[Refactoring MCP Server Tool Execution]**
**Learning:** MCP tool handlers often repeat the same pattern: lock context, serialize params, execute tool, format output.
**Action:** Extracted this logic into a generic `execute_tool_core<T, P>` helper method in `ArthropodServer`. This reduced boilerplate significantly and centralized error handling and context management. Used `Result<String, McpError>` as return type to allow flexible output formatting (e.g., prepending banners).
