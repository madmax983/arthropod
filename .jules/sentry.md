# Sentry's Journal

## 2024-05-22 - [Initial Audit]
**Learning:** Started fresh audit of `flux-state`.
**Action:** Adding comprehensive unit tests to `signal.rs`, `computed.rs`, and `effect.rs` to cover core reactive primitives.

## 2024-05-23 - [Reactive Graph Correctness]
**Learning:** Verified that `flux-state` correctly handles "Glitch Freedom" (diamond dependencies) and dynamic dependency pruning. The runtime correctly avoids unnecessary re-computations and executions.
**Action:** Used `AtomicUsize` counters to precisely measure execution counts in diamond patterns. This pattern is highly effective for verifying reactive graph behavior without relying on timing or race conditions.

## 2024-10-24 - [Panic Safety in Reactive Runtime]
**Learning:** Panics within an `Effect` closure were leaking into the `Runtime`'s thread-local `tracking_context`, causing subsequent unrelated signal reads to be incorrectly tracked as dependencies of the failed effect. This "zombie" dependency would cause a second panic when the unrelated signal was updated.
**Action:** Implemented a `ContextGuard` RAII wrapper in `Runtime` that ensures `pop_context` is always called, even during stack unwinding.

## 2024-10-24 - [Effect Drop Behavior]
**Learning:** `Effect::new` returns a handle that unsubscribes the effect when dropped. This is subtle and caused a test failure where an effect stopped working immediately because its return value was ignored.
**Action:** Always bind the result of `Effect::new` (e.g., `let _keep_alive = ...`) in tests or long-lived scopes to keep the subscription active.

## 2024-10-24 - [Reactive State Desynchronization]
**Learning:** `TextInputState` duplicated state (cursor position) without subscribing to the underlying signal. When the signal was updated externally (e.g., text shortened), the cursor position remained invalid, causing silent failures in subsequent edits.
**Action:** Implemented "lazy clamping" in all state mutation methods. Always check bounds against the *current* signal value before performing operations. Do not assume cached state is valid in a reactive system.

## 2024-05-24 - [Silent Stale Value Bug]
**Learning:** If a `Computed` value panics during recomputation, it is prematurely unmarked as `stale` before the new value is stored. This causes subsequent reads to return the old (stale) value without re-attempting the computation, even if dependencies have changed.
**Action:** Modified `ContextGuard` to restore the node's `stale` status if the computation panics. This ensures correct panic recovery while preserving "optimistic staleness removal" needed for cycle handling.

## 2024-05-24 - [Debug Reactivity]
**Learning:** `Debug` formatting of reactive primitives (Signal, Computed) was non-reactive, meaning effects using only `println!("{:?}", signal)` would not re-run on updates.
**Action:** Always verify that "observation" (even via Debug) triggers "dependency tracking" in reactive systems. Added `runtime.track(id)` to `Debug` implementations.

## 2024-05-25 - [String Manipulation Safety]
**Learning:** `TextInputState` relies heavily on `char_idx_to_byte_idx` to safely map character indices (cursor position) to byte indices for `String` mutation. This prevents panics when handling multi-byte Unicode characters (e.g., Emoji).
**Action:** Added comprehensive unit tests covering Unicode insertion/deletion and boundary conditions to ensure `String::insert` and `String::remove` never panic due to invalid byte indices.

## 2024-10-25 - [Missing Update Coverage]
**Learning:** Found an untested execution path inside `WriteSignal::update()`. `WriteSignal::update()` executes a closure safely bypassing the tracking mechanisms. If it fails, the system might not properly invoke `self.runtime.notify(self.id)`.
**Action:** Created `sentry_correctness.rs` to add tests for `update()` mutating the value and correctly updating the runtime graph by triggering subsequent `Effect` and subscriber calculations.
**Graceful Failure over Panics in Window Handles**
**Learning:** Returning `unreachable!()` in a Windows `wndproc` or `unwrap()`ing a potentially null `HWND` during handle retrieval creates unnecessary panic risks that could crash the application on edge cases or unexpected OS messages.
**Action:** Always prefer safe `ok_or` conversions into standard handle errors (like `raw_window_handle::HandleError::Unavailable`) and properly defer to `DefWindowProcW` instead of forcefully asserting message types.
**[Reactive Graph Coverage & Debug Safety]
**Learning:** Forcing a type mismatch via `std::mem::transmute` just to hit a Debug path is extremely unsafe and introduces UB. Sometimes 100% coverage requires exposing safe internals rather than resorting to UB, or simply accepting acceptable unreached error branches.
**Action:** Avoid `std::mem::transmute` entirely when mocking type mismatches. If an error branch cannot be safely hit from the public API, write safe tests to maximize valid path coverage instead.

## 2024-10-25 - [Form and Focus State Testing]
**Learning:** The `input-engine`'s headless state components (`TextInputState`, `FormState`) interact closely with `flux_state::Signal` and `ValidationState`. Found a potential division-by-zero panic in `cycle_focus_index` when the UI tree contains 0 focusable elements and improved test coverage significantly across form data collection and validation logic.
**Action:** Add boundary value tests for focus cycling calculations (`total == 0`) and always include `#[cfg(test)]` modules that mock `flux_state::Runtime` instances for headless UI testing.
**[Explicit Panic Coverage]
**Learning:** Found several explicit panic points inside `crates/flux-state/src/runtime.rs` that were missing test coverage, specifically when missing/invalid `NodeId`s were requested.
**Action:** Added `#should_panic` tests to exercise these `unwrap_or_else(|| panic!(...))` error bounds, completing full panic path coverage in `runtime.rs`.
