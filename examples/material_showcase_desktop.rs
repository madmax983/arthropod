//! Stellar Material UI desktop showcase.
//!
//! Run with:
//! `cargo run --example material_showcase_desktop`

use arthropod::prelude::*;
use glam::Vec4;
use material_ui::{
    components::{
        MaterialButton, MaterialCard, MaterialCheckbox, MaterialForm, MaterialIcon,
        MaterialTextInput,
    },
    data_display::{Chip, Table},
    inputs::{Fab, Rating, Slider, Switch},
    navigation::{AppBar, BottomNavItem, BottomNavigation, Tab, Tabs},
    surfaces::Paper,
    theme::MaterialTheme,
};

fn main() -> Result<(), AppError> {
    App::run("Arthropod Material Showcase", 1200, 820, |ctx| {
        let theme = MaterialTheme::from_seed(Vec4::new(0.38, 0.43, 0.96, 1.0));
        ctx.set_extension(theme);

        let tabs = ctx.signal(0_usize);
        let bottom_nav = ctx.signal(0_usize);
        let volume = ctx.signal(0.72_f32);
        let score = ctx.signal(4.5_f32);
        let notifications = ctx.signal(true);
        let releases_enabled = ctx.signal(false);
        let search = ctx.signal(String::new());
        let workspace_name = ctx.signal(String::new());

        Column::new((
            AppBar::new("Arthropod • Material UI Showcase").center_aligned(),
            Paper::new().elevation(1).child(Column::new((
                Text::new("Built with our new MD3 components")
                    .size(26.0)
                    .color(Color::rgba(0.08, 0.09, 0.15, 1.0)),
                Text::new(
                    "A single desktop surface for command actions, feature toggles, and status views.",
                )
                .size(16.0)
                .color(Color::rgba(0.22, 0.24, 0.33, 1.0)),
                Tabs::new(
                    vec![
                        Tab::new("Overview"),
                        Tab::new("Components"),
                        Tab::new("Analytics"),
                    ],
                    tabs,
                ),
                MaterialCard::new().elevated().child(Column::new((
                    Row::new((
                        MaterialButton::new("Launch").on_click(|| println!("Launch clicked")),
                        MaterialButton::new("Preview").tonal().on_click(|| {
                            println!("Preview clicked")
                        }),
                        MaterialButton::new("Settings")
                            .outlined()
                            .on_click(|| println!("Settings clicked")),
                        Fab::new("★").label("Promote").on_click(|| println!("Promote")),
                    ))
                    .gap(12.0),
                    Row::new((
                        Switch::new(notifications).label("Live notifications"),
                        MaterialCheckbox::new(releases_enabled).label("Enable release channel"),
                        Row::new((MaterialIcon::settings().size(18.0), Chip::new("Desktop")))
                            .gap(8.0),
                    ))
                    .gap(14.0),
                    Slider::new(volume).width(360.0),
                    Rating::new(score),
                ))),
                MaterialCard::new().outlined().child(Column::new((
                    Text::new("Team activity").size(20.0),
                    Table::new(
                        vec!["Name", "Role", "Status"],
                        vec![
                            vec!["Ari", "Design", "Reviewing"],
                            vec!["Mina", "Frontend", "Implementing"],
                            vec!["Jules", "QA", "Validating"],
                        ],
                    ),
                ))),
                MaterialForm::new()
                    .field(
                        "workspace_name",
                        MaterialTextInput::new(workspace_name)
                            .label("Workspace Name")
                            .placeholder("Nebula Ops"),
                    )
                    .field(
                        "quick_filter",
                        MaterialTextInput::new(search)
                            .label("Quick Filter")
                            .placeholder("Search components"),
                    )
                    .on_submit(|| println!("Material showcase form submitted")),
            )))
            .padding(20.0),
            BottomNavigation::new(
                vec![
                    BottomNavItem::new("⌂", "Home"),
                    BottomNavItem::new("◎", "Insights"),
                    BottomNavItem::new("⚙", "Settings"),
                ],
                bottom_nav,
            ),
        ))
        .gap(16.0)
        .padding(16.0)
    })
}
