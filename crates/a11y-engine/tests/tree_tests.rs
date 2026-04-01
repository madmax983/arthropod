//! Tests for accessibility tree
//!
//! Following TDD - tests written before full implementation.

use a11y_engine::{A11yNode, A11yTree, AccessibleName, Role};
use plat_core::Rect;

#[test]
fn test_create_empty_tree() {
    let tree = A11yTree::new();
    let root = tree.root();

    assert!(tree.get_node(root).is_some());
    let root_node = tree.get_node(root).unwrap();
    assert!(root_node.parent.is_none());
    assert!(root_node.children.is_empty());
}

#[test]
fn test_add_node_to_root() {
    let mut tree = A11yTree::new();
    let root = tree.root();

    let button = A11yNode {
        role: Role::Button,
        name: AccessibleName::Text("Click me".into()),
        ..Default::default()
    };

    let button_id = tree.add_node(root, button);

    // Verify node was added
    assert!(tree.get_node(button_id).is_some());

    // Verify parent relationship
    let button_node = tree.get_node(button_id).unwrap();
    assert_eq!(button_node.parent, Some(root));

    // Verify child relationship
    let root_node = tree.get_node(root).unwrap();
    assert_eq!(root_node.children.len(), 1);
    assert_eq!(root_node.children[0], button_id);
}

#[test]
fn test_add_multiple_children() {
    let mut tree = A11yTree::new();
    let root = tree.root();

    let button1 = tree.add_node(
        root,
        A11yNode {
            role: Role::Button,
            name: AccessibleName::Text("Button 1".into()),
            ..Default::default()
        },
    );

    let button2 = tree.add_node(
        root,
        A11yNode {
            role: Role::Button,
            name: AccessibleName::Text("Button 2".into()),
            ..Default::default()
        },
    );

    let root_node = tree.get_node(root).unwrap();
    assert_eq!(root_node.children.len(), 2);
    assert!(root_node.children.contains(&button1));
    assert!(root_node.children.contains(&button2));
}

#[test]
fn test_nested_nodes() {
    let mut tree = A11yTree::new();
    let root = tree.root();

    let group = tree.add_node(
        root,
        A11yNode {
            role: Role::Group,
            ..Default::default()
        },
    );

    let button = tree.add_node(
        group,
        A11yNode {
            role: Role::Button,
            name: AccessibleName::Text("Nested".into()),
            ..Default::default()
        },
    );

    // Verify hierarchy
    let button_node = tree.get_node(button).unwrap();
    assert_eq!(button_node.parent, Some(group));

    let group_node = tree.get_node(group).unwrap();
    assert_eq!(group_node.parent, Some(root));
    assert_eq!(group_node.children.len(), 1);
    assert_eq!(group_node.children[0], button);
}

#[test]
fn test_update_node() {
    let mut tree = A11yTree::new();
    let root = tree.root();

    let button_id = tree.add_node(
        root,
        A11yNode {
            role: Role::Button,
            name: AccessibleName::Text("Original".into()),
            ..Default::default()
        },
    );

    // Update node
    tree.update_node(button_id, |node| {
        node.name = AccessibleName::Text("Updated".into());
        node.bounds = Rect::new(100.0, 200.0, 50.0, 30.0);
    });

    // Verify update
    let button = tree.get_node(button_id).unwrap();
    match &button.name {
        AccessibleName::Text(text) => assert_eq!(text, "Updated"),
        _ => panic!("Expected Text name"),
    }
    assert_eq!(button.bounds, Rect::new(100.0, 200.0, 50.0, 30.0));
}

#[test]
fn test_remove_node() {
    let mut tree = A11yTree::new();
    let root = tree.root();

    let button = tree.add_node(
        root,
        A11yNode {
            role: Role::Button,
            ..Default::default()
        },
    );

    // Verify node exists
    assert!(tree.get_node(button).is_some());

    // Remove node
    tree.remove_node(button);

    // Verify node removed
    assert!(tree.get_node(button).is_none());

    // Verify parent updated
    let root_node = tree.get_node(root).unwrap();
    assert!(root_node.children.is_empty());
}

#[test]
fn test_remove_node_with_children() {
    let mut tree = A11yTree::new();
    let root = tree.root();

    let group = tree.add_node(
        root,
        A11yNode {
            role: Role::Group,
            ..Default::default()
        },
    );

    let button = tree.add_node(
        group,
        A11yNode {
            role: Role::Button,
            ..Default::default()
        },
    );

    // Remove group (should remove children too)
    tree.remove_node(group);

    // Verify both removed
    assert!(tree.get_node(group).is_none());
    assert!(tree.get_node(button).is_none());

    // Verify root updated
    let root_node = tree.get_node(root).unwrap();
    assert!(root_node.children.is_empty());
}

#[test]
fn test_query_by_role() {
    let mut tree = A11yTree::new();
    let root = tree.root();

    let button1 = tree.add_node(
        root,
        A11yNode {
            role: Role::Button,
            ..Default::default()
        },
    );

    let _textbox = tree.add_node(
        root,
        A11yNode {
            role: Role::Textbox,
            ..Default::default()
        },
    );

    let button2 = tree.add_node(
        root,
        A11yNode {
            role: Role::Button,
            ..Default::default()
        },
    );

    let buttons = tree.query_by_role(Role::Button);
    assert_eq!(buttons.len(), 2);
    assert!(buttons.contains(&button1));
    assert!(buttons.contains(&button2));

    let textboxes = tree.query_by_role(Role::Textbox);
    assert_eq!(textboxes.len(), 1);
}

#[test]
fn test_dirty_tracking() {
    let mut tree = A11yTree::new();
    let root = tree.root();

    let button = tree.add_node(
        root,
        A11yNode {
            role: Role::Button,
            ..Default::default()
        },
    );

    // Node should be dirty after add
    assert!(tree.is_dirty(button));

    // Clear dirty flag
    tree.clear_dirty();
    assert!(!tree.is_dirty(button));

    // Update should mark dirty
    tree.update_node(button, |node| {
        node.name = AccessibleName::Text("Updated".into());
    });

    assert!(tree.is_dirty(button));
}

#[test]
fn test_get_dirty_nodes() {
    let mut tree = A11yTree::new();
    let root = tree.root();

    let button1 = tree.add_node(
        root,
        A11yNode {
            role: Role::Button,
            ..Default::default()
        },
    );

    let button2 = tree.add_node(
        root,
        A11yNode {
            role: Role::Button,
            ..Default::default()
        },
    );

    tree.clear_dirty();

    // Update only button1
    tree.update_node(button1, |node| {
        node.state.disabled = true;
    });

    let dirty: std::collections::HashSet<_> = tree.get_dirty_nodes().collect();
    assert_eq!(dirty.len(), 1);
    assert!(dirty.contains(&button1));
    assert!(!dirty.contains(&button2));
}

#[test]
fn test_node_count() {
    let mut tree = A11yTree::new();
    let root = tree.root();

    assert_eq!(tree.node_count(), 1); // Just root

    let _button1 = tree.add_node(
        root,
        A11yNode {
            role: Role::Button,
            ..Default::default()
        },
    );

    assert_eq!(tree.node_count(), 2);

    let _button2 = tree.add_node(
        root,
        A11yNode {
            role: Role::Button,
            ..Default::default()
        },
    );

    assert_eq!(tree.node_count(), 3);
}
