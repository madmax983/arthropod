//! Tests for accessibility node types
//!
//! Following TDD - tests written before implementation.

use a11y_engine::{
    A11yAction, A11yNode, A11yRelations, A11yState, AccessibleName, CheckedState, Role,
};
use plat_core::Rect;

#[test]
fn test_create_button_node() {
    let node = A11yNode {
        role: Role::Button,
        name: AccessibleName::Text("Click me".into()),
        description: Some("Submit form".into()),
        state: A11yState::default(),
        actions: vec![A11yAction::Click, A11yAction::Focus],
        relations: A11yRelations::default(),
        bounds: Rect::new(10.0, 20.0, 100.0, 50.0),
        parent: None,
        children: vec![],
        scene_node: None,
    };

    assert_eq!(node.role, Role::Button);
    match &node.name {
        AccessibleName::Text(text) => assert_eq!(text, "Click me"),
        _ => panic!("Expected Text name"),
    }
    assert_eq!(node.description, Some("Submit form".into()));
    assert_eq!(node.actions.len(), 2);
    assert!(node.actions.contains(&A11yAction::Click));
}

#[test]
fn test_checkbox_states() {
    let mut state = A11yState::default();
    assert_eq!(state.checked, None);
    assert!(!state.disabled);
    assert!(!state.focused);

    state.checked = Some(CheckedState::Checked);
    state.disabled = true;

    assert_eq!(state.checked, Some(CheckedState::Checked));
    assert!(state.disabled);
}

#[test]
fn test_indeterminate_checkbox() {
    let state = A11yState {
        checked: Some(CheckedState::Mixed),
        ..Default::default()
    };

    assert_eq!(state.checked, Some(CheckedState::Mixed));
}

#[test]
fn test_heading_levels() {
    let h1 = Role::Heading { level: 1 };
    let h2 = Role::Heading { level: 2 };
    let h6 = Role::Heading { level: 6 };

    assert_eq!(h1, Role::Heading { level: 1 });
    assert_ne!(h1, h2);
    assert_eq!(h6, Role::Heading { level: 6 });
}

#[test]
fn test_accessible_name_variants() {
    let direct = AccessibleName::Text("Button".into());
    let computed = AccessibleName::ComputedFromChildren;

    match direct {
        AccessibleName::Text(text) => assert_eq!(text, "Button"),
        _ => panic!("Expected Text variant"),
    }

    assert!(matches!(computed, AccessibleName::ComputedFromChildren));
}

#[test]
fn test_relations_labelled_by() {
    let mut relations = A11yRelations::default();
    assert!(relations.labelled_by.is_empty());

    // We'll implement A11yId properly in node.rs
    // For now, just test the structure
    assert!(relations.described_by.is_empty());
    assert!(relations.controls.is_empty());
}

#[test]
fn test_state_combinations() {
    let state = A11yState {
        checked: None,
        expanded: Some(true),
        disabled: false,
        focused: true,
        selected: false,
        hidden: false,
        readonly: false,
        required: true,
        invalid: false,
    };

    assert_eq!(state.expanded, Some(true));
    assert!(state.focused);
    assert!(state.required);
    assert!(!state.disabled);
}

#[test]
fn test_all_actions() {
    let actions = vec![
        A11yAction::Click,
        A11yAction::Focus,
        A11yAction::Expand,
        A11yAction::Collapse,
        A11yAction::Check,
        A11yAction::Uncheck,
        A11yAction::Select,
        A11yAction::Increment,
        A11yAction::Decrement,
        A11yAction::ShowContextMenu,
    ];

    assert_eq!(actions.len(), 10);
    assert!(actions.contains(&A11yAction::Click));
    assert!(actions.contains(&A11yAction::Increment));
}

#[test]
fn test_node_default() {
    let node = A11yNode::default();

    assert_eq!(node.role, Role::Group); // Default to generic group
    assert!(matches!(
        node.name,
        AccessibleName::ComputedFromChildren
    ));
    assert!(node.description.is_none());
    assert!(node.actions.is_empty());
    assert!(node.children.is_empty());
}
