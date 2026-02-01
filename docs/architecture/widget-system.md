# Widget System Architecture

This document details the architecture of the `widget-core` system, specifically the `WidgetContext` which acts as the central coordinator for building widget hierarchies.

## Widget Context

The `WidgetContext` employs a "Flattened Context" architecture (see [ADR 0014](../adr/0014-flattened-widget-context.md)) to simplify widget implementation while maintaining modularity under the hood.

### Class Diagram

```mermaid
classDiagram
    class WidgetContext {
        +Scene scene
        -LayoutContext layout_context
        -InteractionContext interaction_context
        -DecorationContext decoration_context
        -InputContext input_context
        -FormContext form_context
        +Option~DesignTokens~ design_tokens
        +create_node(parent: NodeId, content: NodeContent) NodeId
        +set_layout_style(node: NodeId, style: FlexStyle)
        +add_clickable(node: NodeId, callback: Fn)
        +set_background_color(node: NodeId, color: Vec4)
        +add_text_input_state(node: NodeId, ...)
        +add_validator(node: NodeId, validator: Validator)
        +into_scene() Scene
        +layout_styles() HashMap
        +clickables() HashMap
        +background_colors() HashMap
        +text_input_states() IndexMap
        +validators() HashMap
    }

    class LayoutContext {
        -HashMap~NodeId, FlexStyle~ styles
        +set_style(node: NodeId, style: FlexStyle)
        +get_all_styles() HashMap
    }

    class InteractionContext {
        -HashMap~NodeId, Callback~ clickables
        -HashSet~NodeId~ hover_states
        +add_clickable(node: NodeId, callback: Fn)
        +add_hover_state(node: NodeId)
        +get_clickables() HashMap
    }

    class DecorationContext {
        -HashMap~NodeId, Vec4~ backgrounds
        +set_background_color(node: NodeId, color: Vec4)
        +get_background_colors() HashMap
    }

    class InputContext {
        -IndexMap~NodeId, TextInputState~ text_input_states
        -HashMap~NodeId, ReactiveTextState~ reactive_text_states
        +add_text_input_state(...)
        +add_reactive_text_state(...)
    }

    class FormContext {
        -HashMap~NodeId, FormState~ form_states
        -HashMap~NodeId, ValidationState~ validators
        +add_form_state(...)
        +set_validator(...)
    }

    class Scene {
        -NodeTree nodes
        +add_node(parent: NodeId, content: NodeContent) NodeId
        +reparent_node(child: NodeId, old_parent: NodeId, new_parent: NodeId)
    }

    WidgetContext *-- LayoutContext
    WidgetContext *-- InteractionContext
    WidgetContext *-- DecorationContext
    WidgetContext *-- InputContext
    WidgetContext *-- FormContext
    WidgetContext *-- Scene : Builds
```

## Widget Build Flow

The process of building a widget involves passing a mutable `WidgetContext` down the tree.

### Sequence Diagram

```mermaid
sequenceDiagram
    participant App
    participant RootWidget
    participant ChildWidget
    participant WidgetContext
    participant Scene
    participant SubContexts

    App->>WidgetContext: new()
    App->>RootWidget: build(&mut ctx)

    activate RootWidget
    RootWidget->>WidgetContext: create_node(root, content)
    WidgetContext->>Scene: add_node(root, content)
    Scene-->>WidgetContext: NodeId(1)
    WidgetContext-->>RootWidget: NodeId(1)

    RootWidget->>WidgetContext: set_layout_style(NodeId(1), style)
    WidgetContext->>SubContexts: layout_context.set_style(NodeId(1), style)

    RootWidget->>ChildWidget: build(&mut ctx, parent=NodeId(1))
    activate ChildWidget
    ChildWidget->>WidgetContext: create_node(NodeId(1), content)
    WidgetContext->>Scene: add_node(NodeId(1), content)
    Scene-->>WidgetContext: NodeId(2)
    WidgetContext-->>ChildWidget: NodeId(2)

    ChildWidget->>WidgetContext: add_clickable(NodeId(2), callback)
    WidgetContext->>SubContexts: interaction_context.add_clickable(NodeId(2), callback)

    ChildWidget-->>RootWidget: NodeId(2)
    deactivate ChildWidget

    RootWidget-->>App: NodeId(1)
    deactivate RootWidget

    App->>WidgetContext: into_scene()
    WidgetContext-->>App: Scene (populated)
    App->>WidgetContext: layout_styles()
    WidgetContext-->>App: Styles HashMap
    Note over App: App transfers state to ECS
```
