# 🔭 Vantage: Spec for Internationalization (i18n)

**Status:** Draft
**Owner:** Vantage (Product Manager)
**Date:** 2024-05-25

## 1. The "User Story" 👤

> **"As a Global Product Manager building applications with Arthropod, I want an integrated Internationalization (i18n) system, so that I can easily support multiple languages and regions without rewriting UI components or hardcoding strings."**

## 2. The "So What?" (Business Value) 💰

**Problem:**
Currently, text strings in Arthropod are hardcoded into widget macros (e.g., `txt!("Hello World")`). To support multiple languages, developers must manually build translation dictionaries, write reactive wrapper components, and manage language state globally. This leads to:
- **Scalability Blockers:** Hardcoded text makes it impossible to deploy applications to non-English markets.
- **Maintenance Nightmares:** Manual translation logic scattered across the codebase is error-prone and difficult for localization teams to manage.
- **Inconsistent UX:** Poor handling of right-to-left (RTL) text, date formatting, and number localization creates a disjointed experience for international users.
- **Enterprise Dealbreaker:** Multi-language support is a strict requirement for almost all enterprise software.

**Solution:**
A first-class Internationalization Engine built into the framework that provides declarative translation keys, reactive locale state, and automatic formatting for dates, numbers, and currencies.

**Impact:**
- Unlocks global markets by making localization seamless.
- Drastically reduces developer overhead for managing multi-language applications.
- Ensures high-quality UX across diverse regions and writing systems.

## 3. Gap Analysis 🔍

| Feature | React (`react-intl`) | Vue (`vue-i18n`) | Arthropod (Current) | Arthropod (Target) |
| :--- | :--- | :--- | :--- | :--- |
| **Translation Keys** | Yes | Yes | Manual | **Yes (Built-in)** |
| **Reactive Locale** | Yes (Context) | Yes | Manual | **Yes (Signals)** |
| **Pluralization** | Yes | Yes | No | **Yes** |
| **RTL Layout Support** | Manual | Manual | No | **Yes (Layout Engine Integration)** |
| **Number/Date Formatting**| Yes (Intl API) | Yes (Intl API) | Manual | **Yes** |

## 4. Acceptance Criteria ✅

To consider this feature "Done", the Engineering team must demonstrate:

1.  **Translation Dictionary:**
    -   Must support loading translations from standard formats (e.g., JSON, Fluent, or gettext).
    -   Must support interpolation (injecting variables into strings) and pluralization rules.
2.  **Reactive Locale Management:**
    -   Must provide a global signal for the active locale (e.g., `Locale::current()`).
    -   Changing the locale must automatically trigger surgical re-renders of only the text nodes that depend on translations.
3.  **Widget Integration:**
    -   Must provide a standardized macro or widget for translated text (e.g., `t!("greeting.welcome")` instead of `txt!("Welcome")`).
4.  **Formatting APIs:**
    -   Must provide utilities for formatting dates, times, currencies, and numbers according to the active locale.

## 5. Out of Scope (Phase 1) 🚫

-   **Automatic RTL Layout Flipping:** Phase 2. (Automatically mirroring flexbox layouts for RTL languages like Arabic or Hebrew). Phase 1 will focus purely on text translation and formatting.
-   **Live Translation Editing:** Phase 3. (In-app tools for translators to edit strings visually).
-   **Machine Translation Integration:** Out of scope. The system should only consume pre-translated resource files.

## 6. Metrics for Success 📊

-   **Performance:** Changing the locale in an application with 1,000 translated strings must resolve and re-render in < 16ms.
-   **DX Audit:** A developer can implement a language toggle and translate a basic form (5 fields) in under 30 lines of framework configuration code.