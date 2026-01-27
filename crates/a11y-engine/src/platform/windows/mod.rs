//! Windows UI Automation (UIA) implementation via AccessKit
//!
//! This module previously contained a direct UIA implementation (provider.rs, pattern.rs)
//! which had critical issues (broken COM, unsafe transmute UB).
//!
//! **That implementation has been replaced with AccessKit** - a battle-tested library
//! by Matt Campbell (former Microsoft a11y team) that properly handles Windows UIA.
//!
//! See `platform::accesskit_bridge` for the current implementation.

pub mod util;
