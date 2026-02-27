# Time Travel (Ghost Replay)

> **Status:** Experimental (Alpha)
> **Feature Flag:** `nova`

The Time Travel system (internally known as `ghost_replay`) allows you to record and replay user input events. This is useful for debugging, automated testing, and creating "ghost" demos.

## Getting Started

To use Time Travel, you must enable the `nova` feature in your `Cargo.toml`:

```toml
[dependencies]
arthropod = { version = "0.1", features = ["nova"] }
```

## Core Concepts

### `GhostRecorder`

A resource that stores a sequence of `GhostEvent`s.

```rust
pub struct GhostRecorder {
    pub events: Vec<GhostEvent>,
    pub recording: bool,
    // ...
}
```

### `GhostReplayer`

A resource that plays back recorded events by simulating a visual cursor on the Scene.

```rust
pub struct GhostReplayer {
    pub playing: bool,
    pub speed: f32, // Playback speed multiplier (default: 1.0)
    // ...
}
```

## Integration

Unlike other systems, Ghost Replay requires manual integration into your event loop to capture events.

### 1. Initialize the System

Call `init_ghost_replay` during app setup to register the necessary resources and update systems.

```rust
use arthropod::experimental::ghost_replay::init_ghost_replay;

// In your App::new or setup function:
init_ghost_replay(&mut app);
```

### 2. Record Events

You must manually call `record_event` in your `Application::on_event` handler. This allows the recorder to intercept raw window events.

```rust
use arthropod::experimental::ghost_replay::record_event;

fn on_event(&mut self, event: Event, control_flow: &mut ControlFlow) {
    // Pass the event to the recorder
    // (Requires mutable access to the ECS World)
    record_event(self.app.world_mut(), &event);

    // ... handle other events
}
```

## Usage Example

Here is how to control the recorder and replayer from your code (e.g., via keyboard shortcuts).

```rust
// Start Recording
if let Some(mut recorder) = app.world_mut().get_resource_mut::<GhostRecorder>() {
    recorder.start();
}

// Stop Recording
if let Some(mut recorder) = app.world_mut().get_resource_mut::<GhostRecorder>() {
    recorder.stop();
}

// Playback
if let Some(mut replayer) = app.world_mut().get_resource_mut::<GhostReplayer>() {
    // Get recorded events (you might want to clone them or move them)
    let events = app.world().resource::<GhostRecorder>().events.clone();
    replayer.play(events);
}
```

## Visuals

When replaying, the system automatically spawns a "Ghost Cursor" (a semi-transparent purple square) in the Scene graph to visualize the recorded mouse movements and clicks.
