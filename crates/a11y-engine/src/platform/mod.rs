//! Platform abstraction for accessibility bridges
//!
//! Each platform (Windows UIA, macOS NSAccessibility, Linux AT-SPI) implements this trait
//! to synchronize the platform-agnostic A11yTree with the OS accessibility API.

use crate::node::A11yId;

pub mod accesskit_bridge;
pub mod action_handler;

#[cfg(target_os = "windows")]
pub mod windows;

/// Trait for platform-specific accessibility bridges
pub trait A11yBridge {
    /// Update a node that has changed (or is new)
    fn update_node(&mut self, id: A11yId);

    /// Remove a node from the platform tree
    fn remove_node(&mut self, id: A11yId);

    /// Set navigation focus to a specific node
    fn focus_node(&mut self, id: A11yId);
}
