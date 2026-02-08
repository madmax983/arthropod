---
name: Bevy Integration Roadmap
about: Integrate Arthropod as a Bevy UI plugin
title: "[ROADMAP] Bevy Plugin Integration - arthropod-bevy"
labels: enhancement, roadmap, bevy-integration
assignees: ''
---

## 🎮 Vision

Integrate Arthropod as a first-class Bevy UI plugin, providing the Bevy ecosystem with high-performance, reactive, declarative UI that addresses [bevy#254](https://github.com/bevyengine/bevy/issues/254) and related UI ergonomics issues.

## 🎯 Problem Statement

**Bevy's UI Pain Points:**
- ❌ No reactive state management (manual UI updates on every change)
- ❌ Verbose, imperative construction (20+ lines for simple buttons)
- ❌ Limited widget library (basic nodes only)
- ❌ Poor ergonomics for complex layouts
- ❌ No declarative macros or builder patterns

**What Arthropod Brings:**
- ✅ MobX-style reactive state (`flux-state`)
- ✅ 15+ production-ready widgets
- ✅ Declarative macros: `col!()`, `btn!()`, `txt!()`
- ✅ Flexbox layout via `taffy`
- ✅ Theme system with design tokens
- ✅ **Proven performance**: 127μs for 1,000 widgets (0.76% of 60fps budget)

## 🏗️ Technical Approach

### Current Architecture Alignment

| Component | Arthropod | Bevy | Status |
|-----------|-----------|------|--------|
| **ECS** | `bevy_ecs 0.15` | `bevy_ecs 0.15` | ✅ 100% compatible |
| **Rendering** | WGPU 28.0 | WGPU (bevy_render) | ✅ Same backend |
| **Math** | `glam` | `glam` | ✅ Shared types |
| **Scene Graph** | Custom | Bevy hierarchy | ⚠️ Needs mapping |
| **Reactive State** | `flux-state` | None | ✅ New capability |

### Integration Points

**1. Windowing (Easy)**
- Replace `plat-core` with `bevy_winit`
- Use Bevy's `Window` resource
- Map Bevy events to Arthropod dispatcher

**2. Rendering (Medium)**
- Create `ArthropodUINode` for Bevy's `RenderGraph`
- Reuse instance buffer approach
- Let Bevy manage GPU resources
- Support multiple UI roots per camera

**3. ECS Integration (Easy - already compatible!)**
- `FrameworkContext` as Bevy resource
- Arthropod systems in Bevy's schedule
- Widget entities coexist with game entities
- Scene as Bevy resource

**4. Assets (Medium)**
- Integrate with `bevy_asset` for fonts
- Use `bevy_image` for textures
- Theme as loadable asset

**5. Input (Easy)**
- Subscribe to Bevy's input events
- Mouse, keyboard, touch mapping
- Focus management

## 📋 Implementation Plan

### Phase 1: Proof of Concept (1-2 weeks)
**Goal:** Arthropod UI rendering in Bevy window

- [ ] Create `crates/arthropod-bevy` plugin crate
- [ ] Replace windowing: `plat-core` → `bevy_winit`
- [ ] Basic render graph integration
- [ ] Demo: Counter app in Bevy
- [ ] Verify ECS systems run correctly

**Success Criteria:**
- Arthropod widgets render in Bevy window
- Reactive updates work (button clicks, state changes)
- 60fps maintained for 1,000+ widgets

### Phase 2: Full Integration (2-3 weeks)
**Goal:** Production-ready plugin

- [ ] Asset system integration (fonts via `bevy_asset`)
- [ ] Complete event system bridge
- [ ] Multiple UI roots per camera
- [ ] Camera-relative positioning
- [ ] Z-ordering with 3D scene
- [ ] Focus and keyboard navigation

**Success Criteria:**
- Load fonts/themes as Bevy assets
- UI overlay on 3D game scene
- Mouse hover works with camera transforms
- Tab navigation between widgets

### Phase 3: Polish & Release (1-2 weeks)
**Goal:** Community-ready crate

- [ ] Comprehensive documentation
- [ ] 5+ example games (HUD, menu, inventory, dialog, settings)
- [ ] Bevy version compatibility matrix
- [ ] Performance benchmarks (compare to `bevy_ui`, `egui`)
- [ ] Publish to crates.io as `bevy_arthropod`
- [ ] Blog post and showcase

**Success Criteria:**
- Examples build and run on Windows/Linux/macOS
- Documentation covers all features
- Published to crates.io
- Community feedback cycle started

## 🎨 Example Usage

**Current Bevy UI (verbose, imperative):**
```rust
commands.spawn(NodeBundle {
    style: Style {
        width: Val::Px(100.0),
        height: Val::Px(50.0),
        flex_direction: FlexDirection::Column,
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        padding: UiRect::all(Val::Px(10.0)),
        // ... many more lines
    },
    background_color: Color::rgb(0.2, 0.2, 0.2).into(),
    ..default()
});
```

**With Arthropod (declarative, reactive):**
```rust
fn setup_ui(mut commands: Commands, runtime: Res<Runtime>) {
    let health = ctx.signal(100);
    let (read, write) = health.split();

    commands.spawn(ArthropodUIBundle {
        root: col!([
            txt!(Computed::new(runtime, move ||
                format!("Health: {}", read.get())
            ), size: 24.0),
            btn!("Heal", primary, on_click: move || {
                write.update(|h| *h = (*h + 10).min(100));
            }),
            btn!("Attack", on_click: || {
                // Trigger game event
            }),
        ], gap: 10.0, padding: 20.0),
    });
}
```

## 📊 Market Opportunity

- **Bevy**: 35,000+ GitHub stars, leading Rust game engine
- **UI is #1 pain point**: [bevy#254](https://github.com/bevyengine/bevy/issues/254) has 600+ reactions
- **Existing solutions**: `bevy-egui` (heavy), `kayak_ui` (abandoned), `sickle_ui` (early)
- **Arthropod advantages**: Performance + ergonomics + completeness

## 🔗 Related Resources

- [Bevy UI Roadmap](https://github.com/bevyengine/bevy/issues/254)
- [Bevy Render Architecture](https://bevyengine.org/learn/book/getting-started/plugins/)
- [Arthropod Performance Benchmarks](./docs/performance/benchmark-results.md)
- [flux-state Reactivity Guide](./crates/flux-state/README.md)

## 🤝 Call for Contributors

This is a significant undertaking that could benefit the entire Bevy ecosystem. Looking for contributors interested in:
- Bevy render graph integration
- Asset system design
- Game UI/UX patterns
- Cross-platform testing

## ✅ Definition of Done

- [ ] `bevy_arthropod` published to crates.io
- [ ] Works with Bevy 0.15+
- [ ] 5+ working examples
- [ ] Documentation complete
- [ ] Benchmarks show competitive performance
- [ ] Community feedback positive
- [ ] RFC submitted to Bevy (optional)

---

**Strategic Value:** This gives Arthropod two paths forward:
1. **Standalone GUI framework** - compete with iced, egui, Slint
2. **Bevy UI solution** - tap into massive game dev community

Both benefit from the same core architecture and performance work.
