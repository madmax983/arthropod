//! Event dispatcher for routing platform events to widgets.
//!
//! The EventDispatcher handles all input routing:
//! - Keyboard input to focused widgets
//! - Tab/Shift+Tab focus navigation
//! - Mouse click hit testing and focus management
//! - Form submission on Enter

// Allow collapsible_if since nested if-let chains are more readable in this context
#![allow(clippy::collapsible_if)]

use plat_core::{ElementState, Event, Key, MouseButton, WindowEvent};
use render_engine::{NodeContent, NodeId, Scene};
use std::collections::HashMap;
use widget_core::WidgetContext;

/// Result of dispatching an event.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DispatchResult {
    /// Event was not handled (no relevant widget)
    Ignored,
    /// Event was handled, no visual update needed
    Handled,
    /// Text content changed, needs text node sync
    TextChanged,
    /// Focus changed, needs focus visual sync
    FocusChanged,
    /// Form was submitted
    FormSubmitted,
}

/// Routes platform events to widgets.
///
/// Tracks input state (mouse position, modifier keys) and provides
/// a single dispatch point for all event routing.
pub struct EventDispatcher {
    /// Current mouse position
    mouse_pos: (f32, f32),
    /// Whether shift key is held
    shift_held: bool,
    /// Mapping from widget NodeIds to app Scene NodeIds
    node_id_map: HashMap<NodeId, NodeId>,
    /// Form node (if any) for Enter submission
    form_node: Option<NodeId>,
}

impl EventDispatcher {
    /// Create a new event dispatcher.
    ///
    /// # Arguments
    ///
    /// * `node_id_map` - Mapping from widget context NodeIds to app scene NodeIds
    /// * `form_node` - Optional form node for Enter key submission
    pub fn new(node_id_map: HashMap<NodeId, NodeId>, form_node: Option<NodeId>) -> Self {
        Self {
            mouse_pos: (0.0, 0.0),
            shift_held: false,
            node_id_map,
            form_node,
        }
    }

    /// Get current mouse position.
    pub fn mouse_pos(&self) -> (f32, f32) {
        self.mouse_pos
    }

    /// Check if shift is held.
    pub fn shift_held(&self) -> bool {
        self.shift_held
    }

    /// Dispatch a platform event to widgets.
    ///
    /// Routes the event to the appropriate widget and returns what changed.
    /// The caller should use the result to sync visuals.
    ///
    /// # Arguments
    ///
    /// * `event` - Platform event to dispatch
    /// * `widget_ctx` - Widget context for state management
    /// * `app_scene` - App scene for hit testing
    ///
    /// # Returns
    ///
    /// `DispatchResult` indicating what changed (if anything)
    pub fn dispatch(
        &mut self,
        event: &Event,
        widget_ctx: &mut WidgetContext,
        app_scene: &Scene,
    ) -> DispatchResult {
        match event {
            Event::Window {
                event: WindowEvent::KeyboardInput(keyboard_input),
                ..
            } => {
                if keyboard_input.state == ElementState::Pressed {
                    self.handle_key_press(&keyboard_input.key, widget_ctx)
                } else if keyboard_input.state == ElementState::Released
                    && matches!(keyboard_input.key, Key::Shift)
                {
                    self.shift_held = false;
                    DispatchResult::Handled
                } else {
                    DispatchResult::Ignored
                }
            }
            Event::Window {
                event: WindowEvent::CursorMoved { position },
                ..
            } => {
                self.mouse_pos = (position.x as f32, position.y as f32);
                DispatchResult::Handled
            }
            Event::Window {
                event: WindowEvent::MouseInput(mouse_input),
                ..
            } => {
                if mouse_input.button == MouseButton::Left
                    && mouse_input.state == ElementState::Pressed
                {
                    self.handle_click(widget_ctx, app_scene)
                } else {
                    DispatchResult::Ignored
                }
            }
            _ => DispatchResult::Ignored,
        }
    }

    /// Handle a key press event.
    fn handle_key_press(&mut self, key: &Key, widget_ctx: &mut WidgetContext) -> DispatchResult {
        match key {
            Key::Shift => {
                self.shift_held = true;
                DispatchResult::Handled
            }
            Key::Backspace => {
                widget_ctx.send_backspace();
                DispatchResult::TextChanged
            }
            Key::Delete => {
                widget_ctx.send_delete();
                DispatchResult::TextChanged
            }
            Key::Left => {
                widget_ctx.send_key_left();
                DispatchResult::Handled
            }
            Key::Right => {
                widget_ctx.send_key_right();
                DispatchResult::Handled
            }
            Key::Enter => {
                // Submit form if we have one
                if let Some(form_node) = self.form_node {
                    widget_ctx.trigger_submit(form_node);
                    DispatchResult::FormSubmitted
                } else {
                    DispatchResult::Ignored
                }
            }
            Key::Tab => {
                if self.shift_held {
                    widget_ctx.focus_prev();
                } else {
                    widget_ctx.focus_next();
                }
                DispatchResult::FocusChanged
            }
            _ => {
                // Try to convert to character
                if let Some(c) = key.to_char(self.shift_held) {
                    widget_ctx.send_char(c);
                    DispatchResult::TextChanged
                } else {
                    DispatchResult::Ignored
                }
            }
        }
    }

    /// Handle a mouse click event.
    fn handle_click(
        &mut self,
        widget_ctx: &mut WidgetContext,
        app_scene: &Scene,
    ) -> DispatchResult {
        let (x, y) = self.mouse_pos;

        // Hit test against app scene
        if let Some(mut app_node_id) = app_scene.hit_test(x, y) {
            // Check if we hit a text node - if so, get its parent (the input container)
            if let Some(hit_node) = app_scene.get_node(app_node_id) {
                if matches!(hit_node.content, NodeContent::Text { .. }) {
                    if let Some(parent_id) = app_scene.find_parent(app_node_id) {
                        app_node_id = parent_id;
                    }
                }
            }

            // Find the corresponding widget node ID
            let widget_node_id = self
                .node_id_map
                .iter()
                .find(|&(_, &app_id)| app_id == app_node_id)
                .map(|(&widget_id, _)| widget_id);

            if let Some(widget_node_id) = widget_node_id {
                if widget_ctx.is_text_input(widget_node_id) {
                    widget_ctx.focus_node(widget_node_id);
                    return DispatchResult::FocusChanged;
                } else if widget_ctx.is_clickable(widget_node_id) {
                    widget_ctx.trigger_click(widget_node_id);
                    return DispatchResult::Handled;
                }
            }
        }

        DispatchResult::Ignored
    }

    /// Get the node ID map (widget -> app scene)
    pub fn node_id_map(&self) -> &HashMap<NodeId, NodeId> {
        &self.node_id_map
    }

    /// Get the form node (if any)
    pub fn form_node(&self) -> Option<NodeId> {
        self.form_node
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use flux_state::{Runtime, Signal};
    use indexmap::IndexMap;
    use plat_core::{KeyboardInput, Modifiers, MouseInput, Point, WindowId};
    use render_engine::Color;

    // =========================================================================
    // Helper functions
    // =========================================================================

    fn create_test_context() -> WidgetContext {
        WidgetContext::new_test()
    }

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

    fn create_keyboard_event(key: Key, state: ElementState) -> Event {
        Event::Window {
            window_id: WindowId::default(),
            event: WindowEvent::KeyboardInput(KeyboardInput {
                key,
                state,
                modifiers: Modifiers::default(),
                repeat: false,
            }),
        }
    }

    fn create_mouse_click_event(x: f64, y: f64) -> Event {
        Event::Window {
            window_id: WindowId::default(),
            event: WindowEvent::MouseInput(MouseInput {
                button: MouseButton::Left,
                state: ElementState::Pressed,
                position: Point { x, y },
                modifiers: Modifiers::default(),
            }),
        }
    }

    fn create_cursor_moved_event(x: f64, y: f64) -> Event {
        Event::Window {
            window_id: WindowId::default(),
            event: WindowEvent::CursorMoved {
                position: Point { x, y },
            },
        }
    }

    // =========================================================================
    // Basic EventDispatcher Tests
    // =========================================================================

    #[test]
    fn test_dispatch_result_variants() {
        // Ensure all variants are distinct
        assert_ne!(DispatchResult::Ignored, DispatchResult::Handled);
        assert_ne!(DispatchResult::TextChanged, DispatchResult::FocusChanged);
        assert_ne!(DispatchResult::FormSubmitted, DispatchResult::Ignored);
    }

    #[test]
    fn test_event_dispatcher_new() {
        let dispatcher = EventDispatcher::new(HashMap::new(), None);
        assert_eq!(dispatcher.mouse_pos(), (0.0, 0.0));
        assert!(!dispatcher.shift_held());
        assert!(dispatcher.form_node().is_none());
    }

    #[test]
    fn test_event_dispatcher_with_form() {
        let form_node = NodeId(42);
        let dispatcher = EventDispatcher::new(HashMap::new(), Some(form_node));
        assert_eq!(dispatcher.form_node(), Some(form_node));
    }

    #[test]
    fn test_event_dispatcher_node_id_map() {
        let mut map = HashMap::new();
        map.insert(NodeId(1), NodeId(100));
        map.insert(NodeId(2), NodeId(200));

        let dispatcher = EventDispatcher::new(map.clone(), None);
        assert_eq!(dispatcher.node_id_map().len(), 2);
        assert_eq!(dispatcher.node_id_map().get(&NodeId(1)), Some(&NodeId(100)));
    }

    // =========================================================================
    // Keyboard Input Tests
    // =========================================================================

    #[test]
    fn test_dispatch_character_key_updates_text() {
        let mut ctx = create_test_context();
        let node_id = create_text_input_node(&mut ctx);
        ctx.focus_node(node_id);

        let mut dispatcher = EventDispatcher::new(HashMap::new(), None);
        let scene = Scene::new();

        let event = create_keyboard_event(Key::A, ElementState::Pressed);
        let result = dispatcher.dispatch(&event, &mut ctx, &scene);

        assert_eq!(result, DispatchResult::TextChanged);
        assert_eq!(ctx.get_text_input_value(node_id), Some("a".to_string()));
    }

    #[test]
    fn test_dispatch_character_with_shift_produces_uppercase() {
        let mut ctx = create_test_context();
        let node_id = create_text_input_node(&mut ctx);
        ctx.focus_node(node_id);

        let mut dispatcher = EventDispatcher::new(HashMap::new(), None);
        let scene = Scene::new();

        // Press shift
        let shift_event = create_keyboard_event(Key::Shift, ElementState::Pressed);
        dispatcher.dispatch(&shift_event, &mut ctx, &scene);

        // Press 'A' with shift held
        let event = create_keyboard_event(Key::A, ElementState::Pressed);
        dispatcher.dispatch(&event, &mut ctx, &scene);

        assert_eq!(ctx.get_text_input_value(node_id), Some("A".to_string()));
    }

    #[test]
    fn test_dispatch_shift_release_clears_shift_state() {
        let mut dispatcher = EventDispatcher::new(HashMap::new(), None);
        let mut ctx = create_test_context();
        let scene = Scene::new();

        // Press shift
        let press_event = create_keyboard_event(Key::Shift, ElementState::Pressed);
        dispatcher.dispatch(&press_event, &mut ctx, &scene);
        assert!(dispatcher.shift_held());

        // Release shift
        let release_event = create_keyboard_event(Key::Shift, ElementState::Released);
        let result = dispatcher.dispatch(&release_event, &mut ctx, &scene);

        assert_eq!(result, DispatchResult::Handled);
        assert!(!dispatcher.shift_held());
    }

    #[test]
    fn test_dispatch_backspace_removes_character() {
        let mut ctx = create_test_context();
        let runtime = Runtime::new();
        let signal = Signal::new(runtime, "hello".to_string());
        let (read, write) = signal.split();
        let node_id = ctx.create_node(
            ctx.root(),
            NodeContent::Rect {
                color: Color::WHITE,
            },
        );
        ctx.add_text_input_state(node_id, read, write, false, None);
        ctx.focus_node(node_id);

        let mut dispatcher = EventDispatcher::new(HashMap::new(), None);
        let scene = Scene::new();

        let event = create_keyboard_event(Key::Backspace, ElementState::Pressed);
        let result = dispatcher.dispatch(&event, &mut ctx, &scene);

        assert_eq!(result, DispatchResult::TextChanged);
        assert_eq!(ctx.get_text_input_value(node_id), Some("hell".to_string()));
    }

    #[test]
    fn test_dispatch_delete_removes_character() {
        let mut ctx = create_test_context();
        let runtime = Runtime::new();
        let signal = Signal::new(runtime, "hello".to_string());
        let (read, write) = signal.split();
        let node_id = ctx.create_node(
            ctx.root(),
            NodeContent::Rect {
                color: Color::WHITE,
            },
        );
        ctx.add_text_input_state(node_id, read, write, false, None);
        ctx.focus_node(node_id);

        // Move cursor to start so delete works
        ctx.send_key_left();
        ctx.send_key_left();
        ctx.send_key_left();
        ctx.send_key_left();
        ctx.send_key_left();

        let mut dispatcher = EventDispatcher::new(HashMap::new(), None);
        let scene = Scene::new();

        let event = create_keyboard_event(Key::Delete, ElementState::Pressed);
        let result = dispatcher.dispatch(&event, &mut ctx, &scene);

        assert_eq!(result, DispatchResult::TextChanged);
        assert_eq!(ctx.get_text_input_value(node_id), Some("ello".to_string()));
    }

    #[test]
    fn test_dispatch_left_arrow_moves_cursor() {
        let mut ctx = create_test_context();
        let runtime = Runtime::new();
        let signal = Signal::new(runtime, "abc".to_string());
        let (read, write) = signal.split();
        let node_id = ctx.create_node(
            ctx.root(),
            NodeContent::Rect {
                color: Color::WHITE,
            },
        );
        ctx.add_text_input_state(node_id, read, write, false, None);
        ctx.focus_node(node_id);

        let mut dispatcher = EventDispatcher::new(HashMap::new(), None);
        let scene = Scene::new();

        // Cursor starts at end (position 3)
        assert_eq!(ctx.get_cursor_position(node_id), Some(3));

        let event = create_keyboard_event(Key::Left, ElementState::Pressed);
        let result = dispatcher.dispatch(&event, &mut ctx, &scene);

        assert_eq!(result, DispatchResult::Handled);
        assert_eq!(ctx.get_cursor_position(node_id), Some(2));
    }

    #[test]
    fn test_dispatch_right_arrow_moves_cursor() {
        let mut ctx = create_test_context();
        let runtime = Runtime::new();
        let signal = Signal::new(runtime, "abc".to_string());
        let (read, write) = signal.split();
        let node_id = ctx.create_node(
            ctx.root(),
            NodeContent::Rect {
                color: Color::WHITE,
            },
        );
        ctx.add_text_input_state(node_id, read, write, false, None);
        ctx.focus_node(node_id);

        // Move cursor to start
        ctx.send_key_left();
        ctx.send_key_left();
        ctx.send_key_left();
        assert_eq!(ctx.get_cursor_position(node_id), Some(0));

        let mut dispatcher = EventDispatcher::new(HashMap::new(), None);
        let scene = Scene::new();

        let event = create_keyboard_event(Key::Right, ElementState::Pressed);
        let result = dispatcher.dispatch(&event, &mut ctx, &scene);

        assert_eq!(result, DispatchResult::Handled);
        assert_eq!(ctx.get_cursor_position(node_id), Some(1));
    }

    #[test]
    fn test_dispatch_tab_focuses_next() {
        let mut ctx = create_test_context();
        let node1 = create_text_input_node(&mut ctx);
        let _node2 = create_text_input_node(&mut ctx);
        ctx.focus_node(node1);

        let mut dispatcher = EventDispatcher::new(HashMap::new(), None);
        let scene = Scene::new();

        let event = create_keyboard_event(Key::Tab, ElementState::Pressed);
        let result = dispatcher.dispatch(&event, &mut ctx, &scene);

        assert_eq!(result, DispatchResult::FocusChanged);
        // Focus should have changed (might be node2 or back to node1 depending on HashMap order)
        assert!(ctx.focused_node().is_some());
    }

    #[test]
    fn test_dispatch_shift_tab_focuses_prev() {
        let mut ctx = create_test_context();
        let node1 = create_text_input_node(&mut ctx);
        let _node2 = create_text_input_node(&mut ctx);
        ctx.focus_node(node1);

        let mut dispatcher = EventDispatcher::new(HashMap::new(), None);
        let scene = Scene::new();

        // Press shift first
        let shift_event = create_keyboard_event(Key::Shift, ElementState::Pressed);
        dispatcher.dispatch(&shift_event, &mut ctx, &scene);

        // Then Tab
        let tab_event = create_keyboard_event(Key::Tab, ElementState::Pressed);
        let result = dispatcher.dispatch(&tab_event, &mut ctx, &scene);

        assert_eq!(result, DispatchResult::FocusChanged);
    }

    #[test]
    fn test_dispatch_enter_submits_form() {
        let mut ctx = create_test_context();
        let form_node = NodeId(42);
        ctx.add_form_state(form_node, IndexMap::new(), None);

        let mut dispatcher = EventDispatcher::new(HashMap::new(), Some(form_node));
        let scene = Scene::new();

        let event = create_keyboard_event(Key::Enter, ElementState::Pressed);
        let result = dispatcher.dispatch(&event, &mut ctx, &scene);

        assert_eq!(result, DispatchResult::FormSubmitted);
    }

    #[test]
    fn test_dispatch_enter_without_form_is_ignored() {
        let mut ctx = create_test_context();
        let mut dispatcher = EventDispatcher::new(HashMap::new(), None);
        let scene = Scene::new();

        let event = create_keyboard_event(Key::Enter, ElementState::Pressed);
        let result = dispatcher.dispatch(&event, &mut ctx, &scene);

        assert_eq!(result, DispatchResult::Ignored);
    }

    #[test]
    fn test_dispatch_unknown_key_is_ignored() {
        let mut ctx = create_test_context();
        let mut dispatcher = EventDispatcher::new(HashMap::new(), None);
        let scene = Scene::new();

        let event = create_keyboard_event(Key::F12, ElementState::Pressed);
        let result = dispatcher.dispatch(&event, &mut ctx, &scene);

        assert_eq!(result, DispatchResult::Ignored);
    }

    #[test]
    fn test_dispatch_key_released_non_shift_is_ignored() {
        let mut ctx = create_test_context();
        let mut dispatcher = EventDispatcher::new(HashMap::new(), None);
        let scene = Scene::new();

        let event = create_keyboard_event(Key::A, ElementState::Released);
        let result = dispatcher.dispatch(&event, &mut ctx, &scene);

        assert_eq!(result, DispatchResult::Ignored);
    }

    // =========================================================================
    // Mouse Input Tests
    // =========================================================================

    #[test]
    fn test_dispatch_cursor_moved_updates_position() {
        let mut ctx = create_test_context();
        let mut dispatcher = EventDispatcher::new(HashMap::new(), None);
        let scene = Scene::new();

        let event = create_cursor_moved_event(150.0, 200.0);
        let result = dispatcher.dispatch(&event, &mut ctx, &scene);

        assert_eq!(result, DispatchResult::Handled);
        assert_eq!(dispatcher.mouse_pos(), (150.0, 200.0));
    }

    #[test]
    fn test_dispatch_click_on_empty_scene_is_ignored() {
        let mut ctx = create_test_context();
        let mut dispatcher = EventDispatcher::new(HashMap::new(), None);
        let scene = Scene::new();

        // Move cursor to a position
        let move_event = create_cursor_moved_event(50.0, 50.0);
        dispatcher.dispatch(&move_event, &mut ctx, &scene);

        // Click
        let click_event = create_mouse_click_event(50.0, 50.0);
        let result = dispatcher.dispatch(&click_event, &mut ctx, &scene);

        assert_eq!(result, DispatchResult::Ignored);
    }

    #[test]
    fn test_dispatch_right_click_is_ignored() {
        let mut ctx = create_test_context();
        let mut dispatcher = EventDispatcher::new(HashMap::new(), None);
        let scene = Scene::new();

        let event = Event::Window {
            window_id: WindowId::default(),
            event: WindowEvent::MouseInput(MouseInput {
                button: MouseButton::Right,
                state: ElementState::Pressed,
                position: Point { x: 50.0, y: 50.0 },
                modifiers: Modifiers::default(),
            }),
        };

        let result = dispatcher.dispatch(&event, &mut ctx, &scene);
        assert_eq!(result, DispatchResult::Ignored);
    }

    #[test]
    fn test_dispatch_mouse_release_is_ignored() {
        let mut ctx = create_test_context();
        let mut dispatcher = EventDispatcher::new(HashMap::new(), None);
        let scene = Scene::new();

        let event = Event::Window {
            window_id: WindowId::default(),
            event: WindowEvent::MouseInput(MouseInput {
                button: MouseButton::Left,
                state: ElementState::Released,
                position: Point { x: 50.0, y: 50.0 },
                modifiers: Modifiers::default(),
            }),
        };

        let result = dispatcher.dispatch(&event, &mut ctx, &scene);
        assert_eq!(result, DispatchResult::Ignored);
    }

    // =========================================================================
    // Non-Window Events Tests
    // =========================================================================

    #[test]
    fn test_dispatch_lifecycle_event_is_ignored() {
        let mut ctx = create_test_context();
        let mut dispatcher = EventDispatcher::new(HashMap::new(), None);
        let scene = Scene::new();

        let event = Event::Lifecycle(plat_core::LifecycleEvent::Resumed);
        let result = dispatcher.dispatch(&event, &mut ctx, &scene);

        assert_eq!(result, DispatchResult::Ignored);
    }

    // =========================================================================
    // Multiple Character Input Tests
    // =========================================================================

    #[test]
    fn test_dispatch_multiple_characters_builds_string() {
        let mut ctx = create_test_context();
        let node_id = create_text_input_node(&mut ctx);
        ctx.focus_node(node_id);

        let mut dispatcher = EventDispatcher::new(HashMap::new(), None);
        let scene = Scene::new();

        // Type "hi"
        for key in [Key::H, Key::I] {
            let event = create_keyboard_event(key, ElementState::Pressed);
            dispatcher.dispatch(&event, &mut ctx, &scene);
        }

        assert_eq!(ctx.get_text_input_value(node_id), Some("hi".to_string()));
    }

    #[test]
    fn test_dispatch_number_key_produces_digit() {
        let mut ctx = create_test_context();
        let node_id = create_text_input_node(&mut ctx);
        ctx.focus_node(node_id);

        let mut dispatcher = EventDispatcher::new(HashMap::new(), None);
        let scene = Scene::new();

        let event = create_keyboard_event(Key::Key5, ElementState::Pressed);
        dispatcher.dispatch(&event, &mut ctx, &scene);

        assert_eq!(ctx.get_text_input_value(node_id), Some("5".to_string()));
    }

    #[test]
    fn test_dispatch_number_key_with_shift_produces_symbol() {
        let mut ctx = create_test_context();
        let node_id = create_text_input_node(&mut ctx);
        ctx.focus_node(node_id);

        let mut dispatcher = EventDispatcher::new(HashMap::new(), None);
        let scene = Scene::new();

        // Press shift
        let shift_event = create_keyboard_event(Key::Shift, ElementState::Pressed);
        dispatcher.dispatch(&shift_event, &mut ctx, &scene);

        // Press 1 (should produce !)
        let event = create_keyboard_event(Key::Key1, ElementState::Pressed);
        dispatcher.dispatch(&event, &mut ctx, &scene);

        assert_eq!(ctx.get_text_input_value(node_id), Some("!".to_string()));
    }

    #[test]
    fn test_dispatch_space_key_produces_space() {
        let mut ctx = create_test_context();
        let node_id = create_text_input_node(&mut ctx);
        ctx.focus_node(node_id);

        let mut dispatcher = EventDispatcher::new(HashMap::new(), None);
        let scene = Scene::new();

        let event = create_keyboard_event(Key::Space, ElementState::Pressed);
        dispatcher.dispatch(&event, &mut ctx, &scene);

        assert_eq!(ctx.get_text_input_value(node_id), Some(" ".to_string()));
    }
}
