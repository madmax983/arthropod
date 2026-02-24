# arthropod-ecs

The ECS integration layer for the Arthropod GUI framework. It bridges the gap between the `render-engine` scene graph and the `bevy_ecs` runtime.

## Architecture

Arthropod uses a **Hybrid ECS** approach:

- **Scene Graph (`render-engine`)**: Optimized for rendering and hierarchy traversal.
- **ECS (`bevy_ecs`)**: Optimized for cross-cutting concerns (systems, events, querying).

### The Bridge

Entities in the ECS can "own" or reference nodes in the Scene Graph. This is done via `NodeId`s stored in components.

```rust
// ECS Entity
commands.spawn((
    Renderable,                 // Tag: this entity has a visual representation
    ReactiveColor(signal),      // Component: its color is reactive
    // The link to the Scene Graph is managed internally via NodeId
));
```

## Key Components

- **`Renderable`**: Marks an entity as having a corresponding node in the scene graph.
- **`ReactiveColor`**: Binds a `flux-state` signal to the visual color of a node.
- **`ReactiveTransform`**: Binds a signal to the node's position/scale.
- **`ReactiveText`**: Binds a signal to the text content.
- **`Hoverable`**: Marks an entity as interactive (mouse hover detection).

## Systems

- **`apply_reactive_changes_system`**: Propagates signal updates from `flux-state` to the `Scene`.
- **`collect_renderables_system`**: Gathers visible nodes for rendering.
- **`update_all_reactive_system`**: Triggers reactive updates.

## Performance

This split architecture allows for:

1.  **Fast Layout**: Layout is computed on the specialized Scene tree (O(N) traversal).
2.  **Fast Rendering**: Rendering is done by traversing the Scene tree (Painter's Algorithm).
3.  **Fast Logic**: Application logic runs in parallel ECS systems.
4.  **Fine-Grained Updates**: Only entities with changed signals are updated in the Scene.

## License

Part of the Arthropod project.
