//! Chat UI Example - Messaging Interface
//!
//! **Real-world pattern:** Chat/messaging application interface
//!
//! **Widgets showcased:**
//! - List (dynamic message history)
//! - Stack (unread badge overlay)
//! - Card (message bubbles)
//! - Row + Spacer (left/right message alignment)
//! - Column (overall layout structure)
//! - TextInput (message composition)
//! - Button (send action)
//!
//! **Key learnings:**
//! - List for dynamic, scrollable content
//! - Stack for overlay badges (unread count)
//! - Row + Spacer for alternating alignment (sender vs receiver)
//! - Card for styled message bubbles
//! - TextInput + Button for message composition
//!
//! **Run with:** cargo run --example chat_ui

use arthropod::prelude::*;

fn main() -> Result<(), AppError> {
    App::run("Chat", 500, 700, |ctx| {
        // Create signal for message input
        let message_input = ctx.signal(String::new());

        // Helper to create message bubble
        fn message(sender: &'static str, text: &'static str, is_own: bool) -> Row<(Spacer, Card<(Column<(Text, Text)>,)>)> {
            let bubble_color = if is_own {
                Color::rgba(0.0, 0.47, 0.84, 0.15) // Blue tint for own messages
            } else {
                Color::rgba(0.9, 0.9, 0.9, 1.0) // Gray for others
            };

            let message_card = Card::new((Column::new((
                Text::new(sender)
                    .size(12.0)
                    .color(Color::rgba(0.5, 0.5, 0.5, 1.0)),
                Text::new(text)
                    .size(14.0)
                    .color(Color::rgba(0.1, 0.1, 0.1, 1.0)),
            ))
            .gap(4.0),))
            .padding(12.0);

            if is_own {
                // Right-aligned (own messages)
                Row::new((Spacer::flex(), message_card))
            } else {
                // Left-aligned (other messages)
                Row::new((Spacer::flex(), message_card))
            }
        }

        Column::new((
            // Header with title and status
            Stack::new((
                Card::new((Row::new((
                    Text::new("Chat")
                        .size(18.0)
                        .color(Color::rgba(0.1, 0.1, 0.1, 1.0)),
                    Spacer::flex(),
                    Text::new("● Online")
                        .size(12.0)
                        .color(Color::rgba(0.0, 0.7, 0.3, 1.0)),
                ))
                .gap(12.0),))
                .padding(16.0),
                // Unread badge overlay (top-right corner)
                Row::new((
                    Spacer::flex(),
                    Card::new((Text::new("3")
                        .size(10.0)
                        .color(Color::rgba(1.0, 1.0, 1.0, 1.0)),))
                    .padding(4.0),
                )),
            )),
            Divider::horizontal(),
            // Message list
            {
                let mut messages = List::column();
                messages.push(message("Alice", "Hey! How are you?", false));
                messages.push(message("You", "I'm good, thanks! Working on the new UI.", true));
                messages.push(message("Alice", "That sounds exciting!", false));
                messages.push(message(
                    "You",
                    "Yeah, testing the new widget system. Pretty cool!",
                    true,
                ));
                messages.push(message("Alice", "Can't wait to see it! 🎉", false));
                messages.gap(8.0)
            },
            Spacer::flex(),
            Divider::horizontal(),
            // Message input area
            Row::new((
                TextInput::new(message_input).placeholder("Type a message..."),
                Button::new("Send").primary(),
            ))
            .gap(8.0)
            .padding(12.0),
        ))
        .gap(0.0)
    })
}
