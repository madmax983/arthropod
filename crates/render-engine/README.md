# render-engine

The rendering backend for the Arthropod GUI framework. It provides a retained-mode scene graph and GPU-accelerated rendering via `wgpu`.

## Features

- **Retained Mode Scene Graph**: Persistent tree structure for efficient updates.
- **GPU Acceleration**: Built on `wgpu` for cross-platform support (Vulkan, Metal, DX12).
- **Batching**: Automatically batches draw calls to minimize GPU overhead.
- **Text Rendering**: High-quality text rendering with subpixel positioning.
- **Styling**: Unified `VisualStyle` based on Figma's design model (fills, strokes, effects).

## Architecture

### The Scene Graph

The `Scene` is the central data structure. It manages a hierarchy of `SceneNode`s.

- **Nodes**: Lightweight structs containing transform, bounds, and content.
- **IDs**: Nodes are referenced by `NodeId`, allowing O(1) lookups and updates.
- **Flat Storage**: Nodes are stored in a flat `HashMap` or Arena, improving cache locality.

### Coordinate System

- **Origin**: Top-Left (0, 0).
- **X-Axis**: Increases to the right.
- **Y-Axis**: Increases downwards.
- **Units**: Logical pixels (DPI-independent).

### Rendering Model

- **Painter's Algorithm**: Nodes are drawn in tree traversal order (depth-first).
- **Z-Ordering**: Children are drawn on top of parents. Later siblings are drawn on top of earlier siblings.
- **Clipping**: Parent nodes can clip their children to their bounds (`clips_content: true`).

## Usage

While most users will interact with `render-engine` via the high-level `widget-core` API, you can use it directly for custom rendering or low-level tools.

```rust
use render_engine::{Scene, SceneNode, NodeContent, Color, Transform2D};
use style_engine::VisualStyle;

// 1. Create a scene
let mut scene = Scene::new();
let root = scene.root();

// 2. Create a child node (Red Rectangle)
let red_rect = scene.add_node(root, SceneNode::new(NodeContent::Styled {
    style: Box::new(VisualStyle::new().solid_fill(Color::RED.as_vec4())),
}));

// 3. Position the node
let mut node = scene.get_node_mut(red_rect).unwrap();
node.transform = Transform2D::translate(100.0, 50.0);
node.bounds = plat_core::Rect::new(0.0, 0.0, 200.0, 100.0);
```

## Primitives

- **Rectangle**: `NodeContent::Styled` with `VisualStyle` (fills, strokes, corner radii).
- **Text**: `NodeContent::Styled` with `TextContent`.
- **Vector Paths**: `NodeContent::Styled` with `VectorPath` geometry.
- **Images**: `NodeContent::Styled` with `ImageFill`.

## Integration

`render-engine` is designed to be used as a Resource in a `bevy_ecs` world, often managed by `arthropod-ecs`.

## License

Part of the Arthropod project.
