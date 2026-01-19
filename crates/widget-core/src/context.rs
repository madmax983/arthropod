//! Widget build context
//!
//! Provides API for widgets to build scene nodes and configure components.

use render_engine::{Scene, NodeId, NodeContent, SceneNode};
use layout_engine::{FlexStyle, FlexDirection};
use arthropod_ecs::FrameworkContext;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use glam::Vec4;
use flux_state::ReadSignal;
use crate::validation::Validator;

use flux_state::WriteSignal;
use crate::form::{FormData, SubmitCallback};

/// Text input state for a node
#[derive(Clone)]
pub struct TextInputState {
    pub read_signal: ReadSignal<String>,
    pub write_signal: WriteSignal<String>,
    pub cursor_position: usize,
    pub readonly: bool,
    pub max_length: Option<usize>,
}

/// Validation state for a node
pub struct ValidationState {
    pub validator: Validator,
    pub error: Option<String>,
}

/// Form state for tracking form fields and validation
pub struct FormState {
    pub field_mapping: HashMap<String, NodeId>, // field name -> field node ID
    pub is_valid: bool,
    pub on_submit: Option<SubmitCallback>,
    pub submit_error: Option<String>,
}

/// Widget building context
///
/// Provides access to:
/// - Scene graph for hierarchy
/// - ECS context for components
/// - Layout engine for flexbox
/// - Text engine for text shaping
pub struct WidgetContext {
    scene: Scene,
    #[allow(dead_code)]
    ecs_context: Option<FrameworkContext>,
    layout_styles: HashMap<NodeId, FlexStyle>,
    hover_states: HashSet<NodeId>,
    clickables: HashMap<NodeId, Arc<dyn Fn() + Send + Sync>>,
    background_colors: HashMap<NodeId, Vec4>,
    text_input_states: HashMap<NodeId, TextInputState>,
    focused_node: Option<NodeId>,
    validators: HashMap<NodeId, ValidationState>,
    placeholders: HashSet<NodeId>,
    form_states: HashMap<NodeId, FormState>,
}

impl WidgetContext {
    /// Create a new widget context (for testing)
    pub fn new_test() -> Self {
        Self {
            scene: Scene::new(),
            ecs_context: None,
            layout_styles: HashMap::new(),
            hover_states: HashSet::new(),
            clickables: HashMap::new(),
            background_colors: HashMap::new(),
            text_input_states: HashMap::new(),
            focused_node: None,
            validators: HashMap::new(),
            placeholders: HashSet::new(),
            form_states: HashMap::new(),
        }
    }

    /// Get the scene
    pub fn scene(&self) -> &Scene {
        &self.scene
    }

    /// Get mutable scene
    pub fn scene_mut(&mut self) -> &mut Scene {
        &mut self.scene
    }

    /// Create a new scene node
    pub fn create_node(&mut self, parent: NodeId, content: NodeContent) -> NodeId {
        let node = SceneNode::new(content);
        self.scene.add_node(parent, node)
    }

    /// Get the root node ID
    pub fn root(&self) -> NodeId {
        self.scene.root()
    }

    /// Check if node has layout component
    pub fn has_layout_node(&self, _node_id: NodeId) -> bool {
        // TODO: Query ECS for LayoutNode component
        true // For now, assume all nodes have layout
    }

    /// Get layout style for node
    pub fn get_layout_style(&self, node_id: NodeId) -> Option<FlexStyle> {
        self.layout_styles.get(&node_id).cloned()
    }

    /// Check if node is a text node
    pub fn is_text_node(&self, node_id: NodeId) -> bool {
        if let Some(node) = self.scene.get_node(node_id) {
            matches!(node.content, NodeContent::Text { .. })
        } else {
            false
        }
    }

    /// Check if node has reactive text component
    pub fn has_reactive_text(&self, _node_id: NodeId) -> bool {
        // TODO: Query ECS for ReactiveText component
        true // For now, assume reactive nodes exist
    }

    /// Set layout style for a node
    pub fn set_layout_style(&mut self, node_id: NodeId, style: FlexStyle) {
        self.layout_styles.insert(node_id, style);
        // TODO: Add LayoutNode component to ECS entity
    }

    /// Re-parent a node from old parent to new parent
    pub fn reparent_node(&mut self, child_id: NodeId, old_parent: NodeId, new_parent: NodeId) {
        self.scene.reparent_node(child_id, old_parent, new_parent);
    }

    /// Add hover state tracking to a node
    pub fn add_hover_state(&mut self, node_id: NodeId) {
        self.hover_states.insert(node_id);
    }

    /// Check if node has hover state
    pub fn has_hover_state(&self, node_id: NodeId) -> bool {
        self.hover_states.contains(&node_id)
    }

    /// Add clickable component to a node
    pub fn add_clickable(&mut self, node_id: NodeId, callback: Arc<dyn Fn() + Send + Sync>) {
        self.clickables.insert(node_id, callback);
    }

    /// Check if node is clickable
    pub fn has_clickable(&self, node_id: NodeId) -> bool {
        self.clickables.contains_key(&node_id)
    }

    /// Set background color for a node
    pub fn set_background_color(&mut self, node_id: NodeId, color: Vec4) {
        self.background_colors.insert(node_id, color);
    }

    /// Check if node has background color
    pub fn has_background_color(&self, node_id: NodeId) -> bool {
        self.background_colors.contains_key(&node_id)
    }

    /// Get background color for a node
    pub fn get_background_color(&self, node_id: NodeId) -> Option<Vec4> {
        self.background_colors.get(&node_id).copied()
    }

    /// Simulate click on a node (for testing)
    pub fn trigger_click(&mut self, node_id: NodeId) {
        if let Some(callback) = self.clickables.get(&node_id) {
            callback();
        }
    }

    /// Simulate hover on a node (for testing)
    pub fn trigger_hover(&mut self, node_id: NodeId, hovered: bool) {
        if hovered {
            self.hover_states.insert(node_id);
        } else {
            self.hover_states.remove(&node_id);
        }
    }

    /// Add text input state to a node
    pub fn add_text_input_state(
        &mut self,
        node_id: NodeId,
        read_signal: ReadSignal<String>,
        write_signal: WriteSignal<String>,
        readonly: bool,
        max_length: Option<usize>,
    ) {
        let cursor_position = read_signal.get_untracked().len();
        self.text_input_states.insert(node_id, TextInputState {
            read_signal,
            write_signal,
            cursor_position,
            readonly,
            max_length,
        });
    }

    /// Focus a node
    pub fn focus_node(&mut self, node_id: NodeId) {
        self.focused_node = Some(node_id);
    }

    /// Check if node is focused
    pub fn is_focused(&self, node_id: NodeId) -> bool {
        self.focused_node == Some(node_id)
    }

    /// Blur a node
    pub fn blur_node(&mut self, node_id: NodeId) {
        if self.focused_node == Some(node_id) {
            self.focused_node = None;
        }
    }

    /// Get cursor position for a text input
    pub fn get_cursor_position(&self, node_id: NodeId) -> Option<usize> {
        self.text_input_states.get(&node_id).map(|state| state.cursor_position)
    }

    /// Send a character to focused input
    pub fn send_char(&mut self, c: char) {
        if let Some(focused_id) = self.focused_node {
            if let Some(state) = self.text_input_states.get_mut(&focused_id) {
                if state.readonly {
                    return;
                }

                let mut current_value = state.read_signal.get_untracked();

                // Check max_length
                if let Some(max_len) = state.max_length {
                    if current_value.len() >= max_len {
                        return;
                    }
                }

                // Insert character at cursor position
                current_value.insert(state.cursor_position, c);
                state.cursor_position += 1;

                // Update signal
                state.write_signal.set(current_value);
            }
        }
    }

    /// Send backspace to focused input
    pub fn send_backspace(&mut self) {
        if let Some(focused_id) = self.focused_node {
            if let Some(state) = self.text_input_states.get_mut(&focused_id) {
                if state.readonly {
                    return;
                }

                if state.cursor_position > 0 {
                    let mut current_value = state.read_signal.get_untracked();
                    current_value.remove(state.cursor_position - 1);
                    state.cursor_position -= 1;

                    // Update signal
                    state.write_signal.set(current_value);
                }
            }
        }
    }

    /// Send delete to focused input
    pub fn send_delete(&mut self) {
        if let Some(focused_id) = self.focused_node {
            if let Some(state) = self.text_input_states.get_mut(&focused_id) {
                if state.readonly {
                    return;
                }

                let current_value = state.read_signal.get_untracked();
                if state.cursor_position < current_value.len() {
                    let mut new_value = current_value;
                    new_value.remove(state.cursor_position);

                    // Update signal
                    state.write_signal.set(new_value);
                }
            }
        }
    }

    /// Send left arrow key to focused input
    pub fn send_key_left(&mut self) {
        if let Some(focused_id) = self.focused_node {
            if let Some(state) = self.text_input_states.get_mut(&focused_id) {
                if state.cursor_position > 0 {
                    state.cursor_position -= 1;
                }
            }
        }
    }

    /// Send right arrow key to focused input
    pub fn send_key_right(&mut self) {
        if let Some(focused_id) = self.focused_node {
            if let Some(state) = self.text_input_states.get_mut(&focused_id) {
                let current_value = state.read_signal.get_untracked();
                if state.cursor_position < current_value.len() {
                    state.cursor_position += 1;
                }
            }
        }
    }

    /// Set validator for a node
    pub fn set_validator(
        &mut self,
        node_id: NodeId,
        validator: Validator,
        initial_result: Result<(), String>,
    ) {
        self.validators.insert(node_id, ValidationState {
            validator,
            error: initial_result.err(),
        });
    }

    /// Check if node has validation error
    pub fn has_validation_error(&self, node_id: NodeId) -> bool {
        self.validators.get(&node_id)
            .and_then(|state| state.error.as_ref())
            .is_some()
    }

    /// Get validation error for a node
    pub fn get_validation_error(&self, node_id: NodeId) -> Option<String> {
        self.validators.get(&node_id)
            .and_then(|state| state.error.clone())
    }

    /// Add placeholder marker to a node
    pub fn add_placeholder(&mut self, node_id: NodeId) {
        self.placeholders.insert(node_id);
    }

    /// Check if node has placeholder
    pub fn has_placeholder(&self, node_id: NodeId) -> bool {
        self.placeholders.contains(&node_id)
    }

    /// Get current value of a text input (for testing)
    pub fn get_text_input_value(&self, node_id: NodeId) -> Option<String> {
        self.text_input_states.get(&node_id)
            .map(|state| state.read_signal.get_untracked())
    }

    /// Add form state to a node
    pub fn add_form_state(
        &mut self,
        node_id: NodeId,
        field_mapping: HashMap<String, NodeId>,
        on_submit: Option<SubmitCallback>,
    ) {
        self.form_states.insert(node_id, FormState {
            field_mapping,
            is_valid: true, // Will be updated by revalidate_form
            on_submit,
            submit_error: None,
        });
    }

    /// Check if form is valid
    pub fn is_form_valid(&self, node_id: NodeId) -> bool {
        self.form_states.get(&node_id)
            .map(|state| state.is_valid)
            .unwrap_or(true)
    }

    /// Get all field errors for a form
    pub fn get_form_field_errors(&self, node_id: NodeId) -> HashMap<String, String> {
        let mut errors = HashMap::new();

        if let Some(form_state) = self.form_states.get(&node_id) {
            for (field_name, field_node_id) in &form_state.field_mapping {
                if let Some(error) = self.get_validation_error(*field_node_id) {
                    errors.insert(field_name.clone(), error);
                }
            }
        }

        errors
    }

    /// Revalidate a form (check all field validators)
    pub fn revalidate_form(&mut self, node_id: NodeId) {
        // Re-run validators on all fields with current values
        let field_node_ids: Vec<NodeId> = if let Some(form_state) = self.form_states.get(&node_id) {
            form_state.field_mapping.values().copied().collect()
        } else {
            return;
        };

        // Collect current values first to avoid borrowing issues
        let mut field_values = HashMap::new();
        for field_node_id in &field_node_ids {
            if let Some(current_value) = self.get_text_input_value(*field_node_id) {
                field_values.insert(*field_node_id, current_value);
            }
        }

        // Re-run validation for each field
        for (field_node_id, current_value) in field_values {
            if let Some(validator_state) = self.validators.get_mut(&field_node_id) {
                // Run validator on current value
                let result = (validator_state.validator)(&current_value);
                validator_state.error = result.err();
            }
        }

        // Get field errors after re-validation
        let field_errors = self.get_form_field_errors(node_id);

        // Update form is_valid state
        if let Some(form_state) = self.form_states.get_mut(&node_id) {
            form_state.is_valid = field_errors.is_empty();
        }
    }

    /// Trigger form submission
    pub fn trigger_submit(&mut self, node_id: NodeId) {
        // Revalidate first
        self.revalidate_form(node_id);

        // Only submit if valid
        if !self.is_form_valid(node_id) {
            return;
        }

        // Collect form data
        let mut form_data = FormData::new();

        if let Some(form_state) = self.form_states.get(&node_id) {
            for (field_name, field_node_id) in &form_state.field_mapping {
                if let Some(value) = self.get_text_input_value(*field_node_id) {
                    form_data.insert(field_name.clone(), value);
                }
            }

            // Call submit callback
            if let Some(callback) = &form_state.on_submit {
                let result = callback(form_data);

                // Store submit error if any
                if let Some(form_state_mut) = self.form_states.get_mut(&node_id) {
                    form_state_mut.submit_error = result.err();
                }
            }
        }
    }

    /// Check if form has submit error
    pub fn has_submit_error(&self, node_id: NodeId) -> bool {
        self.form_states.get(&node_id)
            .and_then(|state| state.submit_error.as_ref())
            .is_some()
    }

    /// Get submit error for a form
    pub fn get_submit_error(&self, node_id: NodeId) -> Option<String> {
        self.form_states.get(&node_id)
            .and_then(|state| state.submit_error.clone())
    }
}

/// Helper to check if a FlexStyle is a row layout
pub fn is_row_layout(style: &FlexStyle) -> bool {
    style.direction == FlexDirection::Row
}
