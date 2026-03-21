# 🗣️ Echo: Widget macros emit confusing "no rules expected" errors instead of catching typos

## The "Experience"

### 1. The Walkthrough
**Scenario:** "I am a new user trying to use the `col!`, `row!`, and `form!` macros, but I made a typo in the property name."

**Action:** I used `col!` and accidentally typed `typo_gap: 10.0` instead of `gap: 10.0`.

```rust
use arthropod::prelude::*;

fn main() {
    let _ = col!([
        txt!("Hello")
    ], typo_gap: 10.0);
}
```

### 2. The Friction Points
*   **Confusing Macro Errors:** The compiler spits out `error: no rules expected 'typo_gap'`.
*   This error is classic Rust macro opacity. It just says "no rules expected this token", which doesn't explain *why* it's wrong or what the valid parameters are for `col!`.
*   Conversely, if I do `txt!("Hello", wrong_arg: 24.0)`, I get `Unknown property or flag: wrong_arg: 24.0`. This is vastly better because I know exactly what I did wrong!

### 3. The Report
**Title:** "🗣️ Echo: Layout macros (col!, row!, form!) throw confusing macro parsing errors on typos"

**Description:**
* 🤦 **The Confusion:** "I made a typo `typo_gap` in `col!` instead of `gap`. The compiler yelled `no rules expected 'typo_gap'`. I had to guess if the property doesn't exist, if I missed a comma, or if I messed up the tuple syntax."
* 🕵️ **The Reality:** "The `txt!` macro correctly catches unknown flags and emits a clean `compile_error!`. However, container macros like `col!`, `row!`, and `form!` lack a robust catch-all arm for unknown tokens (`$($unknown:tt)*`), causing the macro parser to abruptly fail with a generic error."
* 💡 **The Fix:** "Add a catch-all match arm using `$($unknown:tt)*` to `__col_apply!`, `__row_apply!`, and `__form_apply!` in the arthropod framework. This should emit an explicit `compile_error!` with a clear message (e.g., 'Unknown property or flag') to catch all invalid syntax and typos, preventing confusing 'no rules expected' compiler errors."
