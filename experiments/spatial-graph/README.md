# Arthropod Moonshot: Spatial Graph 🌌

This experiment implements an infinite, zoomable, node-based canvas for the Arthropod Neural Exoskeleton.

## The Mechanism ⚙️

The core of the Spatial Graph is the **Coordinate Transformation System**.

1.  **`Viewport` Resource**: Manages the `Camera` state (position, zoom, viewport size).
2.  **`SpatialNode` Component**: Defines an entity's position and size in the infinite world space.
3.  **`spatial_transform_system`**: A lightweight ECS system that projects `SpatialNode` world coordinates into `SceneNode` screen coordinates (bounds) every frame.

The system uses `glam` for SIMD-accelerated vector math to ensure transformations are lightning fast.

```rust
// World -> Screen
let screen_pos = (world_pos - camera.pos) * camera.zoom + center;
```

## The Magic ✨

The magic lies in the **Zero-Cost Abstraction** of the infinite space.

-   **Infinite Canvas**: Users can pan and zoom indefinitely.
-   **Decoupling**: The logical spatial representation (`SpatialNode`) is completely decoupled from the rendering representation (`SceneNode`).
-   **Performance**: By using ECS for the transformation step, we can process thousands of nodes in parallel (future optimization) and only update the `Scene` (which is optimized for dirty tracking) when necessary.

## Usage

Run the infinite canvas example:

```bash
cargo run -p spatial-graph --example infinite_canvas
```

(Note: This example runs in headless mode in this environment, but simulates the camera movement and rendering loop).
