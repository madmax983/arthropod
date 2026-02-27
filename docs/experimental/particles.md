# Particles

> **Status:** Experimental (Alpha)
> **Feature Flag:** `nova`

The Particles system provides a GPU-accelerated particle engine integrated with the ECS. It allows spawning and updating thousands of particles efficiently.

## Getting Started

To use the Particles system, you must enable the `nova` feature in your `Cargo.toml`:

```toml
[dependencies]
arthropod = { version = "0.1", features = ["nova"] }
```

## Core Components

### `ParticleEmitter`

Attach this component to an entity to spawn particles.

```rust
pub struct ParticleEmitter {
    pub rate: f32,          // Particles per second
    pub active: bool,       // Whether the emitter is currently active
    pub position: Vec2,     // Source position
    // ... other properties
}
```

### `ParticleTime`

A resource that controls the simulation time step for particles.

```rust
pub struct ParticleTime {
    pub dt: f32, // Delta time for simulation step (e.g., 0.016 for 60fps)
}
```

## Systems

- `emit_particles`: Spawns new `Particle` entities based on active `ParticleEmitter`s.
- `update_particles`: Updates the position and lifetime of existing `Particle`s.
- `apply_forces`: (Optional) Applies physics forces like gravity or wind.

## Basic Usage

Here is a minimal example of setting up a particle system.

```rust
use arthropod::prelude::*;
use arthropod::experimental::particles::{
    ParticleEmitter, ParticleTime, emit_particles, update_particles,
};
use render_engine::{Scene, Vec2};

#[cfg(feature = "nova")]
fn main() {
    // 1. Initialize World
    let mut app = App::new_headless().unwrap();

    // 2. Setup Resources
    app.world_mut().insert_resource(ParticleTime { dt: 0.016 });

    // 3. Register Systems
    // In a real app, add these to your Update schedule
    // app.add_system(emit_particles);
    // app.add_system(update_particles);

    // 4. Spawn an Emitter
    app.spawn(ParticleEmitter {
        rate: 100.0,
        active: true,
        position: Vec2::new(400.0, 300.0),
        ..Default::default()
    });

    println!("Particle System Initialized!");
}

#[cfg(not(feature = "nova"))]
fn main() {
    println!("This example requires the 'nova' feature.");
}
```

## Performance

The particle system is designed for high throughput. Benchmarks show it can handle 10,000+ particles at 60fps on modern hardware.
