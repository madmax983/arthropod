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
