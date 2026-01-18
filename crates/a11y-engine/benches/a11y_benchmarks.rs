//! Benchmarks for accessibility engine performance
//!
//! Measures key operations to ensure < 100 μs overhead per frame.

use a11y_engine::{A11yNode, A11yTree, AccessibleName, Role};
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use plat_core::Rect;

/// Benchmark adding nodes to the tree
fn bench_add_node(c: &mut Criterion) {
    c.bench_function("a11y_tree_add_node", |b| {
        b.iter(|| {
            let mut tree = A11yTree::new();
            let root = tree.root();

            let node = A11yNode {
                role: Role::Button,
                name: AccessibleName::Text("Button".into()),
                ..Default::default()
            };

            black_box(tree.add_node(root, node));
        });
    });
}

/// Benchmark updating node properties
fn bench_update_node(c: &mut Criterion) {
    let mut tree = A11yTree::new();
    let root = tree.root();
    let node_id = tree.add_node(
        root,
        A11yNode {
            role: Role::Button,
            ..Default::default()
        },
    );

    c.bench_function("a11y_tree_update_node", |b| {
        b.iter(|| {
            tree.update_node(node_id, |node| {
                node.state.disabled = !node.state.disabled;
                node.bounds = Rect::new(100.0, 200.0, 50.0, 30.0);
            });
        });
    });
}

/// Benchmark querying nodes by role
fn bench_query_by_role(c: &mut Criterion) {
    let mut group = c.benchmark_group("a11y_tree_query_by_role");

    for size in [100, 1_000, 10_000] {
        let mut tree = A11yTree::new();
        let root = tree.root();

        // Add mix of buttons and textboxes
        for i in 0..size {
            let role = if i % 2 == 0 {
                Role::Button
            } else {
                Role::Textbox
            };

            tree.add_node(
                root,
                A11yNode {
                    role,
                    ..Default::default()
                },
            );
        }

        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, _| {
            b.iter(|| {
                let buttons = tree.query_by_role(Role::Button);
                black_box(buttons);
            });
        });
    }

    group.finish();
}

/// Benchmark getting dirty nodes
fn bench_get_dirty_nodes(c: &mut Criterion) {
    let mut group = c.benchmark_group("a11y_tree_get_dirty_nodes");

    for dirty_count in [10, 100, 1_000] {
        let mut tree = A11yTree::new();
        let root = tree.root();

        // Add 10,000 nodes total
        let mut node_ids = Vec::new();
        for _ in 0..10_000 {
            let id = tree.add_node(
                root,
                A11yNode {
                    role: Role::Button,
                    ..Default::default()
                },
            );
            node_ids.push(id);
        }

        tree.clear_dirty();

        // Mark some as dirty
        for i in 0..dirty_count {
            tree.update_node(node_ids[i], |node| {
                node.state.disabled = true;
            });
        }

        group.bench_with_input(
            BenchmarkId::from_parameter(dirty_count),
            &dirty_count,
            |b, _| {
                b.iter(|| {
                    let dirty = tree.get_dirty_nodes();
                    black_box(dirty);
                });
            },
        );
    }

    group.finish();
}

/// Benchmark remove node (with children)
fn bench_remove_node(c: &mut Criterion) {
    c.bench_function("a11y_tree_remove_node_with_children", |b| {
        b.iter(|| {
            let mut tree = A11yTree::new();
            let root = tree.root();

            // Create group with 10 children
            let group = tree.add_node(
                root,
                A11yNode {
                    role: Role::Group,
                    ..Default::default()
                },
            );

            for _ in 0..10 {
                tree.add_node(
                    group,
                    A11yNode {
                        role: Role::Button,
                        ..Default::default()
                    },
                );
            }

            // Benchmark removing group (cascades to children)
            black_box(tree.remove_node(group));
        });
    });
}

/// Benchmark tree with realistic UI structure
fn bench_realistic_ui(c: &mut Criterion) {
    let mut group = c.benchmark_group("a11y_realistic_ui");

    for widget_count in [100, 1_000, 10_000] {
        group.bench_with_input(
            BenchmarkId::from_parameter(widget_count),
            &widget_count,
            |b, &count| {
                b.iter(|| {
                    let mut tree = A11yTree::new();
                    let root = tree.root();

                    // Simulate realistic UI: 60% buttons, 20% textboxes, 10% groups, 10% other
                    for i in 0..count {
                        let role = match i % 10 {
                            0..=5 => Role::Button,
                            6..=7 => Role::Textbox,
                            8 => Role::Group,
                            _ => Role::List,
                        };

                        let parent = if i % 20 == 0 {
                            // Create some groups
                            let group = tree.add_node(
                                root,
                                A11yNode {
                                    role: Role::Group,
                                    ..Default::default()
                                },
                            );
                            group
                        } else {
                            root
                        };

                        tree.add_node(
                            parent,
                            A11yNode {
                                role,
                                name: AccessibleName::Text(format!("Widget {}", i)),
                                bounds: Rect::new(
                                    ((i as f32 * 10.0) % 800.0),
                                    ((i as f32 / 80.0) * 50.0),
                                    100.0,
                                    40.0,
                                ),
                                ..Default::default()
                            },
                        );
                    }

                    black_box(tree);
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_add_node,
    bench_update_node,
    bench_query_by_role,
    bench_get_dirty_nodes,
    bench_remove_node,
    bench_realistic_ui,
);
criterion_main!(benches);
