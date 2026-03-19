# 🔭 Vantage: Spec for Strict Widget Macros

**Status:** Draft
**Owner:** Vantage (Product Manager)
**Date:** 2024-05-22

## 1. The "User Story" 👤

> **"As an Application Developer building UI with Arthropod, I want the compiler to immediately reject invalid or misspelled properties in widget macros (like `txt!`, `btn!`, `col!`), so that I don't waste time debugging why my UI isn't rendering as expected."**

## 2. The "So What?" (Business Value) 💰

**Problem:**
Currently, our declarative UI macros silently ignore unrecognized properties. For example, typing `wrong_arg: 24.0` instead of `size: 24.0` in a `txt!` macro compiles successfully but fails to apply the expected styling. This leads to:
- **Lost Developer Time:** Developers spend minutes or hours troubleshooting silent failures instead of building features.
- **Frustration & Churn:** Poor Developer Experience (DX) is the leading cause of framework abandonment. If developers don't trust the tools to catch simple mistakes, they won't adopt Arthropod.
- **Maintenance Burden:** Typos silently lingering in the codebase make refactoring and debugging significantly harder later on.

**Solution:**
Enforce strict validation on all widget macros at compile time. Any unrecognized or misspelled property must generate a hard compiler error.

**Impact:**
- Drastically improves DX by providing instant feedback.
- Builds trust in the framework's reliability.
- Reduces support load and "why doesn't this work?" questions in the community.

## 3. Gap Analysis 🔍

| Feature | React (JSX/TSX) | SwiftUI | Arthropod (Current) | Arthropod (Target) |
| :--- | :--- | :--- | :--- | :--- |
| **Property Validation** | Strict (via TS/PropTypes) | Strict (Compiler Error) | Loose (Silently Ignored) | **Strict (Compiler Error)** |
| **Feedback Loop** | Instant (IDE/Build) | Instant (IDE/Build) | Delayed (Runtime Debugging) | **Instant (IDE/Build)** |
| **DX Confidence** | High | High | Low | **High** |

## 4. Acceptance Criteria ✅

To consider this feature "Done", the Engineering team must demonstrate:

1.  **Immediate Failure on Typos:**
    -   Passing an invalid property name (e.g., `txt!("Hello", typo_size: 14.0)`) must fail compilation with a clear error indicating the property is unknown.
2.  **Strict Property Matching:**
    -   Macros must only accept explicitly defined properties. Catch-all patterns that swallow tokens are forbidden.
3.  **Clear Error Messages:**
    -   The compiler error must be actionable (e.g., "Unknown property 'typo_size'"). It should not be a generic parser error like "no rules expected this token" if possible, though failing to compile is the primary requirement.
4.  **No Regression in Valid Usage:**
    -   All existing, correctly formatted macro invocations in our examples and test suites must continue to compile and function without modification.

## 5. Out of Scope (Phase 1) 🚫

-   **"Did you mean?" suggestions:** While IDE-like suggestions would be excellent, generating dynamic "did you mean 'size'?" hints within the macro error output is out of scope for this initial strictness pass.
-   **Runtime Property Validation:** This spec only covers compile-time checks via macros.

## 6. Metrics for Success 📊

-   **DX Audit:** The `🗣️ Echo: Widget macros swallow typo errors silently` scenario must pass (i.e., result in a compiler error) on the next evaluation.
-   **Bug Reports:** Zero user bug reports related to "property not applying" caused by simple typos.
