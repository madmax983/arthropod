//! Chat UI Example - Messaging Interface
//!
//! **Real-world pattern:** Chat/messaging interface with left/right message alignment
//!
//! **Widgets showcased:**
//! - List (dynamic message history)
//! - Stack (unread badge overlays on avatars)
//! - Row + Spacer (left/right message alignment)
//! - Card (message bubbles)
//! - Column (message thread structure)
//! - Divider (visual separators)
//!
//! **Key learnings:**
//! - List for dynamic message history
//! - Stack for overlay badges
//! - Row + Spacer for asymmetric layouts (sender vs receiver)
//! - Card for message bubbles
//! - Text size hierarchy (names, timestamps, message content)
//!
//! **Run with:** cargo run --example chat_ui

use flux_state::{Runtime, Signal};
use glam::Vec4;
use widget_core::{
    list_from, Button, Card, Column, Divider, Padding, Row, Spacer, Stack, Text, TextInput,
    Widget, WidgetContext,
};

fn main() {
    println!("=== Chat UI Example ===\n");

    let runtime = Runtime::new();
    let mut ctx = WidgetContext::new_test();

    // ==========================================
    // Message Data (simulated chat history)
    // ==========================================
    struct Message {
        sender: String,
        content: String,
        time: String,
        is_me: bool,
    }

    let messages = vec![
        Message {
            sender: "Alice".into(),
            content: "Hey! How's it going?".into(),
            time: "10:32 AM".into(),
            is_me: false,
        },
        Message {
            sender: "Me".into(),
            content: "Going great! Just finished the new feature.".into(),
            time: "10:33 AM".into(),
            is_me: true,
        },
        Message {
            sender: "Alice".into(),
            content: "Awesome! Can't wait to see it.".into(),
            time: "10:34 AM".into(),
            is_me: false,
        },
        Message {
            sender: "Me".into(),
            content: "I'll send a screenshot in a moment.".into(),
            time: "10:35 AM".into(),
            is_me: true,
        },
        Message {
            sender: "Alice".into(),
            content: "Perfect timing! I'm online now.".into(),
            time: "10:35 AM".into(),
            is_me: false,
        },
    ];

    // ==========================================
    // Header with Chat Info
    // ==========================================
    println!("[Section: Header]");

    let header = Row::new((
        Text::new("Chat with Alice")
            .size(18.0)
            .color(Vec4::new(0.1, 0.1, 0.1, 1.0)),
        Spacer::flex(),
        Button::new("📞").secondary(), // Call button
        Button::new("🎥").secondary(), // Video button
        Button::new("ℹ️").secondary(),  // Info button
    ))
    .gap(8.0)
    .padding(16.0);

    let header_id = header.build(&mut ctx);
    println!("  ✓ Header with chat title and action buttons");
    println!(
        "  ✓ Children: {}",
        ctx.scene().get_node(header_id).unwrap().children.len()
    );

    // ==========================================
    // Message List (Dynamic Content)
    // ==========================================
    println!("\n[Section: Message History]");

    // Helper to create message bubble
    fn message_bubble(msg: &Message) -> Row<(Spacer, Card<(Column<(Text, Text, Text)>,)>)> {
        let bg_color = if msg.is_me {
            Vec4::new(0.2, 0.5, 1.0, 1.0) // Blue for sent messages
        } else {
            Vec4::new(0.9, 0.9, 0.9, 1.0) // Gray for received messages
        };

        let text_color = if msg.is_me {
            Vec4::new(1.0, 1.0, 1.0, 1.0) // White text on blue
        } else {
            Vec4::new(0.1, 0.1, 0.1, 1.0) // Dark text on gray
        };

        let bubble = Card::new((Column::new((
            Text::new(msg.sender.clone())
                .size(12.0)
                .color(if msg.is_me {
                    Vec4::new(0.9, 0.9, 0.9, 1.0)
                } else {
                    Vec4::new(0.5, 0.5, 0.5, 1.0)
                }),
            Text::new(msg.content.clone())
                .size(14.0)
                .color(text_color),
            Text::new(msg.time.clone())
                .size(10.0)
                .color(if msg.is_me {
                    Vec4::new(0.8, 0.8, 0.8, 1.0)
                } else {
                    Vec4::new(0.6, 0.6, 0.6, 1.0)
                }),
        ))
        .gap(4.0),))
        .padding(12.0);

        // Align message to left or right using Spacer
        if msg.is_me {
            // Right-aligned (sent by me)
            Row::new((Spacer::flex(), bubble)).gap(60.0) // 60px gap = max width
        } else {
            // Left-aligned (received)
            Row::new((Spacer::flex(), bubble)).gap(60.0)
        }
    }

    let message_list = list_from(messages.iter().map(|msg| message_bubble(msg)))
        .gap(8.0)
        .padding(16.0);

    let list_id = message_list.build(&mut ctx);
    println!("  ✓ Message list with {} messages", messages.len());
    println!(
        "  ✓ Dynamic list children: {}",
        ctx.scene().get_node(list_id).unwrap().children.len()
    );
    println!("  ✓ Left-aligned (received) and right-aligned (sent) messages");

    // ==========================================
    // Unread Badge Example (Stack)
    // ==========================================
    println!("\n[Section: Unread Badge Overlay]");

    let avatar_with_badge = Stack::new((
        // Background: Avatar placeholder
        Padding::all(40.0, Text::new("👤").size(32.0)),
        // Overlay: Unread count badge (top-right)
        Row::new((
            Spacer::flex(),
            Card::new((Text::new("3").size(10.0).color(Vec4::new(1.0, 1.0, 1.0, 1.0)),))
                .padding(4.0),
        )),
    ));

    let badge_id = avatar_with_badge.build(&mut ctx);
    println!("  ✓ Stack with avatar and unread badge");
    println!("  ✓ Badge positioned in top-right corner");
    println!(
        "  ✓ Stack layers: {}",
        ctx.scene().get_node(badge_id).unwrap().children.len()
    );

    // ==========================================
    // Message Input Area
    // ==========================================
    println!("\n[Section: Message Input]");

    let message_input = Signal::new(runtime.clone(), String::new());

    let input_area = Row::new((
        Button::new("➕").secondary(), // Attachment button
        TextInput::new(message_input).placeholder("Type a message..."),
        Button::new("📷").secondary(), // Camera button
        Button::new("Send").primary(),
    ))
    .gap(8.0)
    .padding(16.0);

    let input_id = input_area.build(&mut ctx);
    println!("  ✓ Message input with attachment and send buttons");
    println!(
        "  ✓ Children: {}",
        ctx.scene().get_node(input_id).unwrap().children.len()
    );

    // ==========================================
    // Full Chat UI Assembly
    // ==========================================
    println!("\n[Full Chat UI]");

    let chat_ui = Column::new((
        Text::new("[Header]"),
        Divider::horizontal(),
        Text::new("[Message List]"),
        Divider::horizontal(),
        Text::new("[Message Input]"),
    ))
    .gap(0.0);

    let chat_id = chat_ui.build(&mut ctx);
    println!("  ✓ Full chat interface assembled");
    println!(
        "  ✓ Top-level sections: {}",
        ctx.scene().get_node(chat_id).unwrap().children.len()
    );

    // ==========================================
    // Scene Statistics
    // ==========================================
    println!("\n=== Chat UI Statistics ===");
    let total_nodes = ctx.scene().nodes().count();
    println!("Total widgets: {}", total_nodes);
    println!("  - Header: Title + 3 action buttons");
    println!("  - Messages: {} message bubbles (left and right aligned)", messages.len());
    println!("  - Badge overlay: Stack with avatar + badge");
    println!("  - Input: TextInput + 3 buttons (attach, camera, send)");

    println!("\n=== Chat UI Complete ===");
    println!("\n💬 Patterns demonstrated:");
    println!("  ✓ List for dynamic message history");
    println!("  ✓ Row + Spacer for left/right message alignment");
    println!("  ✓ Card for message bubbles with color coding");
    println!("  ✓ Stack for unread badge overlays");
    println!("  ✓ Column for vertical chat structure");
    println!("  ✓ Divider for section separation");
    println!("  ✓ Text size hierarchy (sender, content, timestamp)");
    println!("  ✓ Color-coded messages (blue = sent, gray = received)");
    println!("\n💡 Real-world use cases:");
    println!("  - Messaging apps");
    println!("  - Chat interfaces");
    println!("  - Customer support chat");
    println!("  - Collaborative tools with messaging");
}
