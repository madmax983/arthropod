# 🗣️ Echo: DX Audit for Arthropod & Nova Story Engine

## Topic 1: The Linux Quick Start Dead End

🤦 **The Confusion:**
I copied the `Quick Start` code block from `README.md` into my Linux machine. It told me "App::run requires Windows or macOS. Use App::new_headless() on Linux." So I switched to `App::new_headless()`. But there is absolutely no example showing how to build widgets (using the macros like `col!` and `txt!`) when using `App::new_headless()`. The macros emit obscure compiler errors without a `WidgetContext`.

🕵️ **The Reality:**
`App::run()` automatically creates an `AppContext` and builds the widget tree, which works great on Windows/Mac. But `App::new_headless()` doesn't provide the same closure or `WidgetContext` directly. As a Linux user, I am completely stranded. I can't even get "Hello World" to show up without deep-diving into ECS and `WidgetContext` setup.

💡 **The Fix:**
Provide a copy-pasteable example in the `README.md` showing exactly how to use `App::new_headless()` to build and interact with widgets, or implement a headless macro/runner that works exactly like `App::run()`. Don't just say "Use App::new_headless() on Linux" and leave me to figure out the rest.


## Topic 2: Import Overload for Nova Story

🤦 **The Confusion:**
To get the Nova Story Engine working based on the `docs/experimental/nova-story.md` example, I had to import 5 different things just for one basic feature: `NarrativeGenerator`, `StoryRuntime`, `register_story`, `Story`, and `Passage`. That's way too many imports just to say "Here's my story".

🕵️ **The Reality:**
The experimental module structure requires users to pull in internal engine mechanics (`NarrativeGenerator`, `StoryRuntime`, `register_story`) alongside the actual story content models (`Story`, `Passage`).

💡 **The Fix:**
Consolidate the API. I should just be able to write `app.add_story(Story::new(...))` without having to manually register systems, spawn a `NarrativeGenerator` entity, and insert a `StoryRuntime` resource. Give me one macro or a simple facade.


## Topic 3: Jargon-Heavy Nova Story Docs

🤦 **The Confusion:**
The Nova Story Engine docs hit me with: "This is a fully reactive, ECS-driven narrative runtime." What does that mean? Also, step 5 of the tutorial is "Spawn the NarrativeGenerator (renders the current state to the Scene)". I'm writing a story, why am I manually spawning an entity on the Scene root?

🕵️ **The Reality:**
The tutorial exposes the underlying architecture (ECS, Resources, Entities, Scene Graph) directly to the user. Users don't care about the Entity-Component-System when they just want to write an interactive narrative.

💡 **The Fix:**
Hide the jargon. The example should just be "Define your story -> Run it." The internal state management and ECS components (`NarrativeGenerator`) should be abstracted away from the public user API.


## Topic 4: Confusing Unresolved Import Error

🤦 **The Confusion:**
When trying to use `arthropod::experimental::story::*` in my fresh `main.rs`, the compiler threw `error[E0432]: unresolved import`. I thought I spelled it wrong or the crate was broken.

🕵️ **The Reality:**
The `story` module is gated behind `#[cfg(feature = "nova")]`. If I don't include `features = ["nova"]` in `Cargo.toml`, it acts like it doesn't exist. There's a tiny note in the README about it, but the error itself is unhelpful.

💡 **The Fix:**
Add a `compile_error!` or a more prominent `#[deprecated]` style warning that triggers if someone tries to use the Nova APIs without the feature flag, or at least ensure the `README` warning is the very first thing they see when they encounter `E0432`.
