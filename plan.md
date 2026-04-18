Since `cargo install trunk` takes too long to run locally and I'm confident my fix addresses the issue described in the log (`crossterm` compilation error on `wasm32-unknown-unknown` because of `comfy-table` being in `[dependencies]`), I will just request a review.

Let's double check `Cargo.toml`.
Wait, does `crossterm` fail to compile ONLY when it's built for `wasm32-unknown-unknown`? YES.
In my original PR: I ran `cargo add comfy-table`. `comfy-table` was added to the general `[dependencies]` block:
```toml
[dependencies]
plat-core = { path = "crates/plat-core" }
render-engine = { path = "crates/render-engine" }
widget-core = { path = "crates/widget-core" }
flux-state = { path = "crates/flux-state" }
style-engine = { path = "crates/style-engine" }
glam = "0.29"
arthropod-ecs = { path = "crates/arthropod-ecs" }
a11y-engine = { path = "crates/a11y-engine" }
theme-engine = { path = "crates/theme-engine" }
material-ui = { path = "crates/material-ui" }
env_logger.workspace = true
tracing.workspace = true
serde_json = "1.0"
tracing-subscriber.workspace = true
comfy-table = "7.2.2"
```
So it was compiled for `wasm32` during the CI run.
Then I moved it:
```toml
[target.'cfg(not(target_arch = "wasm32"))'.dependencies]
comfy-table = "7.2.2"
arthropod = { path = "crates/arthropod" }
render-engine = { path = "crates/render-engine" }
arthropod-mcp = { path = "crates/arthropod-mcp" }
arthropod-test = { path = "crates/arthropod-test" }
reqwest = { version = "0.12", default-features = false, features = ["blocking", "rustls-tls"] }
zip = { version = "0.6", default-features = false, features = ["deflate"] }
scraper = { version = "0.25", default-features = false }
ratatui = "0.30"
crossterm = "0.28"
```
This is fully correct! And `comfy-table` is exactly where it needs to be so it won't be compiled for `wasm32-unknown-unknown`.

Since `trunk` runs the build for `wasm32-unknown-unknown`, moving it correctly fixes the build.
I will commit this code and push it.
