use arthropod_ecs::components::{LayoutStyle, ProgressBarState, SceneNodeRef};
use arthropod_ecs::systems::update_all_reactive_system;
use bevy_ecs::prelude::*;
use flux_state::{Runtime, Signal};
use layout_engine::FlexStyle;
use render_engine::{NodeId, Scene};

#[test]
fn test_progress_bar_out_of_bounds_no_loop() {
    let mut world = World::new();
    let runtime = Runtime::new();
    // 1.5 is out-of-bounds. We want to test that it doesn't cause infinite invalidation
    // and is correctly clamped to 1.0.
    let progress_signal = Signal::new(runtime.clone(), 1.5_f32);
    let (read_sig, _write_sig) = progress_signal.split();

    world.insert_resource(Scene::new());

    let entity = world
        .spawn((
            SceneNodeRef(NodeId(1)),
            LayoutStyle(FlexStyle {
                ..Default::default()
            }),
            ProgressBarState {
                progress: read_sig,
                last_progress: 1.0,
                total_width: 100.0,
            },
        ))
        .id();

    let mut schedule = Schedule::default();
    schedule.add_systems(update_all_reactive_system);

    // Run first time
    world.clear_trackers();
    schedule.run(&mut world);

    let state = world.get::<ProgressBarState>(entity).unwrap();
    // Should be clamped to 1.0
    assert!(
        (state.last_progress - 1.0).abs() < 0.0001,
        "Progress should be clamped to 1.0"
    );

    // Run second time with NO changes
    world.clear_trackers();
    schedule.run(&mut world);

    // Check if it was marked as changed
    let mut query = world.query::<Ref<ProgressBarState>>();
    let is_changed = query.get(&world, entity).unwrap().is_changed();
    assert!(
        !is_changed,
        "ProgressBarState should not be marked as changed when unchanged (fixes the infinite invalidation bug)"
    );

    // Check if LayoutStyle was marked as changed
    let mut query = world.query::<Ref<LayoutStyle>>();
    let is_changed = query.get(&world, entity).unwrap().is_changed();
    assert!(
        !is_changed,
        "LayoutStyle should not be marked as changed when unchanged"
    );
}

#[test]
fn test_progress_bar_does_not_trigger_change_detection_when_unchanged() {
    let mut world = World::new();
    let runtime = Runtime::new();
    let progress_signal = Signal::new(runtime.clone(), 0.5_f32);
    let (read_sig, _write_sig) = progress_signal.split();

    world.insert_resource(Scene::new());

    let entity = world
        .spawn((
            SceneNodeRef(NodeId(1)),
            LayoutStyle(FlexStyle {
                ..Default::default()
            }),
            ProgressBarState {
                progress: read_sig,
                last_progress: 0.5,
                total_width: 100.0,
            },
        ))
        .id();

    let mut schedule = Schedule::default();
    schedule.add_systems(update_all_reactive_system);

    // Run first time
    schedule.run(&mut world);
    world.clear_trackers();

    // Run second time with NO changes
    schedule.run(&mut world);

    // Check if it was marked as changed
    let mut query = world.query::<Ref<ProgressBarState>>();
    let is_changed = query.get(&world, entity).unwrap().is_changed();
    assert!(
        !is_changed,
        "ProgressBarState should not be marked as changed when unchanged"
    );

    // Check if LayoutStyle was marked as changed
    let mut query = world.query::<Ref<LayoutStyle>>();
    let is_changed = query.get(&world, entity).unwrap().is_changed();
    assert!(
        !is_changed,
        "LayoutStyle should not be marked as changed when unchanged"
    );
}
