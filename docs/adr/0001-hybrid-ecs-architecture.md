# ADR 0001: Hybrid ECS Architecture for Scene Management

**Status:** Accepted (Updated for Phase 1 completion)

**Date:** 2026-01-17 (Created), 2026-01-18 (Updated)

**Deciders:** Architecture Team

**Update:** Phase 1 completed 2026-01-18. Scene and WgpuBackend moved to ECS Resources, arthropod::App builder implemented.

## Context

Arthropod requires both efficient hierarchical operations (layout, event bubbling) and high-performance bulk operations (rendering, animation). Traditional approaches force a choice between:

1. **Pure Scene Graph**: Excellent for hierarchy and parent-child relationships, but poor for bulk operations (rendering all visible nodes)
2. **Pure ECS**: Excellent for bulk operations, but awkward for hierarchical queries and tree traversal

We needed an architecture that provides optimal performance for both types of operations while maintaining a clean, enterprise-friendly API.

## Decision

We implement a **hybrid architecture** that combines:

- **Custom Scene Tree**: HashMap-based node storage for O(1) access and cache-friendly tree traversal
- **ECS Integration**: bevy_ecs for cross-cutting systems (rendering, animation, reactive updates)
- **NodeId Bridge**: Scene nodes and ECS entities reference each other via NodeId

### What Uses Custom Tree

- Scene hierarchy and parent-child relationships
- Layout computation (parent→child recursion)
- Event bubbling (child→parent traversal)
- Spatial indexing (future: quadtree for hit testing)

### What Uses ECS

- Rendering system (bulk query for visible entities)
- Animation system (parallel property interpolation)
- Reactive updates (poll flux-state signals into components)
- Component composition (Hoverable, Draggable, Focusable)
- System scheduling (parallel execution of independent systems)

### Integration Pattern

```
arthropod::App (High-level API)
    ├── FrameworkContext (bevy_ecs::World)
    │   ├── Scene (Resource) ← HashMap<NodeId, SceneNode>
    │   ├── WgpuBackend (Resource) ← GPU rendering
    │   └── Entities with SceneNodeRef components
    └── Runtime (Rc<Runtime>) ← Reactive state runtime

Scene and WgpuBackend live in ECS World as Resources
Systems access them via world.resource::<Scene>()
Runtime stays in App (RefCell is !Sync, can't be Resource)
```

**Usage Pattern:**
```rust
use arthropod::prelude::*;

// Create app - automatically sets up Scene, WgpuBackend as Resources
let mut app = AppBuilder::new()
    .with_window_config(config)
    .build(event_loop)?;

// Access Scene as Resource
let node_id = {
    let mut scene = app.world_mut().resource_mut::<Scene>();
    scene.add_node(root, node)
};

// Spawn entity linked to scene node
app.spawn(node_id)
    .insert(Renderable)
    .insert(ReactiveColor::new(color_signal));

// Update ECS systems, render to GPU
app.update();
app.render_to_gpu()?;
```

## Consequences

### Positive

- **Performance**: O(1) scene node access, O(n) cache-friendly iteration for both tree and bulk ops
- **Flexibility**: Can optimize each operation type independently
- **Clean API**: arthropod::App builder abstracts ECS complexity, Scene as Resource simplifies access
- **Unified Resource Management**: Scene and WgpuBackend as ECS Resources eliminate manual lifecycle management
- **Ergonomic Imports**: `use arthropod::prelude::*;` provides everything needed
- **Gradual Migration**: Can move operations to ECS incrementally
- **Parallel Systems**: ECS scheduler enables parallel execution (future)
- **Component Composition**: Standard ECS pattern for behaviors (Hoverable + Draggable + Renderable)
- **Thread Safety**: Resources are Send+Sync, enabling future parallel systems

### Negative

- **Runtime Limitation**: Runtime uses RefCell (!Sync), cannot be Resource, must use accessor method
- **Manual Sync**: Scene node bounds must be manually updated (not in reactive system yet)
- **Learning Curve**: Developers must understand Resource access patterns
- **Memory Overhead**: Some data duplication (NodeId in both Scene and ECS)

### Mitigations

- **Clear Ownership**: Scene owns node data, ECS owns behavior/components
- **arthropod::App API**: Provides convenient accessors (app.runtime(), app.world(), app.spawn())
- **arthropod::prelude**: Single import provides all common types
- **Documentation**: Clear guidance on Resource access patterns
- **TDD Tests**: Comprehensive test coverage ensures API correctness
- **Testing**: Integration tests verify Scene/ECS synchronization

## Performance Characteristics

### Tree Operations (Custom Scene)
- Layout: O(n) depth-first traversal ⚡
- Event bubbling: O(depth) parent traversal ⚡
- Find by ID: O(1) HashMap lookup ⚡
- Hit testing: O(log n) with spatial index (future) ⚡

### Bulk Operations (ECS)
- Render all visible: O(n) archetype iteration ⚡
- Animate all: O(n) parallel component updates ⚡
- Query by component: O(n) cache-friendly ⚡
- Reactive updates: O(changed) only dirty entities ⚡

## Alternatives Considered

### 1. Pure ECS (bevy_hierarchy pattern)

**Pros:**
- Single data model
- Proven pattern in game engines
- Excellent query performance

**Cons:**
- Hierarchical queries are awkward (Parent<T> components, recursive queries)
- Layout computation requires complex query chains
- Event bubbling is unnatural in ECS
- Less intuitive for GUI domain

**Rejected because:** GUI operations are fundamentally hierarchical, fighting ECS patterns.

### 2. Pure Scene Graph

**Pros:**
- Natural for hierarchical operations
- Simple mental model
- Standard GUI pattern

**Cons:**
- Poor bulk query performance (must traverse entire tree)
- Difficult to implement parallel systems
- Limited component composition
- Rendering every frame requires full tree traversal

**Rejected because:** Rendering and animation performance are critical.

### 3. ECS with Cached Hierarchy

**Pros:**
- Single ECS data model
- Cache can optimize hierarchy queries

**Cons:**
- Cache invalidation is complex
- Still awkward for tree operations
- More complex than hybrid approach

**Rejected because:** Added complexity without clear benefits over hybrid.

## Implementation Notes

### Current State (Phase 1 - Complete)

**Architecture:**
- ✅ **Scene as ECS Resource**: Scene stored in bevy_ecs World via `#[derive(Resource)]`
- ✅ **WgpuBackend as ECS Resource**: GPU backend stored as Resource for unified lifecycle
- ✅ **arthropod::App builder**: High-level API (`AppBuilder::new().build()`)
- ✅ **arthropod::prelude**: Single import for all common types
- ✅ **Runtime in App**: `Rc<Runtime>` kept in App struct (RefCell is !Sync)
- ✅ **TDD test coverage**: 5 comprehensive tests for App builder API

**Components:**
- ✅ SceneNodeRef, ReactiveColor, ReactiveTransform, ReactiveOpacity, Renderable

**Systems:**
- ✅ Reactive updates (poll flux-state signals)
- ✅ Render collection (query Renderable entities)

**Examples Updated:**
- ✅ colored_rectangles: Demonstrates reactive state + hover interactions
- ✅ single_rect_test: Minimal rendering test with App API
- ✅ debug_render: RenderDoc capture with App API

**Performance:**
- ✅ ECS overhead < 2 μs for 1,000 widgets (measured via criterion)
- ✅ Resource access is zero-cost abstraction
- ✅ See `docs/performance/benchmark-results.md` for full metrics

### Future Enhancements (Phase 2+)

- Layout as ECS system (with tree awareness)
- Event system with ECS components (Hoverable, Clickable, Draggable)
- Animation system with parallel interpolation
- Spatial indexing integrated with ECS queries
- Dirty tracking via ECS change detection

## References

- [Arthropod Design Doc](../design/arthropod-design-doc.md)
- [ECS Integration Plan](../design/ecs-integration-plan.md)
- bevy_ecs documentation: https://docs.rs/bevy_ecs/
- Unity DOTS (similar hybrid): https://unity.com/dots
