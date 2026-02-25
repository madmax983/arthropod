# 31. Ghost Replay System

Date: 2024-05-20

## Status

Accepted

## Context

Debugging complex user interaction flows—such as drag-and-drop operations, hover states, or focus transitions—is difficult. Bugs often depend on the precise timing, velocity, and path of the mouse cursor.

Traditional reproduction steps ("Move mouse quickly to the right") are subjective and imprecise. Furthermore, automated integration tests (like those using `arthropod-test`) run headlessly, making it impossible to visually inspect *why* a test failed during a specific interaction sequence.

We need a mechanism to:
1.  **Record** a session of user inputs (mouse movements, clicks) with precise timestamps.
2.  **Replay** that session visually within the engine to observe the interaction.
3.  **Visualize** the cursor path to understand user behavior during the session.

## Decision

We will implement a **Ghost Replay System** within the `arthropod::experimental` module, guarded by the `nova` feature.

The system consists of three main components:

1.  **GhostRecorder**: A Resource that listens to the main event loop. It captures `CursorMoved`, `Click`, and `Release` events, storing them with a relative timestamp from the start of the recording.
2.  **GhostReplayer**: A System that iterates through a recorded session. It manages the playback state (playing, stopped, speed) and updates the scene.
3.  **Visual Feedback**: Instead of injecting synthetic events into the OS event loop (which is complex and platform-dependent), the replayer instantiates a **Ghost Cursor**—a semi-transparent visual node in the Scene Graph.
    *   **Purple Circle**: Represents the cursor position.
    *   **Red Flash**: Represents a Click event.

This approach decouples the *visualization* of the input from the *execution* of the input, making it safe to run in overlay mode without interfering with the actual system cursor.

## Consequences

### Positive
*   **Visual Debugging**: Developers can "watch" a bug reproduction or a user session to identify ergonomic issues or race conditions.
*   **Deterministic Analysis**: The replay follows the exact timestamped sequence, allowing for frame-by-frame analysis of interaction logic.
*   **Performance**: The recording format is extremely lightweight (simple structs), allowing for long sessions without memory pressure.

### Negative
*   **Visualization Only**: The current implementation primarily visualizes the input. It does not automatically re-inject events into the application logic to "drive" the application state (though it could be extended to do so). Its primary purpose is inspection, not automation.
*   **Drift**: If the application runs at a significantly different frame rate during replay than recording, the visual interpolation of the ghost cursor might look slightly different (though timestamps are respected).
*   **Scope**: Currently limited to mouse interactions. Keyboard and touch gestures are not yet supported.
