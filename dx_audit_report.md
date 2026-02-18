# 🗣️ Echo Report: Getting Started

## Experience Audit

### 1. The "README Run"
**Scenario:** I copied the "Quick Start" code from `README.md` into `examples/echo_quickstart.rs` and ran it.

**Command:** `cargo run --example echo_quickstart`

**Result:**
- **Compilation:** ✅ Success! It compiled without errors.
- **Runtime:** ❌ Failed (Expected on Linux).
  - Error: `WindowCreation("Platform initialization failed: The current platform is not supported. Arthropod currently supports Windows and macOS.")`

### 2. The Friction Points
- **OS Support:** As a Linux user (or someone running in a cloud IDE/CI), I hit a wall immediately. The error message is clear, but it's disappointing.
- **"Nova" Confusion:** The README has a big warning: "**REQUIRES FEATURE NOVA**: Experimental features ... must have the nova feature flag enabled." I wasn't sure if "Quick Start" used experimental features. Turns out it doesn't, which is good, but the warning made me hesitate.

### 3. Suggestions
- **Add a Note in Quick Start:** "Note: Currently supports Windows and macOS. Linux support is planned." (It's in "Features", but I didn't read that far before copying code).
- **Consider a Headless/TUI Mode:** Allow `App::run` to fallback to a headless mode or TUI on unsupported platforms so I can at least see "Hello World" in the console.

## Conclusion
The Quick Start code is valid and compiles. The error message for unsupported platforms is clear. The DX is acceptable, but could be friendlier to Linux users.
