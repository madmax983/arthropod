//! Integration demo: Colored rectangles with reactive state + ECS
//!
//! Demonstrates:
//! - Window creation with plat-core
//! - Scene graph with render-engine (persistent, not rebuilt each frame)
//! - wgpu rendering backend
//! - Reactive state with flux-state
//! - ECS integration with arthropod-ecs
//! - Reactive ECS components that automatically update scene nodes

use arthropod_ecs::{FrameworkContext, ReactiveColor, Renderable};
use flux_state::{Effect, Runtime, Signal, WriteSignal};
use plat_core::{
    Application, ControlFlow, Event, EventLoop, Rect, Size, Window, WindowConfig, WindowEvent,
};
use render_engine::{
    backend::{RenderBackend, WgpuBackend},
    Color,
    NodeContent,
    NodeId,
    Scene,
    SceneNode,
    Transform2D,
};
use std::rc::Rc;
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
        println!("║           PERFORMANCE STATS ({} frames)             ║", self.frame_count);
        println!("╠══════════════════════════════════════════════════════╣");
        println!("║  ECS Update:  {:>8.2?} (avg) {:>8.2?} (min) {:>8.2?} (max) ║",
                 avg_update,
                 self.min(&self.update_times),
                 self.max(&self.update_times));
        println!("║  ECS Render:  {:>8.2?} (avg) {:>8.2?} (min) {:>8.2?} (max) ║",
                 avg_render,
                 self.min(&self.render_times),
                 self.max(&self.render_times));
        println!("║  GPU Render:  {:>8.2?} (avg) {:>8.2?} (min) {:>8.2?} (max) ║",
                 avg_gpu,
                 self.min(&self.gpu_times),
                 self.max(&self.gpu_times));
        println!("║  Total Frame: {:>8.2?} (avg) {:>8.2?} (min) {:>8.2?} (max) ║",
                 avg_total,
                 self.min(&self.total_times),
                 self.max(&self.total_times));
        println!("║  Frame Rate:  {:>8.1} fps (avg)                        ║",
                 1.0 / avg_total.as_secs_f64());
        println!("╚══════════════════════════════════════════════════════╝\n");
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

    fn clear(&mut self) {
        self.update_times.clear();
        self.render_times.clear();
        self.gpu_times.clear();
        self.total_times.clear();
        self.frame_count = 0;
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
    window: Window,
    backend: WgpuBackend,

    // ECS Integration
    scene: Scene,
    context: FrameworkContext,
    node_ids: Vec<NodeId>, // NodeIds for updating bounds on resize

    // Runtime must be kept alive for signals to work
    #[allow(dead_code)]
    runtime: Rc<Runtime>,

    // State
    mouse_pos: WriteSignal<(f32, f32)>,
    window_size_write: WriteSignal<(f32, f32)>,

    // Static data
    rects: Vec<RectData>,

    // Keep effects alive (they dispose on drop)
    #[allow(dead_code)]
    hover_effects: Vec<Effect>,

    // Performance tracking
    perf_stats: PerfStats,
}

impl Application for DemoApp {
    fn new(event_loop: &EventLoop) -> Self {
        // Create window
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

        let window = event_loop
            .create_window(config)
            .expect("Failed to create window");
        let size = window.inner_size();

        // Create wgpu backend
        let backend = WgpuBackend::new(&window, size.width, size.height)
            .expect("Failed to create wgpu backend");

        // Create reactive runtime
        let runtime = Runtime::new();

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

        // Create persistent Scene and ECS FrameworkContext
        let mut scene = Scene::new();
        let mut context = FrameworkContext::new();
        let root = scene.root();

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

            // Add node to scene
            let node_id = scene.add_node(root, rect_node);
            node_ids.push(node_id);

            // Spawn ECS entity with ReactiveColor component
            context
                .spawn(node_id)
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
            let rect_index = node_ids.len() - 1; // For debug logging

            let effect = Effect::new(runtime.clone(), move || {
                let (mx, my) = mouse.get();
                let (win_w, win_h) = window_size.get();

                // Calculate bounds for current window size
                let bounds_x = x_ratio * win_w;
                let bounds_y = y_ratio * win_h;
                let bounds_width = width_ratio * win_w;
                let bounds_height = height_ratio * win_h;

                let is_hovering = mx >= bounds_x
                    && mx <= bounds_x + bounds_width
                    && my >= bounds_y
                    && my <= bounds_y + bounds_height;

                println!(
                    "Rect {} - Mouse: ({:.0}, {:.0}), Bounds: ({:.0}, {:.0}, {:.0}x{:.0}), Window: ({:.0}x{:.0}), Hovering: {}",
                    rect_index,
                    mx,
                    my,
                    bounds_x,
                    bounds_y,
                    bounds_width,
                    bounds_height,
                    win_w,
                    win_h,
                    is_hovering
                );

                let new_color = if is_hovering { hover_color } else { base_color };
                // Update the signal - ECS system will propagate to scene node
                write.set(new_color);
            });

            hover_effects.push(effect);
        }

        Self {
            window,
            backend,
            scene,
            context,
            node_ids,
            runtime,
            mouse_pos: write_mouse,
            window_size_write: write_window_size,
            rects,
            hover_effects,
            perf_stats: PerfStats::new(),
        }
    }

    fn on_event(&mut self, event: Event, control_flow: &mut ControlFlow) {
        if let Event::Window { event, .. } = event {
            match event {
                WindowEvent::CloseRequested => {
                    *control_flow = ControlFlow::Exit;
                }
                WindowEvent::Resized(new_size) => {
                    println!("RESIZE EVENT: {}x{}", new_size.width, new_size.height);
                    self.backend.resize(new_size.width, new_size.height);

                    // Update window size signal to trigger hover recalculation
                    self.window_size_write
                        .set((new_size.width as f32, new_size.height as f32));

                    // Update scene node bounds for new window size
                    for (i, node_id) in self.node_ids.iter().enumerate() {
                        let bounds = self.rects[i]
                            .bounds_for_size(new_size.width as f32, new_size.height as f32);
                        if let Some(node) = self.scene.get_node_mut(*node_id) {
                            node.bounds = bounds;
                        }
                    }

                    self.window.request_redraw();
                }
                WindowEvent::CursorMoved { position } => {
                    // Update mouse position signal (triggers color effects)
                    println!("CursorMoved event: ({:.0}, {:.0})", position.x, position.y);
                    self.mouse_pos.set((position.x as f32, position.y as f32));
                    // Request redraw to show updated colors
                    self.window.request_redraw();
                }
                _ => {}
            }
        }
    }

    fn on_redraw(&mut self, _window_id: plat_core::WindowId) {
        let frame_start = Instant::now();

        // Update ECS systems - this will poll all ReactiveColor signals
        // and update the scene node colors automatically
        let update_start = Instant::now();
        self.context.update(&mut self.scene);
        let update_time = update_start.elapsed();

        // Collect renderables from ECS - generates RectInstances
        let render_start = Instant::now();
        let instances = self.context.render(&self.scene);
        let render_time = render_start.elapsed();

        // Render instances directly using ECS-friendly API
        let gpu_start = Instant::now();
        if let Err(e) = self.backend.render_instances(&instances) {
            eprintln!("Render error: {}", e);
        }
        let gpu_time = gpu_start.elapsed();

        let total_time = frame_start.elapsed();

        // Record performance statistics
        self.perf_stats
            .record_frame(update_time, render_time, gpu_time, total_time);
    }
}

fn main() {
    // Initialize tracing with pretty formatting and filtering
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .with_target(true)
        .with_thread_ids(false)
        .with_line_number(true)
        .init();

    println!("Arthropod ECS Integration Demo");
    println!("================================");
    println!();
    println!("This demo showcases:");
    println!("  ✓ Window creation (plat-core)");
    println!("  ✓ Persistent Scene graph (render-engine)");
    println!("  ✓ ECS integration (arthropod-ecs with bevy_ecs)");
    println!("  ✓ Reactive ECS components (ReactiveColor)");
    println!("  ✓ wgpu rendering backend with ECS-generated instances");
    println!("  ✓ Reactive state management (flux-state)");
    println!("  ✓ Mouse input handling");
    println!("  ✓ Hover interactions with automatic color updates");
    println!("  ✓ Performance instrumentation & benchmarking");
    println!();
    println!("You should see 4 colored rectangles:");
    println!("  - Red (top-left)");
    println!("  - Green (top-right)");
    println!("  - Blue (bottom-left)");
    println!("  - Yellow (bottom-right)");
    println!();
    println!("HOVER OVER THE RECTANGLES to see them change color!");
    println!("(Colors update via ECS ReactiveColor components)");
    println!();
    println!("Performance stats will be reported every 2 seconds showing:");
    println!("  - ECS Update time (reactive signal polling)");
    println!("  - ECS Render time (instance collection)");
    println!("  - GPU Render time (wgpu rendering)");
    println!("  - Total frame time and FPS");
    println!();
    println!("Set RUST_LOG=debug for detailed logs");
    println!();

    plat_core::run::<DemoApp>().expect("Failed to run application");
}
