## 2024-05-22 - Runtime Locking Architecture
**Learning:** In a reactive runtime, holding a global lock while executing user closures (even for simple reads) is a deadlock trap if the user closure accesses other reactive primitives.
**Action:** Always decouple the "lookup" lock from the "value" lock. Use `Arc` to share value handles so the global runtime lock can be released immediately after lookup, allowing fine-grained locking on individual values.
