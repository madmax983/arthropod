# Chronos Architecture

**Chronos** provides time-travel (undo/redo) capabilities for Arthropod applications by wrapping `flux-state` primitives.

## Class Diagram

```mermaid
classDiagram
    class Timeline {
        -past: Vec~Transaction~
        -future: Vec~Transaction~
        -current_batch: Option~Transaction~
        +new() Arc~Mutex~Timeline~~
        +undo()
        +redo()
        +begin_transaction()
        +commit_transaction()
        -record(op: Operation)
    }

    class Transaction {
        -ops: Vec~Operation~
    }

    class Operation {
        -undo: Box~Fn~
        -redo: Box~Fn~
    }

    class RetroSignal~T~ {
        -read: ReadSignal~T~
        -write: WriteSignal~T~
        -timeline: Arc~Mutex~Timeline~~
        +new(rt, timeline, val) RetroSignal
        +set(val: T)
        +update(f: Fn)
        +get() T
    }

    class Signal~T~ {
        <<flux-state>>
    }

    Timeline *-- Transaction
    Transaction *-- Operation
    RetroSignal --> Timeline : Records to
    RetroSignal --> Signal : Wraps
```

## Sequence Diagrams

### Recording a Change

When a `RetroSignal` is updated, it captures the current state and the new state into an `Operation` and pushes it to the `Timeline`.

```mermaid
sequenceDiagram
    participant User
    participant RetroSignal
    participant Timeline
    participant WriteSignal

    User->>RetroSignal: set(new_value)
    activate RetroSignal

    RetroSignal->>RetroSignal: Capture old_value
    RetroSignal->>RetroSignal: Create undo/redo closures

    RetroSignal->>Timeline: record(Operation)
    activate Timeline
    Timeline->>Timeline: Clear future
    Timeline->>Timeline: Push to past (or batch)
    deactivate Timeline

    RetroSignal->>WriteSignal: set(new_value)
    activate WriteSignal
    WriteSignal-->>RetroSignal: (triggers reactivity)
    deactivate WriteSignal

    deactivate RetroSignal
```

### Undoing a Change

Undoing pops the last transaction from the history and executes its `undo` closure, which writes the old value back to the underlying signal.

```mermaid
sequenceDiagram
    participant User
    participant Timeline
    participant Closure
    participant WriteSignal

    User->>Timeline: undo()
    activate Timeline

    Timeline->>Timeline: Pop Transaction from past
    Timeline->>Timeline: Push Transaction to future

    loop For each Operation (reverse)
        Timeline->>Closure: (op.undo)()
        activate Closure
        Closure->>WriteSignal: set(old_value)
        activate WriteSignal
        WriteSignal-->>Closure: (triggers reactivity)
        deactivate WriteSignal
        deactivate Closure
    end

    deactivate Timeline
```
