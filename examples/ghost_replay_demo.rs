#[cfg(feature = "nova")]
#[allow(clippy::collapsible_if)]
mod demo {
    use arthropod::experimental::ghost_replay::{
        GhostRecorder, GhostReplayer, init_ghost_replay, record_event,
    };
    use arthropod::prelude::*;
    use plat_core::{
        Application, ControlFlow, ElementState, Event, EventLoop, Key, MouseButton, Size,
        WindowConfig, WindowEvent, WindowId,
    };
    use render_engine::{Color, NodeContent, Scene, SceneNode, VisualStyle};

    pub struct DemoApp {
        app: App,
    }

    impl Application for DemoApp {
        fn new(event_loop: &EventLoop) -> Self {
            let config = WindowConfig {
                title: "Ghost Replay Demo".to_string(),
                size: Size {
                    width: 800,
                    height: 600,
                },
                resizable: true,
                decorations: true,
                visible: true,
                ..Default::default()
            };
            let mut app = App::new_windowed(config, event_loop).expect("Failed to create app");

            // Initialize Ghost Replay
            init_ghost_replay(&mut app);

            println!("=== Ghost Replay Demo ===");
            println!("Controls:");
            println!("  R: Start Recording");
            println!("  S: Stop Recording / Stop Playback");
            println!("  P: Start Playback");
            println!("  Click anywhere to draw dots (to verify input).");

            Self { app }
        }

        fn on_event(&mut self, event: Event, control_flow: &mut ControlFlow) {
            // Manually record event (in a real WidgetApp this is done automatically)
            // We need to pass &mut World
            record_event(self.app.world_mut(), &event);

            match &event {
                Event::Window {
                    event: WindowEvent::CloseRequested,
                    ..
                } => {
                    *control_flow = ControlFlow::Exit;
                }
                Event::Window {
                    event: WindowEvent::KeyboardInput(input),
                    ..
                } => {
                    if input.state == ElementState::Pressed {
                        match input.key {
                            Key::R => {
                                println!("Recording started...");
                                if let Some(mut recorder) =
                                    self.app.world_mut().get_resource_mut::<GhostRecorder>()
                                {
                                    recorder.start();
                                }
                            }
                            Key::S => {
                                println!("Stopped.");
                                if let Some(mut recorder) =
                                    self.app.world_mut().get_resource_mut::<GhostRecorder>()
                                {
                                    recorder.stop();
                                }
                                if let Some(mut replayer) =
                                    self.app.world_mut().get_resource_mut::<GhostReplayer>()
                                {
                                    replayer.stop();
                                }
                            }
                            Key::P => {
                                println!("Playback started...");
                                // Extract events to avoid borrow conflict
                                let events = {
                                    if let Some(recorder) =
                                        self.app.world().get_resource::<GhostRecorder>()
                                    {
                                        recorder.events.clone()
                                    } else {
                                        Vec::new()
                                    }
                                };

                                if events.is_empty() {
                                    println!("No events recorded!");
                                } else {
                                    println!("Replaying {} events...", events.len());
                                    if let Some(mut replayer) =
                                        self.app.world_mut().get_resource_mut::<GhostReplayer>()
                                    {
                                        replayer.play(events);
                                    }
                                    // Request redraw to start animation loop
                                    if let Some(window) = self.app.window() {
                                        window.request_redraw();
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
                _ => {}
            }

            // Separate match for drawing to get position easily
            if let Event::Window {
                event: WindowEvent::MouseInput(input),
                ..
            } = &event
            {
                if input.state == ElementState::Pressed && input.button == MouseButton::Left {
                    // Use a block to limit mutable borrow of scene
                    {
                        let mut scene = self.app.world_mut().resource_mut::<Scene>();
                        let root = scene.root();
                        let mut node = SceneNode::new(NodeContent::Styled {
                            style: Box::new(VisualStyle::new().solid_fill(Color::WHITE.as_vec4())),
                        });
                        node.bounds = plat_core::Rect::new(
                            input.position.x as f32 - 2.0,
                            input.position.y as f32 - 2.0,
                            4.0,
                            4.0,
                        );
                        scene.add_node(root, node);
                    }

                    if let Some(window) = self.app.window() {
                        window.request_redraw();
                    }
                }
            }
        }

        fn on_redraw(&mut self, _window_id: WindowId) {
            self.app.update();

            // Render
            if let Err(e) = self.app.render_to_gpu() {
                eprintln!("Render error: {}", e);
            }

            // Constant redraw for playback animation if playing
            let is_playing =
                if let Some(replayer) = self.app.world().get_resource::<GhostReplayer>() {
                    replayer.playing
                } else {
                    false
                };

            if is_playing {
                if let Some(window) = self.app.window() {
                    window.request_redraw();
                }
            }
        }
    }
}

fn main() {
    #[cfg(feature = "nova")]
    {
        use plat_core::run;
        println!("Starting Ghost Replay Demo (nova feature enabled)...");
        // We must run it via plat_core::run.
        // demo::DemoApp implements Application.
        if let Err(e) = run::<demo::DemoApp>() {
            eprintln!("Application error: {}", e);
        }
    }
    #[cfg(not(feature = "nova"))]
    {
        println!("This example requires the 'nova' feature.");
        println!("Run with: cargo run --example ghost_replay_demo --features nova");
    }
}
