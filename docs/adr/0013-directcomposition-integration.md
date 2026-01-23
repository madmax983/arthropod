# ADR 0013: DirectComposition Integration for Selective Transparency

**Status:** Accepted

**Date:** 2026-01-22

**Deciders:** Architecture Team

## Context

Arthropod needs to support modern Windows visual effects like Mica and Acrylic backdrop materials. While the `window-vibrancy` crate provides full-window effects, many applications require **selective transparency** - for example, a Mica sidebar with a solid content area.

### The Problem

Standard wgpu swap chains render directly to the window surface without alpha blending awareness from the Desktop Window Manager (DWM). This means:

1. Full-window backdrop effects work (via DWM attributes)
2. But per-region backdrops are impossible - you can't have Mica in one area and solid in another
3. Native Windows apps (Settings, File Explorer) achieve this through DirectComposition

### Requirements

1. **Selective Transparency**: Different backdrop materials in different window regions
2. **wgpu Compatibility**: Must work with existing wgpu rendering pipeline
3. **Layer-Based API**: Clean abstraction for managing material regions
4. **Performance**: Minimal overhead, leverage GPU composition
5. **Graceful Fallback**: Work on systems without DirectComposition support

## Decision

Integrate Windows DirectComposition to create a visual tree that layers backdrop materials behind wgpu's swap chain output.

### Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Desktop Window Manager                    │
│                  (composites with desktop)                   │
└──────────────────────────┬──────────────────────────────────┘
                           ↓
┌─────────────────────────────────────────────────────────────┐
│              DirectComposition Visual Tree                   │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐         │
│  │ Mica Layer  │  │ Solid Layer │  │ wgpu Output │         │
│  │ (backdrop)  │  │ (backdrop)  │  │  (content)  │         │
│  └─────────────┘  └─────────────┘  └─────────────┘         │
└──────────────────────────┬──────────────────────────────────┘
                           ↓
┌─────────────────────────────────────────────────────────────┐
│                    IDCompositionTarget                       │
│                    (bound to HWND)                           │
└─────────────────────────────────────────────────────────────┘
```

### Implementation Layers

**1. COM Infrastructure (`composition.rs`)**
```rust
pub struct CompositionDevice { device: IDCompositionDesktopDevice }
pub struct CompositionTarget { target: IDCompositionTarget }
pub struct CompositionVisual { visual: IDCompositionVisual2 }
pub struct BackdropVisual { visual: IDCompositionVisual2, material: BackdropMaterial }
```

**2. Window Integration (`WindowImpl`)**
```rust
pub struct WindowConfig {
    // ... existing fields ...
    pub composition_mode: bool,  // Opt-in for DirectComposition
}

struct WindowComposition {
    device: CompositionDevice,
    target: CompositionTarget,
    root_visual: CompositionVisual,
}
```

**3. High-Level Layer API (`compositor.rs`)**
```rust
pub struct Compositor { /* manages layers */ }
pub struct Layer { /* visual + material + bounds */ }

// Usage:
let compositor = Compositor::new(&window)?;
let sidebar = compositor.create_layer(BackdropMaterial::Mica)?;
sidebar.set_bounds(Rect::new(0.0, 0.0, 200.0, 600.0));
compositor.commit()?;
```

**4. wgpu Integration (`composition_swap_chain.rs`)**
```rust
// Composition-compatible surface config
pub fn create_composition_surface_config(...) -> SurfaceConfiguration {
    SurfaceConfiguration {
        format: TextureFormat::Bgra8UnormSrgb,  // Required by DirectComposition
        alpha_mode: CompositeAlphaMode::PreMultiplied,  // For DWM blending
        // ...
    }
}
```

### API Design Choice: Layer-Based

We chose a **layer-based API** over alternatives:

| Approach | Pros | Cons |
|----------|------|------|
| Scene-node based | Declarative, follows hierarchy | Couples materials to scene graph |
| Region-based | Decoupled | Manual sync with layout |
| **Layer-based** | Explicit control, flexible z-order | Slightly more complex |

The layer-based approach was chosen because:
- Layers are independent of scene graph lifecycle
- Explicit z-ordering matches DirectComposition's visual tree model
- Easy to understand: create layer, set bounds, commit

## Consequences

### Positive

- **Selective Transparency**: Mica sidebar + solid content now possible
- **Native Integration**: Proper DWM compositor integration
- **Future-Proof**: Ready for Windows UI evolution (SystemBackdrop APIs)
- **Clean API**: Layer abstraction hides COM complexity
- **Opt-In**: Zero cost when `composition_mode: false`

### Negative

- **Windows-Only**: macOS requires different approach (NSVisualEffectView)
- **Complexity**: Additional COM layer to maintain
- **Version Requirements**: Windows 10 1803+ for DirectComposition3
- **Testing**: Requires Windows environment for full testing

### Neutral

- **Performance**: DirectComposition is GPU-accelerated, minimal CPU overhead
- **Memory**: Additional visual tree objects, but lightweight

## Implementation Status

| Component | Status | Notes |
|-----------|--------|-------|
| CompositionDevice | ✅ Complete | D3D11 DXGI backend |
| CompositionTarget | ✅ Complete | HWND binding |
| CompositionVisual | ✅ Complete | Visual tree nodes |
| BackdropVisual | ✅ Complete | Material association |
| Layer API | ✅ Complete | High-level abstraction |
| WindowConfig | ✅ Complete | composition_mode flag |
| wgpu Integration | ✅ Complete | BGRA + PreMultiplied |
| Demo Example | ✅ Complete | directcomposition_demo |

## Alternatives Considered

### 1. window-vibrancy Only
- **Approach**: Use existing crate for full-window effects
- **Rejected**: Cannot do selective transparency

### 2. Bypass wgpu Entirely
- **Approach**: Render directly to DirectComposition surfaces
- **Rejected**: Loses wgpu's cross-platform benefits, high maintenance

### 3. Layer System with Masks
- **Approach**: Render masks to control transparency regions
- **Rejected**: Complex, less performant, doesn't leverage DWM

### 4. Multiple Windows
- **Approach**: Overlay transparent windows for different regions
- **Rejected**: Complex window management, z-order issues

## References

- [DirectComposition Overview](https://docs.microsoft.com/en-us/windows/win32/directcomp/directcomposition-portal)
- [Composition Swap Chains](https://docs.microsoft.com/en-us/windows/win32/comp/comp-swapchain-examples)
- [DWM Backdrop Types](https://docs.microsoft.com/en-us/windows/win32/api/dwmapi/ne-dwmapi-dwm_systembackdrop_type)
- [windows-rs Crate](https://github.com/microsoft/windows-rs)

## Future Work

1. **macOS Support**: Integrate NSVisualEffectView for similar effects
2. **SystemBackdrop API**: Use Windows 11 22H2+ APIs when available
3. **Blur Effects**: Implement Acrylic blur via DirectComposition effects
4. **Animation**: Animate layer transitions using DirectComposition animations
