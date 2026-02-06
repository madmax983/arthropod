//! Settings Panel Example - Application Preferences UI
//!
//! **Real-world pattern:** Settings/preferences interface with sectioned layout
//!
//! **Widgets showcased:**
//! - Column (vertical hierarchical structure)
//! - Divider (section separators with varying thickness)
//! - Checkbox (boolean preference toggles)
//! - Form (grouped preferences with validation)
//! - Padding (visual hierarchy and breathing room)
//! - Row (inline controls and labels)
//!
//! **Key learnings:**
//! - Column + Divider for sectioned content
//! - Checkbox for on/off settings
//! - Form for grouped inputs
//! - Padding for visual hierarchy
//! - Text size for section headers vs labels
//!
//! **Run with:** cargo run --example settings_panel

use flux_state::{Runtime, Signal};
use glam::Vec4;
use widget_core::{
    Button, Checkbox, Column, Divider, Form, Padding, Row, Spacer, Text, TextInput, Widget,
    WidgetContext,
};

fn main() {
    println!("=== Settings Panel Example ===\n");

    let runtime = Runtime::new();
    let mut ctx = WidgetContext::new_test();

    // ==========================================
    // Header
    // ==========================================
    println!("[Section: Header]");
    let header = Column::new((
        Text::new("Settings")
            .size(28.0)
            .color(Vec4::new(0.1, 0.1, 0.1, 1.0)),
        Text::new("Manage your application preferences")
            .size(14.0)
            .color(Vec4::new(0.5, 0.5, 0.5, 1.0)),
    ))
    .gap(4.0)
    .padding(24.0);

    let header_id = header.build(&mut ctx);
    println!("  ✓ Header with title and subtitle");

    // ==========================================
    // Section 1: Account Settings
    // ==========================================
    println!("\n[Section: Account Settings]");

    let account_form = Form::new((
        (
            "username",
            TextInput::new(Signal::new(runtime.clone(), "john_doe".to_string()))
                .placeholder("Username"),
        ),
        (
            "email",
            TextInput::new(Signal::new(runtime.clone(), "john@example.com".to_string()))
                .placeholder("Email")
                .validator(|s| {
                    if s.contains('@') {
                        Ok(())
                    } else {
                        Err("Invalid email".into())
                    }
                }),
        ),
    ))
    .gap(12.0);

    let account_section = Column::new((
        Text::new("Account").size(20.0).color(Vec4::new(0.2, 0.2, 0.2, 1.0)),
        account_form,
        Row::new((Spacer::flex(), Button::new("Save Changes").primary())).gap(8.0),
    ))
    .gap(16.0)
    .padding(24.0);

    let account_id = account_section.build(&mut ctx);
    println!("  ✓ Account form with username and email");
    println!("  ✓ Save button aligned to right");

    // ==========================================
    // Section 2: Notifications
    // ==========================================
    println!("\n[Section: Notifications]");

    let notifications = Column::new((
        Text::new("Notifications")
            .size(20.0)
            .color(Vec4::new(0.2, 0.2, 0.2, 1.0)),
        Checkbox::new(Signal::new(runtime.clone(), true)).label("Enable email notifications"),
        Checkbox::new(Signal::new(runtime.clone(), false)).label("Enable push notifications"),
        Checkbox::new(Signal::new(runtime.clone(), true)).label("Weekly digest emails"),
        Checkbox::new(Signal::new(runtime.clone(), false)).label("Marketing emails"),
    ))
    .gap(12.0)
    .padding(24.0);

    let notifications_id = notifications.build(&mut ctx);
    println!("  ✓ Notification toggles (4 checkboxes)");
    println!("  ✓ Two enabled, two disabled by default");

    // ==========================================
    // Section 3: Appearance
    // ==========================================
    println!("\n[Section: Appearance]");

    let appearance = Column::new((
        Text::new("Appearance")
            .size(20.0)
            .color(Vec4::new(0.2, 0.2, 0.2, 1.0)),
        Row::new((
            Text::new("Theme:").size(14.0),
            Button::new("Light").secondary(),
            Button::new("Dark").primary(),
            Button::new("Auto").secondary(),
        ))
        .gap(8.0),
        Checkbox::new(Signal::new(runtime.clone(), false)).label("Compact mode"),
        Checkbox::new(Signal::new(runtime.clone(), true)).label("Show animations"),
    ))
    .gap(12.0)
    .padding(24.0);

    let appearance_id = appearance.build(&mut ctx);
    println!("  ✓ Theme selector (Light/Dark/Auto buttons)");
    println!("  ✓ Display preferences (2 checkboxes)");

    // ==========================================
    // Section 4: Privacy
    // ==========================================
    println!("\n[Section: Privacy]");

    let privacy = Column::new((
        Text::new("Privacy").size(20.0).color(Vec4::new(0.2, 0.2, 0.2, 1.0)),
        Checkbox::new(Signal::new(runtime.clone(), true)).label("Share usage analytics"),
        Checkbox::new(Signal::new(runtime.clone(), false)).label("Allow crash reports"),
        Checkbox::new(Signal::new(runtime.clone(), true)).label("Remember login"),
        Text::new("We respect your privacy and never sell your data.")
            .size(12.0)
            .color(Vec4::new(0.6, 0.6, 0.6, 1.0)),
    ))
    .gap(12.0)
    .padding(24.0);

    let privacy_id = privacy.build(&mut ctx);
    println!("  ✓ Privacy toggles (3 checkboxes)");
    println!("  ✓ Privacy policy text");

    // ==========================================
    // Full Settings Panel Assembly
    // ==========================================
    println!("\n[Full Settings Panel]");

    let settings_panel = Column::new((
        Text::new("[Header]"),
        Divider::horizontal().thickness(2.0),
        Text::new("[Account Section]"),
        Divider::horizontal(),
        Text::new("[Notifications Section]"),
        Divider::horizontal(),
        Text::new("[Appearance Section]"),
        Divider::horizontal(),
        Text::new("[Privacy Section]"),
        Divider::horizontal().thickness(2.0),
        Padding::all(
            24.0,
            Row::new((
                Button::new("Cancel").secondary(),
                Spacer::flex(),
                Button::new("Reset to Defaults"),
                Button::new("Save All Changes").primary(),
            ))
            .gap(8.0),
        ),
    ))
    .gap(0.0); // Sections have their own padding

    let panel_id = settings_panel.build(&mut ctx);
    println!("  ✓ Full panel assembled with dividers");
    println!("  ✓ Footer with Reset and Save buttons");
    println!(
        "  ✓ Top-level sections: {}",
        ctx.scene().get_node(panel_id).unwrap().children.len()
    );

    // ==========================================
    // Scene Statistics
    // ==========================================
    println!("\n=== Settings Panel Statistics ===");
    let total_nodes = ctx.scene().nodes().count();
    println!("Total widgets: {}", total_nodes);
    println!("  - Header: Title + subtitle");
    println!("  - Account: Form with 2 fields + Save button");
    println!("  - Notifications: 4 checkboxes");
    println!("  - Appearance: Theme buttons + 2 checkboxes");
    println!("  - Privacy: 3 checkboxes + policy text");
    println!("  - Footer: 4 buttons (Cancel, Reset, Save)");

    println!("\n=== Settings Panel Complete ===");
    println!("\n⚙️ Patterns demonstrated:");
    println!("  ✓ Column + Divider for sectioned layout");
    println!("  ✓ Checkbox for boolean toggles");
    println!("  ✓ Form for grouped text inputs");
    println!("  ✓ Row + Spacer for justified button bars");
    println!("  ✓ Text size hierarchy (section headers > labels)");
    println!("  ✓ Thick dividers for major sections");
    println!("  ✓ Padding for visual breathing room");
    println!("\n💡 Real-world use cases:");
    println!("  - Application preferences");
    println!("  - User profile settings");
    println!("  - Admin configuration panels");
    println!("  - Game settings menus");
}
