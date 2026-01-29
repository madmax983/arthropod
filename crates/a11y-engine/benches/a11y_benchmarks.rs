//! Benchmarks for accessibility engine performance
//!
//! Measures key operations to ensure < 100 μs overhead per frame.
//! Per ADR 0010 requirements:
//! - A11yTree add node: < 50 ns
//! - A11yTree update node: < 30 ns
//! - Sync to platform (100 nodes): < 100 microseconds

use a11y_engine::{
    A11yNode, A11yTree, AccessibleName, Role, platform::accesskit_bridge::AccessKitBridge,
};
use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use plat_core::Rect;
use std::sync::{Arc, Mutex};

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

/// Benchmark syncing dirty nodes to AccessKit platform
/// Target: < 100 microseconds for 100 dirty nodes (ADR 0010)
fn bench_sync_to_platform(c: &mut Criterion) {
    let mut group = c.benchmark_group("sync_to_platform");

    for node_count in [10, 50, 100, 500, 1000] {
        group.bench_with_input(
            BenchmarkId::from_parameter(node_count),
            &node_count,
            |b, &count| {
                // Setup: create tree with many dirty nodes
                let mut tree = A11yTree::new();
                let root = tree.root();

                let _node_ids: Vec<_> = (0..count)
                    .map(|i| {
                        tree.add_node(
                            root,
                            A11yNode {
                                role: Role::Button,
                                name: AccessibleName::Text(format!("Button {}", i)),
                                ..Default::default()
                            },
                        )
                    })
                    .collect();

                // Get all dirty nodes
                let dirty: Vec<_> = tree.get_dirty_nodes().into_iter().collect();

                // Create bridge
                let tree_arc = Arc::new(Mutex::new(tree));
                let bridge = AccessKitBridge::new(tree_arc);

                // Benchmark syncing dirty nodes to AccessKit
                b.iter(|| {
                    black_box(bridge.create_tree_update(black_box(&dirty)));
                });
            },
        );
    }

    group.finish();
}

/// Benchmark full ECS sync cycle (ECS → A11yTree update)
/// Simulates sync_accessible_nodes_system updating A11yTree from Scene
fn bench_ecs_a11y_sync(c: &mut Criterion) {
    let mut group = c.benchmark_group("ecs_a11y_sync");

    for widget_count in [100, 500, 1000] {
        group.bench_with_input(
            BenchmarkId::from_parameter(widget_count),
            &widget_count,
            |b, &count| {
                b.iter(|| {
                    // Setup: create tree and simulate ECS sync
                    let mut tree = A11yTree::new();
                    let root = tree.root();

                    // Create nodes
                    let node_ids: Vec<_> = (0..count)
                        .map(|i| {
                            tree.add_node(
                                root,
                                A11yNode {
                                    role: Role::Button,
                                    name: AccessibleName::Text(format!("Button {}", i)),
                                    ..Default::default()
                                },
                            )
                        })
                        .collect();

                    tree.clear_dirty();

                    // Simulate ECS system updating bounds for all nodes
                    for (i, &node_id) in node_ids.iter().enumerate() {
                        tree.update_node(node_id, |node| {
                            node.bounds = Rect::new(
                                (i as f32 * 10.0) % 800.0,
                                (i as f32 / 80.0) * 50.0,
                                100.0,
                                40.0,
                            );
                        });
                    }

                    black_box(tree);
                });
            },
        );
    }

    group.finish();
}

/// Benchmark complete render frame a11y overhead
/// Measures: ECS sync → get dirty → platform sync
fn bench_frame_a11y_overhead(c: &mut Criterion) {
    let mut group = c.benchmark_group("frame_a11y_overhead");

    for dirty_count in [10, 50, 100] {
        group.bench_with_input(
            BenchmarkId::from_parameter(dirty_count),
            &dirty_count,
            |b, &count| {
                b.iter(|| {
                    // Create tree with existing nodes
                    let mut tree = A11yTree::new();
                    let root = tree.root();

                    // Add 1000 total nodes (realistic UI)
                    let mut node_ids = Vec::new();
                    for i in 0..1000 {
                        let id = tree.add_node(
                            root,
                            A11yNode {
                                role: Role::Button,
                                name: AccessibleName::Text(format!("Button {}", i)),
                                ..Default::default()
                            },
                        );
                        node_ids.push(id);
                    }

                    tree.clear_dirty();

                    // Update only a subset (dirty nodes from frame changes)
                    for i in 0..count {
                        tree.update_node(node_ids[i], |node| {
                            node.bounds = Rect::new(
                                black_box(100.0),
                                black_box(200.0),
                                black_box(50.0),
                                black_box(30.0),
                            );
                        });
                    }

                    // Get dirty nodes
                    let dirty: Vec<_> = tree.get_dirty_nodes().into_iter().collect();

                    // Sync to platform
                    let tree_arc = Arc::new(Mutex::new(tree));
                    let bridge = AccessKitBridge::new(tree_arc);
                    let _update = black_box(bridge.create_tree_update(&dirty));
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
    bench_sync_to_platform,
    bench_ecs_a11y_sync,
    bench_frame_a11y_overhead,
);
criterion_main!(benches);
