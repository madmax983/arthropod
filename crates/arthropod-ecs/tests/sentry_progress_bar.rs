use arthropod_ecs::components::{LayoutStyle, ProgressBarState, SceneNodeRef};
use arthropod_ecs::systems::update_progress_bar_direct_system;
use bevy_ecs::prelude::*;
use flux_state::{Runtime, Signal};
use layout_engine::FlexStyle;
use render_engine::NodeId;

#[test]
fn test_progress_bar_does_not_trigger_change_detection_when_unchanged() {
    let mut world = World::new();
    let runtime = Runtime::new();
    let progress_signal = Signal::new(runtime.clone(), 0.5_f32);
    let (read_sig, _write_sig) = progress_signal.split();

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
    schedule.add_systems(update_progress_bar_direct_system);

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
