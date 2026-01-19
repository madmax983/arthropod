//! Integration demo: Colored rectangles with reactive state + ECS
//!
//! Demonstrates:
//! - arthropod::App builder for easy setup
//! - Scene, Runtime, WgpuBackend managed automatically as ECS Resources
//! - Reactive state with flux-state
//! - ECS integration with arthropod-ecs
//! - Hover interactions with reactive color changes

use arthropod::prelude::*;
use arthropod_mcp::{AppConnection, connect_to_mcp_server};
use std::time::{Duration, Instant};

/// Performance statistics tracker
struct PerfStats {
    update_times: Vec<Duration>,
    render_times: Vec<Duration>,
    gpu_times: Vec<Duration>,
    total_times: Vec<Duration>,
    frame_count: usize,
    last_report: Instant,
}

impl PerfStats {
    fn new() -> Self {
        Self {
            update_times: Vec::new(),
            render_times: Vec::new(),
            gpu_times: Vec::new(),
            total_times: Vec::new(),
            frame_count: 0,
            last_report: Instant::now(),
        }
    }

    fn record_frame(
        &mut self,
        update_time: Duration,
        render_time: Duration,
        gpu_time: Duration,
        total_time: Duration,
    ) {
        self.update_times.push(update_time);
        self.render_times.push(render_time);
        self.gpu_times.push(gpu_time);
        self.total_times.push(total_time);
        self.frame_count += 1;

        // Report stats every 2 seconds
        if self.last_report.elapsed() >= Duration::from_secs(2) {
            self.report();
            self.clear();
            self.last_report = Instant::now();
        }
    }

    fn report(&self) {
        if self.frame_count == 0 {
            return;
        }

        let avg_update = self.avg(&self.update_times);
        let avg_render = self.avg(&self.render_times);
        let avg_gpu = self.avg(&self.gpu_times);
        let avg_total = self.avg(&self.total_times);

        println!("\n╔══════════════════════════════════════════════════════╗");
        println!(
            "║           PERFORMANCE STATS ({} frames)             ║",
            self.frame_count
        );
        println!("╠══════════════════════════════════════════════════════╣");
        println!(
            "║  ECS Update:  {:>8.2?} (avg) {:>8.2?} (min) {:>8.2?} (max) ║",
            avg_update,
            self.min(&self.update_times),
            self.max(&self.update_times)
        );
        println!(
            "║  ECS Render:  {:>8.2?} (avg) {:>8.2?} (min) {:>8.2?} (max) ║",
            avg_render,
            self.min(&self.render_times),
            self.max(&self.render_times)
        );
        println!(
            "║  GPU Render:  {:>8.2?} (avg) {:>8.2?} (min) {:>8.2?} (max) ║",
            avg_gpu,
            self.min(&self.gpu_times),
            self.max(&self.gpu_times)
        );
        println!(
            "║  Total Frame: {:>8.2?} (avg) {:>8.2?} (min) {:>8.2?} (max) ║",
            avg_total,
            self.min(&self.total_times),
            self.max(&self.total_times)
        );
        println!(
            "║  Frame Rate:  {:>8.1} fps (avg)                        ║",
            1.0 / avg_total.as_secs_f32()
        );
        println!("╚══════════════════════════════════════════════════════╝");
    }

    fn clear(&mut self) {
        self.update_times.clear();
        self.render_times.clear();
        self.gpu_times.clear();
        self.total_times.clear();
        self.frame_count = 0;
    }

    fn avg(&self, times: &[Duration]) -> Duration {
        if times.is_empty() {
            return Duration::ZERO;
        }
        let sum: Duration = times.iter().sum();
        sum / times.len() as u32
    }

    fn min(&self, times: &[Duration]) -> Duration {
        times.iter().copied().min().unwrap_or(Duration::ZERO)
    }

    fn max(&self, times: &[Duration]) -> Duration {
        times.iter().copied().max().unwrap_or(Duration::ZERO)
    }
}

// Rectangle data for hover detection
struct RectData {
    // Store as normalized coordinates (0.0 to 1.0)
    x_ratio: f32,
    y_ratio: f32,
    width_ratio: f32,
    height_ratio: f32,
    base_color: Color,
    hover_color: Color,
}

impl RectData {
    fn bounds_for_size(&self, window_width: f32, window_height: f32) -> Rect {
        Rect {
            x: self.x_ratio * window_width,
            y: self.y_ratio * window_height,
            width: self.width_ratio * window_width,
            height: self.height_ratio * window_height,
        }
    }
}

struct DemoApp {
    // App encapsulates Window, WgpuBackend, Scene, Runtime, FrameworkContext
    app: arthropod::App,

    // NodeIds for updating bounds on resize
    node_ids: Vec<NodeId>,

    // State signals
    mouse_pos: WriteSignal<(f32, f32)>,
    window_size_write: WriteSignal<(f32, f32)>,

    // Static data
    rects: Vec<RectData>,

    // Keep effects alive (they dispose on drop)
    #[allow(dead_code)]
    hover_effects: Vec<Effect>,

    // Performance tracking
    perf_stats: PerfStats,

    // MCP connection for remote debugging (optional)
    mcp_connection: Option<AppConnection>,

    // Frame counter for periodic MCP updates
    mcp_update_counter: u32,
}

impl Application for DemoApp {
    fn new(event_loop: &EventLoop) -> Self {
        // Create window config
        let config = WindowConfig {
            title: "Arthropod Phase 1 Demo - Hover Interactive Rectangles".to_string(),
            size: Size {
                width: 800,
                height: 600,
            },
            resizable: true,
            decorations: true,
            visible: true,
            ..Default::default()
        };

        // Create app with App builder - handles Window, WgpuBackend, Scene, Runtime automatically!
        let mut app = arthropod::AppBuilder::new()
            .with_window_config(config)
            .build(event_loop)
            .expect("Failed to create app");

        let size = app.window().unwrap().inner_size();

        // Access Runtime from app
        let runtime = app.runtime().clone();

        // Mouse position signal
        let mouse_signal = Signal::new(runtime.clone(), (0.0f32, 0.0f32));
        let (read_mouse, write_mouse) = mouse_signal.split();

        // Window size signal
        let size_signal = Signal::new(runtime.clone(), (size.width as f32, size.height as f32));
        let (read_window_size, write_window_size) = size_signal.split();

        // Define rectangles with normalized coordinates (relative to 800x600 base)
        // This makes them scale with the window
        let rects = vec![
            RectData {
                x_ratio: 50.0 / 800.0,       // 0.0625
                y_ratio: 50.0 / 600.0,       // 0.0833
                width_ratio: 320.0 / 800.0,  // 0.4
                height_ratio: 240.0 / 600.0, // 0.4
                base_color: Color::RED,
                hover_color: Color::rgba(1.0, 0.5, 0.5, 1.0), // Lighter red
            },
            RectData {
                x_ratio: 430.0 / 800.0,      // 0.5375
                y_ratio: 50.0 / 600.0,       // 0.0833
                width_ratio: 320.0 / 800.0,  // 0.4
                height_ratio: 240.0 / 600.0, // 0.4
                base_color: Color::GREEN,
                hover_color: Color::rgba(0.5, 1.0, 0.5, 1.0), // Lighter green
            },
            RectData {
                x_ratio: 50.0 / 800.0,       // 0.0625
                y_ratio: 310.0 / 600.0,      // 0.5167
                width_ratio: 320.0 / 800.0,  // 0.4
                height_ratio: 240.0 / 600.0, // 0.4
                base_color: Color::BLUE,
                hover_color: Color::rgba(0.5, 0.5, 1.0, 1.0), // Lighter blue
            },
            RectData {
                x_ratio: 430.0 / 800.0,                       // 0.5375
                y_ratio: 310.0 / 600.0,                       // 0.5167
                width_ratio: 320.0 / 800.0,                   // 0.4
                height_ratio: 240.0 / 600.0,                  // 0.4
                base_color: Color::rgba(1.0, 1.0, 0.0, 1.0),  // Yellow
                hover_color: Color::rgba(1.0, 1.0, 0.5, 1.0), // Lighter yellow
            },
        ];

        // Access Scene from ECS World (Scene is a Resource now!)
        let root = {
            let scene = app.world().resource::<Scene>();
            scene.root()
        };

        // Create scene nodes and ECS entities for each rectangle
        let mut node_ids = Vec::new();
        let mut hover_effects = Vec::new();

        for rect in &rects {
            // Create color signal for this rectangle
            let signal = Signal::new(runtime.clone(), rect.base_color);
            let (read, write) = signal.split();

            // Create scene node with initial bounds
            let bounds = rect.bounds_for_size(size.width as f32, size.height as f32);
            let rect_node = SceneNode {
                content: NodeContent::Rect {
                    color: rect.base_color,
                },
                transform: Transform2D::identity(),
                bounds,
                children: vec![],
                visible: true,
                opacity: 1.0,
            };

            // Add node to scene (access Scene as Resource)
            let node_id = {
                let mut scene = app.world_mut().resource_mut::<Scene>();
                scene.add_node(root, rect_node)
            };

            node_ids.push(node_id);

            // Spawn ECS entity with ReactiveColor component
            app.spawn(node_id)
                .insert(Renderable)
                .insert(ReactiveColor::new(read));

            // Create effect that updates color signal based on hover
            let x_ratio = rect.x_ratio;
            let y_ratio = rect.y_ratio;
            let width_ratio = rect.width_ratio;
            let height_ratio = rect.height_ratio;
            let base_color = rect.base_color;
            let hover_color = rect.hover_color;
            let mouse = read_mouse.clone();
            let window_size = read_window_size.clone();

            let effect = Effect::new(runtime.clone(), move || {
                let (mouse_x, mouse_y) = mouse.get();
                let (win_w, win_h) = window_size.get();

                // Calculate rectangle bounds
                let rx = x_ratio * win_w;
                let ry = y_ratio * win_h;
                let rw = width_ratio * win_w;
                let rh = height_ratio * win_h;

                // Check if mouse is hovering
                let is_hovering =
                    mouse_x >= rx && mouse_x <= rx + rw && mouse_y >= ry && mouse_y <= ry + rh;

                // Update color based on hover state
                let new_color = if is_hovering { hover_color } else { base_color };
                write.set(new_color);
            });

            hover_effects.push(effect);
        }

        println!("╔══════════════════════════════════════════════════════╗");
        println!("║      Arthropod Phase 1 Demo - Interactive Rects     ║");
        println!("╠══════════════════════════════════════════════════════╣");
        println!("║  Move mouse over rectangles to see them light up!   ║");
        println!("║  Performance stats shown every 2 seconds             ║");
        println!("║                                                      ║");
        println!("║  Architecture:                                       ║");
        println!("║  ✓ arthropod::App builder (clean API!)              ║");
        println!("║  ✓ Scene graph as ECS Resource                      ║");
        println!("║  ✓ Reactive state (flux-state)                      ║");
        println!("║  ✓ ECS systems (bevy_ecs)                           ║");
        println!("║  ✓ GPU rendering (wgpu)                             ║");
        println!("╚══════════════════════════════════════════════════════╝\n");

        // Try to connect to MCP server (optional - don't fail if server isn't running)
        let mcp_connection = connect_to_mcp_server("colored_rectangles").ok();
        if mcp_connection.is_some() {
            println!("✓ Connected to MCP server for remote debugging");
        } else {
            println!(
                "⚠ MCP server not available (run 'cargo run --bin arthropod-mcp' to enable remote debugging)"
            );
        }

        Self {
            app,
            node_ids,
            mouse_pos: write_mouse,
            window_size_write: write_window_size,
            rects,
            hover_effects,
            perf_stats: PerfStats::new(),
            mcp_connection,
            mcp_update_counter: 0,
        }
    }

    #[allow(clippy::single_match)]
    fn on_event(&mut self, event: Event, control_flow: &mut ControlFlow) {
        match event {
            Event::Window { event, .. } => match event {
                WindowEvent::CloseRequested => {
                    *control_flow = ControlFlow::Exit;
                }
                WindowEvent::Resized(new_size) => {
                    // Update window size signal (triggers effects)
                    self.window_size_write
                        .set((new_size.width as f32, new_size.height as f32));

                    // Resize GPU backend
                    self.app.resize(new_size.width, new_size.height);

                    // Update scene node bounds (access Scene as Resource)
                    let mut scene = self.app.world_mut().resource_mut::<Scene>();
                    for (i, node_id) in self.node_ids.iter().enumerate() {
                        if let Some(node) = scene.get_mut(*node_id) {
                            let rect = &self.rects[i];
                            node.bounds =
                                rect.bounds_for_size(new_size.width as f32, new_size.height as f32);
                        }
                    }
                }
                WindowEvent::CursorMoved { position } => {
                    // Update mouse position signal (triggers effects)
                    self.mouse_pos.set((position.x as f32, position.y as f32));

                    // Request redraw to update visuals
                    if let Some(window) = self.app.window() {
                        window.request_redraw();
                    }
                }
                _ => {}
            },
            _ => {}
        }
    }

    fn on_redraw(&mut self, _window_id: WindowId) {
        let frame_start = Instant::now();

        // Run ECS update systems (reactive signals, animations, etc.)
        let update_start = Instant::now();
        self.app.update();
        let update_time = update_start.elapsed();

        // Send scene updates to MCP server periodically (every 60 frames = ~1 second at 60fps)
        self.mcp_update_counter += 1;
        if self.mcp_update_counter >= 60 {
            self.mcp_update_counter = 0;
            if let Some(connection) = &mut self.mcp_connection {
                let scene = self.app.world().resource::<render_engine::Scene>();
                if let Ok(scene_json) = scene.serialize_to_json() {
                    let _ = connection.send_scene_update(scene_json);
                }
            }
        }

        // Run ECS render systems (collect instances)
        let render_start = Instant::now();
        let render_time = render_start.elapsed();

        // Render to GPU
        let gpu_start = Instant::now();
        if let Err(e) = self.app.render_to_gpu() {
            eprintln!("Render error: {}", e);
        }
        let gpu_time = gpu_start.elapsed();

        let total_time = frame_start.elapsed();

        // Record performance stats
        self.perf_stats
            .record_frame(update_time, render_time, gpu_time, total_time);
    }
}

fn main() {
    env_logger::init();
    plat_core::run::<DemoApp>().expect("Failed to run application");
}
