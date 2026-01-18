# ADR 0001: Hybrid ECS Architecture for Scene Management

**Status:** Accepted

**Date:** 2026-01-17

**Deciders:** Architecture Team

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
Scene (HashMap<NodeId, SceneNode>)  ←→  FrameworkContext (bevy_ecs::World)
       ↑                                          ↑
       └────────── NodeId references ─────────────┘

Systems query ECS → read/write Scene nodes via NodeId
```

## Consequences

### Positive

- **Performance**: O(1) scene node access, O(n) cache-friendly iteration for both tree and bulk ops
- **Flexibility**: Can optimize each operation type independently
- **Clean API**: FrameworkContext hides ECS complexity from users
- **Gradual Migration**: Can move operations to ECS incrementally
- **Parallel Systems**: ECS scheduler enables parallel execution (future)
- **Component Composition**: Standard ECS pattern for behaviors (Hoverable + Draggable + Renderable)

### Negative

- **Two Data Structures**: Scene and ECS World must stay in sync
- **Manual Sync**: Scene node bounds must be manually updated (not in reactive system yet)
- **Complexity**: Developers must understand both systems
- **Memory Overhead**: Some data duplication (NodeId in both Scene and ECS)

### Mitigations

- **Clear Ownership**: Scene owns node data, ECS owns behavior/components
- **FrameworkContext API**: Abstracts ECS complexity
- **Documentation**: Clear guidance on when to use Scene vs ECS
- **Testing**: Comprehensive integration tests ensure sync

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

### Current State (Phase 1)

- ✅ Scene: HashMap-based, O(1) access
- ✅ FrameworkContext: Wraps bevy_ecs World
- ✅ Components: SceneNodeRef, ReactiveColor, ReactiveTransform, ReactiveOpacity, Renderable
- ✅ Systems: Reactive updates, render collection
- ✅ Example: colored_rectangles demonstrates hybrid pattern

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
