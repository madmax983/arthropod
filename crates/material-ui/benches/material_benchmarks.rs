use criterion::{Criterion, black_box, criterion_group, criterion_main};
use flux_state::{Runtime, Signal};
use glam::Vec4;
use material_ui::{
    components::{MaterialButton, MaterialCard, MaterialTextInput},
    data_display::Table,
    inputs::{RadioGroup, RadioOption, Select, SelectOption, Slider, Switch},
    navigation::{Tab, Tabs},
    theme::MaterialTheme,
};
use widget_core::{Widget, WidgetContext};

// ---------------------------------------------------------------------------
// Theme benchmarks
// ---------------------------------------------------------------------------

fn bench_from_seed_light(c: &mut Criterion) {
    let seed = Vec4::new(0.4, 0.2, 0.8, 1.0);
    c.bench_function("MaterialTheme::from_seed_light", |b| {
        b.iter(|| MaterialTheme::from_seed_light(black_box(seed)));
    });
}

fn bench_from_seed_dark(c: &mut Criterion) {
    let seed = Vec4::new(0.4, 0.2, 0.8, 1.0);
    c.bench_function("MaterialTheme::from_seed_dark", |b| {
        b.iter(|| MaterialTheme::from_seed_dark(black_box(seed)));
    });
}

fn bench_to_design_tokens(c: &mut Criterion) {
    let theme = MaterialTheme::from_seed(Vec4::new(0.4, 0.2, 0.8, 1.0));
    c.bench_function("MaterialTheme::to_design_tokens", |b| {
        b.iter(|| black_box(&theme).to_design_tokens());
    });
}

// ---------------------------------------------------------------------------
// Widget build benchmarks
// ---------------------------------------------------------------------------

fn bench_material_button(c: &mut Criterion) {
    c.bench_function("MaterialButton::build", |b| {
        b.iter(|| {
            let mut ctx = WidgetContext::new_test();
            let theme = MaterialTheme::from_seed(Vec4::new(0.4, 0.2, 0.8, 1.0));
            ctx.set_extension(theme);
            MaterialButton::new("Click me").build(&mut ctx)
        })
    });
}

fn bench_switch(c: &mut Criterion) {
    let runtime = Runtime::new();
    c.bench_function("Switch::build", |b| {
        b.iter(|| {
            let mut ctx = WidgetContext::new_test();
            let theme = MaterialTheme::from_seed(Vec4::new(0.4, 0.2, 0.8, 1.0));
            ctx.set_extension(theme);
            let signal = Signal::new(runtime.clone(), false);
            Switch::new(signal).label("Toggle").build(&mut ctx)
        })
    });
}

fn bench_radio_group(c: &mut Criterion) {
    let runtime = Runtime::new();
    c.bench_function("RadioGroup::build (5 options)", |b| {
        b.iter(|| {
            let mut ctx = WidgetContext::new_test();
            let theme = MaterialTheme::from_seed(Vec4::new(0.4, 0.2, 0.8, 1.0));
            ctx.set_extension(theme);
            let signal = Signal::new(runtime.clone(), None::<String>);
            RadioGroup::new(
                vec![
                    RadioOption::new("a", "Alpha"),
                    RadioOption::new("b", "Beta"),
                    RadioOption::new("c", "Gamma"),
                    RadioOption::new("d", "Delta"),
                    RadioOption::new("e", "Epsilon"),
                ],
                signal,
            )
            .build(&mut ctx)
        })
    });
}

fn bench_slider(c: &mut Criterion) {
    let runtime = Runtime::new();
    c.bench_function("Slider::build", |b| {
        b.iter(|| {
            let mut ctx = WidgetContext::new_test();
            let theme = MaterialTheme::from_seed(Vec4::new(0.4, 0.2, 0.8, 1.0));
            ctx.set_extension(theme);
            let signal = Signal::new(runtime.clone(), 0.5_f32);
            Slider::new(signal).width(300.0).build(&mut ctx)
        })
    });
}

fn bench_select(c: &mut Criterion) {
    let runtime = Runtime::new();
    c.bench_function("Select::build (5 options)", |b| {
        b.iter(|| {
            let mut ctx = WidgetContext::new_test();
            let theme = MaterialTheme::from_seed(Vec4::new(0.4, 0.2, 0.8, 1.0));
            ctx.set_extension(theme);
            let signal = Signal::new(runtime.clone(), None::<String>);
            Select::new(
                vec![
                    SelectOption::new("a", "Alpha"),
                    SelectOption::new("b", "Beta"),
                    SelectOption::new("c", "Gamma"),
                    SelectOption::new("d", "Delta"),
                    SelectOption::new("e", "Epsilon"),
                ],
                signal,
            )
            .placeholder("Choose...")
            .build(&mut ctx)
        })
    });
}

fn bench_dialog(c: &mut Criterion) {
    use material_ui::feedback::Dialog;
    use widget_core::Text;

    c.bench_function("Dialog::build", |b| {
        b.iter(|| {
            let mut ctx = WidgetContext::new_test();
            let theme = MaterialTheme::from_seed(Vec4::new(0.4, 0.2, 0.8, 1.0));
            ctx.set_extension(theme);
            Dialog::new()
                .title("Confirm")
                .content(Text::new("Are you sure?"))
                .action(MaterialButton::new("Cancel").outlined())
                .action(MaterialButton::new("OK"))
                .build(&mut ctx)
        })
    });
}

fn bench_table(c: &mut Criterion) {
    c.bench_function("Table::build (5x3)", |b| {
        b.iter(|| {
            let mut ctx = WidgetContext::new_test();
            let theme = MaterialTheme::from_seed(Vec4::new(0.4, 0.2, 0.8, 1.0));
            ctx.set_extension(theme);
            Table::new(
                vec!["Name", "Age", "City"],
                vec![
                    vec!["Alice", "30", "Portland"],
                    vec!["Bob", "25", "Seattle"],
                    vec!["Charlie", "35", "Denver"],
                    vec!["Diana", "28", "Austin"],
                    vec!["Eve", "32", "Chicago"],
                ],
            )
            .build(&mut ctx)
        })
    });
}

fn bench_tabs(c: &mut Criterion) {
    let runtime = Runtime::new();
    c.bench_function("Tabs::build (5 tabs)", |b| {
        b.iter(|| {
            let mut ctx = WidgetContext::new_test();
            let theme = MaterialTheme::from_seed(Vec4::new(0.4, 0.2, 0.8, 1.0));
            ctx.set_extension(theme);
            let signal = Signal::new(runtime.clone(), 0_usize);
            Tabs::new(
                vec![
                    Tab::new("Home"),
                    Tab::new("Profile"),
                    Tab::new("Settings"),
                    Tab::new("Help"),
                    Tab::new("About"),
                ],
                signal,
            )
            .build(&mut ctx)
        })
    });
}

fn bench_material_card(c: &mut Criterion) {
    use widget_core::Text;

    c.bench_function("MaterialCard::build", |b| {
        b.iter(|| {
            let mut ctx = WidgetContext::new_test();
            let theme = MaterialTheme::from_seed(Vec4::new(0.4, 0.2, 0.8, 1.0));
            ctx.set_extension(theme);
            MaterialCard::new()
                .outlined()
                .child(Text::new("Card content"))
                .build(&mut ctx)
        })
    });
}

fn bench_material_text_input(c: &mut Criterion) {
    let runtime = Runtime::new();
    c.bench_function("MaterialTextInput::build", |b| {
        b.iter(|| {
            let mut ctx = WidgetContext::new_test();
            let theme = MaterialTheme::from_seed(Vec4::new(0.4, 0.2, 0.8, 1.0));
            ctx.set_extension(theme);
            let signal = Signal::new(runtime.clone(), String::new());
            MaterialTextInput::new(signal)
                .label("Email")
                .placeholder("you@example.com")
                .build(&mut ctx)
        })
    });
}

criterion_group!(
    benches,
    bench_from_seed_light,
    bench_from_seed_dark,
    bench_to_design_tokens,
    bench_material_button,
    bench_switch,
    bench_radio_group,
    bench_slider,
    bench_select,
    bench_dialog,
    bench_table,
    bench_tabs,
    bench_material_card,
    bench_material_text_input,
);
criterion_main!(benches);
