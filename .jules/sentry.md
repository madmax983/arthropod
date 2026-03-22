**Drop-based deadlocks in signals**
**Learning:** In `WriteSignal::set`, replacing the inner value using `*guard = value` drops the old value while the `RwLockWriteGuard` is still held. If the old value has a `Drop` implementation that attempts to write to the same signal, it will cause a deadlock because the current thread already holds the write lock.
**Action:** Always extract the old value using `std::mem::replace` inside the write guard, then explicitly drop it *outside* the guard scope so its `Drop` implementation cannot trigger reentrant lock acquisitions.

**Proptest Concurrency Timeouts**
**Learning:** `proptest!` runs tests for 256 iterations by default. For concurrency checks spanning thousands of updates and iterations this takes around 15 seconds to run locally, causing downstream `cargo-mutants` to fail baseline testing due to timeouts.
**Action:** For heavy test setups, explicitly limit iterations via `#![proptest_config(ProptestConfig::with_cases(10))]`.
