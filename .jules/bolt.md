**[Performance] Scene Graph Optimization**
**Learning:** `Vec::contains` is O(N) and kills performance when N is large. Replacing with `HashSet` for dirty tracking makes it O(1).
**Action:** Always use `HashSet` or `BitSet` for "dirty" sets where N can be large.

**[Performance] Hash Lookup Reduction**
**Learning:** Optimistic insertion (check parent exists, link child, insert node) reduces lookups.
**Action:** Minimize hash lookups in hot paths like scene construction.

**[Performance] Allocation Avoidance**
**Learning:** Avoiding `Box` allocation for solid color nodes yields significant speedup.
**Action:** Provide specialized constructors for common, simple cases.

**[Lock Contention in Reactive Primitives]**
**Learning:** `Signal::get` and `Computed::get` are hot paths. Acquiring the `RuntimeInner` lock multiple times (once for tracking, once for handle retrieval) adds significant overhead (~20-30% of total read time).
**Action:** Combine operations into single-lock methods on the Runtime (`track_and_get_signal`, `track_and_get_computed_if_fresh`). This reduces lock acquisitions from 2-3 to 1 in the happy path.
