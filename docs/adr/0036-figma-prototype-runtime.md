# Figma Prototype Runtime Execution Semantics

**Status:** Proposed

## Context
Our platform needs to handle complex runtime behaviors defined in imported prototype graphs (such as those from Figma). The challenge is effectively executing deterministic navigation behaviors without intertwining them with core app logic. We needed a comprehensive mechanism to orchestrate event dispatch, timeout handling, navigation history, overlay stack rules, back semantics, and URL effects natively within the framework.

## Decision
We implemented a dedicated `PrototypeRuntime` module and exported it via the Arthropod public API. This acts as a deterministic runtime executor for these prototype graphs.

## Consequences
- **Positive:** Interactive prototype testing and execution is possible natively. Complexity related to prototype execution semantics (overlays, histories, timeouts) is neatly centralized inside `PrototypeRuntime` instead of scattering across the core ECS or component update systems.
- **Negative:** Adds complexity to runtime state management as we now track independent overlay stacks and navigation histories for prototype modes.
