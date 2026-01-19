# Arthropod MCP Server - Complete Workflow Example

## Overview

This document demonstrates a complete testing workflow using the Arthropod MCP server. The workflow simulates testing a button with a hover effect using reactive signals.

## Starting the Server

```bash
cargo run --bin arthropod-mcp
```

The server listens on stdin and responds on stdout. All logs go to stderr.

---

## Workflow: Testing a Button with Hover Effect

### Step 1: Create the Scene

**Request:**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "test.create_scene",
  "params": {
    "nodes": [
      {
        "name": "background",
        "content": {
          "type": "Rect",
          "color": [0.1, 0.1, 0.1, 1.0]
        },
        "bounds": {
          "x": 0.0,
          "y": 0.0,
          "width": 800.0,
          "height": 600.0
        }
      },
      {
        "name": "button",
        "content": {
          "type": "RoundedRect",
          "color": [0.2, 0.6, 0.9, 1.0],
          "corner_radius": 8.0
        },
        "bounds": {
          "x": 300.0,
          "y": 250.0,
          "width": 200.0,
          "height": 100.0
        },
        "parent": "background"
      }
    ]
  }
}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "nodes_created": 2,
    "node_ids": {
      "background": 1,
      "button": 2
    }
  }
}
```

**What happened:** Created a dark background with a blue rounded rectangle button as a child.

---

### Step 2: Verify Initial State

**Request:**
```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "method": "test.assert_node_state",
  "params": {
    "node_name": "button",
    "expected": {
      "visible": true,
      "opacity": 1.0,
      "color": [0.2, 0.6, 0.9, 1.0],
      "corner_radius": 8.0,
      "bounds": {
        "x": 300.0,
        "y": 250.0,
        "width": 200.0,
        "height": 100.0
      }
    }
  }
}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "result": {
    "passed": true,
    "node_name": "button",
    "failures": []
  }
}
```

**What happened:** Verified the button was created with correct properties. All assertions passed.

---

### Step 3: Setup Reactive Color Signal

**Request:**
```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "method": "test.setup_reactive_chain",
  "params": {
    "node_name": "button",
    "signal_name": "button_color",
    "signal_type": "color",
    "initial_value": [0.2, 0.6, 0.9, 1.0]
  }
}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "result": {
    "node_name": "button",
    "signal_name": "button_color",
    "signal_type": "color",
    "status": "ready"
  }
}
```

**What happened:**
- Created a reactive color signal initialized to blue
- Spawned an ECS entity linked to the button node
- Added `Renderable` and `ReactiveColor` components
- Registered the signal for remote control

---

### Step 4: Simulate Hover (Change to Green)

**Request:**
```json
{
  "jsonrpc": "2.0",
  "id": 4,
  "method": "state.set_signal",
  "params": {
    "name": "button_color",
    "value": [0.2, 0.9, 0.4, 1.0]
  }
}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "id": 4,
  "result": {
    "name": "button_color",
    "old_value": [0.2, 0.6, 0.9, 1.0],
    "new_value": [0.2, 0.9, 0.4, 1.0]
  }
}
```

**What happened:** Updated the signal value from blue to green. The signal is now "dirty" and ready to propagate.

---

### Step 5: Propagate Signal Changes

**Request:**
```json
{
  "jsonrpc": "2.0",
  "id": 5,
  "method": "state.trigger_update",
  "params": {}
}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "id": 5,
  "result": {
    "duration_us": 342,
    "duration_ms": 0.342
  }
}
```

**What happened:**
- Ran ECS update systems
- `ReactiveColor` component polled the signal
- Detected the color changed from blue to green
- Updated the scene node's content to reflect the new color
- Took 342 microseconds

---

### Step 6: Verify Color Changed

**Request:**
```json
{
  "jsonrpc": "2.0",
  "id": 6,
  "method": "test.assert_node_state",
  "params": {
    "node_name": "button",
    "expected": {
      "color": [0.2, 0.9, 0.4, 1.0]
    },
    "tolerance": 0.001
  }
}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "id": 6,
  "result": {
    "passed": true,
    "node_name": "button",
    "failures": []
  }
}
```

**What happened:** Confirmed the button's color in the scene tree is now green. The reactive system worked!

---

### Step 7: Verify Render Output

**Request:**
```json
{
  "jsonrpc": "2.0",
  "id": 7,
  "method": "test.verify_render_output",
  "params": {
    "expected_count": 2,
    "expected_instances": [
      {
        "color": [0.1, 0.1, 0.1, 1.0],
        "position": {"x": 0.0, "y": 0.0},
        "size": {"width": 800.0, "height": 600.0}
      },
      {
        "color": [0.2, 0.9, 0.4, 1.0],
        "position": {"x": 300.0, "y": 250.0},
        "size": {"width": 200.0, "height": 100.0}
      }
    ]
  }
}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "id": 7,
  "result": {
    "passed": true,
    "actual_count": 2,
    "failures": []
  }
}
```

**What happened:**
- Triggered the render systems
- Generated 2 `RectInstance` objects for the GPU
- Verified:
  - Correct number of instances (2)
  - Background is dark gray at position (0, 0) with size 800x600
  - Button is green at position (300, 250) with size 200x100

---

## Workflow Summary

This workflow demonstrated:

1. **Declarative scene creation** - Created parent/child hierarchy with one call
2. **Property assertions** - Verified initial state
3. **Reactive setup** - One-call signal + component + registration
4. **Signal manipulation** - Changed color remotely
5. **Update propagation** - Triggered reactive system
6. **State verification** - Confirmed scene updated
7. **Render validation** - Verified GPU instances match expectations

## Performance Tracking

All operations are tracked:
```json
{
  "jsonrpc": "2.0",
  "id": 8,
  "method": "perf.get_stats",
  "params": {}
}
```

Would return timing data showing:
- Update took ~342 μs
- Render took ~120 μs
- Total test overhead < 1 ms

## Tool Categories Used

| Category | Tools Used |
|----------|------------|
| **Test** | create_scene, assert_node_state, verify_render_output, setup_reactive_chain |
| **State** | set_signal, trigger_update, trigger_render |
| **Scene** | (implicit via test tools) |
| **ECS** | (implicit via reactive components) |

## Real-World Use Cases

This workflow enables:

### 1. Automated GUI Testing
```bash
# Run test suite via MCP
python test_suite.py | cargo run --bin arthropod-mcp
```

### 2. AI-Driven Testing
Claude Code can now:
- Create test scenarios
- Manipulate UI state
- Verify visual output
- Debug failures

### 3. Visual Regression Testing
```python
# Capture baseline
verify_render_output(baseline="golden/button_hover.json")

# After changes, compare
result = verify_render_output(baseline="golden/button_hover.json")
if not result["passed"]:
    print("Visual regression detected!")
```

### 4. Interactive Debugging
```bash
# Query current state
scene.get_node("button")

# Manipulate
state.set_signal("button_color", [1.0, 0.0, 0.0, 1.0])

# Observe
test.verify_render_output()
```

---

## Available Tools (20 total)

### Scene (6)
- `scene.list_nodes` - List all nodes with filters
- `scene.get_node` - Get detailed node info
- `scene.query_hierarchy` - Get tree structure
- `scene.find_nodes_at_position` - Spatial queries
- `scene.update_node` - Modify properties
- `scene.mark_dirty` - Manual dirty tracking

### ECS (5)
- `ecs.query_entities` - Query by components
- `ecs.get_entity` - Get entity details
- `ecs.count_entities` - Count matching entities
- `ecs.list_archetypes` - Show component combos
- `ecs.verify_linkage` - Check scene-entity links

### State (5)
- `state.register_signal` - Register signal
- `state.set_signal` - Update signal value
- `state.get_signal` - Read signal value
- `state.trigger_update` - Run ECS update
- `state.trigger_render` - Run render systems

### Test (4)
- `test.create_scene` - Declarative scene builder
- `test.assert_node_state` - Property assertions
- `test.verify_render_output` - Render validation
- `test.setup_reactive_chain` - Reactive setup helper

---

## Next Steps

Try it yourself:
```bash
# Start server
cargo run --bin arthropod-mcp

# Send request (in another terminal)
echo '{"jsonrpc":"2.0","id":1,"method":"scene.list_nodes","params":{}}' | cargo run --bin arthropod-mcp
```

Or use the Python helper (future work):
```python
from arthropod_mcp import McpClient

client = McpClient()
result = client.create_scene(nodes=[...])
print(result)
```
