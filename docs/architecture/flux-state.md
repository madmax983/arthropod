# Flux-State Architecture

This document details the internal architecture of `flux-state`, the reactive state management library for Arthropod.

## Core Architecture

`flux-state` separates the public API (`Runtime`) from the internal implementation (`RuntimeInner`) to ensure thread safety and correct synchronization.

### Class Diagram

```mermaid
classDiagram
    class Runtime {
        -inner: Arc~Mutex~RuntimeInner~~
        +new() Arc~Runtime~
        +create_signal(value: Any) NodeId
        +create_computed(fn: Fn) NodeId
        +create_effect(fn: Fn) NodeId
        +track(source: NodeId)
        +notify(source: NodeId)
        +run_effect(id: NodeId)
    }

    class RuntimeInner {
        -signals: HashMap~NodeId, Arc~Any~~
        -computeds: HashMap~NodeId, ComputedNode~
        -effects: HashMap~NodeId, Arc~Fn~~
        -dependencies: HashMap~NodeId, HashSet~NodeId~~
        -subscribers: HashMap~NodeId, HashSet~NodeId~~
        -tracking_context: HashMap~ThreadId, Vec~NodeId~~
        -stale: HashSet~NodeId~
        -pending_effects: Vec~NodeId~
        +cleanup_dependencies(id: NodeId)
        +mark_stale(id: NodeId) bool
        +mark_subscribers_stale(source: NodeId)
        +push_context(id: NodeId)
        +pop_context()
    }

    class Signal~T~ {
        -runtime: Arc~Runtime~
        -id: NodeId
        +get() T
        +set(value: T)
        +update(f: Fn(T) -> T)
    }

    class Computed~T~ {
        -runtime: Arc~Runtime~
        -id: NodeId
        +get() T
    }

    class Effect {
        -runtime: Arc~Runtime~
        -id: NodeId
    }

    Runtime *-- RuntimeInner : Wraps with Mutex
    Signal --> Runtime : Uses
    Computed --> Runtime : Uses
    Effect --> Runtime : Uses
```

## Reactive Update Flow

When a `Signal` is updated, it triggers a chain of notifications through the dependency graph.

### Sequence Diagram: Signal Update

```mermaid
sequenceDiagram
    participant User Code
    participant Signal
    participant Runtime
    participant RuntimeInner
    participant Effect

    User Code->>Signal: set(new_value)
    activate Signal

    Signal->>Runtime: notify(signal_id)
    activate Runtime

    Runtime->>Runtime: lock()
    Runtime->>RuntimeInner: mark_subscribers_stale(signal_id)
    activate RuntimeInner
    RuntimeInner->>RuntimeInner: Mark dependent Effects as stale
    RuntimeInner->>RuntimeInner: Add to pending_effects queue
    deactivate RuntimeInner
    Runtime->>Runtime: unlock()

    Runtime->>Runtime: flush_effects()

    loop While pending_effects is not empty
        Runtime->>Runtime: lock()
        Runtime->>RuntimeInner: Pop effect_id
        Runtime->>RuntimeInner: cleanup_dependencies(effect_id)
        Runtime->>RuntimeInner: push_context(effect_id)
        Runtime->>Runtime: unlock()

        Runtime->>Effect: execute()
        activate Effect
        Effect->>Signal: get()
        Signal->>Runtime: track(signal_id)
        Runtime->>RuntimeInner: Add dependency (Effect -> Signal)
        Effect-->>Runtime: done
        deactivate Effect

        Runtime->>Runtime: lock()
        Runtime->>RuntimeInner: pop_context()
        Runtime->>Runtime: unlock()
    end

    deactivate Runtime
    Signal-->>User Code: done
    deactivate Signal
```

## Thread Safety

-   **Runtime**: Implements `Send + Sync` via `Arc<Mutex<RuntimeInner>>`.
-   **Signals**: Store values as `Arc<dyn Any + Send + Sync>`, ensuring thread-safe access.
-   **Locking Strategy**: Coarse-grained locking around graph operations. Locks are released before executing user callbacks (Effects/Computeds) to prevent deadlocks and allow re-entry into the runtime (e.g., reading other signals).
