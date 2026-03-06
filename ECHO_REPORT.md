# 🗣️ Echo: Getting Started example is broken

## Description

* 🤦 **The Confusion:** Tried to run the `story_demo` without explicitly adding `features = ["nova"]` to my `Cargo.toml`. The compiler did not say `NarrativeGenerator` was not found. Instead, it allowed me to import the "stub" `NarrativeGenerator`, generated some deprecation warnings which I ignored, and then crashed with a runtime panic when I actually tried to use `Story::new("start")`.
* 🕵️ **The Reality:** Turns out I needed to enable feature `nova`. The framework tries to be "helpful" by providing experimental stubs that emit a beautiful CLI warning block at runtime instead of a raw compile error. But because `Story::new` panics immediately, the runtime warning is mixed with a panic stack trace, completely breaking the intended DX.
* 💡 **The Fix:** Add a huge banner in README saying 'REQUIRES FEATURE NOVA'. Even better: remove the stubs from `experimental/mod.rs` so the compiler fails with a standard 'struct not found' or 'unresolved import' error! A compile-time error is instant and impossible to ignore, unlike deprecation warnings and runtime logs.
