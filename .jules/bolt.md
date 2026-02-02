## 2024-05-22 - Runtime Locking Architecture
**Learning:** In a reactive runtime, holding a global lock while executing user closures (even for simple reads) is a deadlock trap if the user closure accesses other reactive primitives.
**Action:** Always decouple the "lookup" lock from the "value" lock. Use `Arc` to share value handles so the global runtime lock can be released immediately after lookup, allowing fine-grained locking on individual values.

## Unstable Syntax Trap
**Learning:** `let_chains` (`if a && let b = ...`) is unstable in Rust, but Clippy might suggest it if running on a newer toolchain, causing build failures on stable.
**Action:** Stick to nested `if` statements for complex conditions involving bindings, even if Clippy suggests collapsing them, unless strict compiler version pinning guarantees support.
