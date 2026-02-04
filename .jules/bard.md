# Bard's Journal 🎻

## 2024-05-22 - The Missing Map
**Confusion:** The project lacked a root `README.md`, making it impossible for new users (and potential contributors) to understand what Arthropod is, how to build it, or where to start.
**Clarification:** Created a comprehensive `README.md` that serves as the map for the repository, explaining the architecture, key crates, and providing a "Hello World" quick start guide.

## 2024-05-22 - The Hidden Engine
**Confusion:** The `flux-state` crate, being the core reactive engine, had no visible README on GitHub, forcing users to dive into source code to understand how to use it.
**Clarification:** Created `crates/flux-state/README.md` mirroring the crate-level documentation to provide immediate visibility into Core Concepts and Quick Start examples.

## 2024-05-22 - Layout Engine Exports
**Confusion:** `layout-engine` uses `taffy` internally but does not publicly re-export `Dimension` enum. Users attempting to construct `FlexStyle` manually might try to use `Dimension::Points` based on Taffy docs.
**Clarification:** `FlexStyle` uses `Option<f32>` for width/height instead of `Dimension`. The conversion happens internally.

## 2024-05-23 - The Illusory Z-Order
**Confusion:** `Scene::hit_test` documentation claimed to return the "topmost" node, implying a managed Z-order. However, the implementation iterates over a HashMap, making the result non-deterministic for overlapping nodes.
**Clarification:** Updated docs to explicitly state that behavior for overlapping nodes is undefined in the current implementation.

## 2024-05-24 - Z-Order Redemption
**Confusion:** Previous investigation suggested `Scene::hit_test` was non-deterministic due to HashMap iteration.
**Clarification:** Re-investigation revealed that `hit_test` iterates over the `children` Vector (not the HashMap directly), ensuring deterministic Z-order (last child added is top-most). Docs and tests were updated to reflect this guarantee.
