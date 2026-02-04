# Forge's Journal

**[WidgetContext is a God Object]**
**Learning:** `WidgetContext` in `widget-core` handles too many responsibilities (layout, input, painting, validation).
**Action:** Extract logic into the state structs it manages (e.g., `TextInputState`, `FormState`) to improve encapsulation.
