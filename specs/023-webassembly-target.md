# Spec: WebAssembly (Wasm) Web Target Support

## 👤 User Story
As a web developer or cross-platform application creator, I want to compile and run my Arthropod GUI applications directly in the browser via WebAssembly (Wasm), so that I can reach users on any platform without requiring native installations.

## 💼 So What? (Business Problem)
Native desktop applications limit user acquisition by requiring downloads and bypassing strict corporate IT installation policies. By targeting the web natively without a rewrite, we expand our potential addressable user base by an order of magnitude and lower the barrier to entry, converting the framework from a niche desktop tool into a universal deployment solution.

## 📊 Metric Definition
- **Success** = The `wasm` bundle payload is < 3MB gzipped for a hello-world app.
- **Success** = Initial time-to-interactive (TTI) is < 1000ms on a standard mobile network.
- **Success** = Framerates consistently hit 60 FPS for standard UI navigation inside WebGL/WebGPU canvases on mid-tier hardware.

## 🔍 Gap Analysis
Currently, compiling Arthropod targets strictly OS-native backends (Windows/macOS), missing the most pervasive platform: the browser. Compared to the market, `egui`, `dioxus`, and `slint` all provide zero-friction web targets. Without Wasm support, Arthropod falls behind these standard libs as an enterprise-grade choice for universal deployment.

## ✅ Acceptance Criteria
- Must compile standard Arthropod GUI applications to `wasm32-unknown-unknown` without compilation errors.
- Must render the GUI natively in standard web browsers using WebGL or WebGPU.
- Must support standard input events (mouse, keyboard, touch) within the browser window.
- The web target must integrate cleanly with the existing reactive state engine and ECS architecture without requiring code forks.

## 🚫 Out of Scope
- Server-Side Rendering (SSR) of initial GUI states.
- Deep integration with browser DOM manipulation or CSS styling out-of-the-box.
- Support for ancient browsers lacking Wasm or minimum required WebGL/WebGPU specifications.
