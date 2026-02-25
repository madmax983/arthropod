**[Performance] Scene Graph Optimization**
**Learning:** `Vec::contains` is O(N) and kills performance when N is large. Replacing with `HashSet` for dirty tracking makes it O(1).
**Action:** Always use `HashSet` or `BitSet` for "dirty" sets where N can be large.

**[Performance] Hash Lookup Reduction**
**Learning:** Optimistic insertion (check parent exists, link child, insert node) reduces lookups.
**Action:** Minimize hash lookups in hot paths like scene construction.

**[Performance] Allocation Avoidance**
**Learning:** Avoiding `Box` allocation for solid color nodes yields significant speedup.
**Action:** Provide specialized constructors for common, simple cases.
