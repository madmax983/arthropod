use criterion::{black_box, criterion_group, criterion_main, Criterion};
use render_engine::NodeContent;
use widget_core::{Layer, WidgetContext};

fn bench_layer_root_lookup(c: &mut Criterion) {
    let ctx = WidgetContext::new_test();
    c.bench_function("layer_root lookup", |b| {
        b.iter(|| ctx.layer_root(black_box(Layer::Dialog)));
    });
}

fn bench_add_to_layer(c: &mut Criterion) {
    c.bench_function("add_to_layer", |b| {
        b.iter_with_setup(WidgetContext::new_test, |mut ctx| {
            ctx.add_to_layer(black_box(Layer::Dialog), NodeContent::Empty);
        });
    });
}

fn bench_move_to_layer(c: &mut Criterion) {
    c.bench_function("move_to_layer", |b| {
        b.iter_with_setup(
            || {
                let mut ctx = WidgetContext::new_test();
                let root = ctx.root();
                let node = ctx.create_node(root, NodeContent::Empty);
                (ctx, node)
            },
            |(mut ctx, node)| {
                ctx.move_to_layer(black_box(node), Layer::Tooltip);
            },
        );
    });
}

criterion_group!(
    benches,
    bench_layer_root_lookup,
    bench_add_to_layer,
    bench_move_to_layer
);
criterion_main!(benches);
