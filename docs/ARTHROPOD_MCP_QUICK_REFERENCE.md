# Arthropod MCP Server - Quick Reference

## Installation

**Location:** `.claude/mcp-servers/arthropod.json` (already configured!)

**Manual Test:**
```bash
cargo run --bin arthropod-mcp
# Then type: {"jsonrpc":"2.0","id":1,"method":"scene.list_nodes","params":{}}
```

---

## Tool Categories (20 total)

### 🎨 Scene Tools (6)
| Tool | Purpose | Example |
|------|---------|---------|
| `scene.list_nodes` | List all nodes | `{ "filter": { "visible_only": true } }` |
| `scene.get_node` | Get node details | `{ "node_name": "my_button" }` |
| `scene.query_hierarchy` | Tree structure | `{ "root_name": "background" }` |
| `scene.find_nodes_at_position` | Spatial query | `{ "x": 150.0, "y": 200.0 }` |
| `scene.update_node` | Modify properties | `{ "node_name": "btn", "updates": { "visible": false } }` |
| `scene.mark_dirty` | Force redraw | `{ "node_name": "my_rect" }` |

### 🔧 ECS Tools (5)
| Tool | Purpose | Example |
|------|---------|---------|
| `ecs.query_entities` | Filter by components | `{ "with_components": ["Renderable"] }` |
| `ecs.get_entity` | Entity details | `{ "entity_id": 42 }` |
| `ecs.count_entities` | Count matches | `{ "with_components": ["SceneNodeRef"] }` |
| `ecs.list_archetypes` | Component combos | `{}` |
| `ecs.verify_linkage` | Check integrity | `{}` |

### 🔄 State Tools (5)
| Tool | Purpose | Example |
|------|---------|---------|
| `state.register_signal` | Create signal | `{ "name": "color1", "signal_type": "color", "initial_value": [1,0,0,1] }` |
| `state.set_signal` | Update signal | `{ "name": "color1", "value": [0,1,0,1] }` |
| `state.get_signal` | Read signal | `{ "name": "color1" }` |
| `state.trigger_update` | Run ECS update | `{}` |
| `state.trigger_render` | Generate instances | `{}` |

### ✅ Test Tools (4)
| Tool | Purpose | Example |
|------|---------|---------|
| `test.create_scene` | Build scene | `{ "nodes": [{ "name": "rect", "content": {...}, "bounds": {...} }] }` |
| `test.assert_node_state` | Verify properties | `{ "node_name": "rect", "expected": { "color": [1,0,0,1] } }` |
| `test.verify_render_output` | Check rendering | `{ "expected_count": 2, "expected_instances": [...] }` |
| `test.setup_reactive_chain` | Quick reactive setup | `{ "node_name": "rect", "signal_name": "c1", "signal_type": "color", "initial_value": [1,0,0,1] }` |

---

## Common Workflows

### 🎯 Workflow 1: Create & Verify Scene
```
1. test.create_scene → Create nodes
2. test.assert_node_state → Verify properties
3. test.verify_render_output → Check rendering
```

### 🎯 Workflow 2: Reactive Testing
```
1. test.create_scene → Create node
2. test.setup_reactive_chain → Setup signal
3. state.set_signal → Change value
4. state.trigger_update → Propagate
5. test.assert_node_state → Verify changed
```

### 🎯 Workflow 3: Debug Rendering
```
1. scene.get_node → Check scene
2. ecs.get_entity → Check ECS
3. ecs.verify_linkage → Find broken links
4. test.verify_render_output → Check instances
```

---

## Node Content Types

### Rect
```json
{
  "type": "Rect",
  "color": [1.0, 0.0, 0.0, 1.0]
}
```

### RoundedRect
```json
{
  "type": "RoundedRect",
  "color": [0.2, 0.6, 0.9, 1.0],
  "corner_radius": 8.0
}
```

---

## Signal Types

| Type | Format | Example |
|------|--------|---------|
| `color` | `[r, g, b, a]` | `[1.0, 0.0, 0.0, 1.0]` (red) |
| `f32` | `number` | `0.5` (opacity) |
| `bool` | `true/false` | `true` (visible) |

---

## Performance Expectations

| Operation | Target | Typical |
|-----------|--------|---------|
| Scene query | < 500 μs | ~100 μs |
| ECS query | < 300 μs | ~50 μs |
| Signal set | < 100 μs | ~20 μs |
| Update | < 500 μs | ~300 μs |
| Render | < 500 μs | ~120 μs |
| **Total overhead** | **< 1 ms** | **~500 μs** |

---

## Color Presets

```json
{
  "RED":    [1.0, 0.0, 0.0, 1.0],
  "GREEN":  [0.0, 1.0, 0.0, 1.0],
  "BLUE":   [0.0, 0.0, 1.0, 1.0],
  "WHITE":  [1.0, 1.0, 1.0, 1.0],
  "BLACK":  [0.0, 0.0, 0.0, 1.0],
  "GRAY":   [0.5, 0.5, 0.5, 1.0]
}
```

---

## Complete Example

```json
// 1. Create scene
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "test.create_scene",
  "params": {
    "nodes": [{
      "name": "button",
      "content": {
        "type": "RoundedRect",
        "color": [0.2, 0.6, 0.9, 1.0],
        "corner_radius": 8.0
      },
      "bounds": {"x": 300.0, "y": 250.0, "width": 200.0, "height": 100.0}
    }]
  }
}

// 2. Setup reactive
{
  "jsonrpc": "2.0",
  "id": 2,
  "method": "test.setup_reactive_chain",
  "params": {
    "node_name": "button",
    "signal_name": "btn_color",
    "signal_type": "color",
    "initial_value": [0.2, 0.6, 0.9, 1.0]
  }
}

// 3. Change color
{
  "jsonrpc": "2.0",
  "id": 3,
  "method": "state.set_signal",
  "params": {
    "name": "btn_color",
    "value": [0.2, 0.9, 0.4, 1.0]
  }
}

// 4. Update
{
  "jsonrpc": "2.0",
  "id": 4,
  "method": "state.trigger_update",
  "params": {}
}

// 5. Verify
{
  "jsonrpc": "2.0",
  "id": 5,
  "method": "test.assert_node_state",
  "params": {
    "node_name": "button",
    "expected": {"color": [0.2, 0.9, 0.4, 1.0]}
  }
}
```

---

## Error Handling

All tools return JSON-RPC 2.0 responses:

**Success:**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": { ... }
}
```

**Error:**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "error": {
    "code": -32602,
    "message": "Invalid params",
    "data": "Node 'foo' not found"
  }
}
```

---

## Files

- **Setup:** `MCP_SETUP_GUIDE.md`
- **Examples:** `MCP_WORKFLOW_EXAMPLE.md`
- **Config:** `.claude/mcp-servers/arthropod.json`
- **Binary:** `target/debug/arthropod-mcp`

---

## Quick Commands

```bash
# Build
cargo build --bin arthropod-mcp

# Test
cargo test -p arthropod-mcp

# Run
cargo run --bin arthropod-mcp

# Release
cargo build --release --bin arthropod-mcp
```

---

## Status

✅ **20 tools implemented**
✅ **67 tests passing**
✅ **Standalone binary builds**
✅ **MCP configuration ready**
✅ **Documentation complete**

**Ready for Claude Code integration!** 🚀
