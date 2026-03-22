//! Widget build context
//!
//! Provides API for widgets to build scene nodes and configure components.
//!
//! # The Builder's Toolkit
//!
//! `WidgetContext` is the primary interface passed to [`crate::Widget::build`].
//! It acts as a **temporary accumulator** during the widget tree construction phase.
//!
//! ## The Build Phase
//!
//! Arthropod constructs the UI in two phases:
//!
//! 1.  **Build Phase**: Widgets are traversed recursively. Each widget uses `WidgetContext` to:
//!     - Create [`render_engine::SceneNode`]s (via [`WidgetContext::create_node`]).
//!     - Define layout properties (via [`WidgetContext::set_layout_style`]).
//!     - Register interactions like clicks or text input.
//!     - Set up reactive bindings (connecting signals to node properties).
//!
//! 2.  **Integration Phase**: Once the build is complete, the `WidgetContext` is consumed.
//!     Its accumulated state (layout maps, event handlers, etc.) is transferred into the
//!     Arthropod ECS (Entity Component System) as components on entities.
//!
//! ## Example: Building a Custom "Counter" Widget
//!
//! This example demonstrates how to implement a custom widget that uses `WidgetContext`
//! to create a visual node, handle clicks, and update layout.
//!
//! ```
//! use widget_core::{Widget, WidgetContext};
//! use render_engine::{NodeContent, NodeId, Color, VisualStyle};
//! use layout_engine::FlexStyle;
//! use std::sync::Arc;
//!
//! struct Counter {
//!     count: i32,
//!     on_increment: Arc<dyn Fn() + Send + Sync>,
//! }
//!
//! impl Widget for Counter {
//!     fn build(&self, ctx: &mut WidgetContext) -> NodeId {
//!         // 1. Create the visual node (a blue square)
//!         let node_id = ctx.create_node(
//!             ctx.root(),
//!             NodeContent::Styled {
//!                 style: Box::new(
//!                     VisualStyle::new()
//!                         .solid_fill(Color::BLUE.as_vec4())
//!                         .text(render_engine::TextContent::new(
//!                             format!("Count: {}", self.count),
//!                             16.0
//!                         ))
//!                 )
//!             }
//!         );
//!
//!         // 2. Configure Layout (Fixed size, centered)
//!         ctx.set_layout_style(node_id, FlexStyle {
//!             width: Some(100.0),
//!             height: Some(50.0),
//!             ..Default::default()
//!         });
//!
//!         // 3. Add Interaction (Click handler)
//!         let callback = self.on_increment.clone();
//!         ctx.add_clickable(node_id, callback);
//!
//!         // 4. Return the node ID so parents can layout this widget
//!         node_id
//!     }
//! }
//! ```

use crate::form_state::{FormState, SubmitCallback};
use crate::input_state::{
    ComputedTextState, ReactiveColorState, ReactiveLayoutFlexGrowState, ReactiveLayoutWidthState,
    ReactiveTextState, TextInputState,
};
use crate::validation::Validator;
use crate::validation_state::ValidationState;
use anim_graph::timeline::Timeline;
use flux_state::{Computed, ReadSignal, WriteSignal};
use glam::Vec4;
use indexmap::IndexMap;
use layout_engine::{FlexDirection, FlexStyle};
use render_engine::{Color, NodeContent, NodeId, Scene, SceneNode};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use theme_engine::DesignTokens;

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
    pub(crate) widget_styles: HashMap<NodeId, crate::Style>,

    // Interaction
    pub(crate) hover_states: HashSet<NodeId>,
    pub(crate) clickables: HashMap<NodeId, Arc<dyn Fn() + Send + Sync>>,

    // Decoration
    pub(crate) background_colors: HashMap<NodeId, Vec4>,

    // Input
    pub(crate) text_input_states: IndexMap<NodeId, TextInputState>,
    pub(crate) reactive_text_states: HashMap<NodeId, ReactiveTextState>,
    pub(crate) computed_text_states: HashMap<NodeId, ComputedTextState>,
    pub(crate) reactive_color_states: HashMap<NodeId, ReactiveColorState>,
    pub(crate) reactive_layout_width_states: HashMap<NodeId, ReactiveLayoutWidthState>,
    pub(crate) reactive_layout_flex_grow_states: HashMap<NodeId, ReactiveLayoutFlexGrowState>,
    pub(crate) progress_bar_states: HashMap<NodeId, (ReadSignal<f32>, f32)>,
    pub(crate) frame_signal: Option<ReadSignal<u64>>,
    pub(crate) focused_node: Option<NodeId>,
    pub(crate) placeholders: HashSet<NodeId>,

    // Form
    pub(crate) validators: HashMap<NodeId, ValidationState>,
    pub(crate) form_states: HashMap<NodeId, FormState>,

    /// Design tokens for theming (optional for backwards compatibility)
    design_tokens: Option<DesignTokens>,

    /// Effects that must be kept alive for reactive synchronization
    effects: Vec<flux_state::Effect>,

    // Timelines
    pub(crate) timeline_f32_states: HashMap<NodeId, (Timeline<f32>, WriteSignal<f32>)>,
    pub(crate) timeline_color_states: HashMap<
        NodeId,
        (
            Timeline<render_engine::Color>,
            WriteSignal<render_engine::Color>,
        ),
    >,
}

impl WidgetContext {
    /// Create a new widget context with a given scene
    pub fn new(scene: Scene) -> Self {
        Self {
            scene,
            layout_styles: HashMap::new(),
            widget_styles: HashMap::new(),
            hover_states: HashSet::new(),
            clickables: HashMap::new(),
            background_colors: HashMap::new(),
            text_input_states: IndexMap::new(),
            reactive_text_states: HashMap::new(),
            computed_text_states: HashMap::new(),
            reactive_color_states: HashMap::new(),
            reactive_layout_width_states: HashMap::new(),
            reactive_layout_flex_grow_states: HashMap::new(),
            progress_bar_states: HashMap::new(),
            frame_signal: None,
            focused_node: None,
            placeholders: HashSet::new(),
            validators: HashMap::new(),
            form_states: HashMap::new(),
            design_tokens: None,
            effects: Vec::new(),
            timeline_f32_states: HashMap::new(),
            timeline_color_states: HashMap::new(),
        }
    }

    /// Create a new widget context (for testing and widget building)
    pub fn new_test() -> Self {
        Self::new(Scene::new())
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
        self.scene.get_node(node_id).is_some_and(|node| {
            matches!(node.content, NodeContent::Styled { ref style } if style.text.is_some())
        })
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
        self.scene.get_node(node_id).and_then(|node| {
            if let NodeContent::Styled { ref style } = node.content {
                style.text.as_ref().map(|t| t.text.clone())
            } else {
                None
            }
        })
    }

    /// Set layout style for a node
    pub fn set_layout_style(&mut self, node_id: NodeId, style: FlexStyle) {
        self.layout_styles.insert(node_id, style);
    }

    /// Set a high-level widget style for a node.
    ///
    /// This style contains pseudo-state information (hover, focus, etc.) and will be
    /// transferred to a `WidgetStyle` ECS component for runtime resolution.
    pub fn set_widget_style(&mut self, node_id: NodeId, style: crate::Style) {
        self.widget_styles.insert(node_id, style);
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

    /// Add computed text state to a node
    pub fn add_computed_text_state(&mut self, node_id: NodeId, computed: Computed<String>) {
        self.computed_text_states
            .insert(node_id, ComputedTextState { computed });
    }

    /// Store an Effect to keep it alive for reactive synchronization.
    ///
    /// Effects must be stored to prevent them from being dropped immediately.
    /// Use this when you have Effects that need to run continuously to sync
    /// reactive state (e.g., Checkbox color synchronization).
    ///
    /// # When to Use
    ///
    /// - **Synchronizing Signals**: When one signal needs to update another based on state changes
    /// - **Side effects in widgets**: Logging, persistence, or other non-value-producing reactions
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use flux_state::{Runtime, Signal, Effect};
    /// # use widget_core::WidgetContext;
    /// # let mut ctx = WidgetContext::new_test();
    /// # let runtime = Runtime::new();
    ///
    /// let checked = Signal::new(runtime.clone(), false);
    /// let (read, write) = checked.split();
    ///
    /// let color = Signal::new(runtime.clone(), render_engine::Color::rgba(0.5, 0.5, 0.5, 1.0));
    /// let (_, color_write) = color.split();
    ///
    /// // Effect synchronizes checkbox state to color
    /// let effect = Effect::new(runtime.clone(), move || {
    ///     if read.get() {
    ///         color_write.set(render_engine::Color::BLUE);
    ///     } else {
    ///         color_write.set(render_engine::Color::rgba(0.5, 0.5, 0.5, 1.0));
    ///     }
    /// });
    ///
    /// ctx.store_effect(effect); // Keep Effect alive!
    /// ```
    ///
    /// # Note
    ///
    /// For derived values (not side effects), prefer `Computed` instead:
    ///
    /// ```rust,no_run
    /// use flux_state::{Runtime, Signal, Computed};
    /// # let runtime = Runtime::new();
    /// # let checked = Signal::new(runtime.clone(), false);
    /// # let (read, _) = checked.split();
    ///
    /// // Better: Use Computed for derived values
    /// let color = Computed::new(runtime.clone(), move || {
    ///     if read.get() {
    ///         render_engine::Color::BLUE
    ///     } else {
    ///         render_engine::Color::rgba(0.5, 0.5, 0.5, 1.0)
    ///     }
    /// });
    /// ```
    pub fn store_effect(&mut self, effect: flux_state::Effect) {
        self.effects.push(effect);
    }

    /// Add reactive color state to a node
    pub fn add_reactive_color_state(&mut self, node_id: NodeId, read_signal: ReadSignal<Color>) {
        self.reactive_color_states
            .insert(node_id, ReactiveColorState { read_signal });
    }

    /// Add reactive layout width state to a node
    pub fn add_reactive_layout_width_state(
        &mut self,
        node_id: NodeId,
        read_signal: ReadSignal<f32>,
        handle: Option<std::sync::Arc<dyn std::any::Any + Send + Sync>>,
    ) {
        self.reactive_layout_width_states.insert(
            node_id,
            ReactiveLayoutWidthState {
                read_signal,
                handle,
            },
        );
    }

    /// Add reactive layout flex grow state to a node
    pub fn add_reactive_layout_flex_grow_state(
        &mut self,
        node_id: NodeId,
        read_signal: ReadSignal<f32>,
    ) {
        self.reactive_layout_flex_grow_states
            .insert(node_id, ReactiveLayoutFlexGrowState { read_signal });
    }

    /// Add progress bar direct state to a node
    pub fn add_progress_bar_state(
        &mut self,
        node_id: NodeId,
        progress: ReadSignal<f32>,
        total_width: f32,
    ) {
        self.progress_bar_states
            .insert(node_id, (progress, total_width));
    }

    /// Add a `Timeline<f32>` to drive a signal for a node.
    ///
    /// The timeline will be transferred to a `TimelineDriver<f32>` ECS component
    /// during widget integration, and ticked each frame by `timeline_system`.
    pub fn add_timeline_f32(
        &mut self,
        node_id: NodeId,
        timeline: Timeline<f32>,
        target: WriteSignal<f32>,
    ) {
        self.timeline_f32_states.insert(node_id, (timeline, target));
    }

    /// Add a `Timeline<Color>` to drive a color signal for a node.
    pub fn add_timeline_color(
        &mut self,
        node_id: NodeId,
        timeline: Timeline<render_engine::Color>,
        target: WriteSignal<render_engine::Color>,
    ) {
        self.timeline_color_states
            .insert(node_id, (timeline, target));
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
        input_engine::text::send_char(&mut self.text_input_states, self.focused_node, c);
    }

    /// Send backspace to focused input
    pub fn send_backspace(&mut self) {
        input_engine::text::send_backspace(&mut self.text_input_states, self.focused_node);
    }

    /// Send delete to focused input
    pub fn send_delete(&mut self) {
        input_engine::text::send_delete(&mut self.text_input_states, self.focused_node);
    }

    /// Send left arrow key to focused input
    pub fn send_key_left(&mut self) {
        input_engine::text::send_key_left(&mut self.text_input_states, self.focused_node);
    }

    /// Send right arrow key to focused input
    pub fn send_key_right(&mut self) {
        input_engine::text::send_key_right(&mut self.text_input_states, self.focused_node);
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
        input_engine::form::get_form_field_errors(node_id, &self.form_states, &self.validators)
    }

    /// Get form state for a form node
    pub fn get_form_state(&self, node_id: NodeId) -> Option<&FormState> {
        self.form_states.get(&node_id)
    }

    /// Revalidate a form (check all field validators)
    pub fn revalidate_form(&mut self, node_id: NodeId) {
        input_engine::form::revalidate_form(
            node_id,
            &mut self.form_states,
            &self.text_input_states,
            &mut self.validators,
        );
    }

    /// Trigger form submission
    pub fn trigger_submit(&mut self, node_id: NodeId) {
        input_engine::form::trigger_submit(
            node_id,
            &mut self.form_states,
            &self.text_input_states,
            &mut self.validators,
        );
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
        input_engine::focus::focus_next(&self.text_input_states, &mut self.focused_node)
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
        input_engine::focus::focus_prev(&self.text_input_states, &mut self.focused_node)
    }

    // =========================================================================
    // Accessor methods for app-shell integration
    // =========================================================================

    /// Get all layout styles (for app-shell integration)
    pub fn layout_styles(&self) -> &HashMap<NodeId, FlexStyle> {
        &self.layout_styles
    }

    /// Get all widget styles (for app-shell integration)
    pub fn widget_styles(&self) -> &HashMap<NodeId, crate::Style> {
        &self.widget_styles
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

    /// Extracts derived text states (`Computed<String>`) configured by widgets.
    ///
    /// The app shell pulls these states and injects them into the `arthropod-ecs` Reactive Text component.
    /// This guarantees that when a source `Signal` changes, the UI will correctly recalculate the derived
    /// layout and render string automatically.
    pub fn computed_text_states(&self) -> &HashMap<NodeId, ComputedTextState> {
        &self.computed_text_states
    }

    /// Get all reactive color states (for app-shell integration)
    pub fn reactive_color_states(&self) -> &HashMap<NodeId, ReactiveColorState> {
        &self.reactive_color_states
    }

    /// Get all reactive layout width states (for app-shell integration)
    pub fn reactive_layout_width_states(&self) -> &HashMap<NodeId, ReactiveLayoutWidthState> {
        &self.reactive_layout_width_states
    }

    /// Get all reactive layout flex grow states (for app-shell integration)
    pub fn reactive_layout_flex_grow_states(
        &self,
    ) -> &HashMap<NodeId, ReactiveLayoutFlexGrowState> {
        &self.reactive_layout_flex_grow_states
    }

    /// Get all progress bar states (for app-shell integration)
    pub fn progress_bar_states(&self) -> &HashMap<NodeId, (ReadSignal<f32>, f32)> {
        &self.progress_bar_states
    }

    /// Get all f32 timeline states (for app-shell integration)
    pub fn timeline_f32_states(&self) -> &HashMap<NodeId, (Timeline<f32>, WriteSignal<f32>)> {
        &self.timeline_f32_states
    }

    /// Get all Color timeline states (for app-shell integration)
    pub fn timeline_color_states(
        &self,
    ) -> &HashMap<
        NodeId,
        (
            Timeline<render_engine::Color>,
            WriteSignal<render_engine::Color>,
        ),
    > {
        &self.timeline_color_states
    }

    /// Get all validators (for app-shell integration)
    pub fn validators(&self) -> &HashMap<NodeId, ValidationState> {
        &self.validators
    }

    /// Get all form states (for app-shell integration)
    pub fn form_states(&self) -> &HashMap<NodeId, FormState> {
        &self.form_states
    }

    /// Get all effects (for app-shell integration)
    pub fn effects(&self) -> &[flux_state::Effect] {
        &self.effects
    }

    /// Take all effects, draining the context.
    ///
    /// Effects have RAII Drop semantics (Drop calls `dispose_effect`),
    /// so they must be moved, not cloned, to avoid double-dispose.
    pub fn take_effects(&mut self) -> Vec<flux_state::Effect> {
        std::mem::take(&mut self.effects)
    }

    /// Get the frame signal if available
    pub fn frame_signal(&self) -> ReadSignal<u64> {
        self.frame_signal
            .clone()
            .expect("Frame signal not initialized in context")
    }

    /// Set the frame signal
    pub fn set_frame_signal(&mut self, signal: ReadSignal<u64>) {
        self.frame_signal = Some(signal);
    }

    /// Take all f32 timeline states, draining the map.
    ///
    /// This moves the timelines out of the context so they can be
    /// inserted as ECS components (Timeline is not Clone).
    pub fn take_timeline_f32_states(
        &mut self,
    ) -> HashMap<NodeId, (Timeline<f32>, WriteSignal<f32>)> {
        std::mem::take(&mut self.timeline_f32_states)
    }

    /// Take all Color timeline states, draining the map.
    pub fn take_timeline_color_states(
        &mut self,
    ) -> HashMap<
        NodeId,
        (
            Timeline<render_engine::Color>,
            WriteSignal<render_engine::Color>,
        ),
    > {
        std::mem::take(&mut self.timeline_color_states)
    }

    /// Take ownership of the scene (consumes self)
    pub fn into_scene(self) -> Scene {
        self.scene
    }

    /// Take the scene out of the context, replacing it with an empty one.
    ///
    /// This allows reclaiming the scene while keeping the context alive (e.g., for state access).
    pub fn take_scene(&mut self) -> Scene {
        std::mem::replace(&mut self.scene, Scene::new())
    }

    /// Apply a resolved style to a node.
    ///
    /// This is the "Unified Style Bridge" that converts a high-level `ResolvedStyle`
    /// (containing both layout and visual properties) into the appropriate
    /// low-level components.
    pub fn apply_style(&mut self, node_id: NodeId, style: &crate::ResolvedStyle) {
        // 1. Apply Layout
        self.set_layout_style(node_id, style.to_flex_style());

        // 2. Apply Visuals to Scene Node
        if let Some(node) = self.scene.get_node_mut(node_id) {
            node.content = render_engine::NodeContent::Styled {
                style: Box::new(style.to_visual_style()),
            };
        }

        // 3. Extract background color for backwards compatibility/hit testing if needed
        if let Some(bg) = &style.background {
            self.set_background_color(node_id, bg.as_color());
        }
    }
}

/// Helper to check if a FlexStyle is a row layout
pub fn is_row_layout(style: &FlexStyle) -> bool {
    style.direction == FlexDirection::Row
}

#[cfg(test)]
mod tests;
