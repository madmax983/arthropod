# 🔭 Vantage: Spec for Internationalization (i18n) System

**Status**: Draft
**Owner**: Vantage (Product Manager)
**Target Release**: Arthropod Core (Feature Promotion)

## 1. 👤 User Story

**As a** Global User,
**I want** to use the application in my native language,
**So that** I can understand the interface, navigation, and error messages without relying on external translation tools or struggling with a language barrier.

**As an** Enterprise Developer,
**I want** a unified system to manage string translations, date formatting, and number localization,
**So that** I can confidently release my application to international markets without hardcoding text strings or writing custom localization logic for every widget.

## 2. 🧐 The "So What?" (Business Value)

Currently, Arthropod widgets rely on hardcoded string literals for text content. This severely limits the framework's viability for enterprise applications, which almost universally require supporting multiple languages to reach a global audience.

Without a built-in i18n system, developers are forced to invent their own solutions, leading to fragmented, inconsistent, and often non-reactive translations.

**Utility is Revenue**:
An Internationalization System ensures that:
- Applications can reach a significantly larger market segment.
- Translations are managed in a standardized way (e.g., via translation files).
- The user interface dynamically updates when the language preference changes, providing a seamless experience.

## 3. 🔍 Gap Analysis

| Feature | Current State | Market Standard (e.g., React-Intl / i18next) | Target State (Arthropod i18n System) |
| :--- | :--- | :--- | :--- |
| **String Localization** | Hardcoded strings | Key-based translation lookup | Centralized key-based resolution |
| **Dynamic Language Switching** | Complete app restart | Reactive updates | Reactive signal-based updates |
| **Pluralization** | Manual string building | Built-in rules | Built-in pluralization support |
| **Formatting** | Manual (e.g., `format!`) | Locale-aware date/number formatting | Locale-aware formatting APIs |
| **Consistency** | Low | High | High |

## 4. 📝 Solution Overview

We will introduce a **Internationalization (i18n) System** into the Arthropod framework.

### Key Capabilities

1.  **Translation Registry**:
    -   A centralized store for localized strings mapped to unique keys.
    -   Support for loading translations from standard formats (e.g., JSON, Fluent, or YAML).

2.  **Reactive Locale State**:
    -   The active locale/language must be stored in a reactive signal.
    -   This allows the entire UI to instantly re-render when the user toggles between languages, without requiring a page reload or app restart.

3.  **Widget Integration**:
    -   Provide macros or APIs (e.g., `t!("greeting.hello")`) that automatically resolve to the correct string based on the active locale.
    -   Widgets that display text should seamlessly consume these translated strings.

4.  **Formatting & Plurals**:
    -   Support for passing variables into translated strings.
    -   Support for pluralization rules specific to different languages.

## 5. 📊 Metrics Definition (Success Definition)

-   **Performance**: Changing the active locale must trigger a reactive update that resolves and renders new strings in < 16ms (1 frame).
-   **Adoption Time**: A developer should be able to integrate English and Spanish translations for a basic form in < 15 minutes.
-   **Coverage**: 100% of text-rendering widgets must be capable of consuming dynamic translation keys instead of static strings.

## 6. ✅ Acceptance Criteria

### Must Have (Phase 1)
-   [ ] **Translation Dictionary**: A mechanism to define and load key-value translation dictionaries for multiple locales.
-   [ ] **Translation Macro/API**: An intuitive API for looking up strings by key within widget definitions.
-   [ ] **Reactive Switching**: A signal-based approach to store the active locale, ensuring all translated widgets update instantly when the locale changes.
-   [ ] **Variable Interpolation**: The ability to pass dynamic values into translated strings (e.g., "Hello, {name}").
-   [ ] **Fallback Locale**: A system to fall back to a default locale (e.g., English) if a specific translation key is missing in the target language.

### Should Have (Phase 2)
-   [ ] **Pluralization**: Support for language-specific plural rules (e.g., "1 item", "2 items").
-   [ ] **Number & Date Formatting**: Locale-aware formatting for numbers, currency, and dates.

### Could Have (Future)
-   [ ] **Right-to-Left (RTL) Support**: Automatic layout flipping and text alignment adjustments for RTL languages like Arabic or Hebrew.
-   [ ] **Translation File Extraction**: A CLI tool to automatically extract all translation keys used in the codebase into a template file.

## 7. 🚫 Out of Scope

-   **Automatic Translation**: Integrating with external APIs (like Google Translate or DeepL) to auto-translate strings at runtime. The system expects pre-translated resource files.
-   **Implementation Details**: Specifying the exact underlying data structures (Structs/Enums) or parsing libraries used to manage the dictionaries.
