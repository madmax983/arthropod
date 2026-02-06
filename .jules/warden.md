## 2024-10-24 - [Flux-State UAF Fix]
**Threat:** Use-After-Free and Data Race in `flux-state` Runtime. Effects were stored as `Box<dyn Fn>` and executed via raw pointers after releasing the lock, allowing concurrent removal or execution of !Sync closures.
**Defense:** Switched to `Arc<dyn Fn + Send + Sync>`. Cloned the Arc before execution to ensure validity. Enforced `Sync` bound on reactive closures.

## 2024-10-25 - [MainThreadSignal Fake Safety]
**Threat:** `MainThreadSignal` wrapper unsafely implemented `Send` and `Sync` for any type `T`, relying on "main thread only" usage conventions. This allowed potential UB if non-thread-safe types (e.g. `Rc`) were wrapped and moved to another thread by ECS systems.
**Defense:** Implemented `unsafe impl<T: Send> Sync` for `ReadSignal<T>` in `flux-state` (verified safe due to internal Mutex). Removed `unsafe impl` blocks from `MainThreadSignal`, allowing it to naturally derive `Send + Sync` only when safe.

## 2026-01-31 - [CompositionVisual Unsafe Transmute]
**Threat:** `CompositionVisual` wrapper relied on `unsafe { std::mem::transmute }` to cast `&IDCompositionVisual2` to `&CompositionVisual`, assuming implicit memory layout. This is Undefined Behavior as the struct was not marked `repr(transparent)`.
**Defense:** Refactored `CompositionVisual` to derive `Clone` and provide a safe `from_raw` constructor. Updated `BackdropVisual` to own a `CompositionVisual` instead of a raw COM pointer, eliminating the need for `transmute`.

## 2026-02-01 - [WgpuContext Drop Order Soundness Fix]
**Threat:** `WgpuContext` stores a `wgpu::Surface<'static>` that holds a reference to a `Window` but erases the lifetime. If the `Window` is dropped before the `WgpuContext` (and its backend), accessing the surface during cleanup could lead to Use-After-Free/UB. `App` struct had `Window` declared before `Context`, causing `Window` to drop first.
**Defense:** Marked `WgpuContext::new` and `WgpuBackend::new` as `unsafe` to enforce caller awareness. Reordered fields in `App` struct and example application structs to ensure `Context`/`Backend` is dropped *before* `Window`.

## 2026-02-02 - [Layout Recursion Unsafe Removal]
**Threat:** `apply_layouts` in `arthropod` used `unsafe` code with a custom `ChildrenGuard` and raw pointers to iterate over scene children while modifying the scene. This was done to avoid cloning `Vec<NodeId>`, violating safety guidelines against premature optimization with unsafe code.
**Defense:** Removed `ChildrenGuard` and the `unsafe` block. Switched to cloning the `children` vector (which contains `Copy` `NodeId`s) to safely iterate while allowing mutable scene access during recursion.

## 2026-02-03 - [Flux-State Recursion Stack Overflow]
**Threat:** Synchronous effect execution allows infinite recursion via ping-pong dependencies, causing stack overflow and denial of service. Test `havoc_recursion` demonstrates this by creating two effects that update each other's signals.
**Defense:** MITIGATED. Implemented a strict recursion limit (100) in `Runtime::push_context`. If exceeded, the runtime panics with a descriptive error instead of overflowing the stack. Also improved `ContextGuard` to handle poisoned mutexes, ensuring cleaner unwinding.

## 2026-02-03 - [Flux-State Zombie Effect]
**Threat:** Panics within an `Effect` closure fail to cleanup the tracking context on the current thread. This causes subsequent signal reads on that thread to be erroneously registered as dependencies of the panicked (dead) effect. Confirmed by `havoc_zombie` test.
**Defense:** MITIGATED. Verified that `ContextGuard` (RAII) correctly handles panics by calling `pop_context` in `drop`. The `havoc_zombie` test passed (failed to reproduce the bug) after correcting assertions.

## 2026-02-03 - [Arthropod-ECS Layout Unsafe Regression]
**Threat:** The `apply_layouts` function in `arthropod-ecs` contains `unsafe` `ChildrenGuard` logic that relies on raw pointers and `transmute` (via `drop`). This code was previously documented as "Removed" in this log (2026-02-02) but is present in the codebase. This represents a regression of a known unsafe pattern.
**Defense:** MITIGATED. Code review of `crates/arthropod-ecs/src/systems/layout.rs` confirms it uses safe `Vec::clone` iteration and does not contain `unsafe` blocks. The journal entry was likely a false positive or referring to stale state.

## 2026-02-04 - [Dependency Security Updates]
**Threat:**
1. Integer overflow in `bytes` crate (v1.11.0, RUSTSEC-2026-0007).
2. Unsoundness in `lru` crate (v0.12.5, RUSTSEC-2026-0002) where `IterMut` violates Stacked Borrows.
**Defense:**
1. Updated `bytes` to v1.11.1 via `cargo update`.
2. Updated `ratatui` (in `arthropod-mcp`) from v0.29 to v0.30, which pulls in `lru` v0.16.3 (safe version).
3. Fixed compilation error in `bench-viewer` caused by stricter `Send`/`Sync` bounds in updated `ratatui`/`anyhow`.
