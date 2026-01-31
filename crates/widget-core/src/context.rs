//! Widget build context
//!
//! Provides API for widgets to build scene nodes and configure components.

use crate::decoration_context::DecorationContext;
use crate::form::SubmitCallback;
use crate::form_context::FormContext;
use crate::input_context::InputContext;
use crate::interaction_context::InteractionContext;
use crate::layout_context::LayoutContext;
use crate::validation::Validator;
use flux_state::{ReadSignal, WriteSignal};
use glam::Vec4;
use layout_engine::{FlexDirection, FlexStyle};
use render_engine::{NodeContent, NodeId, Scene, SceneNode};
use std::collections::HashMap;
use std::sync::Arc;
use theme_engine::DesignTokens;

// Re-export types from sub-contexts to maintain API compatibility
pub use crate::form_context::{FormState, ValidationState};
pub use crate::input_context::{ReactiveTextState, TextInputState};

/// Widget building context
///
/// Provides build-time API for widgets to construct scene nodes and configure
/// behavioral components. After building, use `apply_to_ecs()` to transfer
/// the accumulated widget state (layout, clickables, validators, etc.) to
/// ECS components.
///
/// This is a build-time context only - it accumulates widget state in sub-contexts
/// during widget construction, which is then transferred to ECS components for
/// runtime use.
pub struct WidgetContext {
    scene: Scene,

    // Sub-contexts
    layout_context: LayoutContext,
    interaction_context: InteractionContext,
    decoration_context: DecorationContext,
    input_context: InputContext,
    form_context: FormContext,

    /// Design tokens for theming (optional for backwards compatibility)
    design_tokens: Option<DesignTokens>,
}

impl WidgetContext {
    /// Create a new widget context (for testing and widget building)
    pub fn new_test() -> Self {
        Self {
            scene: Scene::new(),
            layout_context: LayoutContext::new(),
            interaction_context: InteractionContext::new(),
            decoration_context: DecorationContext::new(),
            input_context: InputContext::new(),
            form_context: FormContext::new(),
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
        self.layout_context.get_style(node_id)
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
        self.input_context
            .reactive_text_states
            .contains_key(&node_id)
    }

    /// Get the text content of a node if it has any
    pub fn get_text(&self, node_id: NodeId) -> Option<String> {
        // Check reactive state first
        if let Some(state) = self.input_context.reactive_text_states.get(&node_id) {
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
    ///
    /// Layout styles are accumulated during widget building and transferred to
    /// ECS components via `apply_to_ecs()` after building is complete.
    pub fn set_layout_style(&mut self, node_id: NodeId, style: FlexStyle) {
        self.layout_context.set_style(node_id, style);
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
        self.interaction_context.add_hover_state(node_id);
    }

    /// Check if node has hover state
    pub fn has_hover_state(&self, node_id: NodeId) -> bool {
        self.interaction_context.has_hover_state(node_id)
    }

    /// Add clickable component to a node
    pub fn add_clickable(&mut self, node_id: NodeId, callback: Arc<dyn Fn() + Send + Sync>) {
        self.interaction_context.add_clickable(node_id, callback);
    }

    /// Check if node is clickable
    pub fn has_clickable(&self, node_id: NodeId) -> bool {
        self.interaction_context.has_clickable(node_id)
    }

    /// Set background color for a node
    pub fn set_background_color(&mut self, node_id: NodeId, color: Vec4) {
        self.decoration_context.set_background_color(node_id, color);
    }

    /// Check if node has background color
    pub fn has_background_color(&self, node_id: NodeId) -> bool {
        self.decoration_context.has_background_color(node_id)
    }

    /// Get background color for a node
    pub fn get_background_color(&self, node_id: NodeId) -> Option<Vec4> {
        self.decoration_context.get_background_color(node_id)
    }

    /// Simulate click on a node (for testing)
    pub fn trigger_click(&mut self, node_id: NodeId) {
        self.interaction_context.trigger_click(node_id);
    }

    /// Simulate hover on a node (for testing)
    pub fn trigger_hover(&mut self, node_id: NodeId, hovered: bool) {
        self.interaction_context.trigger_hover(node_id, hovered);
    }

    /// Add reactive text state to a node
    pub fn add_reactive_text_state(&mut self, node_id: NodeId, read_signal: ReadSignal<String>) {
        self.input_context
            .add_reactive_text_state(node_id, read_signal);
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
        self.input_context.add_text_input_state(
            node_id,
            read_signal,
            write_signal,
            readonly,
            max_length,
        );
    }

    /// Focus a node
    pub fn focus_node(&mut self, node_id: NodeId) {
        self.input_context.focus_node(node_id);
    }

    /// Check if node is focused
    pub fn is_focused(&self, node_id: NodeId) -> bool {
        self.input_context.is_focused(node_id)
    }

    /// Blur a node
    pub fn blur_node(&mut self, node_id: NodeId) {
        self.input_context.blur_node(node_id);
    }

    /// Get cursor position for a text input
    pub fn get_cursor_position(&self, node_id: NodeId) -> Option<usize> {
        self.input_context.get_cursor_position(node_id)
    }

    /// Send a character to focused input
    pub fn send_char(&mut self, c: char) {
        self.input_context.send_char(c);
    }

    /// Send backspace to focused input
    pub fn send_backspace(&mut self) {
        self.input_context.send_backspace();
    }

    /// Send delete to focused input
    pub fn send_delete(&mut self) {
        self.input_context.send_delete();
    }

    /// Send left arrow key to focused input
    pub fn send_key_left(&mut self) {
        self.input_context.send_key_left();
    }

    /// Send right arrow key to focused input
    pub fn send_key_right(&mut self) {
        self.input_context.send_key_right();
    }

    /// Set validator for a node
    pub fn set_validator(
        &mut self,
        node_id: NodeId,
        validator: Validator,
        initial_result: Result<(), String>,
    ) {
        self.form_context
            .set_validator(node_id, validator, initial_result);
    }

    /// Check if node has validation error
    pub fn has_validation_error(&self, node_id: NodeId) -> bool {
        self.form_context.has_validation_error(node_id)
    }

    /// Get validation error for a node
    pub fn get_validation_error(&self, node_id: NodeId) -> Option<String> {
        self.form_context.get_validation_error(node_id)
    }

    /// Add placeholder marker to a node
    pub fn add_placeholder(&mut self, node_id: NodeId) {
        self.input_context.add_placeholder(node_id);
    }

    /// Check if node has placeholder
    pub fn has_placeholder(&self, node_id: NodeId) -> bool {
        self.input_context.has_placeholder(node_id)
    }

    /// Get current value of a text input (for testing)
    pub fn get_text_input_value(&self, node_id: NodeId) -> Option<String> {
        self.input_context.get_text_input_value(node_id)
    }

    /// Add form state to a node
    pub fn add_form_state(
        &mut self,
        node_id: NodeId,
        field_mapping: HashMap<String, NodeId>,
        on_submit: Option<SubmitCallback>,
    ) {
        self.form_context
            .add_form_state(node_id, field_mapping, on_submit);
    }

    /// Check if form is valid
    pub fn is_form_valid(&self, node_id: NodeId) -> bool {
        self.form_context.is_form_valid(node_id)
    }

    /// Get all field errors for a form
    pub fn get_form_field_errors(&self, node_id: NodeId) -> HashMap<String, String> {
        self.form_context.get_form_field_errors(node_id)
    }

    /// Get form state for a form node
    pub fn get_form_state(&self, node_id: NodeId) -> Option<&FormState> {
        self.form_context.get_form_state(node_id)
    }

    /// Revalidate a form (check all field validators)
    pub fn revalidate_form(&mut self, node_id: NodeId) {
        let input_context = &self.input_context;
        self.form_context
            .revalidate_form(node_id, |id| input_context.get_text_input_value(id));
    }

    /// Trigger form submission
    pub fn trigger_submit(&mut self, node_id: NodeId) {
        let input_context = &self.input_context;
        self.form_context
            .trigger_submit(node_id, |id| input_context.get_text_input_value(id));
    }

    /// Check if form has submit error
    pub fn has_submit_error(&self, node_id: NodeId) -> bool {
        self.form_context.has_submit_error(node_id)
    }

    /// Get submit error for a form
    pub fn get_submit_error(&self, node_id: NodeId) -> Option<String> {
        self.form_context.get_submit_error(node_id)
    }

    /// Check if node is a text input
    pub fn is_text_input(&self, node_id: NodeId) -> bool {
        self.input_context.is_text_input(node_id)
    }

    /// Check if node is clickable (alias for has_clickable)
    pub fn is_clickable(&self, node_id: NodeId) -> bool {
        self.has_clickable(node_id)
    }

    /// Get the currently focused node
    pub fn focused_node(&self) -> Option<NodeId> {
        self.input_context.focused_node
    }

    /// Focus the next focusable node (Tab navigation).
    pub fn focus_next(&mut self) -> Option<NodeId> {
        self.input_context.focus_next()
    }

    /// Focus the previous focusable node (Shift+Tab navigation).
    pub fn focus_prev(&mut self) -> Option<NodeId> {
        self.input_context.focus_prev()
    }

    // =========================================================================
    // Accessor methods for app-shell integration
    // These allow the app layer to read accumulated widget state and transfer
    // it to ECS components. WidgetContext itself is ECS-agnostic.
    // =========================================================================

    /// Get all layout styles (for app-shell integration)
    pub fn layout_styles(&self) -> &HashMap<NodeId, FlexStyle> {
        self.layout_context.get_all_styles()
    }

    /// Get all clickables (for app-shell integration)
    pub fn clickables(&self) -> &HashMap<NodeId, Arc<dyn Fn() + Send + Sync>> {
        self.interaction_context.get_clickables()
    }

    /// Get all background colors (for app-shell integration)
    pub fn background_colors(&self) -> &HashMap<NodeId, Vec4> {
        self.decoration_context.get_background_colors()
    }

    /// Get all text input states (for app-shell integration)
    pub fn text_input_states(&self) -> &indexmap::IndexMap<NodeId, TextInputState> {
        &self.input_context.text_input_states
    }

    /// Get all reactive text states (for app-shell integration)
    pub fn reactive_text_states(&self) -> &HashMap<NodeId, ReactiveTextState> {
        &self.input_context.reactive_text_states
    }

    /// Get all validators (for app-shell integration)
    pub fn validators(&self) -> &HashMap<NodeId, ValidationState> {
        &self.form_context.validators
    }

    /// Get all form states (for app-shell integration)
    pub fn form_states(&self) -> &HashMap<NodeId, FormState> {
        &self.form_context.form_states
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
