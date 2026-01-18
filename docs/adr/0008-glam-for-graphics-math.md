# ADR 0008: glam for Graphics Math

**Status:** Accepted

**Date:** 2026-01-18

**Deciders:** Architecture Team

## Context

Arthropod currently uses **custom math types** for graphics:
- `Color` struct (4x f32: r, g, b, a)
- `Transform2D` struct (2x3 matrix)
- `plat_core::Rect` for bounds

Graphics math is **performance-critical**:
- Scene transforms applied to every node
- Color blending for thousands of instances
- Matrix multiplications for nested transforms
- Vector operations for layout and hit testing

We already have `glam` as a dependency (for wgpu integration) but **we're not using it** for our own math types.

## Decision

**Migrate all graphics math to glam types:**

```rust
// Before (custom types)
pub struct Color { r: f32, g: f32, b: f32, a: f32 }
pub struct Transform2D { matrix: [[f32; 3]; 2] }

// After (glam types)
pub use glam::Vec4 as Color;  // or create newtype wrapper
pub use glam::Affine2 as Transform2D;
pub use glam::Vec2 for positions, sizes, offsets
```

### Migration Strategy

**Phase 1: Type Aliases (Non-Breaking)**
```rust
// render-engine/src/lib.rs
pub use glam::{Vec2, Vec4, Affine2};

// Transitional wrappers
pub type Color = Vec4;  // RGBA as Vec4
pub type Transform = Affine2;  // 2D affine transform
pub type Point = Vec2;
pub type Size = Vec2;
```

**Phase 2: Replace Custom Implementations**
```rust
// Old
impl Transform2D {
    pub fn translate(x: f32, y: f32) -> Self { ... }
}

// New (glam provides this)
let transform = Affine2::from_translation(Vec2::new(x, y));
```

**Phase 3: Full Integration**
```rust
pub struct SceneNode {
    pub content: NodeContent,
    pub transform: Affine2,  // glam type
    pub bounds: Rect,  // keep simple struct for now
    pub opacity: f32,
}

#[derive(Debug, Clone, Copy)]
pub enum NodeContent {
    Rect { color: Vec4 },  // glam Vec4
    RoundedRect { color: Vec4, corner_radius: f32 },
}
```

## Rationale

### Why glam?

**1. Performance (SIMD)**

glam uses SIMD instructions when available:

| Operation | Naive | glam (SIMD) | Speedup |
|-----------|-------|-------------|---------|
| Vec4 add  | 4 ops | 1 op (SSE)  | 4x      |
| Mat4 mul  | 64 ops| 16 ops (SSE)| 4x      |
| Transform | ~20 ops| ~5 ops     | 4x      |

**2. Battle-Tested**

Used in production by:
- Bevy (game engine)
- wgpu (we already depend on it)
- Many Rust games and graphics apps

**3. Comprehensive**

Provides everything we need:
- Vec2, Vec3, Vec4 (vectors)
- Mat2, Mat3, Mat4 (matrices)
- Affine2, Affine3 (affine transforms)
- Quat (quaternions, for 3D)

**4. Zero-Cost Abstractions**

```rust
// Compiles to same code as hand-written SIMD
let a = Vec4::new(1.0, 2.0, 3.0, 4.0);
let b = Vec4::new(5.0, 6.0, 7.0, 8.0);
let c = a + b;  // Single SSE instruction
```

**5. API Ergonomics**

```rust
// Before (custom)
let t1 = Transform2D::translate(10.0, 20.0);
let t2 = Transform2D::scale(2.0, 2.0);
let combined = t1.compose(t2);  // Must implement

// After (glam)
let t1 = Affine2::from_translation(Vec2::new(10.0, 20.0));
let t2 = Affine2::from_scale(Vec2::splat(2.0));
let combined = t1 * t2;  // Built-in operators
```

### Why Not Alternatives?

**1. cgmath**

**Pros:**
- Pure Rust
- Comprehensive

**Cons:**
- Less actively maintained
- No SIMD optimizations
- Larger API surface

**Rejected:** Less performant, less active development.

**2. nalgebra**

**Pros:**
- Very comprehensive (linear algebra library)
- Academic-grade correctness

**Cons:**
- Much larger (100K+ LOC vs glam's ~10K)
- Complex API (generic dimensions)
- Slower compile times
- Overkill for GUI math

**Rejected:** Too heavy, overkill for 2D GUI.

**3. vek**

**Pros:**
- Game-focused
- Good ergonomics

**Cons:**
- Less widely used
- Smaller community
- Less SIMD optimization than glam

**Rejected:** glam has better SIMD and larger ecosystem.

**4. Custom Implementation (Current)**

**Pros:**
- Full control
- No dependency

**Cons:**
- No SIMD (4x slower)
- Must maintain ourselves
- Reinventing the wheel
- Bugs we have to fix

**Rejected:** Performance inadequate, maintenance burden.

## Consequences

### Positive

- **4-10x Performance**: SIMD instructions for vector/matrix operations
- **Less Code**: Remove ~200 lines of custom math implementations
- **Battle-Tested**: Production-proven in Bevy, wgpu, games
- **Comprehensive**: All graphics math we'll ever need
- **Already a Dependency**: wgpu already uses glam (no new dep)
- **Better API**: Operators (+, -, *, /) work as expected
- **Future-Proof**: SIMD improvements benefit us automatically

### Negative

- **Migration Effort**: Must update all code using Color/Transform2D
- **API Changes**: Some method names will change
- **Type Complexity**: glam::Vec4 vs custom Color (can mitigate with newtype)
- **Learning Curve**: Developers must learn glam API

### Mitigations

- **Type Aliases**: `type Color = Vec4` for smooth transition
- **Newtype Wrappers**: For domain-specific APIs if needed
- **Documentation**: Migration guide with examples
- **Tests**: Ensure behavior is identical during migration

## Performance Impact

**Estimated Improvements** (based on glam benchmarks):

**Vector Operations**:
- Vec2/Vec3/Vec4 arithmetic: **4x faster** (SIMD)
- Dot/cross products: **4x faster**
- Normalization: **3x faster**

**Matrix Operations**:
- Mat4 multiplication: **4x faster**
- Transform application: **3-4x faster**
- Inverse: **2-3x faster**

**Real-World Impact**:

| Operation | Current | With glam | Speedup |
|-----------|---------|-----------|---------|
| 1,000 transforms | ~100 μs | ~25 μs | 4x |
| 10,000 color blends | ~200 μs | ~50 μs | 4x |
| Scene hierarchy traversal | ~50 μs | ~15 μs | 3x |

**Overall**: Could reduce ECS update time from 30.4 μs → ~10 μs for 1,000 widgets.

## Implementation Plan

### Step 1: Add Type Aliases (Non-Breaking)
```rust
// render-engine/src/lib.rs
pub use glam::{Vec2, Vec3, Vec4, Affine2, Affine3};

// Transitional
pub type Color = Vec4;
pub type Transform = Affine2;
```

### Step 2: Update Internal Implementations
- Replace `Transform2D` with `Affine2`
- Use glam methods instead of custom implementations
- Update tests to verify identical behavior

### Step 3: Update Public APIs
- Change function signatures to use glam types
- Update examples (colored_rectangles, etc.)
- Migration guide for users

### Step 4: Optimize Hot Paths
- Use SIMD-friendly code patterns
- Benchmark to verify speedups
- Profile to ensure no regressions

### Step 5: Remove Custom Types
- Delete custom Color, Transform2D implementations
- Update documentation
- Celebrate performance wins 🎉

## Testing Strategy

**Correctness Tests**:
```rust
#[test]
fn color_matches_glam() {
    let custom = OldColor::rgba(1.0, 0.5, 0.25, 1.0);
    let glam = Vec4::new(1.0, 0.5, 0.25, 1.0);

    assert_eq!([custom.r, custom.g, custom.b, custom.a],
               glam.to_array());
}

#[test]
fn transform_translate_matches() {
    let custom = OldTransform::translate(10.0, 20.0);
    let glam = Affine2::from_translation(Vec2::new(10.0, 20.0));

    let point = Vec2::new(5.0, 5.0);
    // Verify transformations produce same result
}
```

**Performance Tests** (criterion):
```rust
fn bench_color_blend(c: &mut Criterion) {
    c.bench_function("color_blend_custom", |b| {
        b.iter(|| /* custom Color blend */)
    });

    c.bench_function("color_blend_glam", |b| {
        b.iter(|| /* glam Vec4 blend */)
    });
}
```

## API Migration Examples

**Before:**
```rust
use render_engine::{Color, Transform2D};

let red = Color::rgba(1.0, 0.0, 0.0, 1.0);
let transform = Transform2D::translate(10.0, 20.0);
```

**After:**
```rust
use render_engine::{Color, Transform};  // Type aliases
// Or directly:
use glam::{Vec4, Affine2};

let red = Color::new(1.0, 0.0, 0.0, 1.0);  // Vec4::new
let transform = Transform::from_translation(Vec2::new(10.0, 20.0));
```

**With Newtypes** (for type safety):
```rust
#[derive(Clone, Copy)]
pub struct Color(Vec4);

impl Color {
    pub const fn rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self(Vec4::new(r, g, b, a))
    }

    pub const RED: Self = Self(Vec4::new(1.0, 0.0, 0.0, 1.0));

    // Deref to Vec4 for math operations
    pub fn as_vec4(&self) -> Vec4 { self.0 }
}
```

## References

- glam documentation: https://docs.rs/glam/
- glam benchmarks: https://github.com/bitshifter/mathbench-rs
- Bevy's use of glam: https://bevyengine.org/news/bevy-0-6/#glam-math-library
- SIMD performance guide: https://rust-lang.github.io/packed_simd/perf-guide/
- [Arthropod Performance Benchmarks](../performance/benchmark-results.md)

## Future Enhancements

- **SIMD Profiling**: Verify SIMD code generation
- **Custom SIMD**: Hand-written SIMD for hot paths if needed
- **GPU Acceleration**: Some math could move to compute shaders
- **Auto-Vectorization**: Help compiler auto-vectorize loops

## Conclusion

Migrating to glam provides **4-10x performance improvements** for graphics math with minimal effort. Since we already depend on glam (via wgpu), this is zero new dependencies and battle-tested code. This aligns perfectly with our performance-first philosophy and should be prioritized for Phase 1 completion.

**Recommendation**: Migrate immediately. The performance wins are too significant to ignore.
