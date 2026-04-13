## Findings

1. **Severity:** High
   - **File:** `crates/arthropod-ecs/src/systems/reactive.rs`
   - **What can break:** `update_all_reactive_system` silently triggers re-evaluation for components and Layout Engine recalculations every single frame, causing extreme CPU spikes and continuous change detection in Bevy ECS, and performance regressions.
   - **Why it breaks:** The monolithic `update_all_reactive_system` queries optional components and wraps helper functions with auto-dereferencing (`.as_deref_mut()`). Converting an `Option<Mut<'_, T>>` via `.as_deref_mut()` to `Option<&mut T>` inherently triggers the `DerefMut` trait on the Bevy `Mut` wrapper. This unconditionally marks the component as changed in ECS state, even if the actual property check within the helper function results in a no-op (because the tracked reactive value didn't actually change). This causes anything dependent on Bevy change detection for these values (such as `layout_system`) to run wastefully on every frame.
   - **Minimal fix:** Update helper functions like `update_layout_width`, `update_color`, etc., to take the `Mut` wrapper directly by mutable reference (e.g., `&mut Option<Mut<'_, T>>`). Inside the helpers, inspect the value immutably first via `.as_deref()` to determine if a change is needed. Only if an actual update is required should you use `as_deref_mut()` or direct mutation, triggering ECS change detection only when necessary.
   - **Required tests:** A characterization test or benchmark verifying that running `update_all_reactive_system` multiple times without underlying signal changes does not trigger ECS `Changed<T>` events. (Some existing tests implicitly verify correct behavior but missing explicit change detection assertions).

## Test gaps
- Missing explicit test for ECS change detection correctly reflecting mutations only when reactive properties change values.

## Patch plan
1.  Modify `update_layout_width`, `update_progress_bar_width`, `update_layout_flex_grow`, `update_color`, `update_text`, `update_computed_text`, `update_transform`, and `update_opacity` in `crates/arthropod-ecs/src/systems/reactive.rs` to take `&mut Option<Mut<'_, T>>` instead of `Option<&mut T>`.
2.  Update the inner logic of these helpers to first use `.as_deref()` to read values and compare them.
3.  Only call `.as_deref_mut()` when an update is actually necessary.
4.  Update the `update_all_reactive_system` call sites to pass mutable references to the options (e.g., `&mut layout` instead of `layout.as_deref_mut()`).
