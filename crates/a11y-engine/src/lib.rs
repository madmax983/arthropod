//! Accessibility engine for Arthropod GUI framework
//!
//! Provides WCAG 2.1 AA compliance and screen reader support through platform-specific bridges.
//!
//! # Architecture
//!
//! ```text
//! Scene Tree  →  A11yTree  →  Platform Bridges
//!                              ├─ Windows (UI Automation)
//!                              ├─ macOS (NSAccessibility)
//!                              └─ Linux (AT-SPI)
//! ```
//!
//! # Performance
//!
//! - Add node: ~50 ns (HashMap insert)
//! - Update node: ~30 ns (mark dirty)
//! - Sync to platform: O(dirty) - only changed nodes
//!
//! # Example
//!
//! ```
//! use a11y_engine::{A11yTree, A11yNode, Role, AccessibleName};
//!
//! let mut tree = A11yTree::new();
//!
//! let button = A11yNode {
//!     role: Role::Button,
//!     name: AccessibleName::Text("Click me".into()),
//!     ..Default::default()
//! };
//!
//! let button_id = tree.add_node(tree.root(), button);
//! ```

pub mod node;
pub mod tree;

pub use node::{
    A11yAction, A11yId, A11yNode, A11yRelations, A11yState, AccessibleName, CheckedState, Role,
};
pub use tree::A11yTree;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_a11y_id_unique() {
        let id1 = A11yId::new();
        let id2 = A11yId::new();
        assert_ne!(id1, id2, "A11yIds should be unique");
    }

    #[test]
    fn test_role_variants() {
        // Test role creation
        let button = Role::Button;
        let heading = Role::Heading { level: 2 };

        assert_eq!(button, Role::Button);
        assert_eq!(heading, Role::Heading { level: 2 });
    }
}
