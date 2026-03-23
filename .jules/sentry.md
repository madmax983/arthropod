**Drop-based deadlocks in signals**
**Learning:** In `WriteSignal::set`, replacing the inner value using `*guard = value` drops the old value while the `RwLockWriteGuard` is still held. If the old value has a `Drop` implementation that attempts to write to the same signal, it will cause a deadlock because the current thread already holds the write lock.
**Action:** Always extract the old value using `std::mem::replace` inside the write guard, then explicitly drop it *outside* the guard scope so its `Drop` implementation cannot trigger reentrant lock acquisitions.

**Proptest Concurrency Timeouts**
**Learning:** `proptest!` runs tests for 256 iterations by default. For concurrency checks spanning thousands of updates and iterations this takes around 15 seconds to run locally, causing downstream `cargo-mutants` to fail baseline testing due to timeouts.
**Action:** For heavy test setups, explicitly limit iterations via `#![proptest_config(ProptestConfig::with_cases(10))]`.

**Stale While Computing Propagation**
**Learning:** In reactive systems, if a dependency updates while a computed node is actively computing, the computed node must be marked as `stale_while_computing` so it re-evaluates next time. However, if this staleness is not propagated to its subscribers (e.g. because of an early-exit check that it's already stale), those subscribers will never know they need to recompute, leading to lost updates.
**Action:** Always ensure that when marking a computing node as stale, the propagation logic still runs to mark its subscribers as stale, even if the node itself was already in the `stale` set.
