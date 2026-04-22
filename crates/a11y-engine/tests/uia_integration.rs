//! Integration tests for AccessKit bridge
//!
//! Tests the complete flow: A11yTree → AccessKitBridge → TreeUpdate

use a11y_engine::{
    A11yTree,
    platform::accesskit_bridge::{AccessKitBridge, a11y_node_to_accesskit},
    {A11yNode, A11yState, AccessibleName, CheckedState, Role},
};
use accesskit::NodeId as AccessKitNodeId;
use std::sync::{Arc, Mutex};

#[test]
fn test_accesskit_bridge_full_integration() {
    // Build A11yTree with hierarchy
    let mut tree = A11yTree::new();
    let root_id = tree.root();

    // Add button
    let button_node = A11yNode {
        role: Role::Button,
        name: AccessibleName::Text("Submit".into()),
        ..Default::default()
    };
    let button_id = tree.add_node(root_id, button_node);

    // Add checkbox
    let checkbox_node = A11yNode {
        role: Role::Checkbox,
        name: AccessibleName::Text("Accept Terms".into()),
        state: A11yState {
            checked: Some(CheckedState::Checked),
            ..Default::default()
        },
        ..Default::default()
    };
    let checkbox_id = tree.add_node(root_id, checkbox_node);

    // Get dirty nodes (newly added nodes are dirty)
    let dirty: Vec<_> = tree.get_dirty_nodes().collect();
    assert_eq!(
        dirty.len(),
        2,
        "Should have button and checkbox as dirty (root not marked dirty on creation)"
    );

    // Create AccessKit bridge
    let tree_arc = Arc::new(Mutex::new(tree));
    let bridge = AccessKitBridge::new(tree_arc.clone());

    // Create TreeUpdate
    let update = bridge.create_tree_update(&dirty);

    // Verify TreeUpdate contains dirty nodes
    assert_eq!(
        update.nodes.len(),
        2,
        "Should have 2 nodes in update (button and checkbox)"
    );

    // Verify button node in update
    let button_ak_id = AccessKitNodeId(button_id.raw());
    let button_node_in_update = update
        .nodes
        .iter()
        .find(|(id, _)| *id == button_ak_id)
        .map(|(_, node)| node);
    assert!(
        button_node_in_update.is_some(),
        "Button should be in TreeUpdate"
    );
    let button = button_node_in_update.unwrap();
    assert_eq!(button.role(), accesskit::Role::Button);

    // Verify checkbox node in update
    let checkbox_ak_id = AccessKitNodeId(checkbox_id.raw());
    let checkbox_node_in_update = update
        .nodes
        .iter()
        .find(|(id, _)| *id == checkbox_ak_id)
        .map(|(_, node)| node);
    assert!(
        checkbox_node_in_update.is_some(),
        "Checkbox should be in TreeUpdate"
    );
    let checkbox = checkbox_node_in_update.unwrap();
    assert_eq!(checkbox.role(), accesskit::Role::CheckBox);
}

#[test]
fn test_accesskit_node_conversion_preserves_hierarchy() {
    // Create parent and child nodes
    let mut tree = A11yTree::new();
    let root = tree.root();

    let parent_node = A11yNode {
        role: Role::Group,
        name: AccessibleName::Text("Parent Group".into()),
        ..Default::default()
    };
    let parent_id = tree.add_node(root, parent_node);

    let child_node = A11yNode {
        role: Role::Button,
        name: AccessibleName::Text("Child Button".into()),
        ..Default::default()
    };
    let child_id = tree.add_node(parent_id, child_node);

    // Get dirty nodes before moving tree
    let dirty: Vec<_> = tree.get_dirty_nodes().collect();

    // Create bridge and update
    let tree_arc = Arc::new(Mutex::new(tree));
    let bridge = AccessKitBridge::new(tree_arc.clone());
    let update = bridge.create_tree_update(&dirty);

    // Verify both nodes are in update
    let parent_ak_id = AccessKitNodeId(parent_id.raw());
    let child_ak_id = AccessKitNodeId(child_id.raw());

    let has_parent = update.nodes.iter().any(|(id, _)| *id == parent_ak_id);
    let has_child = update.nodes.iter().any(|(id, _)| *id == child_ak_id);

    assert!(has_parent, "Parent should be in update");
    assert!(has_child, "Child should be in update");

    // Verify roles are correct
    let parent_node = update
        .nodes
        .iter()
        .find(|(id, _)| *id == parent_ak_id)
        .map(|(_, node)| node)
        .expect("Parent should be in update");
    assert_eq!(parent_node.role(), accesskit::Role::GenericContainer);

    let child_node = update
        .nodes
        .iter()
        .find(|(id, _)| *id == child_ak_id)
        .map(|(_, node)| node)
        .expect("Child should be in update");
    assert_eq!(child_node.role(), accesskit::Role::Button);
}

#[test]
fn test_accesskit_only_dirty_nodes_in_update() {
    let mut tree = A11yTree::new();
    let root = tree.root();

    // Add 3 nodes
    let node1 = tree.add_node(
        root,
        A11yNode {
            role: Role::Button,
            name: AccessibleName::Text("Button 1".into()),
            ..Default::default()
        },
    );
    let node2 = tree.add_node(
        root,
        A11yNode {
            role: Role::Button,
            name: AccessibleName::Text("Button 2".into()),
            ..Default::default()
        },
    );
    let node3 = tree.add_node(
        root,
        A11yNode {
            role: Role::Button,
            name: AccessibleName::Text("Button 3".into()),
            ..Default::default()
        },
    );

    // Clear dirty flags
    tree.clear_dirty();

    // Update only node1 and node3
    tree.update_node(node1, |n| {
        n.name = AccessibleName::Text("Button 1 Updated".into());
    });
    tree.update_node(node3, |n| {
        n.name = AccessibleName::Text("Button 3 Updated".into());
    });

    // Get dirty nodes
    let dirty_set: std::collections::HashSet<_> = tree.get_dirty_nodes().collect();
    assert_eq!(dirty_set.len(), 2, "Only 2 nodes should be dirty");
    assert!(dirty_set.contains(&node1), "node1 should be dirty");
    assert!(dirty_set.contains(&node3), "node3 should be dirty");
    assert!(!dirty_set.contains(&node2), "node2 should not be dirty");

    // Create update with only dirty nodes
    let dirty: Vec<_> = dirty_set.into_iter().collect();
    let tree_arc = Arc::new(Mutex::new(tree));
    let bridge = AccessKitBridge::new(tree_arc);
    let update = bridge.create_tree_update(&dirty);

    // Verify only dirty nodes in update
    assert_eq!(
        update.nodes.len(),
        2,
        "Only dirty nodes should be in update"
    );

    let node_ids_in_update: Vec<AccessKitNodeId> = update.nodes.iter().map(|(id, _)| *id).collect();
    assert!(
        node_ids_in_update.contains(&AccessKitNodeId(node1.raw())),
        "node1 should be in update"
    );
    assert!(
        node_ids_in_update.contains(&AccessKitNodeId(node3.raw())),
        "node3 should be in update"
    );
    assert!(
        !node_ids_in_update.contains(&AccessKitNodeId(node2.raw())),
        "node2 should not be in update"
    );
}

#[test]
fn test_all_roles_convert_correctly() {
    // Test all role conversions that exist in our Role enum
    let roles_to_test = vec![
        // Widgets
        (Role::Button, accesskit::Role::Button),
        (Role::Checkbox, accesskit::Role::CheckBox),
        (Role::Radio, accesskit::Role::RadioButton),
        (Role::Textbox, accesskit::Role::TextInput),
        (Role::Slider, accesskit::Role::Slider),
        (Role::ProgressBar, accesskit::Role::ProgressIndicator),
        // Containers
        (Role::Group, accesskit::Role::GenericContainer),
        (Role::List, accesskit::Role::List),
        (Role::ListItem, accesskit::Role::ListItem),
        (Role::Grid, accesskit::Role::Table),
        (Role::GridCell, accesskit::Role::Cell),
        // Document structure
        (Role::Heading { level: 1 }, accesskit::Role::Heading),
        (Role::Heading { level: 2 }, accesskit::Role::Heading),
        (Role::Paragraph, accesskit::Role::Paragraph),
        (Role::Region, accesskit::Role::Region),
        // Landmarks
        (Role::Main, accesskit::Role::Main),
        (Role::Navigation, accesskit::Role::Navigation),
        (Role::Search, accesskit::Role::Search),
        (Role::Form, accesskit::Role::Form),
        // Special
        (Role::Alert, accesskit::Role::Alert),
        (Role::Dialog, accesskit::Role::Dialog),
        (Role::Tooltip, accesskit::Role::Tooltip),
    ];

    for (arthropod_role, expected_ak_role) in roles_to_test {
        let node = A11yNode {
            role: arthropod_role,
            ..Default::default()
        };

        let ak_node = a11y_node_to_accesskit(&node);
        assert_eq!(
            ak_node.role(),
            expected_ak_role,
            "Role {:?} should convert to {:?}",
            arthropod_role,
            expected_ak_role
        );
    }
}

#[test]
fn test_accesskit_preserves_node_role() {
    // Create node with various properties set
    let node = A11yNode {
        role: Role::Button,
        name: AccessibleName::Text("Submit Form".into()),
        state: A11yState {
            disabled: true,
            focused: false,
            checked: Some(CheckedState::Unchecked),
            ..Default::default()
        },
        description: Some("Submits the current form".into()),
        ..Default::default()
    };

    // Convert to AccessKit
    let ak_node = a11y_node_to_accesskit(&node);

    // Verify role is preserved
    assert_eq!(ak_node.role(), accesskit::Role::Button);
}

#[test]
fn test_accesskit_bridge_handles_node_removal() {
    let mut tree = A11yTree::new();
    let root = tree.root();

    // Add two nodes
    let node1 = tree.add_node(
        root,
        A11yNode {
            role: Role::Button,
            name: AccessibleName::Text("Persistent".into()),
            ..Default::default()
        },
    );
    let node2 = tree.add_node(
        root,
        A11yNode {
            role: Role::Button,
            name: AccessibleName::Text("Temporary".into()),
            ..Default::default()
        },
    );

    // Clear dirty and update only node2
    tree.clear_dirty();
    tree.update_node(node2, |n| {
        n.name = AccessibleName::Text("About to be removed".into());
    });

    // Remove node2
    tree.remove_node(node2);

    // Verify node2 is gone
    assert!(
        tree.get_node(node2).is_none(),
        "Node should be removed from tree"
    );

    // Get dirty nodes before moving tree
    let dirty: Vec<_> = tree.get_dirty_nodes().collect();

    // Create bridge and update - node2 should not appear
    let tree_arc = Arc::new(Mutex::new(tree));
    let bridge = AccessKitBridge::new(tree_arc.clone());
    let update = bridge.create_tree_update(&dirty);

    let removed_ak_id = AccessKitNodeId(node2.raw());
    let node_in_update = update.nodes.iter().any(|(id, _)| *id == removed_ak_id);
    assert!(!node_in_update, "Removed node should not be in update");

    // Verify node1 still exists
    {
        let tree_lock = tree_arc.lock().unwrap();
        assert!(
            tree_lock.get_node(node1).is_some(),
            "Other nodes should remain"
        );
    }
}

#[test]
fn test_accesskit_bridge_multiple_updates() {
    let mut tree = A11yTree::new();
    let root = tree.root();

    // First update: add nodes
    let node1 = tree.add_node(
        root,
        A11yNode {
            role: Role::Button,
            ..Default::default()
        },
    );

    let tree_arc = Arc::new(Mutex::new(tree));
    let bridge = AccessKitBridge::new(tree_arc.clone());

    let dirty1: Vec<_> = {
        let tree_lock = tree_arc.lock().unwrap();
        tree_lock.get_dirty_nodes().collect()
    };
    let update1 = bridge.create_tree_update(&dirty1);
    assert_eq!(
        update1.nodes.len(),
        1,
        "First update should have 1 node (the newly added button)"
    );

    // Second update: modify node
    {
        let mut tree_lock = tree_arc.lock().unwrap();
        tree_lock.clear_dirty();
        tree_lock.update_node(node1, |n| {
            n.name = AccessibleName::Text("Modified".into());
        });
    }

    let dirty2: Vec<_> = {
        let tree_lock = tree_arc.lock().unwrap();
        tree_lock.get_dirty_nodes().collect()
    };
    let update2 = bridge.create_tree_update(&dirty2);
    assert_eq!(
        update2.nodes.len(),
        1,
        "Second update should only have modified node"
    );

    // Third update: add another node
    {
        let mut tree_lock = tree_arc.lock().unwrap();
        tree_lock.clear_dirty();
        tree_lock.add_node(
            root,
            A11yNode {
                role: Role::Checkbox,
                ..Default::default()
            },
        );
    }

    let dirty3: Vec<_> = {
        let tree_lock = tree_arc.lock().unwrap();
        tree_lock.get_dirty_nodes().collect()
    };
    let update3 = bridge.create_tree_update(&dirty3);
    assert_eq!(
        update3.nodes.len(),
        1,
        "Third update should only have new node"
    );
}
