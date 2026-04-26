**[Hashbrown replacement inside functions]
**Learning:** Adding a `///` doc comment inside a function causes a compiler error with `-D warnings` due to `unused_doc_comments`.
**Action:** Use standard `//` comments when explaining a performance change applied to a local variable declaration.
**[Optimizing String Reactivity]
**Learning:** You can't capture a mutable reference to a `Component`'s field in a closure and mutate it from within a `with_untracked` read lock on the signal, because it results in a borrow checker error (`closure requires unique access`).
**Action:** Use `with_untracked` to cheaply check equality first (`*new_text != val.last_value`). If true, drop the read lock and do the actual allocation/update in a separate block.
**[Optimizing String Reactivity]
**Learning:** You can't capture a mutable reference to a `Component`'s field in a closure and mutate it from within a `with_untracked` read lock on the signal, because it results in a borrow checker error (`closure requires unique access`).
**Action:** Use `with_untracked` to cheaply check equality first (`*new_text != val.last_value`). If true, drop the read lock and do the actual allocation/update in a separate block.
**Reuse text string buffers during reactive updates**
**Learning:** Updating a `String` by assignment (`last_value = new_text.clone()`) creates a new heap allocation and drops the previous one.
**Action:** Use `last_value.clone_from(&new_text)` to reuse the existing `String` buffer capacity, eliminating a heap allocation on text updates.
**Optimize Path Interner with `hashbrown::HashMap`**
**Learning:** `std::collections::HashMap` uses SipHash, which can be computationally expensive when computing hashes for small keys. Switching to `hashbrown::HashMap` uses AHash by default, which is much faster for this kind of workload and yielded ~30-50% speedup in cache hit tests.
**Action:** Prefer `hashbrown::HashMap` over `std::collections::HashMap` everywhere in critical paths for faster hash maps.

**Optimizing state tracking in `flux-state`**
**Learning:** `std::collections::HashMap` and `HashSet` use SipHash, which is robust but slow for small integer keys like `NodeId` or `ThreadId`. For internal components managing deep reactive dependencies like `flux-state`, hashing overhead can become a bottleneck.
**Action:** Replace `std::collections::{HashMap, HashSet}` with `hashbrown::{HashMap, HashSet}` to utilize the faster `AHash` algorithm for improved state update and dependency tracking performance.
**Pre-allocating Collections in Path Boolean Ops**
**Learning:** In `style-engine::path::to_multi_polygon`, several collections (`outer_indices`, `outer_to_poly`, `exteriors`, `holes_by_poly`) were being initialized with zero capacity in a hot path algorithm. This causes unnecessary reallocation on the heap as elements are pushed. Using `.with_capacity` based on known iteration bounds improves performance by avoiding memory reallocation loops.
**Action:** Always scrutinize loops where vectors/hashmaps are instantiated right before being populated in bounded loops, and initialize them with `.with_capacity()`.

**[AHash for Cache Collections]
**Learning:** `std::collections::HashSet` uses `SipHash` which is cryptographically resistant but slow for local engine caches that are cleared and repopulated every frame (like `renderable_nodes_cache`).
**Action:** Always prefer `hashbrown::HashSet` and `hashbrown::HashMap` in hot loops and systems where cryptographic resistance is not required. It gives us `AHash` for much faster insertions and lookups.
