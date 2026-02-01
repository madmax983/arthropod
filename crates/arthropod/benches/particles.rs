use arthropod::experimental::particles::{
    ParticleEmitter, ParticleTime, emit_particles, update_particles,
};
use bevy_ecs::prelude::*;
use criterion::{Criterion, criterion_group, criterion_main};
use render_engine::Scene;

fn benchmark_particles(c: &mut Criterion) {
    let mut world = World::new();
    world.insert_resource(Scene::new());
    world.insert_resource(ParticleTime { dt: 0.016 });

    let mut schedule = Schedule::default();
    schedule.add_systems((emit_particles, update_particles));

    // Spawn multiple emitters to generate load
    for i in 0..10 {
        world.spawn(ParticleEmitter {
            rate: 100.0,
            active: true,
            position: render_engine::Vec2::new(i as f32 * 10.0, 0.0),
            ..Default::default()
        });
    }

    c.bench_function("particles_update_1000_pps", |b| {
        b.iter(|| {
            schedule.run(&mut world);
        })
    });
}

criterion_group!(benches, benchmark_particles);
criterion_main!(benches);
