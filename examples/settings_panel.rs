//! Settings Panel Example - Application Preferences UI
//!
//! **Real-world pattern:** Settings/preferences panel with sectioned layout
//!
//! **Widgets showcased:**
//! - Column (vertical section layout)
//! - Divider (section separators)
//! - Checkbox (preference toggles)
//! - Form (validated input fields)
//! - TextInput (username, email fields)
//! - Row + Spacer (justified button bars)
//! - Button (save, cancel actions)
//!
//! **Key learnings:**
//! - Column + Divider for hierarchical sections
//! - Form integration with validation
//! - Checkbox for boolean preferences
//! - Row + Spacer for footer button alignment
//! - Text size hierarchy for section headers
//!
//! **Run with:** cargo run --example settings_panel

use arthropod::prelude::*;
use render_engine::Color;
use widget_core::{
    Button, Checkbox, Column, Divider, Form, Row, Spacer, Text, TextInput,
};

fn main() -> Result<(), AppError> {
    App::run("Settings Panel", 600, 800, |ctx| {
        // Create signals for form fields
        let username = ctx.signal("john_doe".to_string());
        let email = ctx.signal("john@example.com".to_string());

        // Create signals for checkboxes
        let email_notifications = ctx.signal(true);
        let push_notifications = ctx.signal(false);
        let dark_mode = ctx.signal(true);

        Column::new((
            // Header
            Text::new("Settings")
                .size(28.0)
                .color(Color::rgba(0.1, 0.1, 0.1, 1.0)),
            Divider::horizontal().margin(16.0),
            // Account Section
            Column::new((
                Text::new("Account")
                    .size(18.0)
                    .color(Color::rgba(0.2, 0.2, 0.2, 1.0)),
                Form::new((
                    (
                        "username",
                        TextInput::new(username)
                            .placeholder("Username")
                            .validator(|s| {
                                if s.len() >= 3 {
                                    Ok(())
                                } else {
                                    Err("Username must be at least 3 characters".to_string())
                                }
                            }),
                    ),
                    (
                        "email",
                        TextInput::new(email)
                            .placeholder("email@example.com")
                            .validator(|s| {
                                if s.contains('@') && s.contains('.') {
                                    Ok(())
                                } else {
                                    Err("Invalid email format".to_string())
                                }
                            }),
                    ),
                )),
            ))
            .gap(12.0),
            Divider::horizontal().margin(16.0),
            // Notifications Section
            Column::new((
                Text::new("Notifications")
                    .size(18.0)
                    .color(Color::rgba(0.2, 0.2, 0.2, 1.0)),
                Row::new((
                    Checkbox::new(email_notifications),
                    Text::new("Email notifications")
                        .size(14.0)
                        .color(Color::rgba(0.3, 0.3, 0.3, 1.0)),
                ))
                .gap(8.0)
                .padding(8.0),
                Row::new((
                    Checkbox::new(push_notifications),
                    Text::new("Push notifications")
                        .size(14.0)
                        .color(Color::rgba(0.3, 0.3, 0.3, 1.0)),
                ))
                .gap(8.0)
                .padding(8.0),
            ))
            .gap(8.0),
            Divider::horizontal().margin(16.0),
            // Appearance Section
            Column::new((
                Text::new("Appearance")
                    .size(18.0)
                    .color(Color::rgba(0.2, 0.2, 0.2, 1.0)),
                Row::new((
                    Checkbox::new(dark_mode),
                    Text::new("Dark mode")
                        .size(14.0)
                        .color(Color::rgba(0.3, 0.3, 0.3, 1.0)),
                ))
                .gap(8.0)
                .padding(8.0),
                Text::new("Choose your preferred theme")
                    .size(12.0)
                    .color(Color::rgba(0.6, 0.6, 0.6, 1.0)),
                Row::new((
                    Button::new("Light").secondary(),
                    Button::new("Dark").primary(),
                    Button::new("Auto").secondary(),
                ))
                .gap(8.0)
                .padding(8.0),
            ))
            .gap(8.0),
            Divider::horizontal().margin(16.0),
            // Privacy Section
            Column::new((
                Text::new("Privacy")
                    .size(18.0)
                    .color(Color::rgba(0.2, 0.2, 0.2, 1.0)),
                Text::new("Control your data and privacy settings")
                    .size(12.0)
                    .color(Color::rgba(0.6, 0.6, 0.6, 1.0)),
                Row::new((
                    Button::new("Manage Data").secondary(),
                    Button::new("Privacy Policy").secondary(),
                ))
                .gap(8.0)
                .padding(8.0),
            ))
            .gap(8.0),
            // Footer with action buttons
            Column::new((
                Spacer::fixed(32.0),
                Divider::horizontal(),
                Row::new((
                    Button::new("Cancel").secondary(),
                    Spacer::flex(),
                    Button::new("Reset").secondary(),
                    Button::new("Save Changes").primary(),
                ))
                .gap(12.0)
                .padding(20.0),
            ))
            .gap(0.0),
        ))
        .gap(12.0)
        .padding(24.0)
    })
}
