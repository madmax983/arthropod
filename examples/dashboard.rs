//! Dashboard Example - Analytics/Metrics Dashboard
//!
//! **Real-world pattern:** Analytics dashboard with metric cards and data visualization
//!
//! **Widgets showcased:**
//! - Grid (responsive metric card layout)
//! - Card (themed metric containers)
//! - Column/Row (hierarchical layout)
//! - Divider (section separators)
//! - Text (titles, values, labels with size presets)
//! - Button (action buttons)
//!
//! **Key learnings:**
//! - Grid for responsive card layouts
//! - Card for grouped, themed content
//! - Text size hierarchy for visual importance
//! - Combining Column + Divider for sections
//! - Row + Spacer for justified action bars
//!
//! **Run with:** cargo run --example dashboard

use flux_state::{Runtime, Signal};
use glam::Vec4;
use widget_core::{
    Button, Card, Column, Divider, Grid, Row, Spacer, Text, Widget, WidgetContext,
};

fn main() {
    println!("=== Analytics Dashboard Example ===\n");

    let runtime = Runtime::new();
    let mut ctx = WidgetContext::new_test();

    // ==========================================
    // Header Section
    // ==========================================
    println!("[Section: Header]");
    let header = Row::new((
        Text::new("Analytics Dashboard")
            .size(24.0)
            .color(Vec4::new(0.1, 0.1, 0.1, 1.0)),
        Spacer::flex(),
        Button::new("Refresh").primary(),
        Button::new("Export").secondary(),
    ))
    .gap(12.0)
    .padding(20.0);

    let header_id = header.build(&mut ctx);
    println!("  ✓ Header with title and action buttons");
    println!(
        "  ✓ Children: {}",
        ctx.scene().get_node(header_id).unwrap().children.len()
    );

    // ==========================================
    // Metrics Grid (4 cards in 2x2 grid)
    // ==========================================
    println!("\n[Section: Key Metrics]");

    // Helper to create metric card
    fn metric_card(
        title: &str,
        value: &str,
        change: &str,
        positive: bool,
    ) -> Card<(Column<(Text, Text, Text)>,)> {
        let change_color = if positive {
            Vec4::new(0.0, 0.7, 0.3, 1.0) // Green
        } else {
            Vec4::new(0.9, 0.2, 0.2, 1.0) // Red
        };

        Card::new((Column::new((
            Text::new(title).size(14.0).color(Vec4::new(0.5, 0.5, 0.5, 1.0)),
            Text::new(value).size(32.0).color(Vec4::new(0.1, 0.1, 0.1, 1.0)),
            Text::new(change).size(12.0).color(change_color),
        ))
        .gap(8.0),))
        .padding(20.0)
    }

    let metrics = Grid::new(
        (
            metric_card("Total Users", "12,543", "↑ 12.5%", true),
            metric_card("Revenue", "$48,291", "↑ 8.2%", true),
            metric_card("Active Sessions", "1,847", "↓ 3.1%", false),
            metric_card("Conversion Rate", "3.42%", "↑ 0.5%", true),
        ),
        2, // 2 columns
    )
    .gap(16.0)
    .padding(20.0);

    let metrics_id = metrics.build(&mut ctx);
    println!("  ✓ Built 2x2 grid of metric cards");
    println!(
        "  ✓ Grid rows: {}",
        ctx.scene().get_node(metrics_id).unwrap().children.len()
    );
    println!("  ✓ Metrics: Total Users, Revenue, Active Sessions, Conversion Rate");

    // ==========================================
    // Charts Section (placeholder cards)
    // ==========================================
    println!("\n[Section: Charts]");

    let chart_placeholder = Card::new((Column::new((
        Row::new((
            Text::new("Revenue Over Time")
                .size(16.0)
                .color(Vec4::new(0.2, 0.2, 0.2, 1.0)),
            Spacer::flex(),
            Button::new("7D").secondary(),
            Button::new("30D").secondary(),
            Button::new("90D").primary(),
        ))
        .gap(8.0),
        Divider::horizontal(),
        Column::new((
            Text::new("[Chart visualization would go here]")
                .size(14.0)
                .color(Vec4::new(0.6, 0.6, 0.6, 1.0)),
            Text::new("Line chart showing revenue trend")
                .size(12.0)
                .color(Vec4::new(0.7, 0.7, 0.7, 1.0)),
        ))
        .gap(8.0)
        .padding(20.0),
    ))
    .gap(12.0),))
    .padding(20.0);

    let chart_id = chart_placeholder.build(&mut ctx);
    println!("  ✓ Built chart card with time range selector");
    println!("  ✓ Pattern: Card -> Column -> (Title row, Divider, Chart area)");

    // ==========================================
    // Recent Activity Section (list-style)
    // ==========================================
    println!("\n[Section: Recent Activity]");

    fn activity_item(user: &str, action: &str, time: &str) -> Row<(Text, Spacer, Text)> {
        Row::new((
            Text::new(format!("{}: {}", user, action))
                .size(14.0)
                .color(Vec4::new(0.2, 0.2, 0.2, 1.0)),
            Spacer::flex(),
            Text::new(time).size(12.0).color(Vec4::new(0.6, 0.6, 0.6, 1.0)),
        ))
        .gap(16.0)
        .padding(8.0)
    }

    let activity = Card::new((Column::new((
        Text::new("Recent Activity")
            .size(18.0)
            .color(Vec4::new(0.1, 0.1, 0.1, 1.0)),
        Divider::horizontal(),
        activity_item("john@example.com", "Completed purchase", "2m ago"),
        Divider::horizontal(),
        activity_item("sarah@example.com", "Started trial", "5m ago"),
        Divider::horizontal(),
        activity_item("mike@example.com", "Viewed pricing", "8m ago"),
        Divider::horizontal(),
        activity_item("emma@example.com", "Signed up", "12m ago"),
        Divider::horizontal(),
        Row::new((
            Spacer::flex(),
            Button::new("View All").secondary(),
        )),
    ))
    .gap(12.0),))
    .padding(20.0);

    let activity_id = activity.build(&mut ctx);
    println!("  ✓ Built activity feed card");
    println!(
        "  ✓ Items: {}",
        ctx.scene().get_node(activity_id).unwrap().children.len()
    );
    println!("  ✓ Pattern: Alternating content rows and dividers");

    // ==========================================
    // Quick Actions Grid
    // ==========================================
    println!("\n[Section: Quick Actions]");

    fn action_card(
        icon: &str,
        title: &str,
        description: &str,
    ) -> Card<(Column<(Text, Text, Text, Button)>,)> {
        Card::new((Column::new((
            Text::new(icon).size(32.0),
            Text::new(title)
                .size(16.0)
                .color(Vec4::new(0.2, 0.2, 0.2, 1.0)),
            Text::new(description)
                .size(12.0)
                .color(Vec4::new(0.6, 0.6, 0.6, 1.0)),
            Button::new("Go").primary(),
        ))
        .gap(8.0),))
        .padding(16.0)
    }

    let actions = Grid::new(
        (
            action_card("📊", "Reports", "Generate custom reports"),
            action_card("👥", "Users", "Manage user accounts"),
            action_card("⚙️", "Settings", "Configure dashboard"),
            action_card("📈", "Analytics", "View detailed metrics"),
        ),
        4, // 4 columns
    )
    .gap(12.0)
    .padding(20.0);

    let actions_id = actions.build(&mut ctx);
    println!("  ✓ Built 4-column quick actions grid");
    println!(
        "  ✓ Rows: {}",
        ctx.scene().get_node(actions_id).unwrap().children.len()
    );
    println!("  ✓ Each card: Icon, Title, Description, Button");

    // ==========================================
    // Footer
    // ==========================================
    println!("\n[Section: Footer]");

    let footer = Row::new((
        Text::new("Last updated: 2 minutes ago")
            .size(12.0)
            .color(Vec4::new(0.6, 0.6, 0.6, 1.0)),
        Spacer::flex(),
        Button::new("Help").secondary(),
        Button::new("Feedback").secondary(),
    ))
    .gap(8.0)
    .padding(20.0);

    let footer_id = footer.build(&mut ctx);
    println!("  ✓ Footer with timestamp and utility buttons");
    println!(
        "  ✓ Children: {}",
        ctx.scene().get_node(footer_id).unwrap().children.len()
    );

    // ==========================================
    // Full Dashboard Layout
    // ==========================================
    println!("\n[Full Dashboard Layout]");

    let dashboard = Column::new((
        Text::new("[Header would be here]"),
        Text::new("[Metrics grid would be here]"),
        Text::new("[Chart would be here]"),
        Text::new("[Activity feed would be here]"),
        Text::new("[Quick actions would be here]"),
        Text::new("[Footer would be here]"),
    ))
    .gap(0.0); // Sections have their own padding

    let dashboard_id = dashboard.build(&mut ctx);
    println!("  ✓ Full vertical layout assembled");
    println!(
        "  ✓ Top-level sections: {}",
        ctx.scene().get_node(dashboard_id).unwrap().children.len()
    );

    // ==========================================
    // Scene Statistics
    // ==========================================
    println!("\n=== Dashboard Statistics ===");
    let total_nodes = ctx.scene().nodes().count();
    println!("Total widgets in dashboard: {}", total_nodes);
    println!("  - Header: 1 row with 4 children");
    println!("  - Metrics: 4 cards in 2x2 grid");
    println!("  - Chart: 1 card with time selector");
    println!("  - Activity: 1 card with 4 items");
    println!("  - Actions: 4 cards in 1x4 grid");
    println!("  - Footer: 1 row with 4 children");

    println!("\n=== Dashboard Complete ===");
    println!("\n📊 Patterns demonstrated:");
    println!("  ✓ Grid layout for metric cards (2x2, 1x4)");
    println!("  ✓ Card widget for grouped content");
    println!("  ✓ Row + Spacer for justified layouts (header, footer)");
    println!("  ✓ Column + Divider for sectioned content (activity feed)");
    println!("  ✓ Text size hierarchy (titles, values, labels)");
    println!("  ✓ Color coding (positive/negative metrics)");
    println!("  ✓ Nested layouts (card -> column -> rows)");
    println!("\n💡 Real-world use cases:");
    println!("  - Analytics dashboards");
    println!("  - Admin panels");
    println!("  - Monitoring interfaces");
    println!("  - Business intelligence tools");
}
