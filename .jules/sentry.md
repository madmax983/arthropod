**Drop-based deadlocks in signals**
**Learning:** In `WriteSignal::set`, replacing the inner value using `*guard = value` drops the old value while the `RwLockWriteGuard` is still held. If the old value has a `Drop` implementation that attempts to write to the same signal, it will cause a deadlock because the current thread already holds the write lock.
**Action:** Always extract the old value using `std::mem::replace` inside the write guard, then explicitly drop it *outside* the guard scope so its `Drop` implementation cannot trigger reentrant lock acquisitions.
