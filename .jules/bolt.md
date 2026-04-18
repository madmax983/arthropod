**Hashbrown over std::collections**
**Learning:** `hashbrown::HashMap` and `HashSet` provide a performance advantage because they use `AHash` by default instead of `SipHash` which is used in the standard library.
**Action:** When a `HashMap` or `HashSet` is created and the keys do not need to be robust to collision attacks (i.e. most internal structures keyed by things like `NodeId` or `String`), prefer `hashbrown::HashMap` to improve performance.


**Hashbrown over std::collections**
**Learning:** `hashbrown::HashMap` and `HashSet` provide a performance advantage because they use `AHash` by default instead of `SipHash` which is used in the standard library.
**Action:** When a `HashMap` or `HashSet` is created and the keys do not need to be robust to collision attacks (i.e. most internal structures keyed by things like `NodeId` or `String`), prefer `hashbrown::HashMap` to improve performance.
