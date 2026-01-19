# ADR 0011: Model Context Protocol (MCP) Live Debugging Integration

**Status:** Accepted

**Date:** 2026-01-19

**Deciders:** Architecture Team

## Context

Modern GUI frameworks lack AI-native debugging capabilities. Developers using Arthropod need to:

1. **Debug live applications** - Inspect scene state, ECS entities, and reactive signals while the app runs
2. **Iterate rapidly** - Understand why UI isn't behaving as expected without manual print debugging
3. **Leverage AI assistance** - Allow Claude and other AI tools to inspect and diagnose issues in running apps

Traditional approaches require:
- Manual logging and printf debugging
- External debuggers (gdb/lldb) with steep learning curves
- Chrome DevTools-style inspectors (complex to build and maintain)
- No integration with AI assistants

We needed a solution that provides **live introspection** of Arthropod applications while being **AI-native** from the start.

## Decision

We implement **Model Context Protocol (MCP) integration** for live debugging with a two-phase evolution:

### Phase 1: Framework Development (Pre-1.0) - **IMPLEMENTED**

Separate MCP server process for debugging Arthropod framework development:

```
┌─────────────────┐         ┌──────────────────┐
│  Claude Code    │◄─stdio─►│  arthropod-mcp   │
│  (AI Assistant) │         │  (MCP Server)    │
└─────────────────┘         └────────┬─────────┘
                                     │ TCP:7777
                                     ▼
                            ┌──────────────────┐
                            │  Arthropod App   │
                            │  (connects on    │
                            │   startup)       │
                            └──────────────────┘
```

**Components:**
- `arthropod-mcp` binary - Standalone MCP server with stdio transport
- TCP server listening on localhost:7777 for app connections
- Scene serialization via `Scene::serialize_to_json()`
- Live scene updates sent every 60 frames (~1 second at 60fps)
- 14+ MCP tools for inspecting scene, ECS, and reactive state

**Tools provided:**
- `scene_list_nodes` - List all nodes with filtering
- `scene_get_node` - Get detailed node information
- `scene_query_hierarchy` - Query scene tree structure
- `ecs_query_entities` - Query ECS entities by components
- `ecs_get_entity` - Get detailed entity information
- `state_get_signal` - Inspect reactive signal values
- And more...

**Key implementation details:**
1. **Scene serialization** - Added `Serialize`/`Deserialize` to all scene types:
   - `NodeId`, `SceneNode`, `NodeContent`
   - `Color` (serializes as `[r, g, b, a]` array)
   - `Transform2D` (serializes as 6-element affine matrix)
   - `plat_core::Rect`

2. **Live updates** - Apps send scene JSON via TCP:
   ```rust
   // In render loop
   if mcp_update_counter >= 60 {
       let scene = app.world().resource::<Scene>();
       if let Ok(scene_json) = scene.serialize_to_json() {
           connection.send_scene_update(scene_json)?;
       }
   }
   ```

3. **Source indicator** - Tools show which context they're using:
   - `🔴 LIVE APP: app_name (PID: 12345)` - Using live app data
   - `⚙️ Using test harness (no live app connected)` - Using fallback

### Phase 2: User Applications (Post-1.0) - **PLANNED**

Embed MCP server directly in user applications:

```rust
use arthropod::prelude::*;
use arthropod_mcp::McpIntegration;

fn main() {
    let mut app = AppBuilder::new()
        .with_window_config(config)
        .with_mcp(McpIntegration::new())  // Built-in MCP support
        .build(event_loop)?;

    // Optional: Add app-specific tools
    app.mcp_mut().register_tool(MyCustomTool {
        name: "inspect_inventory",
        handler: |ctx| { /* custom logic */ }
    });

    plat_core::run_app(app)?;
}
```

**Benefits:**
- Zero-config debugging - Every app is AI-debuggable by default
- Direct context access - No TCP overhead, no serialization lag
- Extensible tooling - Developers add domain-specific MCP tools
- Marketing differentiation - "AI-native GUI framework"

**API design:**
```rust
pub struct McpIntegration {
    server: ArthropodServer,
    context: Arc<Mutex<FrameworkContext>>,
}

impl McpIntegration {
    pub fn new() -> Self;
    pub fn with_context(context: Arc<Mutex<FrameworkContext>>) -> Self;
    pub fn register_tool<T: Tool>(&mut self, tool: T);
    pub fn start_stdio(&self);  // For Claude Code integration
    pub fn start_tcp(&self, port: u16);  // For remote debugging
}
```

## Consequences

### Positive

1. **Unique differentiation** - No other Rust GUI framework has AI-native debugging
   - Slint, egui, iced all require manual inspection
   - Arthropod apps can be diagnosed by AI assistants

2. **Superior DX during development** - Current implementation enables:
   - Live scene inspection while building the framework
   - Real-time reactive state tracking
   - Zero-latency debugging of signal propagation
   - Validated working: Successfully tracked hover effects changing rectangle colors in real-time

3. **Foundation for AI-native applications** - Post-1.0 enables:
   - Users ship AI-debuggable apps
   - Custom tools for domain-specific logic
   - "AI pair programming" for Arthropod apps

4. **Minimal overhead** -
   - Scene updates: ~1/second (60 frames)
   - Serialization cost: Sub-millisecond for typical UIs
   - TCP communication: Asynchronous, non-blocking

5. **Clean separation** - During framework development:
   - MCP server is separate process (won't crash with app)
   - TCP protocol allows remote debugging
   - Test harness fallback when no app connected

### Negative

1. **Two implementations to maintain** -
   - Pre-1.0: Separate binary + TCP protocol
   - Post-1.0: Embedded integration
   - Mitigation: Share ArthropodServer and tools between both

2. **Serialization overhead** -
   - Must serialize entire scene every 60 frames
   - Mitigation: Only ~1/second, asynchronous, skippable
   - Future: Send deltas instead of full snapshots

3. **Thread safety requirements** -
   - Scene must be `Send + Sync` for serialization
   - Already satisfied by current ECS Resource pattern

4. **Dependency on rmcp SDK** -
   - External dependency (rmcp = "0.13")
   - Current limitation: Some tools disabled due to JSON Schema issues
   - Mitigation: Active upstream, issues being addressed

### Neutral

1. **Learning curve for custom tools** - Users must learn MCP SDK
   - But optional - built-in tools work out of box

2. **stdio vs TCP trade-offs** -
   - stdio: Required for Claude Code integration
   - TCP: Enables remote debugging, multi-process architecture
   - Both needed for different use cases

## Implementation Notes

### Current Status (Phase 1 - COMPLETED)

**Implemented:**
- ✅ TCP server in arthropod-mcp binary (localhost:7777)
- ✅ Scene serialization with full type support
- ✅ Live scene updates from apps (~1/second)
- ✅ 14 MCP tools (scene, ECS, state inspection)
- ✅ Source indication (live app vs test harness)
- ✅ Validated with colored_rectangles example
- ✅ Real-time reactive state tracking (hover effects)

**Files modified:**
- `crates/arthropod-mcp/src/live.rs` - TCP protocol
- `crates/arthropod-mcp/src/server.rs` - Live scene integration
- `crates/render-engine/src/scene.rs` - `serialize_to_json()`
- `crates/render-engine/src/node.rs` - Serde derives
- `crates/plat-core/src/window.rs` - Rect serialization
- `examples/colored_rectangles.rs` - MCP connection + updates

**Dependencies added:**
- `render-engine`: serde, serde_json
- `plat-core`: serde

### Phase 2 Roadmap (Post-1.0)

1. **Q1 2026** - Create `McpIntegration` API in `arthropod` crate
2. **Q2 2026** - Document custom tool development
3. **Q3 2026** - Example gallery with MCP-enabled apps
4. **Q4 2026** - Marketing: "AI-native GUI framework"

## Performance Characteristics

**Measured (Phase 1):**
- Scene serialization: < 1ms for 100 nodes
- TCP send: Asynchronous, non-blocking
- Update frequency: 60 frames = ~16.6ms interval (negligible overhead)

**Expected (Phase 2):**
- Zero TCP overhead (in-process)
- Direct FrameworkContext access
- Optional: Disable MCP in release builds via feature flag

## Alternatives Considered

### 1. Chrome DevTools Protocol (CDP)
- **Pros:** Industry standard, rich tooling
- **Cons:** Complex protocol, no AI integration, browser-centric
- **Decision:** MCP is simpler and AI-native

### 2. Custom JSON-RPC Protocol
- **Pros:** Full control, tailored to Arthropod
- **Cons:** Reinventing wheel, no ecosystem
- **Decision:** MCP provides standardization and AI ecosystem

### 3. No Debugging Tools
- **Pros:** Less code to maintain
- **Cons:** Poor developer experience, harder adoption
- **Decision:** DX is critical for framework success

## Related ADRs

- ADR 0001: Hybrid ECS Architecture - Provides Scene as Resource for serialization
- ADR 0003: Flux-State Reactive Model - Reactive signals inspectable via MCP
- ADR 0002: Bevy ECS Selection - ECS entities queryable via MCP tools

## References

- [Model Context Protocol Specification](https://modelcontextprotocol.io/)
- [rmcp SDK Documentation](https://docs.rs/rmcp/)
- Arthropod MCP Implementation: `crates/arthropod-mcp/`
- Live debugging validation: `examples/colored_rectangles.rs`

## Future Enhancements

1. **Delta updates** - Send only changed nodes instead of full scene
2. **Bidirectional control** - MCP tools can modify scene/state
3. **Performance profiling tools** - Frame time analysis, ECS system timing
4. **Visual inspector** - Browser-based UI showing scene tree
5. **Breakpoint support** - Pause app execution from Claude
6. **State time-travel** - Record/replay reactive state changes
