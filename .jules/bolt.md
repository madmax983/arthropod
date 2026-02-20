## 2024-05-22 - Runtime Locking Architecture
**Learning:** In a reactive runtime, holding a global lock while executing user closures (even for simple reads) is a deadlock trap if the user closure accesses other reactive primitives.
**Action:** Always decouple the "lookup" lock from the "value" lock. Use `Arc` to share value handles so the global runtime lock can be released immediately after lookup, allowing fine-grained locking on individual values.

## Unstable Syntax Trap
**Learning:** `let_chains` (`if a && let b = ...`) is unstable in Rust, but Clippy might suggest it if running on a newer toolchain, causing build failures on stable.
**Action:** Stick to nested `if` statements for complex conditions involving bindings, even if Clippy suggests collapsing them, unless strict compiler version pinning guarantees support.

## Scene Graph Z-Order Traversal
**Learning:** Iterating a flat `HashMap` of scene nodes (`O(N)`) for hit-testing is fast but fundamentally incorrect for overlapping nodes because map iteration order is arbitrary. Correct Z-order requires tree traversal (checking last child first).
**Action:** When implementing hit-testing or rendering in a retained-mode scene graph, always traverse the tree structure to respect hierarchy and sibling order, even if it adds recursion overhead. Optimization (early exit) can offset this cost for "hit" cases.

## Iterative Traversal vs Recursion Performance
**Learning:** Converting a recursive tree traversal to an iterative one using a `Vec` stack (even with `thread_local` reuse) resulted in a ~2x performance regression (68µs -> 175µs for 10k nodes) in micro-benchmarks. The overhead of manual stack management and bounds checks exceeds the compiler-optimized recursive calls.
**Action:** Only replace recursion with iteration when stack depth is a proven crash risk (like here), and accept the minor CPU cost for stability.

**[Heap Reuse in Reactive Graphs]**
**Learning:** Reusing a traversal buffer in `RuntimeInner` eliminates one heap allocation per signal update without fighting the borrow checker, provided the buffer is cleared before use and field access is disjoint.
**Action:** Always look for transient `Vec::new()` in hot loops or recursive structures and hoist them into the parent struct if single-threaded access is guaranteed (e.g., via Mutex).

**[Quadratic Reactive Blowup]**
**Learning:** Checking `pending_effects.contains(&id)` inside the O(N) subscriber notification loop creates an O(N^2) bottleneck. When `stale` set already guards entry, this scan is redundant.
**Action:** Remove redundant collection scans in hot paths when invariants (like `stale` status) guarantee uniqueness. Verified 5x speedup for 20k dependencies (143ms -> 27ms).

**[Batching Lock Acquisitions]**
**Learning:** In systems with fine-grained locking (like reactive runtimes), repeated lock acquisitions for items in a queue (like pending effects) can be a significant bottleneck. Moving the queue to a local buffer under a single lock allows processing without holding the lock or re-acquiring it repeatedly.
**Action:** Look for loops that pop from a shared, locked collection one by one. Replace with `swap` or `append` to take the whole batch.
