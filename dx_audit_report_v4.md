# 🗣️ Echo: Widget macros swallow typo errors silently

## The "Experience"

### 1. The Walkthrough
**Scenario:** "I am a new user trying to use the `col!`, `txt!` and `btn!` macros."

**Action:** I copied the `README.md` Quick Start into a new project and started tweaking parameters to see how it works. I intentionally made some mistakes to see what the compiler would tell me.

### 2. The Friction Points
*   **Silent Typos:** If I write `txt!("Hello", wrong_arg: 24.0)` instead of `size: 24.0`, or `btn!("Click Me", foobar, on_click: ...)` instead of `primary`, the compiler **does not complain**. `cargo check` says everything is fine!
*   **Missing required fields:** If I write `btn!(on_click: || println!("hi"))` without the label text, the compiler doesn't complain.

### 3. The Report
**Title:** "🗣️ Echo: Widget macros silently swallow typos instead of throwing compile errors"

**Description:**
* 🤦 **The Confusion:** "I made a typo in a macro property, like `wrong_arg: 24.0` in `txt!` instead of `size`. The code compiled fine but nothing happened. I wasted time figuring out why my text didn't change size."
* 🕵️ **The Reality:** "The declarative macros (`txt!`, `btn!`, `col!`) appear to use catch-all patterns that silently ignore unhandled or unrecognized tokens. They don't enforce strict property names."
* 💡 **The Fix:** "Make the macros stricter! If I pass an invalid token or property to `txt!` or `btn!`, the compiler should throw an error, not ignore it."
