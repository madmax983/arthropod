## 2024-10-24 - [Flux-State UAF Fix]
**Threat:** Use-After-Free and Data Race in `flux-state` Runtime. Effects were stored as `Box<dyn Fn>` and executed via raw pointers after releasing the lock, allowing concurrent removal or execution of !Sync closures.
**Defense:** Switched to `Arc<dyn Fn + Send + Sync>`. Cloned the Arc before execution to ensure validity. Enforced `Sync` bound on reactive closures.
