# Nova Story Engine

> **Status:** Experimental
> **Feature Flag:** `nova`

The Nova Story Engine is a reactive narrative runtime built on top of `flux-state` and ECS. It allows you to create interactive, choice-driven stories using a graph-based structure.

## Getting Started

To use the Story Engine, you must enable the `nova` feature in your `Cargo.toml`:

```toml
[dependencies]
arthropod = { version = "0.1", features = ["nova"] }
```

## Basic Usage

Here is a minimal example of how to define a story and run it.

```rust
use arthropod::prelude::*;
use arthropod::experimental::story::{
    NarrativeGenerator, StoryRuntime, register_story,
    model::{Story, Passage}
};

fn main() -> Result<(), AppError> {
    // 1. Initialize the App (Headless for this example)
    let mut app = App::new_headless()?;

    // 2. Register Story Systems
    register_story(&mut app);

    // 3. Define your Story
    let mut story = Story::new("start");

    story.add_passage(
        Passage::new("start", "You are standing in a dark room.")
            .add_choice("Turn on the light", "light_on")
            .add_choice("Wait in the dark", "wait")
    );

    story.add_passage(
        Passage::new("light_on", "The light flickers on. You see a door.")
            .add_choice("Open the door", "end")
    );

    story.add_passage(
        Passage::new("wait", "You wait. Nothing happens.")
            .add_choice("Turn on the light", "light_on")
    );

    story.add_passage(
        Passage::new("end", "You leave the room. The End.")
            .add_choice("Restart", "start")
    );

    // 4. Initialize StoryRuntime with your story
    app.world_mut().insert_resource(StoryRuntime::new(story));

    // 5. Spawn the NarrativeGenerator (renders the current state to the Scene)
    let root = app.world().resource::<render_engine::Scene>().root();
    app.spawn(root).insert(NarrativeGenerator);

    // 6. Run the App (Simulation loop)
    // In a real app, this would be your game loop or UI event loop.
    println!("Story Initialized!");
    app.update();

    // Access the runtime to print current text
    let runtime = app.world().resource::<StoryRuntime>();
    println!("Current Text: {}", runtime.current_text());

    Ok(())
}
```

## Key Concepts

### Story & Passages

A `Story` is a collection of `Passage` nodes. Each passage has:
- A unique ID (`String`).
- Text content (`String`).
- A list of `Choice`s leading to other passages.

### StoryRuntime

The `StoryRuntime` is a resource that holds the `Story` data and the current state (current passage ID, history). It provides methods to navigate:
- `current_text()`: Returns the text of the active passage.
- `current_choices()`: Returns available choices.
- `choose(index)`: Transitions to the target passage of the selected choice.

### NarrativeGenerator

`NarrativeGenerator` is a component that, when added to an entity, automatically updates the Scene Graph with the current story text and choices. It reacts to changes in `StoryRuntime`.

## Advanced Features

### Markdown Support

The text in passages supports basic Markdown formatting (e.g., `**bold**`, `# Heading`). The `NarrativeGenerator` parses this and applies styling.

### Custom Rendering

If you want to render the story differently (e.g., custom UI layout), you can query `StoryRuntime` directly in your own systems instead of using `NarrativeGenerator`.

```rust
fn my_story_ui_system(runtime: Res<StoryRuntime>) {
    if runtime.is_changed() {
        println!("New Passage: {}", runtime.current_text());
    }
}
```
