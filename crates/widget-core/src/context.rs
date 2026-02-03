//! Widget build context
//!
//! Provides API for widgets to build scene nodes and configure components.

use crate::form::{FormData, SubmitCallback};
use crate::validation::Validator;
use flux_state::{ReadSignal, WriteSignal};
use glam::Vec4;
use indexmap::IndexMap;
use layout_engine::{FlexDirection, FlexStyle};
use render_engine::{Color, NodeContent, NodeId, Scene, SceneNode};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use theme_engine::DesignTokens;

// Re-export types to maintain API compatibility
// (These are now defined in this file)
// Note: We don't need `pub use crate::form_context::...` anymore as they are here.

/// Text input state for a node
#[derive(Clone)]
pub struct TextInputState {
    pub read_signal: ReadSignal<String>,
    pub write_signal: WriteSignal<String>,
    pub cursor_position: usize,
    pub readonly: bool,
    pub max_length: Option<usize>,
}

/// Reactive text state for a node
#[derive(Clone)]
pub struct ReactiveTextState {
    pub read_signal: ReadSignal<String>,
}

/// Reactive color state for a node
#[derive(Clone)]
pub struct ReactiveColorState {
    pub read_signal: ReadSignal<Color>,
}

/// Validation state for a node
#[derive(Clone)]
pub struct ValidationState {
    pub validator: Validator,
    pub error: Option<String>,
}

/// Form state for tracking form fields and validation
#[derive(Clone)]
pub struct FormState {
    pub field_mapping: IndexMap<String, NodeId>, // field name -> field node ID
    pub is_valid: bool,
    pub on_submit: Option<SubmitCallback>,
    pub submit_error: Option<String>,
}

/// Convert a character index to a byte index in a string.
fn char_idx_to_byte_idx(s: &str, char_idx: usize) -> Option<usize> {
    s.char_indices()
        .nth(char_idx)
        .map(|(byte_idx, _)| byte_idx)
        .or_else(|| {
            // If char_idx equals char count, return the string length
            // (valid insertion point at the end)
            if char_idx == s.chars().count() {
                Some(s.len())
            } else {
                None
            }
        })
}

/// Widget building context
///
/// Provides build-time API for widgets to construct scene nodes and configure
/// behavioral components. After building, use `apply_to_ecs()` to transfer
/// the accumulated widget state (layout, clickables, validators, etc.) to
/// ECS components.
///
/// This is a build-time context only - it accumulates widget state
/// during widget construction, which is then transferred to ECS components for
/// runtime use.
pub struct WidgetContext {
    scene: Scene,

    // Layout
    pub(crate) layout_styles: HashMap<NodeId, FlexStyle>,

    // Interaction
    pub(crate) hover_states: HashSet<NodeId>,
    pub(crate) clickables: HashMap<NodeId, Arc<dyn Fn() + Send + Sync>>,

    // Decoration
    pub(crate) background_colors: HashMap<NodeId, Vec4>,

    // Input
    pub(crate) text_input_states: IndexMap<NodeId, TextInputState>,
    pub(crate) reactive_text_states: HashMap<NodeId, ReactiveTextState>,
    pub(crate) reactive_color_states: HashMap<NodeId, ReactiveColorState>,
    pub(crate) focused_node: Option<NodeId>,
    pub(crate) placeholders: HashSet<NodeId>,

    // Form
    pub(crate) validators: HashMap<NodeId, ValidationState>,
    pub(crate) form_states: HashMap<NodeId, FormState>,

    /// Design tokens for theming (optional for backwards compatibility)
    design_tokens: Option<DesignTokens>,
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
            text_input_states: IndexMap::new(),
            reactive_text_states: HashMap::new(),
            reactive_color_states: HashMap::new(),
            focused_node: None,
            placeholders: HashSet::new(),
            validators: HashMap::new(),
            form_states: HashMap::new(),
            design_tokens: None,
        }
    }

    /// Set design tokens for theming
    ///
    /// When set, widgets will use these tokens for colors instead of hardcoded values.
    /// Typically called by the app layer after querying SystemTheme.
    pub fn set_design_tokens(&mut self, tokens: DesignTokens) {
        self.design_tokens = Some(tokens);
    }

    /// Get design tokens if set
    pub fn design_tokens(&self) -> Option<&DesignTokens> {
        self.design_tokens.as_ref()
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
    pub fn has_reactive_text(&self, node_id: NodeId) -> bool {
        self.reactive_text_states.contains_key(&node_id)
    }

    /// Get the text content of a node if it has any
    pub fn get_text(&self, node_id: NodeId) -> Option<String> {
        // Check reactive state first
        if let Some(state) = self.reactive_text_states.get(&node_id) {
            return Some(state.read_signal.get_untracked());
        }

        // Fallback to static content in node
        if let Some(node) = self.scene.get_node(node_id) {
            if let NodeContent::Text { text, .. } = &node.content {
                return Some(text.clone());
            }
        }

        None
    }

    /// Set layout style for a node
    pub fn set_layout_style(&mut self, node_id: NodeId, style: FlexStyle) {
        self.layout_styles.insert(node_id, style);
    }

    /// Re-parent a node from old parent to new parent
    pub fn reparent_node(&mut self, child_id: NodeId, old_parent: NodeId, new_parent: NodeId) {
        self.scene.reparent_node(child_id, old_parent, new_parent);
    }

    /// Re-parent a node to a new parent (finds and removes from current parent)
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

    /// Add reactive text state to a node
    pub fn add_reactive_text_state(&mut self, node_id: NodeId, read_signal: ReadSignal<String>) {
        self.reactive_text_states
            .insert(node_id, ReactiveTextState { read_signal });
    }

    /// Add reactive color state to a node
    pub fn add_reactive_color_state(&mut self, node_id: NodeId, read_signal: ReadSignal<Color>) {
        self.reactive_color_states
            .insert(node_id, ReactiveColorState { read_signal });
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
        let cursor_position = read_signal.get_untracked().chars().count();
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

                if let Some(max_len) = state.max_length {
                    if current_value.chars().count() >= max_len {
                        return;
                    }
                }

                if let Some(byte_idx) = char_idx_to_byte_idx(&current_value, state.cursor_position)
                {
                    current_value.insert(byte_idx, c);
                    state.cursor_position += 1;
                    state.write_signal.set(current_value);
                }
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

                    if let Some(byte_idx) =
                        char_idx_to_byte_idx(&current_value, state.cursor_position - 1)
                    {
                        current_value.remove(byte_idx);
                        state.cursor_position -= 1;
                        state.write_signal.set(current_value);
                    }
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
                let char_count = current_value.chars().count();

                if state.cursor_position < char_count {
                    let mut new_value = current_value;

                    if let Some(byte_idx) = char_idx_to_byte_idx(&new_value, state.cursor_position)
                    {
                        new_value.remove(byte_idx);
                        state.write_signal.set(new_value);
                    }
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
                let char_count = current_value.chars().count();

                if state.cursor_position < char_count {
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
        field_mapping: IndexMap<String, NodeId>,
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
            if let Some(state) = self.text_input_states.get(field_node_id) {
                field_values.insert(*field_node_id, state.read_signal.get_untracked());
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

        // Collect form data and callback
        let mut form_data = FormData::new();
        let mut callback_opt = None;

        if let Some(form_state) = self.form_states.get(&node_id) {
            callback_opt = form_state.on_submit.clone();

            for (field_name, field_node_id) in &form_state.field_mapping {
                if let Some(state) = self.text_input_states.get(field_node_id) {
                    form_data.insert(field_name.clone(), state.read_signal.get_untracked());
                }
            }
        }

        // Call submit callback
        if let Some(callback) = callback_opt {
            let result = callback(form_data);

            // Store submit error if any
            if let Some(form_state_mut) = self.form_states.get_mut(&node_id) {
                form_state_mut.submit_error = result.err();
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
    pub fn text_input_states(&self) -> &indexmap::IndexMap<NodeId, TextInputState> {
        &self.text_input_states
    }

    /// Get all reactive text states (for app-shell integration)
    pub fn reactive_text_states(&self) -> &HashMap<NodeId, ReactiveTextState> {
        &self.reactive_text_states
    }

    /// Get all reactive color states (for app-shell integration)
    pub fn reactive_color_states(&self) -> &HashMap<NodeId, ReactiveColorState> {
        &self.reactive_color_states
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

    // Helper for testing
    fn create_text_input_node(ctx: &mut WidgetContext) -> NodeId {
        let runtime = Runtime::new();
        let signal = Signal::new(runtime, String::new());
        let (read, write) = signal.split();
        let node_id = ctx.create_node(
            ctx.root(),
            NodeContent::Rect {
                color: Color::WHITE,
            },
        );
        ctx.add_text_input_state(node_id, read, write, false, None);
        node_id
    }

    #[test]
    fn test_char_idx_to_byte_idx_helper() {
        // ASCII
        assert_eq!(char_idx_to_byte_idx("abc", 0), Some(0));
        assert_eq!(char_idx_to_byte_idx("abc", 1), Some(1));
        assert_eq!(char_idx_to_byte_idx("abc", 2), Some(2));
        assert_eq!(char_idx_to_byte_idx("abc", 3), Some(3)); // End position

        // Emoji (4 bytes each)
        assert_eq!(char_idx_to_byte_idx("😀", 0), Some(0));
        assert_eq!(char_idx_to_byte_idx("😀", 1), Some(4)); // End of 4-byte emoji

        // Mixed
        assert_eq!(char_idx_to_byte_idx("a😀b", 0), Some(0)); // 'a'
        assert_eq!(char_idx_to_byte_idx("a😀b", 1), Some(1)); // emoji start
        assert_eq!(char_idx_to_byte_idx("a😀b", 2), Some(5)); // 'b'
        assert_eq!(char_idx_to_byte_idx("a😀b", 3), Some(6)); // end

        // Out of bounds
        assert_eq!(char_idx_to_byte_idx("abc", 10), None);
        assert_eq!(char_idx_to_byte_idx("😀", 5), None);
    }

    // Existing tests follow...
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

        let node_id = ctx.create_node(
            ctx.root(),
            NodeContent::Rect {
                color: Color::WHITE,
            },
        );
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
        let node_id = ctx.create_node(
            ctx.root(),
            NodeContent::Rect {
                color: Color::WHITE,
            },
        );
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
        let node_id = ctx.create_node(
            ctx.root(),
            NodeContent::Rect {
                color: Color::WHITE,
            },
        );
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

    #[test]
    fn test_focus_next_returns_none_with_no_focusable_nodes() {
        let mut ctx = WidgetContext::new_test();
        assert_eq!(ctx.focus_next(), None);
    }

    #[test]
    fn test_focus_next_focuses_first_node_when_nothing_focused() {
        let mut ctx = WidgetContext::new_test();
        let _node1 = create_text_input_node(&mut ctx);
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

    // =========================================================================
    // Unicode Handling Tests
    // =========================================================================

    fn create_text_input_with_value(ctx: &mut WidgetContext, value: &str) -> NodeId {
        let runtime = Runtime::new();
        let signal = Signal::new(runtime, value.to_string());
        let (read, write) = signal.split();
        let node_id = ctx.create_node(
            ctx.root(),
            NodeContent::Rect {
                color: Color::WHITE,
            },
        );
        ctx.add_text_input_state(node_id, read, write, false, None);
        node_id
    }

    #[test]
    fn test_send_char_with_emoji() {
        let mut ctx = WidgetContext::new_test();
        let node_id = create_text_input_with_value(&mut ctx, "😀");
        ctx.focus_node(node_id);

        // Cursor should be at end (1 character, even though it's 4 bytes)
        assert_eq!(ctx.get_cursor_position(node_id), Some(1));

        // Insert 'a' after the emoji - should not panic
        ctx.send_char('a');

        // Value should be "😀a"
        assert_eq!(ctx.get_text_input_value(node_id), Some("😀a".to_string()));
        assert_eq!(ctx.get_cursor_position(node_id), Some(2));
    }

    #[test]
    fn test_send_char_before_emoji() {
        let mut ctx = WidgetContext::new_test();
        let node_id = create_text_input_with_value(&mut ctx, "😀");
        ctx.focus_node(node_id);

        // Move cursor to beginning
        ctx.send_key_left();
        assert_eq!(ctx.get_cursor_position(node_id), Some(0));

        // Insert 'a' before the emoji - should not panic
        ctx.send_char('a');

        // Value should be "a😀"
        assert_eq!(ctx.get_text_input_value(node_id), Some("a😀".to_string()));
        assert_eq!(ctx.get_cursor_position(node_id), Some(1));
    }

    #[test]
    fn test_backspace_emoji() {
        let mut ctx = WidgetContext::new_test();
        let node_id = create_text_input_with_value(&mut ctx, "a😀b");
        ctx.focus_node(node_id);

        // Cursor at end (3 characters)
        assert_eq!(ctx.get_cursor_position(node_id), Some(3));

        // Backspace should remove 'b'
        ctx.send_backspace();
        assert_eq!(ctx.get_text_input_value(node_id), Some("a😀".to_string()));
        assert_eq!(ctx.get_cursor_position(node_id), Some(2));

        // Backspace should remove the emoji (single operation, even though 4 bytes)
        ctx.send_backspace();
        assert_eq!(ctx.get_text_input_value(node_id), Some("a".to_string()));
        assert_eq!(ctx.get_cursor_position(node_id), Some(1));
    }

    #[test]
    fn test_delete_emoji() {
        let mut ctx = WidgetContext::new_test();
        let node_id = create_text_input_with_value(&mut ctx, "a😀b");
        ctx.focus_node(node_id);

        // Move cursor to position 1 (after 'a', before emoji)
        ctx.send_key_left(); // now at 2
        ctx.send_key_left(); // now at 1
        assert_eq!(ctx.get_cursor_position(node_id), Some(1));

        // Delete should remove the emoji
        ctx.send_delete();
        assert_eq!(ctx.get_text_input_value(node_id), Some("ab".to_string()));
        assert_eq!(ctx.get_cursor_position(node_id), Some(1));
    }

    #[test]
    fn test_cursor_movement_with_emoji() {
        let mut ctx = WidgetContext::new_test();
        let node_id = create_text_input_with_value(&mut ctx, "a😀b");
        ctx.focus_node(node_id);

        // Cursor at end (3 characters)
        assert_eq!(ctx.get_cursor_position(node_id), Some(3));

        // Move left through each character
        ctx.send_key_left();
        assert_eq!(ctx.get_cursor_position(node_id), Some(2));

        ctx.send_key_left();
        assert_eq!(ctx.get_cursor_position(node_id), Some(1));

        ctx.send_key_left();
        assert_eq!(ctx.get_cursor_position(node_id), Some(0));

        // Can't go past beginning
        ctx.send_key_left();
        assert_eq!(ctx.get_cursor_position(node_id), Some(0));

        // Move right through each character
        ctx.send_key_right();
        assert_eq!(ctx.get_cursor_position(node_id), Some(1));

        ctx.send_key_right();
        assert_eq!(ctx.get_cursor_position(node_id), Some(2));

        ctx.send_key_right();
        assert_eq!(ctx.get_cursor_position(node_id), Some(3));

        // Can't go past end
        ctx.send_key_right();
        assert_eq!(ctx.get_cursor_position(node_id), Some(3));
    }

    #[test]
    fn test_max_length_with_emoji() {
        let mut ctx = WidgetContext::new_test();
        let runtime = Runtime::new();
        let signal = Signal::new(runtime, String::new());
        let (read, write) = signal.split();
        let node_id = ctx.create_node(
            ctx.root(),
            NodeContent::Rect {
                color: Color::WHITE,
            },
        );

        // Max length of 3 characters
        ctx.add_text_input_state(node_id, read, write, false, Some(3));
        ctx.focus_node(node_id);

        // Add 3 emojis (12 bytes, but only 3 characters)
        ctx.send_char('😀');
        ctx.send_char('😁');
        ctx.send_char('😂');

        assert_eq!(
            ctx.get_text_input_value(node_id),
            Some("😀😁😂".to_string())
        );

        // 4th character should be rejected (max_length is character count, not bytes)
        ctx.send_char('x');
        assert_eq!(
            ctx.get_text_input_value(node_id),
            Some("😀😁😂".to_string())
        );
    }

    #[test]
    fn test_get_text_static() {
        let mut ctx = WidgetContext::new_test();
        let node_id = ctx.create_node(
            ctx.root(),
            NodeContent::Text {
                text: "Hello".to_string(),
                font_size: 16.0,
                color: Color::BLACK,
            },
        );

        assert_eq!(ctx.get_text(node_id), Some("Hello".to_string()));
    }

    #[test]
    fn test_get_text_reactive() {
        let mut ctx = WidgetContext::new_test();
        let runtime = Runtime::new();
        let signal = Signal::new(runtime, "Initial".to_string());
        let (read, write) = signal.split();

        // Simulate what Text widget does:
        // 1. Get initial value
        let initial = read.get_untracked();

        // 2. Create node
        let node_id = ctx.create_node(
            ctx.root(),
            NodeContent::Text {
                text: initial,
                font_size: 16.0,
                color: Color::BLACK,
            },
        );

        // 3. Register reactive state
        ctx.add_reactive_text_state(node_id, read);

        // Initial check
        assert_eq!(ctx.get_text(node_id), Some("Initial".to_string()));

        // Update signal
        write.set("Updated".to_string());

        // Should reflect update
        assert_eq!(ctx.get_text(node_id), Some("Updated".to_string()));
    }
}
