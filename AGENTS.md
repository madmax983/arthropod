# Arthropod - Enterprise Rust GUI Framework

## Project Overview

Arthropod is a cross-platform, enterprise-grade GUI framework written in Rust. It uses a hybrid architecture combining a custom scene tree for hierarchical operations with ECS (Entity-Component-System) for cross-cutting concerns like rendering and animation.

## Development Philosophy

### 1. Performance-First 🚀

**Performance is not an optimization, it's a design principle.**

Every architectural decision in Arthropod prioritizes performance:
- **Measured, not estimated**: All performance claims must be backed by criterion benchmarks
- **Sub-microsecond overhead**: Current ECS overhead < 2 μs for typical UIs (see `docs/performance/benchmark-results.md`)
- **Linear scaling**: Performance must scale linearly with widget count
- **Profile before optimize**: Use `cargo bench` and profiling tools before making claims
- **Document performance**: All ADRs include measured performance characteristics

**Current Performance** (measured via criterion):
- 1,000 widgets: 30.4 μs (0.18% of 60fps budget)
- 10,000 widgets: 400 μs (2.4% of 60fps budget)
- Scene lookups: ~7 ns (O(1) HashMap)
- See `docs/performance/` for full benchmarks

**Performance Requirements**:
- All new features MUST be benchmarked
- Regressions > 10% require justification
- Hot paths (ECS systems, rendering) must stay < 1ms for 1,000 widgets
- Always leave 95%+ of frame budget for application logic

### 2. Test-Driven Development (TDD) ✅

**All code must be test-driven. No exceptions.**

**The TDD Cycle**:
1. **Write the test first** - Define expected behavior
2. **Watch it fail** - Verify test catches the problem
3. **Write minimal code** - Make test pass
4. **Refactor** - Clean up while tests pass
5. **Repeat** - Continue with next requirement

**Testing Requirements**:
- **Unit tests**: Every function/method with logic
- **Integration tests**: Every feature/system interaction
- **Benchmarks**: Every performance-critical path
- **Property tests**: Complex algorithms (use proptest)
- **Coverage**: Aim for 80%+ line coverage

**Example TDD Workflow**:
```rust
// 1. Write test first
#[test]
fn test_reactive_color_updates_scene_node() {
    let runtime = Runtime::new();
    let color_signal = Signal::new(runtime.clone(), Color::RED);
    let (read, write) = color_signal.split();

    let mut scene = Scene::new();
    let node_id = scene.add_node(scene.root(), SceneNode { ... });

    let mut ctx = FrameworkContext::new();
    ctx.spawn(node_id).insert(ReactiveColor::new(read));

    // 2. Define expected behavior
    write.set(Color::BLUE);
    ctx.update(&mut scene);

    // 3. Assert it works
    let node = scene.get_node(node_id).unwrap();
    assert_eq!(extract_color(&node.content), Color::BLUE);
}

// 4. Now implement ReactiveColor to make test pass
// 5. Refactor while tests stay green
```

**When to Write Tests**:
- ✅ **Before** writing implementation (TDD)
- ✅ **During** feature development (integration tests)
- ✅ **After** finding bugs (regression tests)
- ❌ **Never** skip tests because "it's simple"

**Test Organization**:
- Unit tests: Same file as code (`#[cfg(test)] mod tests`)
- Integration tests: `crates/<name>/tests/`
- Benchmarks: `crates/<name>/benches/`
- Examples: `examples/` (also serve as integration tests)

## Architecture

### Core Principles

1. **Hybrid ECS Architecture** (ADR 0001)
   - Custom Scene tree for hierarchy (layout, events, spatial queries)
   - bevy_ecs for bulk operations (rendering, animation, reactive updates)
   - Integration via NodeId references

2. **Reactive State Management** (ADR 0003)
   - flux-state: Signals-based reactive library
   - Fine-grained reactivity (only update what changed)
   - Integrates with ECS via reactive components

3. **GPU-Accelerated Rendering** (ADR 0004)
   - wgpu backend (Vulkan/Metal/D3D12)
   - Instanced rendering (1 draw call for thousands of rectangles)
   - ECS systems generate RectInstances

### Crate Structure

```
arthropod/
├── crates/
│   ├── plat-core/         # Platform abstraction (windows, events, input)
│   ├── render-engine/     # Scene graph, rendering backend (wgpu)
│   ├── flux-state/        # Reactive state (signals, effects)
│   ├── arthropod-ecs/     # ECS integration (bevy_ecs wrapper)
│   ├── anim-graph/        # Animation system (future)
│   ├── arthropod/         # High-level framework API (future)
│   └── arthropod-test/    # Testing utilities
├── examples/              # Example applications
├── docs/
│   ├── adr/              # Architecture Decision Records
│   └── design/           # Design documents
└── .claude/              # Claude Code configuration
```

## Development Workflow

### Testing

```bash
# Run all tests
cargo test --all

# Run specific crate tests
cargo test -p arthropod-ecs

# Run with logging
RUST_LOG=debug cargo test
```

### Code Quality

```bash
# Lint (must pass with -D warnings)
cargo clippy --all-targets --all-features -- -D warnings

# Format
cargo fmt --all

# Check without building
cargo check --all
```

### Running Examples

```bash
# Colored rectangles (demonstrates ECS + reactive state)
cargo run --example colored_rectangles

# Single rectangle test
cargo run --example single_rect_test

# Debug render
cargo run --example debug_render
```

### Building

```bash
# Development build
cargo build

# Release build
cargo build --release

# Build specific example
cargo build --example colored_rectangles
```

## Key Patterns

### 1. ECS Integration Pattern

```rust
// Create scene node
let node_id = scene.add_node(parent, SceneNode { ... });

// Spawn ECS entity linked to scene node
context.spawn(node_id)
    .insert(Renderable)
    .insert(ReactiveColor::new(color_signal));

// Update systems (polls reactive signals)
context.update(&mut scene);

// Render (generates instances from ECS)
let instances = context.render(&scene);
backend.render_instances(&instances)?;
```

### 2. Reactive State Pattern

```rust
// Create signal
let runtime = Runtime::new();
let signal = Signal::new(runtime.clone(), initial_value);
let (read, write) = signal.split();

// Create effect (auto-subscribes to dependencies)
Effect::new(runtime.clone(), move || {
    let value = read.get();
    // This runs whenever value changes
});

// Update signal
write.set(new_value);  // Effects run automatically
```

### 3. Scene Management

```rust
// Create scene (once, persistent)
let mut scene = Scene::new();

// Add nodes
let node_id = scene.add_node(parent, SceneNode {
    content: NodeContent::Rect { color },
    bounds: Rect { x, y, width, height },
    visible: true,
    opacity: 1.0,
    // ...
});

// Update node properties
if let Some(node) = scene.get_node_mut(node_id) {
    node.bounds = new_bounds;
}
```

## Code Conventions

### Naming

- **Crates**: kebab-case (render-engine, arthropod-ecs)
- **Types**: PascalCase (SceneNode, ReactiveColor)
- **Functions**: snake_case (render_instances, get_node_mut)
- **Constants**: SCREAMING_SNAKE_CASE (INITIAL_CAPACITY)

### Documentation

- **Public API**: Must have doc comments
- **Examples**: Include `# Example` section in docs
- **Safety**: Document unsafe code with `# Safety` section
- **Errors**: Document error conditions

### Error Handling

- Use `Result<T, E>` for fallible operations
- Use `thiserror` for custom error types
- Propagate errors with `?` operator
- Only `panic!` for unrecoverable bugs

### Testing

- Unit tests in same file as code (`#[cfg(test)] mod tests`)
- Integration tests in `tests/` directory
- Use descriptive test names: `test_reactive_color_updates_scene_node`
- Test both success and failure cases

## Important Files

### Core Implementation

- `crates/arthropod-ecs/src/context.rs` - FrameworkContext (main ECS API)
- `crates/arthropod-ecs/src/components.rs` - ECS components
- `crates/arthropod-ecs/src/systems/` - ECS systems
- `crates/render-engine/src/backend/wgpu_backend.rs` - GPU rendering
- `crates/render-engine/src/scene.rs` - Scene graph
- `crates/flux-state/src/` - Reactive state primitives

### Examples

- `examples/colored_rectangles.rs` - Complete ECS + reactive demo
- Shows hover interactions, automatic color updates, window resize

### Documentation

- `docs/adr/` - Architecture Decision Records (read these first!)
- `docs/design/arthropod-design-doc.md` - Overall design document

## Common Tasks

### Adding a New ECS Component

1. Define component in `crates/arthropod-ecs/src/components.rs`
2. Create system in `crates/arthropod-ecs/src/systems/`
3. Add system to schedule in `context.rs`
4. Add tests in `crates/arthropod-ecs/tests/ecs_tests.rs`
5. Update example to demonstrate usage

### Adding a New Scene Node Type

1. Add variant to `NodeContent` enum in `render-engine/src/node.rs`
2. Update rendering logic in `wgpu_backend.rs`
3. Update `collect_renderables_system` in `arthropod-ecs/src/systems/render.rs`
4. Add tests
5. Update examples

### Adding a Reactive Component

1. Create signal type in component (e.g., `ReadSignal<T>`)
2. Create reactive component wrapper (e.g., `ReactiveOpacity`)
3. Implement update system (e.g., `update_reactive_opacity_system`)
4. Add to update schedule in `FrameworkContext::build_update_schedule()`
5. Test with signals changing

## Debugging

### RenderDoc Integration

```bash
# Build with renderdoc feature
cargo build --features renderdoc

# Run with RenderDoc attached
# (RenderDoc will capture GPU calls)
```

### Logging

```bash
# Enable debug logging
RUST_LOG=debug cargo run --example colored_rectangles

# Specific module logging
RUST_LOG=arthropod_ecs=trace cargo run --example colored_rectangles

# Multiple modules
RUST_LOG=arthropod_ecs=trace,render_engine=debug cargo run
```

### Performance Profiling

```bash
# Build release for accurate profiling
cargo build --release --example colored_rectangles

# Run with profiler (Windows)
cargo run --release --example colored_rectangles

# Check frame times in logs
```

## Safety Invariants

### ReadSignal

- Thread-safe (Send + Sync) via internal Mutex
- Can be safely shared across threads
- Use directly in ECS components

### SceneResource

- Holds raw pointer to Scene
- Only valid during system execution
- Inserted before systems run, removed after
- See comments in `systems/reactive.rs`

### WgpuBackend

- Surface must not outlive window
- Backend must be dropped before window
- Achieved by struct field order (backend before window)

## Future Work

See `docs/design/arthropod-design-doc.md` for roadmap. Key upcoming features:

- Layout engine (flexbox-like)
- Widget framework (Button, TextField, etc.)
- Event system (hover, click, drag)
- Text rendering (msdf-atlas)
- Advanced rendering (rounded corners, shadows, blur)
- Animation system integration
- Accessibility (a11y-engine)

## Getting Help

### Resources

- **Design Docs**: Start with ADRs in `docs/adr/`
- **Examples**: `examples/colored_rectangles.rs` is most complete
- **Tests**: `crates/arthropod-ecs/tests/` show usage patterns
- **Code Comments**: Implementation files are well-documented

### Key Concepts to Understand

1. **Hybrid Architecture**: Why we use both Scene tree and ECS
2. **Signals**: How reactive state propagates to UI
3. **ECS Systems**: How updates flow through the pipeline
4. **Instanced Rendering**: How we batch GPU draw calls

### Common Pitfalls

- **Forgetting to update context**: Call `context.update(&mut scene)` before render
- **Signal runtime lifetime**: Must keep Runtime alive (use Rc)
- **Scene node bounds**: Must manually update on resize (not in ECS yet)

## Commit Guidelines

Follow existing commit style:

```
Brief summary line (50 chars or less)

Detailed explanation of what changed and why.
Include technical details, architecture decisions.

- Bullet points for key changes
- Test results
- Breaking changes (if any)

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>
```

Examples:
- "Add ECS integration with arthropod-ecs crate"
- "Fix clippy lints across workspace (pedantic mode)"
- "Initial commit: Arthropod Phase 1 - Foundation Complete"
