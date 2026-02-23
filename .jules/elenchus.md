# Elenchus's Journal

## 2024-05-25 - [Audit of flux-state]
**Module:** `crates/flux-state`
**Verdict:** 🟢 Acquitted (mostly)

### Findings

| Test File | Verdict | Reasoning |
| :--- | :--- | :--- |
| `sentry_correctness.rs` | 🟢 Acquitted | rigorous tests for glitch freedom, dynamic pruning, and panic state consistency. |
| `sentry_panic_recovery.rs` | 🟢 Acquitted | Verify panic recovery and re-execution. |
| `sentry_panic.rs` | 🟢 Acquitted | effectively tests against thread-local context pollution. |
| `sentry_orphaned_effect.rs` | 🟢 Acquitted | critical test ensuring failed effects are cleaned up. |
| `sentry_safety.rs` | 🟢 Acquitted | verifies `untracked` and `lazy` behavior contracts. |
| `sentry_dependency.rs` | 🟡 Suspect | largely redundant with `sentry_correctness.rs` but harmless. |
| `havoc_recursion.rs` | 🟢 Acquitted | confirms recursion limit enforcement for Effects. |
| `sentry_cycles.rs` | 🟡 Suspect | asserts "no panic" but misses opportunity to enforce "returns old value" behavior. |

### Recommendations

1.  **Strengthen `sentry_cycles.rs`**: The test currently only asserts that the cycle doesn't crash. It should assert that the cycle resolves to the previous value (re-entrancy returns old value), as this is the observable behavior that `Computed` relies on to break cycles. If this behavior changes (e.g. to a panic), we want to know.

### Actions Taken

*   Audited all `sentry_*` tests in `flux-state`.
*   Verified that `havoc_recursion.rs` correctly tests the recursion limit.
*   Verified that `sentry_cycles.rs` correctly identifies that `Computed` cycles do not panic (unlike `Effect` cycles).
*   Planning to strengthen `sentry_cycles.rs` to assert specific values.

## 2024-05-26 - [Audit of flux-state Phase 2]
**Module:** `crates/flux-state`
**Verdict:** 🟢 Acquitted (Strengthened)

### Findings

| Test File | Verdict | Reasoning |
| :--- | :--- | :--- |
| `sentry_correctness.rs` | 🟢 Acquitted (Strengthened) | `test_glitch_freedom_effect` now asserts consistency of values seen by effect, preventing "Lost Update" bugs from passing unnoticed. |
| `havoc_race.rs` | 🟢 Acquitted | Properly uses `#[should_panic]` to document a known race condition (The Wreckage). |
| `sentry_cycles.rs` | 🟢 Acquitted | Verified that it asserts specific values (A=2, B=1) consistent with the current implementation's cycle-breaking strategy. |

### Actions Taken

*   Updated `test_glitch_freedom_effect` in `sentry_correctness.rs` to assert `3*B == 2*C`, ensuring that the effect always sees a consistent snapshot of the dependency graph (avoiding mixed old/new values).
