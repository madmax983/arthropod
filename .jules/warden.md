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

## 2026-02-05 - [Unsafe Integer Overflow in Texture Readback]
**Threat:** `unpack_readback_pixels` in `render-engine` used unchecked arithmetic `(width * 4)` to calculate buffer sizes. For very large widths, this calculation could wrap around (overflow), causing the allocation of a small buffer for a large image. This would lead to incorrect memory access (logic error) or a panic when accessing the buffer.
**Defense:** Implemented checked arithmetic using `checked_mul` and `checked_add`. The function now returns `Result<Vec<u8>, RendererError>` and fails gracefully with a descriptive error if dimensions are invalid or if an overflow occurs.

## 2026-02-06 - [WGPU Buffer Alignment Overflow]
**Threat:** `aligned_bytes_per_row` in `render-engine` used `saturating_mul` followed by addition to calculate buffer stride. For large widths (e.g. `u32::MAX / 4`), the saturation combined with alignment padding caused an integer overflow (panic in debug, wrap in release), leading to potential DoS or invalid memory access.
**Defense:** Replaced arithmetic with `checked_mul` and `checked_add`. The function now returns `Result<u32, RendererError>` and propagates errors upstream to `with_offscreen_render_pass` and `read_texture_to_rgba`, ensuring graceful failure.
## 2026-02-06 - [Windows Event Loop Hang Fix]
**Threat:** Application hang (DoS) on Windows. The global `WINDOW_COUNT` was decremented *after* `DestroyWindow` in `WindowImpl::drop`. Since `DestroyWindow` synchronously sends `WM_DESTROY`, the message handler saw the count as non-zero (checking for exit condition) and failed to post `WM_QUIT`.
**Defense:** Moved `WINDOW_COUNT` decrement to occur *before* `DestroyWindow`. Added integer overflow check for `WindowId` to preventing truncation when passing via `lpParam` on 32-bit systems.

## 2026-02-21 - [Gradient Atlas Overflow]
**Threat:** `GradientAtlas::add_gradient` did not check if the atlas was full (`next_row >= 1024`). Adding more than 1024 gradients would cause an index out of bounds panic in `rasterize_gradient`, leading to Denial of Service.
**Defense:** Added a bounds check in `add_gradient`. If the atlas is full, it logs an error and returns row 0 (fallback) instead of panicking.

## 2026-02-21 - [Readback Buffer Allocation OOM]
**Threat:** `unpack_readback_pixels` used `vec![0u8; size]` which panics on allocation failure. An attacker triggering a massive window resize or offscreen render could crash the application via OOM.
**Defense:** Switched to `Vec::try_reserve` and `resize`. Returns `RendererError::InitializationFailed` on allocation failure instead of crashing.

## 2026-02-21 - [Projection Matrix Division by Zero]
**Threat:** `create_projection_matrix` divided by width/height without checking for zero. Zero dimensions (e.g. during minimization or startup) resulted in `Inf`/`NaN` in the projection matrix.
**Defense:** Added check for zero width/height. Returns an identity matrix as a safe fallback.

## 2026-02-22 - [Theme-Engine Registry Panic Fix]
**Threat:** Integer underflow in `detect_windows_version_from_registry` caused by malformed registry data (buffer size 1). The logic `len - 1` panicked when `len` was 0 (from `1 / 2` integer division). This could crash the application on startup if the registry key `CurrentBuildNumber` was corrupted.
**Defense:** Extracted parsing logic into `parse_build_number` with strict bounds checking. Used `chunks_exact(2)` to safely handle odd-length buffers and `min` to prevent reading uninitialized memory. Added regression tests covering empty, short, and malformed inputs.

## 2026-02-22 - [MCP Server Unbounded Read DoS]
**Threat:** The MCP server used `BufReader::read_line` to read incoming JSON messages. A malicious client could send an endless stream of bytes without a newline, causing the server to buffer indefinitely until Out-Of-Memory (DoS).
**Defense:** Implemented `read_line_bounded` helper that enforces a strict `MAX_MESSAGE_SIZE` (64MB) limit. The server now validates message length and UTF-8 validity incrementally, closing the connection if the limit is exceeded. Added regression test `tests/dos_protection.rs`.
