1. **Understand Changed Behavior & Invariants**:
   - The diff introduces a monolithic `update_all_reactive_system` that replaces several individual reactive update systems (like colors, text, transform, etc.).
   - It also extracted the update logic into helper functions.
   - However, in `update_progress_bar_width`, I notice:
     ```rust
     fn update_progress_bar_width(
         layout: &mut Option<Mut<'_, LayoutStyle>>,
         progress_bar: &mut Option<Mut<'_, ProgressBarState>>,
     ) {
         if let Some(val) = progress_bar.as_mut() {
             let prog = val.progress.get_untracked().clamp(0.0, 1.0);
             let delta = (prog - val.last_progress).abs();
             if delta >= 0.0001 {
                 let total_width = val.total_width;
                 val.last_progress = prog;
                 if let Some(l) = layout.as_mut() {
                     l.0.width = Some(prog * total_width);
                 }
             }
         }
     }
     ```
     Wait, I read in the git history that there was an issue where `ProgressBarState` updates caused infinite loops because of inconsistent clamping.
     Is there another issue here? The user prompt asks me to perform one focused review pass and identify the *highest-impact* correctness/regression risk.

     Let's look at `update_all_reactive_system` again.
     The system iterates over `ReactiveQuery<'w>`.
     Wait, looking at `update_progress_bar_width` vs `update_layout_flex_grow` or `update_color`, is there an aliasing/correctness issue?

     Ah, `progress_bar.as_mut()` triggers change detection!
     In Bevy ECS, `Option<Mut<T>>::as_mut()` returns an `Option<&mut Mut<T>>`. If we use it, does it trigger change detection? No, `Option<Mut<T>>` is usually accessed via `.as_mut()` but change detection is triggered when it's mutably dereferenced.
     Actually, memory says: "In Bevy ECS, calling `.as_mut()` on an `Option<Mut<T>>` query item does not trigger change detection by itself. Change detection is only triggered when `DerefMut` is invoked (e.g., when actually assigning to a field or explicitly taking a mutable reference to the inner value). Accessing fields immutably via `Deref` (e.g., `val.get_untracked()`) does not mark the component as changed."

     Wait! Look at the helpers:
     ```rust
     fn update_color(
         scene: &mut ResMut<'_, Scene>,
         node_id: render_engine::NodeId,
         color: &mut Option<Mut<'_, ReactiveColor>>,
     ) {
         if let Some(val) = color.as_mut() {
             let new_color = val.signal.get_untracked();
             if new_color != val.last_value {
                 val.last_value = new_color;
     ...
     ```
     By doing `val.last_value = new_color;`, `val` is mutably dereferenced, triggering change detection *only when changed*. This is correct.
     But wait! What about `val.signal.get_untracked()`? It requires `val` to be immutable, but `val` is a `&mut Mut<ReactiveColor>`. It will auto-deref to `ReactiveColor`, which is fine.
     But wait... if `get_untracked()` doesn't trigger change detection... does `val.signal.get_untracked()` trigger it if we call a method taking `&self`? No, calling a method taking `&self` via `Deref` does *not* trigger change detection.
     Wait, `val` is a `&mut Mut<T>`. When we do `val.signal.get_untracked()`, does it trigger change detection?
     If `val` is `&mut Mut<T>`, `val.signal` goes through `DerefMut`? No, field access on a `&mut Mut<T>` goes through `DerefMut`!
     Because `Mut` implements `Deref` and `DerefMut`. Accessing a field `val.signal` requires dereferencing `val`. Since we only need a shared reference to the field, does the compiler use `Deref` or `DerefMut`?
     According to Rust rules, if you do `val.signal` where `val` is `&mut Mut<T>`, it uses `DerefMut` because `val` is a mutable reference? No! `Deref` is used if only a shared reference is needed. Wait, does it?
     Actually, if we do `if let Some(val) = color.as_mut() { ... val.signal.get_untracked() }`, `val` is `&mut Mut<T>`. Wait! If `val` is bound as `&mut Mut<T>`, `val.signal.get_untracked()` might trigger `DerefMut` if `get_untracked` takes `&mut self`, but it doesn't. If `val` is `&mut Mut<T>`, `val.as_ref().signal.get_untracked()` would be safer?
     Wait, memory says: "Change detection is only triggered when `DerefMut` is invoked (e.g., when actually assigning to a field or explicitly taking a mutable reference to the inner value). Accessing fields immutably via `Deref` (e.g., `val.get_untracked()`) does not mark the component as changed." This means `val.signal.get_untracked()` uses `Deref` and does *not* trigger change detection!

     Let's look at `update_text`:
     ```rust
     fn update_text(
         scene: &mut ResMut<'_, Scene>,
         node_id: render_engine::NodeId,
         text: &mut Option<Mut<'_, ReactiveText>>,
     ) {
         if let Some(val) = text.as_mut() {
             // ⚡ Bolt: Check string equality first without allocating, only clone when changed
             let changed = val
                 .signal
                 .with_untracked(|new_text| *new_text != val.last_value);
     ```
     Wait, what about `update_progress_bar_width`?
     ```rust
     fn update_progress_bar_width(
         layout: &mut Option<Mut<'_, LayoutStyle>>,
         progress_bar: &mut Option<Mut<'_, ProgressBarState>>,
     ) {
         if let Some(val) = progress_bar.as_mut() {
             let prog = val.progress.get_untracked().clamp(0.0, 1.0);
             let delta = (prog - val.last_progress).abs();
             if delta >= 0.0001 {
                 let total_width = val.total_width;
                 val.last_progress = prog;
                 if let Some(l) = layout.as_mut() {
                     l.0.width = Some(prog * total_width);
                 }
             }
         }
     }
     ```
     Wait... look at `update_layout_width`:
     ```rust
     fn update_layout_width(
         layout: &mut Option<Mut<'_, LayoutStyle>>,
         width: &mut Option<Mut<'_, ReactiveLayoutWidth>>,
     ) {
         if let Some(val) = width.as_mut() {
             let new_width = val.signal.get_untracked();
             let delta = (new_width - val.last_value).abs();

             if delta >= 0.0001 {
                 val.last_value = new_width;
                 if let Some(l) = layout.as_mut() {
                     l.0.width = Some(new_width);
                 }
             }
         }
     }
     ```
     Wait! In `update_progress_bar_width`:
     ```rust
                 let total_width = val.total_width;
                 val.last_progress = prog;
                 if let Some(l) = layout.as_mut() {
                     l.0.width = Some(prog * total_width);
                 }
     ```
     If we have both `ReactiveLayoutWidth` AND `ProgressBarState` on the same entity, `update_layout_width` sets `l.0.width = Some(new_width)`, and then `update_progress_bar_width` overwrites it with `Some(prog * total_width)`. Is that a bug? They probably shouldn't be used on the same entity.

     Let's look at `update_all_reactive_system`:
     ```rust
     pub fn update_all_reactive_system(mut query: Query<ReactiveQuery<'_>>, mut scene: ResMut<Scene>) {
         for (
             node_ref,
             mut layout,
             mut color,
             mut text,
             mut computed_text,
             mut transform,
             mut opacity,
             mut width,
             mut flex_grow,
             mut progress_bar,
         ) in query.iter_mut()
         {
             update_layout_width(&mut layout, &mut width);
             update_progress_bar_width(&mut layout, &mut progress_bar);
             update_layout_flex_grow(&mut layout, &mut flex_grow);
     ```

     Wait... What if an entity has BOTH `ProgressBarState` and `ReactiveLayoutWidth`? But let's assume it's normal to have only one.

     What about aliasing/borrow checker issues? No, `&mut layout` is passed as a mutable reference to an `Option<Mut<LayoutStyle>>`. It compiles, so the borrow checker is happy.

     Wait! Look at `val.signal.with_untracked` in `update_text` and `update_computed_text` vs `val.signal.get_untracked()`.
     Wait, in `update_computed_text`:
     ```rust
             let changed = val
                 .computed
                 .with_untracked(|new_text| *new_text != val.last_value);

             if changed {
                 let new_val = val.computed.get(); // <--- Wait!!
                 val.last_value = new_val;
     ```
     `val.computed.get()` vs `val.computed.get_untracked()`!
     If it's a reactive system, and it uses `.get()`, `.get()` subscribes the current reactive context!
     But wait... `update_all_reactive_system` is just a normal ECS system, it does NOT run inside a reactive effect! If it calls `.get()` outside of an effect, it panics, or it silently fails, or registers a dependency on a non-existent effect context!
     Let's check `flux_state::Signal::get()` or `flux_state::Computed::get()`. Typically, `get()` tracks the dependency. If there's no active tracking context, it either panics or logs an error.
     Wait, `update_color`, `update_opacity`, `update_transform`, `update_layout_width`, `update_layout_flex_grow` all use `get_untracked()`.
     But `update_computed_text` uses `.get()`:
     ```rust
             if changed {
                 let new_val = val.computed.get();
                 val.last_value = new_val;
     ```
     This is a bug! It should use `get_untracked()`!
     Let me verify this.
So `val.computed.get()` calls `self.with(...)` which calls `self.runtime.track_and_get_computed_if_fresh(...)`.
Inside `track_and_get_computed_if_fresh`, it likely expects an active tracking context (e.g. an Effect) and might panic or erroneously track if none exists.
Wait, let's look at `flux_state/src/runtime.rs` or `flux_state/src/signal.rs` to see what `track_and_get_computed_if_fresh` does.

Let me also check for `.get_untracked()` on `Computed`.
Wait, there's `pub fn get_untracked(&self) -> T` on `Computed`!
```rust
impl<T: Clone + 'static + Send + Sync> Computed<T> {
    pub fn get_untracked(&self) -> T {
        self.with_untracked(|v| v.clone())
    }
}
```
If `get_untracked` exists, `update_computed_text` should definitely use it.

Another thing: What if `ProgressBarState` modifies layout width but the layout component doesn't expect it? No, wait.
Look at `update_progress_bar_width`:
```rust
            let total_width = val.total_width;
            val.last_progress = prog;
            if let Some(l) = layout.as_mut() {
                l.0.width = Some(prog * total_width);
            }
```
In `update_layout_width`:
```rust
                if let Some(l) = layout.as_mut() {
                    l.0.width = Some(new_width);
                }
```
They both modify `l.0.width` directly.
But wait, `update_all_reactive_system` calls:
```rust
        update_layout_width(&mut layout, &mut width);
        update_progress_bar_width(&mut layout, &mut progress_bar);
```
So `update_progress_bar_width` silently overwrites whatever `update_layout_width` did if BOTH components exist on the entity.
While typically an entity wouldn't have both, it's not a severe safety bug, just a minor conflict if misused.

Wait, the prompt says:
> What can break: The `update_computed_text` helper accidentally uses `.get()` instead of `.get_untracked()` to read the computed value.
Let's see if `.get()` causes a panic if not in an effect, or if it just ignores tracking.
Wait! In `track_and_get_computed_if_fresh`:
```rust
        if let Some(observer) = inner.current_context() {
            inner.dependencies.entry(observer).or_default().insert(id);
            inner.subscribers.entry(id).or_default().insert(observer);
        }
```
It looks like it ignores the tracking if `current_context()` returns `None`. So using `.get()` outside an effect isn't necessarily a panic, but wait... what if it panics later when it tries to recompute?
```rust
    pub fn with<R>(&self, f: impl FnOnce(&T) -> R) -> R {
        // Optimistically try to get fresh value with tracking in one go
        let handle = if let Some(h) = self.runtime.track_and_get_computed_if_fresh(self.id) {
            h
        } else {
            // Slow path: value is stale or uninitialized
            self.runtime.recompute(self.id);
            self.runtime.get_computed_handle(self.id)
        };
```
If it calls `self.runtime.recompute(self.id)`, does it panic?
Inside `recompute`:
```rust
    pub(crate) fn recompute(&self, id: NodeId) {
        let (compute_fn, _observer) = {
            let mut inner = self
                .inner
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            ...
```
No, `flux-state` seems relatively robust.
But the issue says:
> What can break:
Wait, `update_computed_text` uses:
```rust
            if changed {
                let new_val = val.computed.get();
                val.last_value = new_val;
```
If `val.computed.get()` is called *while* holding a mutable reference to `Scene` or while iterating over the ECS? No, it's just a method call.
But using `get()` instead of `get_untracked()` *inside an ECS system* might be wrong, because the ECS system isn't an effect, and if we accidentally run it *inside* a `flux-state` effect (e.g. if the system was triggered by something else), we'd accidentally track a dependency. Or maybe `get()` locks the runtime mutex, and if we are in a tight loop it causes contention? Wait, both `get` and `get_untracked` lock the inner runtime mutex.
Is `update_computed_text` using `get()` causing an infinite loop because it subscribes to itself? No, `update_computed_text` doesn't run inside an effect context, so `current_context()` is `None`. Wait, is `update_all_reactive_system` ever called from within a reactive effect? No, it's a Bevy system.

Let me think if there's any other "correctness/regression risk".
Look at `update_all_reactive_system`.
```rust
pub fn update_all_reactive_system(mut query: Query<ReactiveQuery<'_>>, mut scene: ResMut<Scene>) {
    for (
        node_ref,
        mut layout,
        mut color,
        mut text,
        mut computed_text,
        mut transform,
        mut opacity,
        mut width,
        mut flex_grow,
        mut progress_bar,
    ) in query.iter_mut()
    {
```
If an entity has multiple components, but the components are wrapped in `Option`, it iterates `query.iter_mut()`.
Wait! Does `mut query: Query<ReactiveQuery<'_>>` match *every single entity* that has at least *one* of those components?
```rust
pub type ReactiveQuery<'w> = (
    &'w SceneNodeRef,
    Option<&'w mut LayoutStyle>,
    Option<&'w mut ReactiveColor>,
    Option<&'w mut ReactiveText>,
    Option<&'w mut ReactiveComputedText>,
    Option<&'w mut ReactiveTransform>,
    Option<&'w mut ReactiveOpacity>,
    Option<&'w mut ReactiveLayoutWidth>,
    Option<&'w mut ReactiveLayoutFlexGrow>,
    Option<&'w mut ProgressBarState>,
);
```
Wait! In Bevy, if a `Query` has NO required components (or only `&SceneNodeRef` is required), it matches *every single entity* in the world that has a `SceneNodeRef`, regardless of whether it has *any* of the reactive components!
Let's check if there is an `Or<...>` filter?
```rust
pub fn update_all_reactive_system(mut query: Query<ReactiveQuery<'_>>, mut scene: ResMut<Scene>) {
```
There is NO filter!
`update_all_reactive_system` queries `Query<ReactiveQuery<'_>>`.
`ReactiveQuery<'w>` is just a tuple:
```rust
pub type ReactiveQuery<'w> = (
    &'w SceneNodeRef,
    Option<&'w mut LayoutStyle>,
    Option<&'w mut ReactiveColor>,
    Option<&'w mut ReactiveText>,
    Option<&'w mut ReactiveComputedText>,
    Option<&'w mut ReactiveTransform>,
    Option<&'w mut ReactiveOpacity>,
    Option<&'w mut ReactiveLayoutWidth>,
    Option<&'w mut ReactiveLayoutFlexGrow>,
    Option<&'w mut ProgressBarState>,
);
```
Since `SceneNodeRef` is the only required component, this query will match *EVERY SINGLE NODE* in the UI!
Wait, that means for every single node in the entire UI tree, we are doing:
```rust
        update_layout_width(&mut layout, &mut width);
        update_progress_bar_width(&mut layout, &mut progress_bar);
        update_layout_flex_grow(&mut layout, &mut flex_grow);
        update_color(&mut scene, node_ref.0, &mut color);
        update_text(&mut scene, node_ref.0, &mut text);
        update_computed_text(&mut scene, node_ref.0, &mut computed_text);
        update_transform(&mut scene, node_ref.0, &mut transform);
        update_opacity(&mut scene, node_ref.0, &mut opacity);
```
Even if `width`, `progress_bar`, `color`, etc. are all `None`!
But wait, if they are `None`, the helpers do `if let Some(val) = width.as_mut() { ... }` which is practically a no-op and just a quick branch.
Is this a performance issue? The review prompt says:
> Prioritize: correctness, data loss, concurrency, security, API behavior regressions, performance regressions.
This IS a massive performance regression! A UI with 10,000 nodes will run this monolithic query over *every single node* every frame, checking 8 optional components per node.

Wait, the previous implementation had separate systems:
```rust
pub fn update_reactive_colors_system(
    query: Query<(&SceneNodeRef, &ReactiveColor)>,
    mut scene: ResMut<Scene>,
)
```
This only iterated entities that ACTUALLY HAVE `ReactiveColor`!
The new monolithic system:
```rust
pub fn update_all_reactive_system(mut query: Query<ReactiveQuery<'_>>, mut scene: ResMut<Scene>)
```
Iterates `(SceneNodeRef, Option<...>, Option<...>, ...)` for EVERY entity with a `SceneNodeRef`!
If the UI has 5000 static nodes and 5 reactive nodes, this system checks all 5000 nodes instead of 5!
Wait! Is it required to have an `Or<(With<ReactiveColor>, With<ReactiveText>, ...)>` filter?
Yes!
```rust
type ReactiveFilter = Or<(
    With<ReactiveColor>,
    With<ReactiveText>,
    With<ReactiveComputedText>,
    With<ReactiveTransform>,
    With<ReactiveOpacity>,
    With<ReactiveLayoutWidth>,
    With<ReactiveLayoutFlexGrow>,
    With<ProgressBarState>,
)>;
```
This would restore the performance to only iterate over entities that actually have at least one reactive component!
This is a HUGE performance regression.

Let's double check Bevy docs for `Option` in Queries.
Yes, an `Option<T>` term in a Query means "get T if it exists, otherwise return None". It does not restrict the matched entities. The only non-optional term in `ReactiveQuery` is `&SceneNodeRef`. Thus, the query matches EVERY entity with a `SceneNodeRef`.

Also, look at `update_widget_style_system` in the same file:
```rust
pub type WidgetStyleFilter = Or<(Changed<InteractionState>, Changed<WidgetStyle>)>;

pub fn update_widget_style_system(
    mut query: Query<WidgetStyleQuery<'_>, WidgetStyleFilter>,
    mut scene: ResMut<Scene>,
)
```
This correctly uses a filter to only process entities that have changed!
In `update_all_reactive_system`, there is no filter! Wait, since it's polling `flux-state` signals, it can't use `Changed<T>` because the ECS component itself hasn't changed (the signal changed internally). However, it should STILL filter to only entities that actually POSSESS at least one of the reactive components. Otherwise, it polls every `SceneNodeRef` in the world.

Wait, are there any other bugs?
What if `val.computed.get()` panics?
```rust
        if changed {
            let new_val = val.computed.get();
            val.last_value = new_val;
```
If we use `val.computed.get_untracked()`, it is more correct, as we don't want to track anything here, but the performance regression of iterating the whole world is arguably much higher severity (critical performance regression). Both of these should be fixed!

Wait, `get()` vs `get_untracked()`: does `get()` track when not in a context?
In `flux-state`, `get` does:
```rust
    pub fn get(&self) -> T {
        self.with(|v| v.clone())
    }
```
And `with` calls `track_and_get_computed_if_fresh`. If `current_context()` is `None`, it skips tracking and then checks if stale. If it's stale, it calls `recompute()`. `get_untracked()` does the same but skips `track_and_get_computed_if_fresh` and directly calls `get_computed_if_fresh`. The difference is literally just whether it checks `current_context()`. Neither will panic.
However, using `get_untracked()` explicitly documents intent and prevents accidental tracking if the system is ever called from a tracked context.

Wait, the prompt says "Prioritize: correctness, data loss, concurrency, security, API behavior regressions, performance regressions."
A massive performance regression fits perfectly:
"Severity: high"
"What can break: Severe performance regression (O(N) iteration instead of O(M))"
"Why it breaks: `update_all_reactive_system` queries `ReactiveQuery<'_>` which only has `&SceneNodeRef` as a required component. This causes Bevy to iterate over *every single SceneNode in the entire UI* every frame, instead of only iterating the entities that actually have reactive components attached."

Minimal fix:
1. Create a `ReactiveFilter` type using `Or<...>` with `With<...>` for all reactive components.
2. Update the query to `Query<ReactiveQuery<'_>, ReactiveFilter>`.
3. Fix the `val.computed.get()` to `.get_untracked()` in `update_computed_text`.

Let me verify how many elements `Or` supports in Bevy.
Bevy `Or` is implemented for tuples up to 15 elements.
We have 8 reactive components.
```rust
pub type ReactiveFilter = Or<(
    With<ReactiveColor>,
    With<ReactiveText>,
    With<ReactiveComputedText>,
    With<ReactiveTransform>,
    With<ReactiveOpacity>,
    With<ReactiveLayoutWidth>,
    With<ReactiveLayoutFlexGrow>,
    With<ProgressBarState>,
)>;
```
This is perfectly valid in Bevy and will restore O(reactive) performance instead of O(all nodes).
Let me write this up as the plan.
