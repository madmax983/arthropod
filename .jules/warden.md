## 2024-10-24 - [Flux-State UAF Fix]
**Threat:** Use-After-Free and Data Race in `flux-state` Runtime. Effects were stored as `Box<dyn Fn>` and executed via raw pointers after releasing the lock, allowing concurrent removal or execution of !Sync closures.
**Defense:** Switched to `Arc<dyn Fn + Send + Sync>`. Cloned the Arc before execution to ensure validity. Enforced `Sync` bound on reactive closures.

## 2024-10-25 - [MainThreadSignal Fake Safety]
**Threat:** `MainThreadSignal` wrapper unsafely implemented `Send` and `Sync` for any type `T`, relying on "main thread only" usage conventions. This allowed potential UB if non-thread-safe types (e.g. `Rc`) were wrapped and moved to another thread by ECS systems.
**Defense:** Implemented `unsafe impl<T: Send> Sync` for `ReadSignal<T>` in `flux-state` (verified safe due to internal Mutex). Removed `unsafe impl` blocks from `MainThreadSignal`, allowing it to naturally derive `Send + Sync` only when safe.
