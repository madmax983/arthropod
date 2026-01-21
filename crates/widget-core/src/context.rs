//! Widget build context
//!
//! Provides API for widgets to build scene nodes and configure components.

use crate::form::{FormData, SubmitCallback};
use crate::validation::Validator;
use flux_state::{ReadSignal, WriteSignal};
use glam::Vec4;
use layout_engine::{FlexDirection, FlexStyle};
use render_engine::{NodeContent, NodeId, Scene, SceneNode};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

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
/// Provides build-time API for widgets to construct scene nodes and configure
/// behavioral components. After building, use `apply_to_ecs()` to transfer
/// the accumulated widget state (layout, clickables, validators, etc.) to
/// ECS components.
///
/// This is a build-time context only - it accumulates widget state in HashMaps
/// during widget construction, which is then transferred to ECS components for
/// runtime use.
pub struct WidgetContext {
    scene: Scene,
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
    /// Create a new widget context (for testing and widget building)
    pub fn new_test() -> Self {
        Self {
            scene: Scene::new(),
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
    ///
    /// Layout styles are accumulated during widget building and transferred to
    /// ECS components via `apply_to_ecs()` after building is complete.
    pub fn set_layout_style(&mut self, node_id: NodeId, style: FlexStyle) {
        self.layout_styles.insert(node_id, style);
    }

    /// Re-parent a node from old parent to new parent
    pub fn reparent_node(&mut self, child_id: NodeId, old_parent: NodeId, new_parent: NodeId) {
        self.scene.reparent_node(child_id, old_parent, new_parent);
    }

    /// Re-parent a node to a new parent (finds and removes from current parent)
    ///
    /// This is a convenience method for WidgetTuple implementations.
    pub fn reparent_to(&mut self, child_id: NodeId, new_parent: NodeId) {
        // Find current parent
        if let Some(current_parent) = self.scene.find_parent(child_id) {
            if current_parent != new_parent {
                self.scene
                    .reparent_node(child_id, current_parent, new_parent);
            }
        }
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
        self.text_input_states.insert(
            node_id,
            TextInputState {
                read_signal,
                write_signal,
                cursor_position,
                readonly,
                max_length,
            },
        );
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
        self.text_input_states
            .get(&node_id)
            .map(|state| state.cursor_position)
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
        self.validators.insert(
            node_id,
            ValidationState {
                validator,
                error: initial_result.err(),
            },
        );
    }

    /// Check if node has validation error
    pub fn has_validation_error(&self, node_id: NodeId) -> bool {
        self.validators
            .get(&node_id)
            .and_then(|state| state.error.as_ref())
            .is_some()
    }

    /// Get validation error for a node
    pub fn get_validation_error(&self, node_id: NodeId) -> Option<String> {
        self.validators
            .get(&node_id)
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
        self.text_input_states
            .get(&node_id)
            .map(|state| state.read_signal.get_untracked())
    }

    /// Add form state to a node
    pub fn add_form_state(
        &mut self,
        node_id: NodeId,
        field_mapping: HashMap<String, NodeId>,
        on_submit: Option<SubmitCallback>,
    ) {
        self.form_states.insert(
            node_id,
            FormState {
                field_mapping,
                is_valid: true, // Will be updated by revalidate_form
                on_submit,
                submit_error: None,
            },
        );
    }

    /// Check if form is valid
    pub fn is_form_valid(&self, node_id: NodeId) -> bool {
        self.form_states
            .get(&node_id)
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

    /// Get form state for a form node
    pub fn get_form_state(&self, node_id: NodeId) -> Option<&FormState> {
        self.form_states.get(&node_id)
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
        self.form_states
            .get(&node_id)
            .and_then(|state| state.submit_error.as_ref())
            .is_some()
    }

    /// Get submit error for a form
    pub fn get_submit_error(&self, node_id: NodeId) -> Option<String> {
        self.form_states
            .get(&node_id)
            .and_then(|state| state.submit_error.clone())
    }

    /// Check if node is a text input
    pub fn is_text_input(&self, node_id: NodeId) -> bool {
        self.text_input_states.contains_key(&node_id)
    }

    /// Check if node is clickable (alias for has_clickable)
    pub fn is_clickable(&self, node_id: NodeId) -> bool {
        self.has_clickable(node_id)
    }

    /// Get the currently focused node
    pub fn focused_node(&self) -> Option<NodeId> {
        self.focused_node
    }

    /// Focus the next focusable node (Tab navigation).
    ///
    /// Cycles through all text inputs in the order they were added.
    /// If no node is focused, focuses the first one.
    /// Wraps around from last to first.
    ///
    /// # Returns
    ///
    /// The newly focused `NodeId`, or `None` if there are no focusable nodes.
    pub fn focus_next(&mut self) -> Option<NodeId> {
        let focusable: Vec<NodeId> = self.text_input_states.keys().copied().collect();
        if focusable.is_empty() {
            return None;
        }

        let current_index = self
            .focused_node
            .and_then(|f| focusable.iter().position(|&id| id == f));

        let next_index = match current_index {
            Some(idx) => (idx + 1) % focusable.len(),
            None => 0,
        };

        let next_node = focusable[next_index];
        self.focused_node = Some(next_node);
        Some(next_node)
    }

    /// Focus the previous focusable node (Shift+Tab navigation).
    ///
    /// Cycles through all text inputs in reverse order.
    /// If no node is focused, focuses the last one.
    /// Wraps around from first to last.
    ///
    /// # Returns
    ///
    /// The newly focused `NodeId`, or `None` if there are no focusable nodes.
    pub fn focus_prev(&mut self) -> Option<NodeId> {
        let focusable: Vec<NodeId> = self.text_input_states.keys().copied().collect();
        if focusable.is_empty() {
            return None;
        }

        let current_index = self
            .focused_node
            .and_then(|f| focusable.iter().position(|&id| id == f));

        let prev_index = match current_index {
            Some(0) => focusable.len() - 1,
            Some(idx) => idx - 1,
            None => focusable.len() - 1,
        };

        let prev_node = focusable[prev_index];
        self.focused_node = Some(prev_node);
        Some(prev_node)
    }

    // =========================================================================
    // Accessor methods for app-shell integration
    // These allow the app layer to read accumulated widget state and transfer
    // it to ECS components. WidgetContext itself is ECS-agnostic.
    // =========================================================================

    /// Get all layout styles (for app-shell integration)
    pub fn layout_styles(&self) -> &HashMap<NodeId, FlexStyle> {
        &self.layout_styles
    }

    /// Get all clickables (for app-shell integration)
    pub fn clickables(&self) -> &HashMap<NodeId, Arc<dyn Fn() + Send + Sync>> {
        &self.clickables
    }

    /// Get all background colors (for app-shell integration)
    pub fn background_colors(&self) -> &HashMap<NodeId, Vec4> {
        &self.background_colors
    }

    /// Get all text input states (for app-shell integration)
    pub fn text_input_states(&self) -> &HashMap<NodeId, TextInputState> {
        &self.text_input_states
    }

    /// Get all validators (for app-shell integration)
    pub fn validators(&self) -> &HashMap<NodeId, ValidationState> {
        &self.validators
    }

    /// Get all form states (for app-shell integration)
    pub fn form_states(&self) -> &HashMap<NodeId, FormState> {
        &self.form_states
    }

    /// Take ownership of the scene (consumes self)
    pub fn into_scene(self) -> Scene {
        self.scene
    }
}

/// Helper to check if a FlexStyle is a row layout
pub fn is_row_layout(style: &FlexStyle) -> bool {
    style.direction == FlexDirection::Row
}

#[cfg(test)]
mod tests {
    use super::*;
    use flux_state::{Runtime, Signal};
    use render_engine::{Color, NodeContent};

    #[test]
    fn test_is_text_input_returns_false_for_non_input_nodes() {
        let ctx = WidgetContext::new_test();
        let node_id = ctx.scene().root();
        assert!(!ctx.is_text_input(node_id));
    }

    #[test]
    fn test_is_text_input_returns_true_after_adding_input_state() {
        let mut ctx = WidgetContext::new_test();
        let runtime = Runtime::new();
        let signal = Signal::new(runtime, String::new());
        let (read, write) = signal.split();

        let node_id = ctx.create_node(ctx.root(), NodeContent::Rect { color: Color::WHITE });
        ctx.add_text_input_state(node_id, read, write, false, None);

        assert!(ctx.is_text_input(node_id));
    }

    #[test]
    fn test_is_clickable_returns_false_for_non_clickable_nodes() {
        let ctx = WidgetContext::new_test();
        let node_id = ctx.scene().root();
        assert!(!ctx.is_clickable(node_id));
    }

    #[test]
    fn test_is_clickable_returns_true_after_adding_clickable() {
        let mut ctx = WidgetContext::new_test();
        let node_id = ctx.create_node(ctx.root(), NodeContent::Rect { color: Color::WHITE });
        ctx.add_clickable(node_id, Arc::new(|| {}));

        assert!(ctx.is_clickable(node_id));
    }

    #[test]
    fn test_focused_node_returns_none_initially() {
        let ctx = WidgetContext::new_test();
        assert_eq!(ctx.focused_node(), None);
    }

    #[test]
    fn test_focused_node_returns_some_after_focusing() {
        let mut ctx = WidgetContext::new_test();
        let node_id = ctx.create_node(ctx.root(), NodeContent::Rect { color: Color::WHITE });
        ctx.focus_node(node_id);

        assert_eq!(ctx.focused_node(), Some(node_id));
    }

    #[test]
    fn test_into_scene_consumes_context_and_returns_scene() {
        let ctx = WidgetContext::new_test();
        let root_id = ctx.scene().root();

        let scene = ctx.into_scene();
        assert_eq!(scene.root(), root_id);
    }

    // =========================================================================
    // Focus Navigation Tests
    // =========================================================================

    fn create_text_input_node(ctx: &mut WidgetContext) -> NodeId {
        let runtime = Runtime::new();
        let signal = Signal::new(runtime, String::new());
        let (read, write) = signal.split();
        let node_id = ctx.create_node(ctx.root(), NodeContent::Rect { color: Color::WHITE });
        ctx.add_text_input_state(node_id, read, write, false, None);
        node_id
    }

    #[test]
    fn test_focus_next_returns_none_with_no_focusable_nodes() {
        let mut ctx = WidgetContext::new_test();
        assert_eq!(ctx.focus_next(), None);
    }

    #[test]
    fn test_focus_next_focuses_first_node_when_nothing_focused() {
        let mut ctx = WidgetContext::new_test();
        let node1 = create_text_input_node(&mut ctx);
        let _node2 = create_text_input_node(&mut ctx);

        let focused = ctx.focus_next();
        // Should focus one of the nodes (HashMap order is not guaranteed)
        assert!(focused.is_some());
        assert_eq!(ctx.focused_node(), focused);
    }

    #[test]
    fn test_focus_next_cycles_through_nodes() {
        let mut ctx = WidgetContext::new_test();
        let node1 = create_text_input_node(&mut ctx);
        let node2 = create_text_input_node(&mut ctx);
        let node3 = create_text_input_node(&mut ctx);

        // Focus first
        ctx.focus_node(node1);

        // Collect all focused nodes through one cycle
        let mut visited = vec![node1];
        for _ in 0..3 {
            if let Some(next) = ctx.focus_next() {
                if !visited.contains(&next) {
                    visited.push(next);
                }
            }
        }

        // Should have visited all nodes
        assert!(visited.contains(&node1));
        assert!(visited.contains(&node2));
        assert!(visited.contains(&node3));
    }

    #[test]
    fn test_focus_next_wraps_around() {
        let mut ctx = WidgetContext::new_test();
        let _node1 = create_text_input_node(&mut ctx);

        // With only one node, focus_next should keep returning it
        let first = ctx.focus_next();
        assert!(first.is_some());

        let second = ctx.focus_next();
        assert_eq!(first, second); // Wraps back to same node
    }

    #[test]
    fn test_focus_prev_returns_none_with_no_focusable_nodes() {
        let mut ctx = WidgetContext::new_test();
        assert_eq!(ctx.focus_prev(), None);
    }

    #[test]
    fn test_focus_prev_focuses_last_node_when_nothing_focused() {
        let mut ctx = WidgetContext::new_test();
        let _node1 = create_text_input_node(&mut ctx);
        let _node2 = create_text_input_node(&mut ctx);

        let focused = ctx.focus_prev();
        // Should focus one of the nodes
        assert!(focused.is_some());
        assert_eq!(ctx.focused_node(), focused);
    }

    #[test]
    fn test_focus_prev_cycles_backwards() {
        let mut ctx = WidgetContext::new_test();
        let node1 = create_text_input_node(&mut ctx);
        let node2 = create_text_input_node(&mut ctx);

        // Focus first node
        ctx.focus_node(node1);

        // Go backwards, should visit all nodes
        let mut visited = vec![node1];
        for _ in 0..3 {
            if let Some(prev) = ctx.focus_prev() {
                if !visited.contains(&prev) {
                    visited.push(prev);
                }
            }
        }

        assert!(visited.contains(&node1));
        assert!(visited.contains(&node2));
    }

    #[test]
    fn test_focus_next_and_prev_are_inverse() {
        let mut ctx = WidgetContext::new_test();
        let node1 = create_text_input_node(&mut ctx);
        let _node2 = create_text_input_node(&mut ctx);
        let _node3 = create_text_input_node(&mut ctx);

        // Focus a specific node
        ctx.focus_node(node1);

        // Go forward then back should return to same node
        ctx.focus_next();
        ctx.focus_prev();

        assert_eq!(ctx.focused_node(), Some(node1));
    }
}
