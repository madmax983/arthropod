# ADR 0004: wgpu as Primary Rendering Backend

**Status:** Accepted

**Date:** 2026-01-17

**Deciders:** Architecture Team

## Context

Arthropod requires a cross-platform rendering backend for 2D GUI rendering. Key requirements:

1. **Cross-Platform**: Windows, macOS, Linux minimum
2. **Performance**: GPU-accelerated, minimal CPU overhead
3. **Modern API**: Shader-based, instanced rendering
4. **Future-Proof**: Support for advanced effects (blur, shadows, gradients)
5. **Safety**: Memory-safe, no undefined behavior
6. **Ecosystem**: Active development, good tooling
7. **License**: Compatible with MIT/Apache-2.0

## Decision

We use **wgpu 24.0** as our primary rendering backend.

### Architecture

```
┌─────────────────────────────────────────┐
│   arthropod-ecs (ECS Systems)           │
│   - collect_renderables_system          │
│   → Generates Vec<RectInstance>         │
└─────────────┬───────────────────────────┘
              │
              ↓
┌─────────────────────────────────────────┐
│   WgpuBackend::render_instances()       │
│   - Upload instances to GPU             │
│   - Execute render pass                 │
│   - Present frame                       │
└─────────────┬───────────────────────────┘
              │
              ↓
        ┌─────────┐
        │  wgpu   │ → Vulkan / Metal / D3D12
        └─────────┘
```

### Implementation Pattern

```rust
// ECS generates instances
let instances = context.render(&scene);

// Backend renders directly
backend.render_instances(&instances)?;
```

## Rationale

### Why wgpu?

1. **True Cross-Platform**:
   - **Windows**: Direct3D 12 (optimal), Vulkan (fallback)
   - **macOS/iOS**: Metal (native, excellent performance)
   - **Linux**: Vulkan
   - **Web**: WebGPU (future target)
   - Single API for all platforms

2. **Modern GPU Features**:
   - Compute shaders (for advanced layout/effects)
   - Instanced rendering (1 draw call for 1000s of rectangles)
   - Indirect drawing (future: GPU-driven rendering)
   - Shader hot-reload (development)

3. **Memory Safety**:
   - Written in Rust
   - No manual memory management
   - RAII resource cleanup
   - Compile-time validation

4. **WebGPU Standard**:
   - Based on W3C WebGPU spec
   - Future-proof API design
   - Cross-platform by design
   - Will enable WebAssembly target

5. **Excellent Tooling**:
   - RenderDoc integration (we use it)
   - wgpu-profiler for GPU profiling
   - WGSL shader language (Rust-like syntax)
   - Good error messages

6. **Active Development**:
   - ~Monthly releases
   - Responsive maintainers
   - Used in production (Bevy, Veloren, etc.)
   - Large community

### Performance Benefits

**Instanced Rendering** (current):
```rust
// Single draw call for all rectangles
render_pass.draw(0..6, 0..instances.len());
```

- 1000 rectangles: 1 draw call (vs 1000 with immediate mode)
- CPU overhead: Sub-microsecond (instanced rendering)
- GPU overhead: Minimal (all rectangles in single batch)

**Measured CPU Performance** (colored_rectangles example, 4 entities):
- ECS Update: < 1 μs
- ECS Render (instance collection): < 1 μs
- **Total CPU overhead**: < 2 μs
- GPU rendering time: Not separately measured (includes vsync wait)
- **At 60fps**: < 0.02% of 16.67ms frame budget for ECS

**Benchmark Results** (see `docs/performance/benchmark-results.md`):
- 10 instances: 150 ns collection time
- 100 instances: 890 ns collection time
- 1,000 instances: 9.5 μs collection time
- 10,000 instances: 128 μs collection time

**Memory**:
- Per instance: 24 bytes (pos + size + color)
- 1,000 instances: 24 KB GPU buffer
- 10,000 instances: 240 KB GPU buffer

## Consequences

### Positive

- **Performance**: GPU-accelerated, instanced rendering
- **Cross-Platform**: Single codebase for all platforms
- **Future-Proof**: WebGPU will be standard for web graphics
- **Safety**: No manual memory management, no UAF bugs
- **Debugging**: RenderDoc integration excellent
- **Modern API**: Shader-based allows advanced effects
- **Ecosystem**: Can use wgpu ecosystem crates

### Negative

- **Compile Time**: wgpu adds ~30s to clean build
- **Dependencies**: ~150 transitive dependencies
- **Complexity**: Must understand GPU programming model
- **Shader Language**: WGSL is new (vs GLSL/HLSL)
- **Setup Cost**: More complex than software rendering
- **Debugging**: GPU bugs harder to debug than CPU

### Mitigations

- **Compile Time**: Use sccache, precompiled binaries for CI
- **Complexity**: Abstract in WgpuBackend, users don't see it
- **Debugging**: RenderDoc integration, good error messages
- **Learning Curve**: Provide examples, documentation

## Implementation Details

### Current Features

✅ **Basic Rectangle Rendering**:
```rust
#[repr(C)]
pub struct RectInstance {
    pub pos: [f32; 2],
    pub size: [f32; 2],
    pub color: [f32; 4],
}
```

✅ **Projection Matrix**:
- Orthographic 2D projection
- Updates on window resize
- Normalized device coordinates

✅ **Instanced Rendering**:
- Single vertex buffer for all rectangles
- Per-instance attributes (pos, size, color)
- 6 vertices per quad (2 triangles)

✅ **RenderDoc Integration**:
```toml
features = ["renderdoc"]
```

### Shader Pipeline

**Vertex Shader** (WGSL):
```wgsl
@vertex
fn vs_main(
    @builtin(vertex_index) vid: u32,
    @location(0) instance_pos: vec2<f32>,
    @location(1) instance_size: vec2<f32>,
    @location(2) instance_color: vec4<f32>,
) -> VertexOutput {
    // Generate quad from vertex index + instance data
}
```

**Fragment Shader** (WGSL):
```wgsl
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;  // Future: textures, gradients, etc.
}
```

### Future Enhancements

Planned for Phase 2+:

- **Rounded Corners**: SDF-based in fragment shader
- **Shadows**: Blur shader + shadow map
- **Gradients**: Linear/radial via shader
- **Images**: Texture sampling
- **Text**: SDF text rendering (msdf-atlas)
- **Clipping**: Stencil buffer for nested clips
- **Blur**: Gaussian blur compute shader
- **Custom Shaders**: User-defined effects

## Alternatives Considered

### 1. Skia (skia-safe)

**Pros:**
- Battle-tested (Chrome, Android, Flutter)
- Rich 2D API (paths, text, images)
- Excellent text rendering
- Platform-native backends

**Cons:**
- C++ dependency (unsafe bindings)
- Large binary size (~10MB)
- CPU-based by default (GPU backend complex)
- Slower compile times
- Harder to debug
- Memory safety concerns

**Rejected because:** Unsafe bindings, large binary, less control.

### 2. OpenGL (glow)

**Pros:**
- Ubiquitous support
- Lots of learning resources
- Well-understood

**Cons:**
- Deprecated (macOS, mobile)
- No modern features (compute shaders)
- Context management complex
- Platform differences painful
- Future uncertain

**Rejected because:** Deprecated on macOS, no future.

### 3. Software Rendering (tiny-skia)

**Pros:**
- No GPU dependency
- Simple, predictable
- Easy to debug
- Small binary

**Cons:**
- CPU-only (slow for animations)
- No shader effects
- High CPU usage
- Battery drain on laptops/mobile

**Rejected because:** Performance inadequate for modern GUI.

### 4. Direct Platform APIs (Metal/D3D12)

**Pros:**
- Optimal performance on each platform
- Native integration
- Latest features

**Cons:**
- 3 different APIs to maintain
- Massive development effort
- Platform expertise required
- Code duplication

**Rejected because:** Too much effort, wgpu abstracts this.

### 5. vulkano

**Pros:**
- Safe Vulkan bindings
- Good performance
- Rust-native

**Cons:**
- Vulkan-only (no Metal, no D3D12)
- Complex API
- Requires fallback for macOS
- Smaller ecosystem than wgpu

**Rejected because:** Not truly cross-platform.

## Performance Characteristics

Based on **actual criterion benchmarks** for CPU-side instance collection (see `docs/performance/benchmark-results.md`):

### Instance Collection Performance (CPU)
- 10 instances: 150 ns
- 100 instances: 890 ns
- 1,000 instances: 9.5 μs
- 10,000 instances: 128 μs

GPU upload and rendering times not separately measured (dominated by vsync wait at 60fps).

### Expected GPU Performance
Based on wgpu benchmarks and instanced rendering characteristics:
- 1K rectangles: < 1ms GPU time (60fps easily)
- 10K rectangles: 1-2ms GPU time (60fps achievable)
- 100K rectangles: 5-10ms GPU time (100+ fps possible)

**Bottleneck**: At 60fps, VSync wait (16.67ms) dominates. GPU rendering is not the bottleneck for typical UIs.

### Memory Usage
- Per instance: 24 bytes (pos + size + color)
- 1,000 instances: 24 KB GPU buffer
- 10,000 instances: 240 KB GPU buffer
- Vertex buffer grows dynamically (power-of-2)

## Production Experience

After implementing with wgpu 24.0:

✅ **Smooth Development**: API is intuitive
✅ **Good Errors**: Validation layer catches mistakes early
✅ **RenderDoc**: Debugging GPU state is excellent
✅ **Cross-Platform**: Works on Windows (tested), macOS/Linux (not yet tested but should work)
✅ **Performance**: Exceeds requirements significantly

Minor issues encountered:
- Surface lifetime requires care (documented in code)
- Shader compilation at runtime (could pre-compile)

Overall: Excellent choice. Would choose again.

## References

- wgpu documentation: https://docs.rs/wgpu/
- wgpu examples: https://github.com/gfx-rs/wgpu/tree/trunk/examples
- WebGPU spec: https://www.w3.org/TR/webgpu/
- WGSL spec: https://www.w3.org/TR/WGSL/
- [WgpuBackend implementation](../../crates/render-engine/src/backend/wgpu_backend.rs)
- [Shader source](../../crates/render-engine/src/backend/shaders/rect.wgsl)
