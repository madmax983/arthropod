1. *Add `draggable` to `crates/arthropod/src/experimental/mod.rs`.*
   - Register the new module in the `experimental` module tree.
2. *Create `crates/arthropod/src/experimental/draggable.rs`.*
   - Implement `DraggableNode` component.
   - Implement `update_draggable_nodes` system to check if node is hovered and mouse is clicked, then update its translation using `MousePosition` and `Transform2D`.
   - Implement `register_draggable` function.
3. *Update `crates/arthropod/src/app/core.rs`.*
   - Add a call to `crate::experimental::draggable::register_draggable(&mut app);` inside `App::new_with_backend` when `nova` feature is enabled.
4. *Run tests and formatting.*
   - Ensure `cargo test`, `cargo fmt`, and `cargo clippy` pass.
5. *Pre commit instructions.*
   - Run `pre_commit_instructions` tool to perform required verifications.
6. *Commit and submit.*
   - Create PR with title `🌟 Nova: Draggable Nodes` and appropriate description.
