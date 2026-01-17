**Arthropod**

Enterprise-Grade Cross-Platform Rust GUI Framework

Design Document v0.1

January 2026

# **Table of Contents**

[**Table of Contents	1**](#heading=)

[**Executive Summary	2**](#heading=)

[Key Design Decisions	2](#heading=)

[**Core Philosophy	3**](#heading=)

[Design Principles	3](#heading=)

[**Architecture Overview	4**](#heading=)

[**Layer 0: Platform Abstraction (plat-core)	5**](#heading=)

[Platform Support Matrix	5](#heading=)

[Core Responsibilities	5](#heading=)

[**Layer 1: Rendering Engine (render-engine)	6**](#heading=)

[Render Backends	6](#heading=)

[Enterprise Features	6](#heading=)

[**Animation System (anim-graph)	7**](#heading=)

[Architecture Overview	7](#heading=)

[Animation Primitives	7](#heading=)

[Platform-Native Easing	7](#heading=)

[Animation State Machine	7](#heading=)

[Gesture-Animation Binding	8](#heading=)

[Declarative Animation API	8](#heading=)

[Coordinated Animations	8](#heading=)

[Reduced Motion Support	8](#heading=)

[Scene Graph Integration	8](#heading=)

[Testing Integration	9](#heading=)

[**Layer 2: Layout Engine (layout-engine)	10**](#heading=)

[Layout Models	10](#heading=)

[Performance Characteristics	10](#heading=)

[**Layer 3: Widget Framework (widget-core)	11**](#heading=)

[Reactive Primitives	11](#heading=)

[Widget Trait	11](#heading=)

[Enterprise Widget Requirements	11](#heading=)

[**State Management (flux-state)	12**](#heading=)

[Two Levels of State	12](#heading=)

[Store Architecture	12](#heading=)

[Fine-Grained Subscriptions	12](#heading=)

[Built-in Time Travel	12](#heading=)

[**Styling System	13**](#heading=)

[Layer 1: Platform Native Defaults	13](#heading=)

[Layer 2: Semantic Design Tokens	13](#heading=)

[Layer 3: CSS-like Custom Styling	13](#heading=)

[**Layer 4: Accessibility Engine (a11y-engine)	14**](#heading=)

[Accessibility Tree	14](#heading=)

[Platform Bridges	14](#heading=)

[WCAG 2.1 Compliance	14](#heading=)

[**Mobile Strategy: First-Class Support	15**](#heading=)

[Platform Navigation Patterns	15](#heading=)

[Safe Areas and Adaptivity	15](#heading=)

[Touch-First Input	15](#heading=)

[Adaptive Widgets	15](#heading=)

[**WebAssembly Strategy: Same Codebase	16**](#heading=)

[Rendering Modes	16](#heading=)

[Bundle Size Targets	16](#heading=)

[Web-Specific Features	16](#heading=)

[**Layer 5: Application Shell (app-shell)	17**](#heading=)

[Window Management	17](#heading=)

[Command System	17](#heading=)

[Crash Recovery	17](#heading=)

[**Cross-Cutting Concern: Text Stack	18**](#heading=)

[Text Pipeline	18](#heading=)

[Enterprise Text Requirements	18](#heading=)

[**Development Experience: Fast Compilation	19**](#heading=)

[Workspace Structure	19](#heading=)

[Compilation Optimizations	19](#heading=)

[Asset Hot Reload	19](#heading=)

[**Testing Harness (arthropod-test)	20**](#heading=)

[The Testing Advantage	20](#heading=)

[Core Test Harness	20](#heading=)

[Accessibility-First Locators	20](#heading=)

[Multi-Level Actions	20](#heading=)

[Visual Testing	20](#heading=)

[Accessibility Auditing	22](#heading=)

[Built-in Audit Rules	22](#heading=)

[State Testing	22](#heading=)

[Time Control	22](#heading=)

[Fuzz Testing	23](#heading=)

[Performance Testing	23](#heading=)

[Test Utilities	23](#heading=)

[CI Integration	23](#heading=)

[**Implementation Roadmap	24**](#heading=)

[Phase 1: Foundation (Months 1-3)	24](#heading=)

[Phase 2: Core Widgets (Months 4-6)	24](#heading=)

[Phase 3: Styling & Platform Feel (Months 7-9)	24](#heading=)

[Phase 4: Accessibility (Months 10-12)	24](#heading=)

[Phase 5: Mobile (Months 13-18)	24](#heading=)

[Phase 6: WebAssembly (Months 19-24)	25](#heading=)

[**Open Questions	26**](#heading=)

[Technical Questions	26](#heading=)

[Strategic Questions	26](#heading=)

[**Appendix: Competitive Landscape	27**](#heading=)

# **Executive Summary**

Arthropod is a proposed enterprise-grade, cross-platform GUI framework written in Rust. It aims to solve the fundamental tension in GUI development: achieving native look-and-feel while maintaining a unified codebase across desktop, mobile, and web platforms.

The framework is built on a **layered abstraction architecture** with six core layers: Platform Abstraction, Rendering Engine, Layout Engine, Widget Framework, Accessibility Engine, and Application Shell. This design enables pixel-perfect native experiences while sharing the vast majority of application code across platforms.

## **Key Design Decisions**

**Opinionated State Management:** Built-in fine-grained reactivity with structured stores, enabling O(1) updates without virtual DOM diffing. Includes time-travel debugging and transaction support out of the box.

**Native-First Styling:** Automatic platform material integration (Mica, Vibrancy, Material You) with semantic design tokens and CSS-like escape hatches for full customization.

**Mobile as First-Class:** Not an afterthought. Platform navigation patterns, safe areas, touch gestures, and adaptive widgets are core primitives.

**Single Codebase for WebAssembly:** Hybrid rendering strategy (DOM for accessibility, Canvas for complex graphics) enables the same Rust code to run natively and in browsers.

# **Core Philosophy**

The fundamental tension in cross-platform GUI development is native feel versus code sharing. Enterprise applications demand both: pixel-perfect native experiences and a unified, maintainable codebase. Arthropod resolves this through layered abstraction rather than picking one extreme.

## **Design Principles**

**1\. Platform Respect:** Default to native materials, typography, and interaction patterns. Users should feel at home on their platform.

**2\. Progressive Disclosure:** Simple things should be simple. The framework handles platform differences automatically, but provides escape hatches when needed.

**3\. Performance by Default:** Fine-grained reactivity means minimal work on updates. No virtual DOM diffing. Damage tracking for partial redraws.

**4\. Accessibility as Foundation:** The accessibility tree is built alongside the widget tree, not bolted on. WCAG 2.1 AA compliance is achievable by default.

**5\. Type Safety:** Leverage Rust&\#x2019;s type system to catch errors at compile time. Invalid states should be unrepresentable.

# **Architecture Overview**

Arthropod consists of six primary layers, each with clear responsibilities and well-defined interfaces. Higher layers depend on lower layers, never the reverse.

| Layer | Crate | Responsibility |
| :---- | :---- | :---- |
| Layer 0 | plat-core | Platform abstraction: windows, input, GPU |
| Layer 1 | render-engine | Retained-mode scene graph, multiple backends |
| Layer 2 | layout-engine | Flexbox, Grid, and constraint-based layout |
| Layer 3 | widget-core | Reactive widget framework and primitives |
| Layer 4 | a11y-engine | Accessibility tree and platform bridges |
| Layer 5 | app-shell | Application shell, windows, commands, DI |

# **Layer 0: Platform Abstraction (plat-core)**

The foundation layer providing a thin abstraction over OS primitives. This layer exposes raw GPU handles rather than abstracting rendering, allowing higher layers to choose their rendering strategy.

## **Platform Support Matrix**

| Platform | Windowing | Graphics | Accessibility |
| :---- | :---- | :---- | :---- |
| Windows | Win32 | DX12 / Vulkan | UI Automation |
| macOS | Cocoa | Metal | NSAccessibility |
| Linux | GTK / Wayland | Vulkan | AT-SPI |
| iOS | UIKit | Metal | UIAccessibility |
| Android | NativeActivity | Vulkan / GLES | AccessibilityService |
| Web | DOM / Canvas | WebGPU / WebGL | ARIA |

## **Core Responsibilities**

**Window Management:** Creation, lifecycle, resize events, DPI awareness, multi-monitor support.

**Input Normalization:** Unified event model for keyboard, mouse, touch, pen, and gamepad across all platforms.

**GPU Context:** Surface creation and management. Exposes raw handles for higher layers.

**System Integration:** Clipboard, drag-drop, file dialogs, system tray, notifications.

**Platform Queries:** Theme detection, accessibility preferences, locale, safe areas.

# **Layer 1: Rendering Engine (render-engine)**

A retained-mode scene graph with pluggable backends. The scene graph enables efficient damage tracking and incremental updates&\#x2014;critical for enterprise dashboards with thousands of elements.

## **Render Backends**

| Backend | Use Case |
| :---- | :---- |
| WgpuBackend | Primary cross-platform backend via wgpu |
| SkiaBackend | High-quality 2D rendering, excellent text |
| NativeBackend | Platform compositor (CALayer, DirectComposition) |
| SoftwareBackend | CPU fallback for testing and screenshots |

## **Enterprise Features**

**Damage Tracking:** Only redraw changed regions. Critical for large dashboards where a single value change shouldn&\#x2019;t repaint 10,000 cells.

**Hardware Hit Testing:** GPU-accelerated picking for complex interactive graphics.

**Color Management:** ICC profile support, HDR, wide gamut (Display P3, Rec. 2020).

**Subpixel Text:** Platform-aware subpixel rendering with correct gamma correction.

# **Animation System (anim-graph)**

The scene graph answers &\#x201C;what exists and where&\#x201D;&\#x2014;the animation graph answers &\#x201C;how do things change over time.&\#x201D; Modern UI expectations require smooth micro-interactions, interruptible transitions, gesture-driven animation, and physics-based motion. A dedicated animation system delivers these while respecting accessibility preferences like reduced motion.

## **Architecture Overview**

| Layer | Responsibility |
| :---- | :---- |
| Primitives | Tween, Spring, Keyframes&\#x2014;the building blocks |
| State Machine | Complex UI states with automatic transitions |
| Gesture Binding | Connect drag/pan directly to animation progress |
| Controller | Drives scene graph properties, respects reduced motion |

## **Animation Primitives**

pub enum Animation\<T: Animatable\> {    // Simple interpolation    Tween {        from: T,        to: T,        duration: Duration,        easing: Easing,    },        // Physics-based (interruptible, natural feel)    Spring {        target: T,        stiffness: f32,    // Higher \= snappier        damping: f32,      // Higher \= less oscillation        mass: f32,    },        // Complex multi-step    Keyframes {        frames: Vec\<Keyframe\<T\>\>,        timing: KeyframeTiming,    },        // Composition    Sequence(Vec\<Animation\<T\>\>),    Parallel(Vec\<Animation\<T\>\>),}

## **Platform-Native Easing**

| Easing | Use Case |
| :---- | :---- |
| PlatformDefault | Uses native curve per-platform |
| IosSpring | Critically damped spring (iOS feel) |
| MaterialStandard | Material Design standard curve |
| MaterialEmphasized | Material Design emphasized curve |
| CubicBezier(f32, f32, f32, f32) | Custom bezier curve |

## **Animation State Machine**

For complex UI states like draggable cards that can be dismissed, expanded, or snapped back:

let card\_machine \= AnimationStateMachine::new(CardState::Idle)    .state(CardState::Idle, |s| s        .property(opacity, 1.0)        .property(scale, 1.0)        .property(y\_offset, 0.0)    )    .state(CardState::Dismissed, |s| s        .property(opacity, 0.0)        .property(y\_offset, 300.0)    )    .transition(CardState::Dragging, CardState::Dismissed, |t| t        .animation(Spring::snappy())        .when(|ctx| ctx.velocity.y \> 500.0 || ctx.offset.y \> 150.0)    )    .transition(CardState::Dragging, CardState::Idle, |t| t        .animation(Spring::bouncy())  // Snap back        .when(|ctx| ctx.velocity.y \<= 500.0)    );

## **Gesture-Animation Binding**

The magic of native-feeling UIs is connecting gestures directly to animation progress:

gesture\_detector(|g| {    g.on\_pan(|pan| {        // Map drag distance to animation progress (0.0 \- 1.0)        let progress \= (pan.translation.y / 300.0).clamp(0.0, 1.0);        card\_animation.seek(progress);    })    .on\_pan\_end(|pan| {        if pan.velocity.y \> 500.0 {            // Fling to dismiss \- continues from current progress            card\_animation.animate\_to\_end(Spring::from\_velocity(pan.velocity));        } else {            // Snap back            card\_animation.animate\_to\_start(Spring::bouncy());        }    })})

## **Declarative Animation API**

For simple cases, SwiftUI-style implicit animations:

fn build(\&self, cx: Scope) \-\> impl Element {    let is\_expanded \= cx.signal(false);        container()        .padding(if is\_expanded.get() { 32.0 } else { 16.0 })        .background(if is\_expanded.get() { Color::BLUE } else { Color::GRAY })        // Any property change animates automatically        .animation(Spring::default())        .on\_click(move || is\_expanded.toggle())}

## **Coordinated Animations**

Staggered lists and orchestrated sequences:

// Staggered list animationfor (i, item) in items.iter().enumerate() {    list\_item(item)        .transition(            Transition::enter()                .from(opacity, 0.0)                .from(y\_offset, 20.0)                .delay(Duration::from\_millis(50 \* i as u64))                .animation(Spring::default())        )}

## **Reduced Motion Support**

Baked into the system, not an afterthought. When prefers-reduced-motion is enabled:

| Animation Type | Behavior |
| :---- | :---- |
| Spring | Becomes instant |
| Tween | Reduced to 1ms duration |
| Keyframes | Jumps to final frame |

## **Scene Graph Integration**

The animation graph drives properties in the scene graph. They are separate systems with a clear interface:

// Animation system outputs property changesanimation\_controller.tick(delta\_time);// Scene graph consumes themfor (node\_id, property, value) in animation\_controller.updated\_properties() {    scene\_graph.set\_property(node\_id, property, value);}

## **Testing Integration**

The animation system integrates with arthropod-test:

\#\[test\]fn card\_dismiss\_animation() {    let app \= TestHarness::new(CardDemo::default());    let card \= app.find(test\_id("dismissable-card"));        // Simulate drag gesture    app.touch().pan(        card.center(),        card.center() \+ Vector::new(0.0, 200.0),        Duration::from\_millis(200),    );        // Check intermediate state    assert\!(card.opacity() \< 1.0);        // Complete animation    app.advance\_time(Duration::from\_millis(500));        // Card should be dismissed    assert\!(\!card.exists());}\#\[test\]fn respects\_reduced\_motion() {    let app \= TestHarness::new(MyApp::default())        .with\_reduced\_motion(true);        app.find(role(Role::Button)).click();        // Animation should complete instantly    app.advance\_frame();    assert\_eq\!(app.find(test\_id("panel")).opacity(), 1.0);}

# **Layer 2: Layout Engine (layout-engine)**

Provides Flexbox, CSS Grid, and constraint-based layout with incremental computation. The layout cache memoizes unchanged subtrees for performance.

## **Layout Models**

**Flexbox:** The default for most UI. Row/column layouts with flexible sizing, alignment, and wrapping.

**Grid:** Two-dimensional layouts for complex dashboards and forms.

**Absolute:** Manual positioning relative to a container. Useful for overlays and custom layouts.

**Constraints (Cassowary):** Opt-in constraint solver for complex requirements like form alignment across dynamic content, responsive dashboards with min/max sizes, and intrinsic sizing for data tables.

## **Performance Characteristics**

Layout is computed incrementally. When a widget&\#x2019;s constraints change, only affected subtrees are recalculated. The constraint solver is used selectively&\#x2014;most UI uses flexbox, with constraints reserved for genuinely complex cases.

# **Layer 3: Widget Framework (widget-core)**

A hybrid reactive model combining fine-grained signals (like Leptos/SolidJS) with a structured state store. This approach eliminates virtual DOM diffing while providing predictable state management.

## **Reactive Primitives**

Signal\<T\>: Reactive value that notifies dependents on change. O(1) updates.

Computed\<T\>: Derived value that automatically tracks dependencies and recomputes when they change.

Effect: Side effect that runs when its dependencies change. Used for DOM updates, network requests, etc.

## **Widget Trait**

Widgets implement a simple trait that returns an Element tree. The framework handles lifecycle, subscriptions, and updates automatically.

\#\[widget\]fn counter(cx: Scope) \-\> impl Element {    let count \= cx.signal(0);        row(|r| {        r.child(button("-").on\_click(|| count.update(|c| \*c \-= 1)));        r.child(text(count));  // Auto-subscribes to count        r.child(button("+").on\_click(|| count.update(|c| \*c \+= 1)));    })}

## **Enterprise Widget Requirements**

**Virtualization:** Render only visible items for lists and tables with 100k+ rows.

**Keyboard Navigation:** Roving tabindex, arrow key navigation, type-ahead selection.

**Focus Management:** Focus trapping for modals, focus restoration, programmatic focus.

**Form Validation:** Async validation, field dependencies, error aggregation.

# **State Management (flux-state)**

Arthropod is opinionated about state management. This enables deep integration with the widget lifecycle, built-in devtools, and time-travel debugging.

## **Two Levels of State**

**Widget-Local State:** Signals and computed values for UI state that doesn&\#x2019;t need to be shared (form inputs, hover states, animation progress).

**Application State:** Structured store with reducers for state that spans components (user session, document data, feature flags).

## **Store Architecture**

\#\[derive(State)\]pub struct AppState {    pub user: Option\<User\>,    pub documents: HashMap\<DocId, Document\>,    pub ui: UiState,}\#\[derive(Action)\]enum AppAction {    SetUser(User),    UpdateDocument { id: DocId, content: String },    ToggleSidebar,}

## **Fine-Grained Subscriptions**

Selectors automatically create fine-grained subscriptions. A widget only rebuilds when the specific data it reads changes, not on any state change.

// This widget ONLY rebuilds when user.name changeslet user\_name \= cx.select(|s: \&AppState| {    s.user.as\_ref().map(|u| \&u.name)});

## **Built-in Time Travel**

Since the framework controls state, undo/redo is built-in with transaction grouping for complex operations.

store.transaction("Batch resize", |tx| {    for id in selected\_images {        tx.dispatch(ResizeImage { id, scale: 0.5 });    }}); // Single undo step

# **Styling System**

The styling system has three layers, enabling native feel by default while allowing full customization.

## **Layer 1: Platform Native Defaults**

At the lowest level, Arthropod queries the operating system for native materials and colors. On Windows 11, this means Mica backgrounds. On macOS, NSVisualEffectView vibrancy. On Android, Material You dynamic colors.

| Platform | Native Materials |
| :---- | :---- |
| Windows 11 | Mica, Acrylic, system accent color |
| macOS | Vibrancy (.sidebar, .content, .menu), accent color |
| iOS | UIBlurEffect, dynamic system colors |
| Android | Material You dynamic colors, elevation |
| Linux | GTK/Qt theme colors |
| Web | backdrop-filter, CSS custom properties |

## **Layer 2: Semantic Design Tokens**

Platform-agnostic tokens that resolve to appropriate values per-platform. Tokens can reference platform values or be overridden.

pub struct DesignTokens {    pub surface\_primary: TokenValue,   // Main background    pub surface\_elevated: TokenValue,  // Cards, modals    pub text\_primary: TokenValue,    pub accent: TokenValue,            // From OS or custom    pub space\_md: f32,                 // 16.0    // ...}

## **Layer 3: CSS-like Custom Styling**

For full control, a familiar CSS-like syntax with Rust type safety.

let card \= style\! {    background: tokens.surface\_elevated;    border\_radius: 12.0;    padding: tokens.space\_md;        &:hover {        box\_shadow: tokens.shadow\_lg;    }        @media (prefers\_color\_scheme: dark) {        border: (1.0, rgba(255, 255, 255, 0.1));    }};

# **Layer 4: Accessibility Engine (a11y-engine)**

Accessibility is built into the architecture, not added after. The accessibility tree is separate from the widget tree but synchronized, enabling platform-specific optimizations.

## **Accessibility Tree**

Each accessible element has a role, name, description, state, and actions. The tree is updated incrementally as the UI changes.

pub struct A11yNode {    role: Role,              // Button, Checkbox, Grid, etc.    name: AccessibleName,    // What screen readers announce    description: Option\<String\>,    state: A11yState,        // Checked, Expanded, Disabled, etc.    actions: Vec\<A11yAction\>,// Click, Focus, Expand, etc.    relations: A11yRelations,// LabelledBy, DescribedBy, etc.    bounds: Rect,            // Screen coordinates for navigation}

## **Platform Bridges**

Each platform has different accessibility API patterns. Windows caches aggressively while macOS queries lazily. The bridges handle these differences transparently.

**Windows:** UI Automation provider with caching patterns

**macOS:** NSAccessibility protocol with lazy attribute resolution

**Linux:** AT-SPI D-Bus interface

**iOS:** UIAccessibility traits and custom actions

**Android:** AccessibilityNodeInfo provider

**Web:** ARIA attributes on DOM nodes (hybrid mode)

## **WCAG 2.1 Compliance**

The framework provides tools to achieve WCAG 2.1 AA compliance by default, including focus indicators meeting 3:1 contrast ratio, touch targets of at least 44x44 CSS pixels, live regions for dynamic content announcements, and keyboard operability for all interactive elements.

# **Mobile Strategy: First-Class Support**

Mobile is not a port&\#x2014;it&\#x2019;s a first-class target. Platform navigation patterns, touch interactions, and adaptive layouts are core primitives.

## **Platform Navigation Patterns**

pub enum NavigationStyle {    Stack {        enable\_back\_gesture: bool,  // iOS swipe-back    },    Tabs {        position: TabPosition,      // Bottom (iOS/Android), Top    },    Drawer {        edge: Edge,        behavior: DrawerBehavior,   // Modal, Push, Persistent    },    BottomSheet {        detents: Vec\<SheetDetent\>,  // Collapsed, Half, Full    },}

## **Safe Areas and Adaptivity**

Safe area handling is automatic. The framework queries device metrics and provides insets for notches, home indicators, and system UI.

// Content automatically avoids notches and home indicatorssafe\_area(SafeAreaEdges::ALL, || {    column(|c| {        c.child(header());        c.child(content().flex(1.0));        c.child(bottom\_bar());    })})

## **Touch-First Input**

Gesture recognizers with conflict resolution handle tap, long-press, pan, pinch, and rotate. The framework manages gesture disambiguation (e.g., pan vs. tap after slop threshold).

## **Adaptive Widgets**

Some widgets adapt to their platform automatically. A date picker shows as a native wheel on iOS, a Material dialog on Android, and an inline calendar on desktop.

# **WebAssembly Strategy: Same Codebase**

The same Rust code runs natively and in browsers via WebAssembly. A hybrid rendering strategy balances control with accessibility.

## **Rendering Modes**

| Mode | Pros | Cons |
| :---- | :---- | :---- |
| Canvas | Pixel-perfect, full control | No native a11y, larger bundle |
| **Hybrid** | Native text selection, a11y | More complexity |
| DOM | Smallest bundle, best a11y | Less rendering control |

**Recommendation:** Hybrid as default. DOM for text and standard widgets (accessibility), Canvas for complex custom rendering (charts, editors).

## **Bundle Size Targets**

**Minimal build:** \< 500KB gzipped (Latin text, core widgets)

**Full build:** \< 2MB gzipped (full ICU, all widgets)

## **Web-Specific Features**

**Routing:** History API integration with hash fallback

**PWA:** Service worker support, offline fallback, web manifest generation

**Permissions:** Graceful handling of clipboard, notifications, geolocation APIs

# **Layer 5: Application Shell (app-shell)**

The glue layer that makes applications feel enterprise-grade. Window management, command system, dependency injection, and crash recovery.

## **Window Management**

pub enum WindowKind {    Primary,    Secondary { parent: WindowId },    Modal { owner: WindowId },    Popup { anchor: PopupAnchor },    SystemTray,}

Multi-window support with cross-window drag-drop, proper z-ordering, and platform-appropriate behavior (MDI on Windows, separate windows on macOS).

## **Command System**

A centralized command dispatcher for keyboard shortcuts, menu items, and the command palette. Commands are registered with metadata for discoverability.

commands.register(    Command::new("file.save")        .title("Save")        .shortcut(Modifiers::CTRL, Key::S)        .action(|ctx| ctx.dispatch(SaveDocument)));

## **Crash Recovery**

Application state can be serialized for crash recovery. On restart, the framework offers to restore the previous session.

# **Cross-Cutting Concern: Text Stack**

Text rendering is deceptively complex. Enterprise applications need proper support for complex scripts, bidirectional text, and high-quality typography.

## **Text Pipeline**

| Stage | Library / Responsibility |
| :---- | :---- |
| Unicode Segmentation | icu4x &\#x2014; grapheme, word, line breaks |
| Bidirectional | unicode-bidi &\#x2014; RTL/LTR reordering |
| Shaping | rustybuzz &\#x2014; OpenType shaping |
| Font Fallback | Platform queries \+ embedded fallback chain |
| Rasterization | swash or fontdue &\#x2014; glyph rendering |

## **Enterprise Text Requirements**

**Complex Scripts:** Arabic (with kashida), Thai (without word breaks), Devanagari (with complex conjuncts)

**CJK:** Proper line breaking rules, proportional vs. monospace, vertical text

**Variable Fonts:** Animation between weight/width axes, optical sizing

**Inline Objects:** Emoji with skin tones, @mentions, inline chips

# **Development Experience: Fast Compilation**

Rather than full hot reload, optimize the compile-edit-run cycle for fast iteration.

## **Workspace Structure**

Separate crates by change frequency. Business logic rarely changes; UI code changes constantly.

my\_app/├── crates/│   ├── app-core/      \# Business logic (changes rarely)│   ├── app-widgets/   \# Custom widgets (changes sometimes)│   └── app-main/      \# UI, routing (changes often)└── Cargo.toml         \# Workspace

## **Compilation Optimizations**

**Fast Linker:** Use mold (Linux) or lld (Windows/macOS) for 2-5x faster linking

**Selective Optimization:** Dependencies at opt-level 2, app code at opt-level 0

**Dynamic Linking (dev):** Optional Bevy-style dylib for faster iteration

## **Asset Hot Reload**

In debug builds, assets (images, styles, translations) reload without recompilation. Only Rust code changes require a rebuild.

# **Testing Harness (arthropod-test)**

Because Arthropod controls the entire stack&\#x2014;rendering, accessibility tree, state, and input&\#x2014;it can provide testing capabilities that most GUI frameworks cannot. This is a major differentiator for enterprise adoption where comprehensive testing is mandatory.

## **The Testing Advantage**

Most cross-platform GUI testing is painful. Selenium and WebDriver suffer from fragile CSS selectors and timing issues. Native UI testing requires platform-specific tools. Existing Rust GUI frameworks provide almost no testing infrastructure.

Arthropod can do better because we own the accessibility tree (enabling semantic queries), the renderer (enabling deterministic snapshots), the state layer (enabling injection and time-travel), and the input system (enabling simulation at any level).

## **Core Test Harness**

\#\[test\]fn counter\_increments() {    // Headless app instantiation \- no display needed    let app \= TestHarness::new(CounterApp::default());        // Query by accessibility semantics    let count \= app.find(role(Role::Text).name\_contains("count"));    assert\_eq\!(count.text(), "0");        // Interact via semantic actions    app.find(role(Role::Button).name("+")).click();        // State is synchronous in tests \- no flaky waits    assert\_eq\!(count.text(), "1");}

## **Accessibility-First Locators**

Element location uses accessibility semantics rather than brittle CSS selectors. This approach tests what users actually experience and naturally enforces accessibility.

// By accessibility role and properties (preferred)app.find(role(Role::Button).name("Submit"));app.find(role(Role::TextField).labelled\_by("Email"));app.find(role(Role::Checkbox).checked(true));// By test ID (explicit, stable)app.find(test\_id("login-form"));// By state selector (from flux-state)app.find\_by\_state(|s: \&AppState| \&s.user.avatar);// Compound queriesapp.find(role(Role::ListItem)    .within(test\_id("search-results"))    .nth(0));

## **Multi-Level Actions**

Tests can interact at the semantic level (recommended), the accessibility action level, or with raw input events when precise control is needed.

| Level | Examples |
| :---- | :---- |
| Semantic | click(), type\_text(), select\_option() |
| Accessibility | invoke\_action(A11yAction::Press), Expand, Dismiss |
| Raw Input | mouse().move\_to(), keyboard().key\_press(), touch().pan() |

## **Visual Testing**

Deterministic rendering enables pixel-perfect screenshot comparison with automatic platform-specific baselines.

\#\[test\]fn login\_form\_renders\_correctly() {    let app \= TestHarness::new(LoginPage::default());        // Full screenshot comparison    app.assert\_screenshot("login\_page\_default");        // Element-specific snapshot    app.find(test\_id("login-form"))        .assert\_screenshot("login\_form\_empty");        // With tolerance for anti-aliasing    app.assert\_screenshot\_with\_tolerance("login\_page", 0.01);        // Mask dynamic content    app.assert\_screenshot\_masked("dashboard", vec\!\[        test\_id("current-time"),        test\_id("user-avatar"),    \]);}

Baselines are stored per-platform (Windows, macOS, Linux) and automatically selected during test runs.

## **Accessibility Auditing**

Built-in WCAG 2.1 compliance checking catches accessibility regressions automatically.

\#\[test\]fn app\_meets\_wcag\_aa() {    let app \= TestHarness::new(MyApp::default());        // Full WCAG 2.1 AA audit    let violations \= app.audit\_accessibility(WCAG\_2\_1\_AA);    assert\!(violations.is\_empty(), "{violations:\#?}");}\#\[test\]fn color\_contrast\_is\_sufficient() {    let app \= TestHarness::new(MyApp::default());        let violations \= app.audit(AuditRule::ColorContrast {         min\_ratio: 4.5,      // AA for normal text        min\_ratio\_large: 3.0 // AA for large text    });    assert\!(violations.is\_empty());}

### **Built-in Audit Rules**

| Category | Rules |
| :---- | :---- |
| Visual | ColorContrast, FocusIndicatorVisible, TextResizing |
| Interaction | TouchTargetSize, KeyboardAccessible, NoKeyboardTrap |
| Semantic | ImagesHaveAltText, FormInputsHaveLabels, HeadingHierarchy |
| Preferences | ReducedMotionRespected, HighContrastSupported |

## **State Testing**

Deep integration with flux-state enables state injection, extraction, and time-travel testing.

\#\[test\]fn undo\_redo\_works() {    let app \= TestHarness::new(DocumentEditor::default());        let initial \= app.state\_snapshot::\<AppState\>();        app.find(test\_id("editor")).type\_text("Hello");    let after\_typing \= app.state\_snapshot::\<AppState\>();        // Undo    app.dispatch(AppAction::Undo);    assert\_eq\!(app.state\_snapshot::\<AppState\>(), initial);        // Redo      app.dispatch(AppAction::Redo);    assert\_eq\!(app.state\_snapshot::\<AppState\>(), after\_typing);}\#\[test\]fn ui\_reflects\_injected\_state() {    let app \= TestHarness::new(MyApp::default());        // Inject state directly    app.set\_state(AppState {        user: Some(User { name: "Alice".into(), .. }),        ..Default::default()    });        // Verify UI updated    assert\!(app.find(test\_id("greeting")).text().contains("Alice"));}

## **Time Control**

Tests can control time for deterministic testing of animations, debouncing, and timeouts.

\#\[test\]fn debounced\_search\_works() {    let app \= TestHarness::new(SearchPage::default());        app.find(role(Role::SearchBox)).type\_text("rust gui");        // Search shouldn't fire immediately (debounced)    assert\!(app.find(test\_id("results")).is\_empty());        // Advance time past debounce threshold    app.advance\_time(Duration::from\_millis(300));        // Now results should appear    assert\!(\!app.find(test\_id("results")).is\_empty());}

## **Fuzz Testing**

Random interaction testing finds edge cases and ensures the application never crashes regardless of input.

\#\[test\]fn app\_survives\_random\_input() {    let app \= TestHarness::new(MyApp::default());        app.fuzz(FuzzConfig {        duration: Duration::from\_secs(10),        actions: vec\!\[            FuzzAction::Click { weight: 10 },            FuzzAction::Type { weight: 5, charset: Charset::Alphanumeric },            FuzzAction::KeyPress { weight: 3, keys: vec\!\[Key::Tab, Key::Enter\] },            FuzzAction::Scroll { weight: 2 },        \],        seed: 12345,  // Reproducible    });        // App should not crash and should still be accessible    assert\!(app.is\_responsive());    app.audit\_accessibility(WCAG\_2\_1\_AA);}

## **Performance Testing**

Measure frame times, scroll performance, and startup metrics with built-in benchmarking.

\#\[test\]fn large\_list\_scrolls\_smoothly() {    let app \= TestHarness::new(DataGrid::default())        .with\_state(AppState {             items: (0..100\_000).map(Item::dummy).collect(),            ..Default::default()        });        let metrics \= app.benchmark(|app| {        for \_ in 0..100 {            app.touch().pan(                Point::new(200.0, 400.0),                Point::new(200.0, 100.0),                Duration::from\_millis(16),            );            app.advance\_frame();        }    });        assert\!(metrics.avg\_frame\_time \< Duration::from\_millis(16));    assert\!(metrics.dropped\_frames \== 0);}

## **Test Utilities**

Common testing scenarios are handled with built-in utilities.

| Utility | Purpose |
| :---- | :---- |
| wait\_for() | Wait for async conditions with timeout |
| mock\_network() | Mock HTTP requests and responses |
| set\_viewport() | Test responsive layouts (iPhone, iPad, Desktop) |
| set\_color\_scheme() | Test light/dark mode |
| set\_locale() | Test RTL, CJK, and localization |
| set\_reduced\_motion() | Test accessibility preferences |

## **CI Integration**

The test harness is designed for headless CI environments with JUnit/HTML reporting and automatic screenshot capture on failure.

\# arthropod-test.toml\[test\]parallelism \= "auto"headless \= truedefault\_timeout \= "5s"\[visual\]baseline\_dir \= "snapshots"update\_baselines \= false\[reporting\]format \= \["junit", "html"\]on\_failure \= "screenshot"

# **Implementation Roadmap**

## **Phase 1: Foundation (Months 1-3)**

plat-core: Window/input abstraction for macOS and Windows

render-engine: wgpu backend, basic scene graph

flux-state: Reactive core (signals, computed, effects)

Milestone: Colored rectangles responding to input on two platforms

## **Phase 2: Core Widgets (Months 4-6)**

layout-engine: Flexbox implementation

widget-core: Basic primitives (text, button, input, container)

Text pipeline with font fallback

Milestone: Simple forms with validation

## **Phase 3: Styling & Platform Feel (Months 7-9)**

Platform theme queries

Design token system

Native materials (Mica, Vibrancy)

CSS-like style syntax

Milestone: Apps that feel native on each platform

## **Phase 4: Accessibility (Months 10-12)**

a11y-engine: Tree structure and platform bridges

UI Automation (Windows), NSAccessibility (macOS)

Focus management, keyboard navigation

Milestone: Screen reader support on desktop

## **Phase 5: Mobile (Months 13-18)**

iOS and Android platform integration

Touch gestures and navigation patterns

Adaptive widgets

Milestone: Same app running on desktop and mobile

## **Phase 6: WebAssembly (Months 19-24)**

Hybrid rendering implementation

ARIA accessibility bridge

Bundle optimization

Milestone: Same app running in browsers

# **Open Questions**

## **Technical Questions**

**1\. Async Runtime:** Should the framework be runtime-agnostic (tokio, async-std, smol) or pick one? Runtime-agnostic adds complexity but avoids ecosystem lock-in.

**2\. Text Rendering Backend:** Pure Rust (swash/fontdue) vs. Skia for text? Skia is battle-tested but adds a large native dependency.

**3\. Constraint Solver:** Embed Cassowary or build something simpler? Full constraint solving may be overkill for most layouts.

**4\. Android Integration:** Pure NativeActivity vs. JNI bridge to Android Views? The latter enables native widgets but adds complexity.

## **Strategic Questions**

**1\. Open Source Strategy:** Fully open from day one, or develop privately then open-source?

**2\. Governance Model:** Single-company stewardship vs. foundation model (like Rust itself)?

**3\. Enterprise Licensing:** Dual licensing (MIT \+ commercial) for enterprise support contracts?

**4\. Community Building:** How to build a contributor community given the project&\#x2019;s complexity?

# **Appendix: Competitive Landscape**

| Framework | Native Feel | Mobile | WASM | A11y |
| :---- | :---- | :---- | :---- | :---- |
| Tauri | Via WebView | Beta | N/A | Via HTML |
| egui | Custom | Limited | Yes | Limited |
| Iced | Custom | No | Yes | Limited |
| Slint | Custom \+ Native | Embedded | Yes | Yes |
| Flutter | Custom | Yes | Yes | Yes |
| **Arthropod** | **Native First** | **First-Class** | **Same Code** | **Core** |

**Arthropod&\#x2019;s differentiation:** The combination of native-first styling with first-class mobile support, same-codebase WASM, and accessibility as a core primitive. No existing Rust framework delivers all four.