# 🗣️ Echo: Getting Started example is broken

## The "Experience"

### 1. The "README Run"
**Scenario:** "I am a new user trying to run the basic Quick Start example from the README."

**Action:** I copied the "Quick Start" code from `README.md` into my own project and tried to run it using `cargo run`.

**Result:**
- **Compilation:** ✅ Success! It compiled perfectly.
- **Runtime:** ❌ Failed with `App::run requires Windows or macOS. Use App::new_headless() on Linux.`
- **Observation:** As a Linux user, I was immediately greeted with an error. The error message is clear, but the fact that a cross-platform framework's quick start fails out-of-the-box on Linux is a rough first impression.

### 2. The "Error Check"
**Scenario:** "I intentionally misconfigured a layout macro to see how the system responds."

**Action:** I tried passing a named argument `foo: 10.0` into the `txt!` macro.

**Result:**
- **Compilation:** ❌ Failed with `error: no rules expected 'foo'`.
- **Observation:** This error message is classic Rust macro opacity. It just says "no rules expected this token", which doesn't explain *why* it's wrong or what the valid parameters are for `txt!`. It requires the user to dive into the documentation for `txt!` rather than intuitively telling them that `size`, `color`, or standard text attributes are expected.

### 3. The "Import Scan" & Feature Flags
**Scenario:** "I am a new user trying to add `Nova`'s story feature, as advertised."

**Action:** I copied the `NarrativeGenerator` and `register_story` usage into my app, but forgot to enable the `nova` feature flag in my `Cargo.toml`.

**Result:**
- **Compilation:** ✅ Success! But it emitted a bunch of `#[deprecated]` warnings stating: `This feature is EXPERIMENTAL and requires the 'nova' feature`.
- **Runtime:** ❌ Silent failure logic-wise, with a generic log printed: `ERROR: 'register_story' - This feature is EXPERIMENTAL and requires the 'nova' feature. Enable it in Cargo.toml.`
- **Observation:** Why did my code compile if the feature isn't enabled? I completely ignored the warnings because developers ignore warnings during prototyping. When I ran my app, it didn't do anything, and the error was buried in the console logs. This is terrible DX. If a feature isn't enabled, the code simply shouldn't compile (e.g., "struct not found" or "unresolved import").

## The Report

### 🗣️ Echo: "Missing features should cause compile errors, not runtime logs!"

* 🤦 **The Confusion:** "Tried to run code using `NarrativeGenerator` but forgot the `nova` feature. The compiler let me do it anyway (just gave warnings), but then my app didn't work and silently failed at runtime."
* 🕵️ **The Reality:** "The `experimental` module provides empty 'stubs' for missing features. This is intended to be helpful, but it actually hides the fact that the feature is missing at compile time."
* 💡 **The Fix:** "Remove the stubs from `experimental/mod.rs`! Let the compiler fail with 'struct not found' or 'unresolved import' (standard Rust behavior). A hard compile error is instant feedback; a runtime log is easily missed."
