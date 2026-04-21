**[Hashbrown replacement inside functions]
**Learning:** Adding a `///` doc comment inside a function causes a compiler error with `-D warnings` due to `unused_doc_comments`.
**Action:** Use standard `//` comments when explaining a performance change applied to a local variable declaration.
**Reuse text string buffers during reactive updates**
**Learning:** Updating a `String` by assignment (`last_value = new_text.clone()`) creates a new heap allocation and drops the previous one.
**Action:** Use `last_value.clone_from(&new_text)` to reuse the existing `String` buffer capacity, eliminating a heap allocation on text updates.
