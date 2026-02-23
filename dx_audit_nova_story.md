# 🗣️ Echo Report: Nova Feature Confusion

## The "Experience"

### 1. The Walkthrough
**Scenario:** "I am a new user trying to add `Nova`'s story feature."

**Action:** I copied the `NarrativeGenerator` usage from `examples/story_demo.rs` into my own project file `examples/repro_story_usage.rs`.

```rust
use arthropod::prelude::*;
use arthropod::experimental::story::{NarrativeGenerator, register_story};

fn main() -> Result<(), AppError> {
    println!("I want to generate a story!");
    let mut app = App::new_headless()?;
    register_story(&mut app);

    app.spawn(app.world().resource::<render_engine::Scene>().root())
        .insert(NarrativeGenerator)
        .id();

    app.update();

    println!("App updated. Did anything happen?");
    Ok(())
}
```

**Result:**
- **Compilation:** ✅ Success! It compiled with some warnings (deprecation).
- **Runtime:** ❌ Silent Failure (with log).
  - The app printed "I want to generate a story!" and "App updated. Did anything happen?".
  - It also printed an error to stderr: `ERROR: 'register_story' requires 'nova' feature. Enable it in Cargo.toml.`
  - **BUT:** The app continued running! I didn't see the error immediately because it was mixed with other output.

### 2. The Friction Points
- **Silent Failure:** The app continued running even though a core feature (story generation) failed. This is dangerous.
- **Warnings Ignored:** As a developer, I often ignore warnings during rapid prototyping. Deprecation warnings are usually "fix later", not "your app is broken now".
- **Confusing Stubs:** The `experimental` module provides "stubs" for missing features. This is intended to be helpful, but it actually hides the fact that the feature is missing at compile time.

## The Verdict

### 3. The Report
**Title:** "🗣️ Echo: 'Nova' Feature Stubs cause confusion"

**Description:**
* 🤦 **The Confusion:** "I tried to use `NarrativeGenerator` without the `nova` feature. It compiled with some warnings I ignored. When I ran it, nothing happened. I wasted time debugging."
* 🕵️ **The Reality:** "The `experimental` module uses 'stubs' marked as `#[deprecated]` when the feature is disabled. This allows compilation but fails silently (logs to stderr) at runtime."
* 💡 **The Fix:** "Remove the stubs! Let the compiler fail with 'struct not found' (standard Rust behavior) or use `compile_error!` if possible. A compile error is much better than a runtime log."

### 4. Verification
The "helpful" stubs are actually harmful DX because they defer errors to runtime. A compilation error `struct not found` is instant feedback and impossible to ignore.
