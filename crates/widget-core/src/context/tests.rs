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
        NodeContent::Styled {
            style: Box::new(render_engine::VisualStyle::new().solid_fill(Color::WHITE.as_vec4())),
        },
    );
    ctx.add_text_input_state(node_id, read, write, false, None);
    node_id
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
        NodeContent::Styled {
            style: Box::new(render_engine::VisualStyle::new().solid_fill(Color::WHITE.as_vec4())),
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
        NodeContent::Styled {
            style: Box::new(render_engine::VisualStyle::new().solid_fill(Color::WHITE.as_vec4())),
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
        NodeContent::Styled {
            style: Box::new(render_engine::VisualStyle::new().solid_fill(Color::WHITE.as_vec4())),
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
        NodeContent::Styled {
            style: Box::new(render_engine::VisualStyle::new().solid_fill(Color::WHITE.as_vec4())),
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
        NodeContent::Styled {
            style: Box::new(render_engine::VisualStyle::new().solid_fill(Color::WHITE.as_vec4())),
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
        NodeContent::Styled {
            style: Box::new(
                render_engine::VisualStyle::new()
                    .solid_fill(Color::BLACK.as_vec4())
                    .text(render_engine::TextContent::new("Hello", 16.0)),
            ),
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
        NodeContent::Styled {
            style: Box::new(
                render_engine::VisualStyle::new()
                    .solid_fill(Color::BLACK.as_vec4())
                    .text(render_engine::TextContent::new(initial, 16.0)),
            ),
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
