//! Material UI Widget Gallery
//!
//! Showcases all Material Design 3 widgets available in the material-ui crate.
//! Run with: cargo run --example material_gallery
//!
//! This is a build-only example (no window system integration). It
//! demonstrates that every widget compiles, resolves theme colors, and
//! produces valid scene-graph nodes.

use flux_state::{Runtime, Signal};
use glam::Vec4;
use material_ui::{
    components::{
        MaterialButton, MaterialCard, MaterialCheckbox, MaterialDivider, MaterialForm,
        MaterialIcon, MaterialProgressBar, MaterialTextInput,
    },
    data_display::{Avatar, Badge, Chip, ImageList, Skeleton, Table, Tooltip},
    feedback::{Dialog, Snackbar},
    inputs::{
        Autocomplete, ButtonGroup, Fab, Link, RadioGroup, RadioOption, Rating, Select,
        SelectOption, Slider, Switch, ToggleButtonGroup, ToggleOption,
    },
    navigation::{
        AppBar, BottomNavItem, BottomNavigation, Breadcrumbs, Drawer, DrawerItem, Pagination, Step,
        Stepper, Tab, Tabs,
    },
    surfaces::{Accordion, Paper},
    theme::MaterialTheme,
};
use widget_core::{Text, Widget, WidgetContext};

fn main() {
    let runtime = Runtime::new();
    let mut ctx = WidgetContext::new_test();

    // Register Material theme from a purple seed color
    let theme = MaterialTheme::from_seed(Vec4::new(0.404, 0.314, 0.643, 1.0));
    ctx.set_extension(theme);

    let mut widget_count: u32 = 0;

    // =========================================================================
    // Input Widgets
    // =========================================================================
    println!("Building Input Widgets...");

    // Switch
    let switch_signal = Signal::new(runtime.clone(), false);
    let switch = Switch::new(switch_signal).label("Dark Mode");
    let _ = switch.build(&mut ctx);
    widget_count += 1;

    // RadioGroup
    let radio_signal = Signal::new(runtime.clone(), None::<String>);
    let radio_group = RadioGroup::new(
        vec![
            RadioOption::new("sm", "Small"),
            RadioOption::new("md", "Medium"),
            RadioOption::new("lg", "Large"),
        ],
        radio_signal,
    );
    let _ = radio_group.build(&mut ctx);
    widget_count += 1;

    // Slider
    let slider_signal = Signal::new(runtime.clone(), 0.5_f32);
    let slider = Slider::new(slider_signal).width(300.0);
    let _ = slider.build(&mut ctx);
    widget_count += 1;

    // Rating
    let rating_signal = Signal::new(runtime.clone(), 3.0_f32);
    let rating = Rating::new(rating_signal);
    let _ = rating.build(&mut ctx);
    widget_count += 1;

    // Select
    let select_signal = Signal::new(runtime.clone(), None::<String>);
    let select = Select::new(
        vec![
            SelectOption::new("opt1", "Option 1"),
            SelectOption::new("opt2", "Option 2"),
            SelectOption::new("opt3", "Option 3"),
        ],
        select_signal,
    )
    .placeholder("Choose...");
    let _ = select.build(&mut ctx);
    widget_count += 1;

    // FAB
    let fab = Fab::new("+").label("Create").on_click(|| {});
    let _ = fab.build(&mut ctx);
    widget_count += 1;

    // ToggleButtonGroup
    let toggle_signal = Signal::new(runtime.clone(), vec!["bold".to_string()]);
    let toggle_group = ToggleButtonGroup::new(
        vec![
            ToggleOption::new("bold", "B"),
            ToggleOption::new("italic", "I"),
            ToggleOption::new("underline", "U"),
        ],
        toggle_signal,
    )
    .multi();
    let _ = toggle_group.build(&mut ctx);
    widget_count += 1;

    // Autocomplete
    let auto_signal = Signal::new(runtime.clone(), None::<String>);
    let autocomplete = Autocomplete::new(vec!["Apple", "Banana", "Cherry", "Date"], auto_signal)
        .placeholder("Search fruit...");
    let _ = autocomplete.build(&mut ctx);
    widget_count += 1;

    // ButtonGroup
    let btn_group =
        ButtonGroup::new(vec!["Cut", "Copy", "Paste"]).on_click(|i| println!("Button {i}"));
    let _ = btn_group.build(&mut ctx);
    widget_count += 1;

    // Link
    let link = Link::new("Visit documentation").on_click(|| {});
    let _ = link.build(&mut ctx);
    widget_count += 1;

    // =========================================================================
    // Data Display Widgets
    // =========================================================================
    println!("Building Data Display Widgets...");

    // Avatar
    let avatar = Avatar::new("MK").size(48.0);
    let _ = avatar.build(&mut ctx);
    widget_count += 1;

    // Badge
    let badge = Badge::count(5, Text::new("Notifications"));
    let _ = badge.build(&mut ctx);
    widget_count += 1;

    // Chip
    let chip = Chip::filter("Completed").selected(true);
    let _ = chip.build(&mut ctx);
    widget_count += 1;

    // Table
    let table = Table::new(
        vec!["Name", "Role", "Status"],
        vec![
            vec!["Alice", "Engineer", "Active"],
            vec!["Bob", "Designer", "Away"],
        ],
    );
    let _ = table.build(&mut ctx);
    widget_count += 1;

    // Tooltip
    let tooltip = Tooltip::new("Click to save", Text::new("Save"));
    let _ = tooltip.build(&mut ctx);
    widget_count += 1;

    // Skeleton
    let skeleton = Skeleton::text();
    let _ = skeleton.build(&mut ctx);
    widget_count += 1;

    // ImageList
    let image_list = ImageList::new(6).columns(3);
    let _ = image_list.build(&mut ctx);
    widget_count += 1;

    // =========================================================================
    // Feedback Widgets
    // =========================================================================
    println!("Building Feedback Widgets...");

    // Dialog
    let dialog = Dialog::new()
        .title("Confirm Delete")
        .content(Text::new("Are you sure?"))
        .action(MaterialButton::new("Cancel").outlined())
        .action(MaterialButton::new("Delete").on_click(|| {}));
    let _ = dialog.build(&mut ctx);
    widget_count += 1;

    // Snackbar
    let snackbar = Snackbar::new("File saved successfully").action("Undo", || {});
    let _ = snackbar.build(&mut ctx);
    widget_count += 1;

    // =========================================================================
    // Navigation Widgets
    // =========================================================================
    println!("Building Navigation Widgets...");

    // AppBar
    let app_bar = AppBar::new("Material Gallery").center_aligned();
    let _ = app_bar.build(&mut ctx);
    widget_count += 1;

    // Tabs
    let tab_signal = Signal::new(runtime.clone(), 0_usize);
    let tabs = Tabs::new(
        vec![Tab::new("Home"), Tab::new("Profile"), Tab::new("Settings")],
        tab_signal,
    );
    let _ = tabs.build(&mut ctx);
    widget_count += 1;

    // Breadcrumbs
    let breadcrumbs = Breadcrumbs::new(vec!["Home", "Products", "Details"]);
    let _ = breadcrumbs.build(&mut ctx);
    widget_count += 1;

    // Pagination
    let page_signal = Signal::new(runtime.clone(), 1_usize);
    let pagination = Pagination::new(10, page_signal);
    let _ = pagination.build(&mut ctx);
    widget_count += 1;

    // BottomNavigation
    let nav_signal = Signal::new(runtime.clone(), 0_usize);
    let bottom_nav = BottomNavigation::new(
        vec![
            BottomNavItem::new("H", "Home"),
            BottomNavItem::new("S", "Search"),
            BottomNavItem::new("P", "Profile"),
        ],
        nav_signal,
    );
    let _ = bottom_nav.build(&mut ctx);
    widget_count += 1;

    // Stepper
    let stepper = Stepper::new(vec![
        Step::new("Account").completed(),
        Step::new("Address").active(),
        Step::new("Payment"),
    ]);
    let _ = stepper.build(&mut ctx);
    widget_count += 1;

    // Drawer
    let drawer = Drawer::new(vec![
        DrawerItem::new("Inbox").selected(),
        DrawerItem::new("Sent"),
        DrawerItem::new("Drafts"),
    ])
    .header("Mail");
    let _ = drawer.build(&mut ctx);
    widget_count += 1;

    // =========================================================================
    // Surface Widgets
    // =========================================================================
    println!("Building Surface Widgets...");

    // Paper
    let paper = Paper::new()
        .elevation(2)
        .child(Text::new("Elevated surface"));
    let _ = paper.build(&mut ctx);
    widget_count += 1;

    // Accordion
    let accordion_signal = Signal::new(runtime.clone(), true);
    let accordion = Accordion::new("Details", Text::new("Expandable content"), accordion_signal);
    let _ = accordion.build(&mut ctx);
    widget_count += 1;

    // =========================================================================
    // MD3 Component Wrappers
    // =========================================================================
    println!("Building MD3 Components...");

    // MaterialButton variants
    let _ = MaterialButton::new("Filled").build(&mut ctx);
    widget_count += 1;
    let _ = MaterialButton::new("Tonal").tonal().build(&mut ctx);
    widget_count += 1;
    let _ = MaterialButton::new("Outlined").outlined().build(&mut ctx);
    widget_count += 1;
    let _ = MaterialButton::new("Text").text_variant().build(&mut ctx);
    widget_count += 1;
    let _ = MaterialButton::new("Elevated").elevated().build(&mut ctx);
    widget_count += 1;

    // MaterialTextInput
    let input_signal = Signal::new(runtime.clone(), String::new());
    let text_input = MaterialTextInput::new(input_signal)
        .label("Email")
        .placeholder("Enter email");
    let _ = text_input.build(&mut ctx);
    widget_count += 1;

    // MaterialCheckbox
    let check_signal = Signal::new(runtime.clone(), false);
    let checkbox = MaterialCheckbox::new(check_signal).label("Accept terms");
    let _ = checkbox.build(&mut ctx);
    widget_count += 1;

    // MaterialCard
    let card = MaterialCard::new()
        .outlined()
        .child(Text::new("Card content"));
    let _ = card.build(&mut ctx);
    widget_count += 1;

    // MaterialProgressBar
    let progress_signal = Signal::new(runtime.clone(), 0.7_f32);
    let (progress_read, _) = progress_signal.split();
    let progress = MaterialProgressBar::new(progress_read);
    let _ = progress.build(&mut ctx);
    widget_count += 1;

    // MaterialIcon
    let icon = MaterialIcon::check().size(32.0);
    let _ = icon.build(&mut ctx);
    widget_count += 1;

    // MaterialDivider
    let _ = MaterialDivider::horizontal().build(&mut ctx);
    widget_count += 1;

    // MaterialForm
    let name_signal = Signal::new(runtime.clone(), String::new());
    let form = MaterialForm::new()
        .field("Name", MaterialTextInput::new(name_signal).label("Name"))
        .on_submit(|| println!("Submitted!"));
    let _ = form.build(&mut ctx);
    widget_count += 1;

    println!("\nAll {widget_count} Material UI widgets built successfully!");
}
