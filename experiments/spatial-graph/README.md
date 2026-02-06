# Arthropod Moonshot: Spatial Graph 🌌

This experiment implements an infinite, zoomable, node-based canvas for the Arthropod Neural Exoskeleton.

## The Mechanism ⚙️

The core of the Spatial Graph is the **Coordinate Transformation System**.

1.  **`Viewport` Resource**: Manages the `Camera` state (position, zoom, viewport size).
2.  **`SpatialNode` Component**: Defines an entity's position and size in the infinite world space.
3.  **`spatial_transform_system`**: A lightweight ECS system that projects `SpatialNode` world coordinates into `SceneNode` screen coordinates (bounds) every frame. It also performs **Frustum Culling**, automatically toggling `SceneNode.visible` for off-screen nodes.
4.  **`Stats` Resource**: Tracks real-time metrics like total vs. visible node counts.

The system uses `glam` for SIMD-accelerated vector math to ensure transformations are lightning fast.

```rust
// World -> Screen
let screen_pos = (world_pos - camera.pos) * camera.zoom + center;
```

## The Magic ✨

The magic lies in the **Zero-Cost Abstraction** of the infinite space.

-   **Infinite Canvas**: Users can pan and zoom indefinitely.
-   **Decoupling**: The logical spatial representation (`SpatialNode`) is completely decoupled from the rendering representation (`SceneNode`).
-   **Performance**: The integrated **Frustum Culling** ensures that even with millions of nodes in the graph, the rendering engine only processes what the user can see.
-   **Synesthesia**: Real-time system metrics (`Stats`) are exposed as first-class citizens, allowing the UI to visualize its own performance characteristics.

## Usage

Run the infinite canvas example:

```bash
cargo run -p spatial-graph --example infinite_canvas
```

(Note: This example runs in headless mode in this environment, but simulates the camera movement and rendering loop).
