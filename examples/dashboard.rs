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

use arthropod::prelude::*;
use render_engine::Color;
use widget_core::{Button, Card, Column, Divider, Grid, Row, Spacer, Text};

fn main() -> Result<(), AppError> {
    App::run("Analytics Dashboard", 1000, 700, |_ctx| {
        // Helper to create metric card
        fn metric_card(
            title: &'static str,
            value: &'static str,
            change: &'static str,
            positive: bool,
        ) -> Card<(Column<(Text, Text, Text)>,)> {
            let change_color = if positive {
                Color::rgba(0.0, 0.7, 0.3, 1.0) // Green
            } else {
                Color::rgba(0.9, 0.2, 0.2, 1.0) // Red
            };

            Card::new((Column::new((
                Text::new(title)
                    .size(14.0)
                    .color(Color::rgba(0.5, 0.5, 0.5, 1.0)),
                Text::new(value)
                    .size(32.0)
                    .color(Color::rgba(0.1, 0.1, 0.1, 1.0)),
                Text::new(change).size(12.0).color(change_color),
            ))
            .gap(8.0),))
            .padding(20.0)
        }

        // Helper for activity items
        fn activity_item(
            user: &'static str,
            action: &'static str,
            time: &'static str,
        ) -> Row<(Text, Spacer, Text)> {
            Row::new((
                Text::new(format!("{}: {}", user, action))
                    .size(14.0)
                    .color(Color::rgba(0.2, 0.2, 0.2, 1.0)),
                Spacer::flex(),
                Text::new(time)
                    .size(12.0)
                    .color(Color::rgba(0.6, 0.6, 0.6, 1.0)),
            ))
            .gap(16.0)
            .padding(8.0)
        }

        // Main dashboard layout
        Column::new((
            // Header
            Row::new((
                Text::new("Analytics Dashboard")
                    .size(24.0)
                    .color(Color::rgba(0.1, 0.1, 0.1, 1.0)),
                Spacer::flex(),
                Button::new("Refresh").primary(),
                Button::new("Export").secondary(),
            ))
            .gap(12.0)
            .padding(20.0),
            Divider::horizontal(),
            // Metrics Grid (2x2)
            Grid::new(
                (
                    metric_card("Total Users", "12,543", "↑ 12.5%", true),
                    metric_card("Revenue", "$48,291", "↑ 8.2%", true),
                    metric_card("Active Sessions", "1,847", "↓ 3.1%", false),
                    metric_card("Conversion Rate", "3.42%", "↑ 0.5%", true),
                ),
                2, // 2 columns
            )
            .gap(16.0)
            .padding(20.0),
            // Charts Section
            Card::new((Column::new((
                Row::new((
                    Text::new("Revenue Over Time")
                        .size(16.0)
                        .color(Color::rgba(0.2, 0.2, 0.2, 1.0)),
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
                        .color(Color::rgba(0.6, 0.6, 0.6, 1.0)),
                    Text::new("Line chart showing revenue trend")
                        .size(12.0)
                        .color(Color::rgba(0.7, 0.7, 0.7, 1.0)),
                ))
                .gap(8.0)
                .padding(20.0),
            ))
            .gap(12.0),))
            .padding(20.0),
            // Recent Activity
            Card::new((Column::new((
                Text::new("Recent Activity")
                    .size(18.0)
                    .color(Color::rgba(0.1, 0.1, 0.1, 1.0)),
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
                    Button::new("View All Activity").secondary(),
                ))
                .padding(8.0),
            ))
            .gap(8.0),))
            .padding(20.0),
        ))
        .gap(16.0)
    })
}
