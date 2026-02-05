# Bard's Journal 🎻

## 2024-05-22 - [App Architecture Clarity]
**Confusion:** The role of `App::integrate_widgets` vs `WidgetApp` manual syncing was unclear. It seemed like `integrate_widgets` was incomplete due to TODOs, but it actually only handles ECS component transfer, while `WidgetApp` handles event-driven state (text input, focus).
**Clarification:** Documented `integrate_widgets` as the "ECS Bridge" for static/reactive components, distinguishing it from the event loop controller logic.

## 2024-05-22 - [Signal Ownership]
**Confusion:** Users might think `Signal::new` returns a handle they can drop, but `Signal` is an `Arc` wrapper.
**Clarification:** `flux-state` docs already cover this, but added explicit note about `Signal` being cheap to clone and share.
