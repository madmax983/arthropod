# Bard's Journal 🎻

## 2024-05-22 - [App Architecture Clarity]
**Confusion:** The role of `App::integrate_widgets` vs `WidgetApp` manual syncing was unclear. It seemed like `integrate_widgets` was incomplete due to TODOs, but it actually only handles ECS component transfer, while `WidgetApp` handles event-driven state (text input, focus).
**Clarification:** Documented `integrate_widgets` as the "ECS Bridge" for static/reactive components, distinguishing it from the event loop controller logic.

## 2024-05-22 - [Signal Ownership]
**Confusion:** Users might think `Signal::new` returns a handle they can drop, but `Signal` is an `Arc` wrapper.
**Clarification:** `flux-state` docs already cover this, but added explicit note about `Signal` being cheap to clone and share.

## 2024-05-23 - [Macro Syntax Magic]
**Confusion:** The `txt!(@read)` syntax was confusing because it looked like invalid Rust code and the example didn't explain that `@` is syntactic sugar handled by the macro.
**Clarification:** Documented `txt!(@read)` in `crates/widget-core/src/text.rs` and `crates/widget-macros/src/lib.rs`, explaining that it's a special syntax for passing reactive signals.
## 2024-05-24 - [App::run Documentation Update]
**Confusion:** The documentation examples for `App::run` in `crates/arthropod/src/app/core.rs` and `crates/arthropod/src/app/mod.rs` were using `Form::new` instead of the declarative macros (`col!`, `txt!`, `btn!`) that are the recommended way to build UIs in the framework. This didn't align with the examples in the main `lib.rs` and `widget-core`.
**Clarification:** Updated the `App::run` examples in both files to build a reactive "Counter App" using declarative macros, making the recommended approach clearer and more consistent across the codebase.
## 2024-05-25 - [App::run Macro Documentation Fix]
**Confusion:** The documentation example for `App::run` using `col!`, `txt!`, and `btn!` macros failed to compile in CI because it used hallucinated UI patterns (`heading1` literal, `align` property on `col!`) and incorrectly passed a `Signal<i32>` directly to a text widget instead of `ReadSignal<String>`.
**Clarification:** Corrected the example to properly demonstrate standard API patterns: passing a `ReadSignal<String>` using the `@` prefix (`txt!(@read_name)`), wrapping columns in `Center::new(...)` instead of passing `align`, and using explicit `size: <f32>` values.
## 2024-05-26 - [Rustdoc Warnings in Macros]
**Confusion:** Rustdoc warnings were generated in `widget-macros` due to using literal macro syntax (`#[positional]`, `#[children]`) which it mistook for intra-doc links, and angle brackets (`Signal<String>`) which it mistook for unclosed HTML tags.
**Clarification:** Wrapped these elements in backticks (e.g., \`#[positional]\` and \`Signal<String>\`) to ensure they are parsed as code elements, resolving all rustdoc warnings.
## 2024-05-27 - [Feature Gated Missing Docs]
**Confusion:** Rustdoc warnings for `missing_docs` were successfully passing when running `cargo rustdoc -p flux-state -- -D missing_docs` despite `nova` feature-gated structures not being documented.
**Clarification:** You must explicitly pass the feature flags (e.g., `--features nova`) when running `cargo rustdoc` to ensure the tool checks and enforces documentation on conditionally compiled code.
## 2025-01-24 - [Figma and Prototype Runtime API Clarity]\n**Confusion:** The `FigmaRuntime` and `PrototypeRuntime` structs had undocumented public APIs, making it unclear how events should be routed, how layouts are triggered during interactions, and how runtime errors bubble up.\n**Clarification:** Documented the core public types, methods, and error enums for both runtimes, specifically clarifying how `dispatch` routing works and the responsibilities of the `FigmaRuntime` bridge versus the inner `PrototypeRuntime` execution engine.
## 2025-01-24 - [Module and Enum Documentation Coverage]
**Confusion:** The `experimental`, `figma`, `figma_codegen`, `figma_runtime`, and `prototype_runtime` modules in `crates/arthropod/src/lib.rs` and the `core`, `integration`, and `widget` modules in `crates/arthropod/src/app/mod.rs` were missing module-level documentation (`///`), which could lead to confusion about their architectural roles. Additionally, the variants of `AppError` lacked inline documentation, making failure modes opaque.
**Clarification:** Added explicit `///` module-level documentation describing the architectural purpose of each un-documented module, and provided specific failure details for each `AppError` variant to improve error-handling clarity.
## 2025-03-27 - [Widget Core Documentation Coverage]
**Confusion:** The `layer` module, `Style` struct fields/methods, and `icon` mapping functions in `widget-core` lacked documentation, which triggers `missing_docs` warnings and makes the public styling APIs opaque.
**Clarification:** Added explicit documentation to the `LayerManager` concept, `Style` builder methods, icon mappings, and provided an executable doctest for the `style!` macro to make the CSS-like syntax immediately clear to users.
## 2025-05-15 - [Figma Import Models Documentation]
**Confusion:** The `crates/arthropod/src/figma/models.rs` file triggered dozens of `missing_docs` warnings for variants of pure schema-mapping enums like `ConstraintAxis::Min`. Documenting these with tautological phrases (e.g., "The Min constraint") creates noise and doesn't actually help users understand the system.
**Clarification:** I added a file-level `#![allow(missing_docs)]` to `models.rs` to silence warnings for the raw struct and enum variants, as they are essentially generated schema equivalents. To maintain usability, I added comprehensive documentation to the root `ImportedFigmaDocument` struct and the `import_figma_document` function in `schema.rs` to explain the "what" and "why" of the whole module without cluttering the details.
## 2025-05-18 - [Documenting Material UI Theme Subsystems]
**Confusion:** It was unclear what the fields of `MaterialTheme`, `ColorScheme`, `TypographyScale`, and `ElevationScale` were actually for in MD3 context.
**Clarification:** Added detailed documentation to all fields explaining their functional role in the MD3 system (e.g. `primary_container`, `surface_container_highest`, `display_large`, `level5`) along with executable doc tests.

## 2025-05-18 - [Documenting Arthropod ECS]
**Confusion:** Basic `/// The signal` style documentation for internal components like `ReactiveColor`, `ReactiveOpacity`, and ECS query types fails the Bard philosophy by repeating property names without communicating architectural reasons.
**Clarification:** Re-documented `arthropod-ecs` components to explain *why* fields exist, e.g., detailing how `last_value` enables epsilon dirty-checking and why `ReadSignal` is used to prevent cycles. Also added complete executable `## Examples` sections to all exposed constructor methods like `ProgressBarState::new` and `AccessibleNode::new`.
## 2025-05-18 - [Documenting MCP Server Parameters]
**Confusion:** The structs in `crates/arthropod-mcp/src/server.rs` representing JSON-RPC parameters (e.g., `GetNodeParams`, `ListNodesParams`) were missing `///` documentation on their fields, leading to `missing_docs` linter errors.
**Clarification:** Documented the fields for all parameter structs used in the `ArthropodServer` MCP tool handlers to explain their purpose and constraints.

## 2025-05-18 - [Documenting Spatial Graph Experiment]
**Confusion:** The `experiments/spatial-graph` crate lacked module-level documentation and doc comments for its primary components (`SpatialNode`, `Viewport`, `Camera`, `Stats`), making it unclear how the 2D infinite world space mapped to the screen.
**Clarification:** Added module-level `//!` comments to all files and explicit `///` comments with `## Examples` to the key public structs and methods to clarify the projection math.
## 2025-05-18 - [Doc-Test Imports in Workspace]
**Confusion:** Writing doc-tests (`///`) using an assumed prefix like `use experiments_spatial_graph::...` causes the tests to fail with `unresolved module or unlinked crate` errors.
**Clarification:** You must use the exact crate name as defined in `Cargo.toml` (with hyphens replaced by underscores), for example, `use spatial_graph::components::SpatialNode;`.
