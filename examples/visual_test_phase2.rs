//! Visual Test: Phase 2 Rendering Pipeline
//!
//! Demonstrates Phase 2 features:
//! - Linear, Radial, Angular, and Diamond gradients
//! - Stroke rendering (solid and gradient strokes, various alignments)
//! - Drop shadows with various blur radii and offsets
//! - Combined effects (gradient + stroke + shadow)
//! - **GRADIENT TEXT** - The Phase 2 showstopper!
//!
//! This test showcases the unified PrimitivePipeline with GradientAtlas,
//! SDF-based stroke rendering, and drop shadow effects.
//!
//! Run with: cargo run --example visual_test_phase2

use glam::{Vec2, Vec4};
use plat_core::{
    Application, ControlFlow, Event, EventLoop, Rect, Size, Window, WindowConfig, WindowEvent,
    WindowId,
};
use render_engine::{
    Color, ColorStop, CornerRadii, NodeContent, NodeId, Paint, Scene, SceneNode, StrokeStyle,
    VisualStyle,
    backend::{RenderBackend, WgpuBackend},
};

// Import gradient types, StrokeAlign, and TextContent from style_engine
use style_engine::{
    AngularGradient, DiamondGradient, LinearGradient, RadialGradient, StrokeAlign, TextContent,
};

struct VisualTestPhase2 {
    backend: WgpuBackend,
    window: Window,
    scene: Scene,
}

impl Application for VisualTestPhase2 {
    fn new(event_loop: &EventLoop) -> Self {
        println!("=== Phase 2 Visual Test ===");
        println!("Testing Phase 2 rendering features:");
        println!("  ✨ Linear gradients (horizontal, vertical, diagonal, multi-stop)");
        println!("  ✨ Radial, Angular, and Diamond gradients");
        println!("  ✨ Stroke rendering (center, inside, outside alignment)");
        println!("  ✨ Drop shadows (single, multiple, colored, large blur)");
        println!("  ✨ Combined effects (gradient + stroke + shadow)");
        println!("  🌈 GRADIENT TEXT - The Phase 2 showstopper!");
        println!("\n🎨 Demonstrating unified PrimitivePipeline with GradientAtlas!\n");

        let config = WindowConfig {
            title: "Phase 2 Visual Test - Gradients, Strokes, Shadows, Gradient Text".to_string(),
            size: Size::new(1400, 1050),
            resizable: true,
            visible: true,
            ..Default::default()
        };

        let window = event_loop
            .create_window(config)
            .expect("Failed to create window");

        let size = window.inner_size();
        // SAFETY: Backend is dropped before window (struct field order)
        let mut backend = unsafe { WgpuBackend::new(&window, size.width, size.height, false) }
            .expect("Failed to create backend");

        backend.set_clear_color(Color::rgba(0.95, 0.95, 0.95, 1.0));

        let scene = create_phase2_test_scene();

        Self {
            backend,
            window,
            scene,
        }
    }

    fn on_event(&mut self, event: Event, control_flow: &mut ControlFlow) {
        match event {
            Event::Window {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                *control_flow = ControlFlow::Exit;
            }
            Event::Window {
                event: WindowEvent::Resized(size),
                ..
            } => {
                self.backend.resize(size.width, size.height);
                self.window.request_redraw();
            }
            _ => {}
        }
    }

    fn on_redraw(&mut self, _window_id: WindowId) {
        if let Err(e) = self.backend.render(&self.scene) {
            eprintln!("Render error: {}", e);
        }
    }
}

fn create_phase2_test_scene() -> Scene {
    let mut scene = Scene::new();
    let root = scene.root();

    // Row 1: Linear Gradients (Y=50)
    println!("Creating Row 1: Linear Gradients (placeholders)");
    create_gradient_row(&mut scene, root, 50.0);

    // Row 2: Radial & Angular Gradients (Y=200)
    println!("Creating Row 2: Radial & Angular Gradients (placeholders)");
    create_radial_angular_row(&mut scene, root, 200.0);

    // Row 3: Strokes (Y=350)
    println!("Creating Row 3: Strokes (placeholders)");
    create_stroke_row(&mut scene, root, 350.0);

    // Row 4: Drop Shadows (Y=500)
    println!("Creating Row 4: Drop Shadows (placeholders)");
    create_shadow_row(&mut scene, root, 500.0);

    // Row 5: Combined Effects (Y=650)
    println!("Creating Row 5: Combined Effects (placeholders)");
    create_combined_effects_row(&mut scene, root, 650.0);

    // Row 6: Gradient Text (Y=800) - THE PHASE 2 SHOWSTOPPER!
    println!("Creating Row 6: Gradient Text 🌈");
    create_gradient_text_row(&mut scene, root, 800.0);

    println!("\n✅ Phase 2 test scene created successfully");
    println!("\nExpected output:");
    println!("  Row 1: Linear gradients (horizontal, vertical, diagonal, multi-stop)");
    println!("  Row 2: Radial, Angular, Diamond gradients");
    println!("  Row 3: Strokes with different alignments");
    println!("  Row 4: Drop shadows (various blur radii)");
    println!("  Row 5: Combined gradient + stroke + shadow effects");
    println!("  Row 6: 🌈 GRADIENT TEXT - Linear, Rainbow, Radial, Angular + Effects!");

    scene
}

/// Row 1: Linear Gradients
fn create_gradient_row(scene: &mut Scene, root: NodeId, y: f32) {
    let x_start = 50.0;
    let spacing = 180.0;
    let width = 150.0;
    let height = 120.0;

    // 1. Horizontal gradient (red → blue)
    let horizontal_grad = SceneNode::new(NodeContent::Styled {
        style: Box::new(
            VisualStyle::new()
                .fill(Paint::Linear(LinearGradient {
                    start: Vec2::new(0.0, 0.5),
                    end: Vec2::new(1.0, 0.5),
                    stops: vec![
                        ColorStop::new(0.0, Vec4::new(1.0, 0.0, 0.0, 1.0)), // Red
                        ColorStop::new(1.0, Vec4::new(0.0, 0.0, 1.0, 1.0)), // Blue
                    ],
                }))
                .corner_radius(8.0),
        ),
    });
    let hgrad_id = scene.add_node(root, horizontal_grad);
    scene.get_node_mut(hgrad_id).unwrap().bounds = Rect::new(x_start, y, width, height);

    // 2. Vertical gradient (green → yellow)
    let vertical_grad = SceneNode::new(NodeContent::Styled {
        style: Box::new(
            VisualStyle::new()
                .fill(Paint::Linear(LinearGradient {
                    start: Vec2::new(0.5, 0.0),
                    end: Vec2::new(0.5, 1.0),
                    stops: vec![
                        ColorStop::new(0.0, Vec4::new(0.0, 0.8, 0.0, 1.0)), // Green
                        ColorStop::new(1.0, Vec4::new(1.0, 1.0, 0.0, 1.0)), // Yellow
                    ],
                }))
                .corner_radius(8.0),
        ),
    });
    let vgrad_id = scene.add_node(root, vertical_grad);
    scene.get_node_mut(vgrad_id).unwrap().bounds = Rect::new(x_start + spacing, y, width, height);

    // 3. Diagonal gradient (purple → orange)
    let diagonal_grad = SceneNode::new(NodeContent::Styled {
        style: Box::new(
            VisualStyle::new()
                .fill(Paint::Linear(LinearGradient {
                    start: Vec2::new(0.0, 0.0),
                    end: Vec2::new(1.0, 1.0),
                    stops: vec![
                        ColorStop::new(0.0, Vec4::new(0.6, 0.0, 0.8, 1.0)), // Purple
                        ColorStop::new(1.0, Vec4::new(1.0, 0.5, 0.0, 1.0)), // Orange
                    ],
                }))
                .corner_radius(8.0),
        ),
    });
    let dgrad_id = scene.add_node(root, diagonal_grad);
    scene.get_node_mut(dgrad_id).unwrap().bounds =
        Rect::new(x_start + spacing * 2.0, y, width, height);

    // 4. Multi-stop gradient (rainbow)
    let rainbow_grad = SceneNode::new(NodeContent::Styled {
        style: Box::new(
            VisualStyle::new()
                .fill(Paint::Linear(LinearGradient {
                    start: Vec2::new(0.0, 0.5),
                    end: Vec2::new(1.0, 0.5),
                    stops: vec![
                        ColorStop::new(0.0, Vec4::new(1.0, 0.0, 0.0, 1.0)), // Red
                        ColorStop::new(0.33, Vec4::new(0.0, 1.0, 0.0, 1.0)), // Green
                        ColorStop::new(0.66, Vec4::new(0.0, 0.0, 1.0, 1.0)), // Blue
                        ColorStop::new(1.0, Vec4::new(1.0, 0.0, 1.0, 1.0)), // Magenta
                    ],
                }))
                .corner_radius(8.0),
        ),
    });
    let rainbow_id = scene.add_node(root, rainbow_grad);
    scene.get_node_mut(rainbow_id).unwrap().bounds =
        Rect::new(x_start + spacing * 3.0, y, width, height);
}

/// Row 2: Radial & Angular Gradients
fn create_radial_angular_row(scene: &mut Scene, root: NodeId, y: f32) {
    let x_start = 50.0;
    let spacing = 180.0;
    let width = 150.0;
    let height = 120.0;

    // 1. Radial gradient (center outward)
    let radial_grad = SceneNode::new(NodeContent::Styled {
        style: Box::new(
            VisualStyle::new()
                .fill(Paint::Radial(RadialGradient {
                    center: Vec2::new(0.5, 0.5),
                    radius: 0.5,
                    stops: vec![
                        ColorStop::new(0.0, Vec4::new(1.0, 1.0, 1.0, 1.0)), // White center
                        ColorStop::new(1.0, Vec4::new(0.0, 0.4, 0.9, 1.0)), // Blue edge
                    ],
                }))
                .corner_radius(8.0),
        ),
    });
    let radial_id = scene.add_node(root, radial_grad);
    scene.get_node_mut(radial_id).unwrap().bounds = Rect::new(x_start, y, width, height);

    // 2. Radial with offset center
    let radial_offset = SceneNode::new(NodeContent::Styled {
        style: Box::new(
            VisualStyle::new()
                .fill(Paint::Radial(RadialGradient {
                    center: Vec2::new(0.3, 0.3), // Offset to top-left
                    radius: 0.7,
                    stops: vec![
                        ColorStop::new(0.0, Vec4::new(1.0, 1.0, 0.0, 1.0)), // Yellow center
                        ColorStop::new(1.0, Vec4::new(0.0, 0.6, 0.6, 1.0)), // Cyan edge
                    ],
                }))
                .corner_radius(8.0),
        ),
    });
    let radial_offset_id = scene.add_node(root, radial_offset);
    scene.get_node_mut(radial_offset_id).unwrap().bounds =
        Rect::new(x_start + spacing, y, width, height);

    // 3. Angular/conic gradient (color wheel)
    let angular_grad = SceneNode::new(NodeContent::Styled {
        style: Box::new(
            VisualStyle::new()
                .fill(Paint::Angular(AngularGradient {
                    center: Vec2::new(0.5, 0.5),
                    angle: 0.0,
                    stops: vec![
                        ColorStop::new(0.0, Vec4::new(1.0, 0.0, 0.0, 1.0)), // Red
                        ColorStop::new(0.33, Vec4::new(0.0, 1.0, 0.0, 1.0)), // Green
                        ColorStop::new(0.66, Vec4::new(0.0, 0.0, 1.0, 1.0)), // Blue
                        ColorStop::new(1.0, Vec4::new(1.0, 0.0, 0.0, 1.0)), // Red (wrap)
                    ],
                }))
                .corner_radius(8.0),
        ),
    });
    let angular_id = scene.add_node(root, angular_grad);
    scene.get_node_mut(angular_id).unwrap().bounds =
        Rect::new(x_start + spacing * 2.0, y, width, height);

    // 4. Diamond gradient
    let diamond_grad = SceneNode::new(NodeContent::Styled {
        style: Box::new(
            VisualStyle::new()
                .fill(Paint::Diamond(DiamondGradient {
                    center: Vec2::new(0.5, 0.5),
                    scale: 0.5,
                    stops: vec![
                        ColorStop::new(0.0, Vec4::new(1.0, 0.5, 0.0, 1.0)), // Orange center
                        ColorStop::new(1.0, Vec4::new(0.5, 0.0, 0.5, 1.0)), // Purple edge
                    ],
                }))
                .corner_radius(8.0),
        ),
    });
    let diamond_id = scene.add_node(root, diamond_grad);
    scene.get_node_mut(diamond_id).unwrap().bounds =
        Rect::new(x_start + spacing * 3.0, y, width, height);
}

/// Row 3: Strokes
fn create_stroke_row(scene: &mut Scene, root: NodeId, y: f32) {
    let x_start = 50.0;
    let spacing = 180.0;
    let width = 150.0;
    let height = 120.0;

    // 1. Stroke center aligned
    let stroke_center = SceneNode::new(NodeContent::Styled {
        style: Box::new(
            VisualStyle::new()
                .solid_fill(Vec4::new(0.95, 0.95, 0.95, 1.0))
                .stroke(StrokeStyle::solid(
                    Paint::Solid(Vec4::new(0.2, 0.4, 0.8, 1.0)),
                    4.0,
                    StrokeAlign::Center,
                ))
                .corner_radius(8.0),
        ),
    });
    let stroke_center_id = scene.add_node(root, stroke_center);
    scene.get_node_mut(stroke_center_id).unwrap().bounds = Rect::new(x_start, y, width, height);

    // 2. Stroke inside aligned
    let stroke_inside = SceneNode::new(NodeContent::Styled {
        style: Box::new(
            VisualStyle::new()
                .solid_fill(Vec4::new(0.9, 0.95, 0.9, 1.0))
                .stroke(StrokeStyle::solid(
                    Paint::Solid(Vec4::new(0.2, 0.8, 0.2, 1.0)),
                    6.0,
                    StrokeAlign::Inside,
                ))
                .corner_radius(8.0),
        ),
    });
    let stroke_inside_id = scene.add_node(root, stroke_inside);
    scene.get_node_mut(stroke_inside_id).unwrap().bounds =
        Rect::new(x_start + spacing, y, width, height);

    // 3. Stroke outside aligned
    let stroke_outside = SceneNode::new(NodeContent::Styled {
        style: Box::new(
            VisualStyle::new()
                .solid_fill(Vec4::new(0.9, 0.9, 0.95, 1.0))
                .stroke(StrokeStyle::solid(
                    Paint::Solid(Vec4::new(0.8, 0.2, 0.2, 1.0)),
                    6.0,
                    StrokeAlign::Outside,
                ))
                .corner_radius(8.0),
        ),
    });
    let stroke_outside_id = scene.add_node(root, stroke_outside);
    scene.get_node_mut(stroke_outside_id).unwrap().bounds =
        Rect::new(x_start + spacing * 2.0, y, width, height);

    // 4. Gradient stroke
    let stroke_gradient = SceneNode::new(NodeContent::Styled {
        style: Box::new(
            VisualStyle::new()
                .solid_fill(Vec4::new(0.95, 0.95, 0.95, 1.0))
                .stroke(StrokeStyle::solid(
                    Paint::Linear(LinearGradient {
                        start: Vec2::new(0.0, 0.5),
                        end: Vec2::new(1.0, 0.5),
                        stops: vec![
                            ColorStop::new(0.0, Vec4::new(1.0, 0.0, 0.5, 1.0)),
                            ColorStop::new(1.0, Vec4::new(0.0, 0.5, 1.0, 1.0)),
                        ],
                    }),
                    5.0,
                    StrokeAlign::Center,
                ))
                .corner_radius(8.0),
        ),
    });
    let stroke_gradient_id = scene.add_node(root, stroke_gradient);
    scene.get_node_mut(stroke_gradient_id).unwrap().bounds =
        Rect::new(x_start + spacing * 3.0, y, width, height);
}

/// Row 4: Drop Shadows
fn create_shadow_row(scene: &mut Scene, root: NodeId, y: f32) {
    let x_start = 50.0;
    let spacing = 180.0;
    let width = 150.0;
    let height = 120.0;

    // 1. Single drop shadow
    let single_shadow = SceneNode::new(NodeContent::Styled {
        style: Box::new(
            VisualStyle::new()
                .solid_fill(Vec4::new(1.0, 1.0, 1.0, 1.0))
                .drop_shadow(Vec2::new(4.0, 4.0), 8.0, Vec4::new(0.0, 0.0, 0.0, 0.4))
                .corner_radius(12.0),
        ),
    });
    let single_shadow_id = scene.add_node(root, single_shadow);
    scene.get_node_mut(single_shadow_id).unwrap().bounds = Rect::new(x_start, y, width, height);

    // 2. Multiple drop shadows
    let multi_shadow = SceneNode::new(NodeContent::Styled {
        style: Box::new(
            VisualStyle::new()
                .solid_fill(Vec4::new(1.0, 1.0, 1.0, 1.0))
                .drop_shadow(Vec2::new(2.0, 2.0), 4.0, Vec4::new(0.0, 0.0, 0.0, 0.3))
                .drop_shadow(Vec2::new(8.0, 8.0), 12.0, Vec4::new(0.0, 0.0, 0.0, 0.2))
                .corner_radius(12.0),
        ),
    });
    let multi_shadow_id = scene.add_node(root, multi_shadow);
    scene.get_node_mut(multi_shadow_id).unwrap().bounds =
        Rect::new(x_start + spacing, y, width, height);

    // 3. Colored shadow
    let colored_shadow = SceneNode::new(NodeContent::Styled {
        style: Box::new(
            VisualStyle::new()
                .solid_fill(Vec4::new(1.0, 1.0, 1.0, 1.0))
                .drop_shadow(
                    Vec2::new(4.0, 4.0),
                    10.0,
                    Vec4::new(0.8, 0.0, 0.4, 0.5), // Pink shadow
                )
                .corner_radius(12.0),
        ),
    });
    let colored_shadow_id = scene.add_node(root, colored_shadow);
    scene.get_node_mut(colored_shadow_id).unwrap().bounds =
        Rect::new(x_start + spacing * 2.0, y, width, height);

    // 4. Large blur radius shadow
    let large_blur_shadow = SceneNode::new(NodeContent::Styled {
        style: Box::new(
            VisualStyle::new()
                .solid_fill(Vec4::new(1.0, 1.0, 1.0, 1.0))
                .drop_shadow(Vec2::new(6.0, 6.0), 20.0, Vec4::new(0.0, 0.0, 0.0, 0.3))
                .corner_radius(12.0),
        ),
    });
    let large_blur_id = scene.add_node(root, large_blur_shadow);
    scene.get_node_mut(large_blur_id).unwrap().bounds =
        Rect::new(x_start + spacing * 3.0, y, width, height);
}

/// Row 5: Combined Effects
fn create_combined_effects_row(scene: &mut Scene, root: NodeId, y: f32) {
    let x_start = 50.0;
    let spacing = 180.0;
    let width = 150.0;
    let height = 120.0;

    // 1. Gradient fill + stroke + shadow (the complete package!)
    let combo1 = SceneNode::new(NodeContent::Styled {
        style: Box::new(
            VisualStyle::new()
                .fill(Paint::Linear(LinearGradient {
                    start: Vec2::new(0.0, 0.0),
                    end: Vec2::new(1.0, 1.0),
                    stops: vec![
                        ColorStop::new(0.0, Vec4::new(0.3, 0.5, 1.0, 1.0)),
                        ColorStop::new(1.0, Vec4::new(0.6, 0.2, 0.9, 1.0)),
                    ],
                }))
                .stroke(StrokeStyle::solid(
                    Paint::Solid(Vec4::new(1.0, 1.0, 1.0, 0.8)),
                    3.0,
                    StrokeAlign::Inside,
                ))
                .drop_shadow(Vec2::new(4.0, 4.0), 12.0, Vec4::new(0.0, 0.0, 0.0, 0.4))
                .corner_radius(16.0),
        ),
    });
    let combo1_id = scene.add_node(root, combo1);
    scene.get_node_mut(combo1_id).unwrap().bounds = Rect::new(x_start, y, width, height);

    // 2. Asymmetric corners + radial gradient + gradient stroke
    let combo2 = SceneNode::new(NodeContent::Styled {
        style: Box::new(
            VisualStyle::new()
                .fill(Paint::Radial(RadialGradient {
                    center: Vec2::new(0.5, 0.5),
                    radius: 0.6,
                    stops: vec![
                        ColorStop::new(0.0, Vec4::new(1.0, 0.8, 0.2, 1.0)),
                        ColorStop::new(1.0, Vec4::new(0.9, 0.2, 0.4, 1.0)),
                    ],
                }))
                .stroke(StrokeStyle::solid(
                    Paint::Linear(LinearGradient {
                        start: Vec2::new(0.0, 0.0),
                        end: Vec2::new(1.0, 1.0),
                        stops: vec![
                            ColorStop::new(0.0, Vec4::new(1.0, 0.0, 0.5, 1.0)),
                            ColorStop::new(1.0, Vec4::new(0.5, 0.0, 1.0, 1.0)),
                        ],
                    }),
                    4.0,
                    StrokeAlign::Center,
                ))
                .corner_radii(CornerRadii::new(24.0, 8.0, 24.0, 8.0)),
        ),
    });
    let combo2_id = scene.add_node(root, combo2);
    scene.get_node_mut(combo2_id).unwrap().bounds = Rect::new(x_start + spacing, y, width, height);

    // 3. Angular gradient + thick stroke + large shadow
    let combo3 = SceneNode::new(NodeContent::Styled {
        style: Box::new(
            VisualStyle::new()
                .fill(Paint::Angular(AngularGradient {
                    center: Vec2::new(0.5, 0.5),
                    angle: 0.0,
                    stops: vec![
                        ColorStop::new(0.0, Vec4::new(0.2, 1.0, 0.4, 1.0)),
                        ColorStop::new(0.5, Vec4::new(0.2, 0.4, 1.0, 1.0)),
                        ColorStop::new(1.0, Vec4::new(0.2, 1.0, 0.4, 1.0)),
                    ],
                }))
                .stroke(StrokeStyle::solid(
                    Paint::Solid(Vec4::new(0.1, 0.1, 0.1, 1.0)),
                    5.0,
                    StrokeAlign::Outside,
                ))
                .drop_shadow(Vec2::new(6.0, 6.0), 16.0, Vec4::new(0.0, 0.5, 0.2, 0.5))
                .corner_radius(12.0),
        ),
    });
    let combo3_id = scene.add_node(root, combo3);
    scene.get_node_mut(combo3_id).unwrap().bounds =
        Rect::new(x_start + spacing * 2.0, y, width, height);

    // 4. Glassmorphism card (semi-transparent with border and shadow)
    let card = SceneNode::new(NodeContent::Styled {
        style: Box::new(
            VisualStyle::new()
                .solid_fill(Vec4::new(1.0, 1.0, 1.0, 0.7))
                .stroke(StrokeStyle::solid(
                    Paint::Solid(Vec4::new(1.0, 1.0, 1.0, 0.5)),
                    2.0,
                    StrokeAlign::Inside,
                ))
                .drop_shadow(Vec2::new(0.0, 8.0), 24.0, Vec4::new(0.0, 0.0, 0.0, 0.15))
                .corner_radius(20.0),
        ),
    });
    let card_id = scene.add_node(root, card);
    scene.get_node_mut(card_id).unwrap().bounds =
        Rect::new(x_start + spacing * 3.0, y, width, height);
}

/// Row 6: Gradient Text - THE PHASE 2 SHOWSTOPPER! 🌈
fn create_gradient_text_row(scene: &mut Scene, root: NodeId, y: f32) {
    let x_start = 50.0;
    let spacing = 300.0;
    let width = 250.0;
    let height = 80.0;

    // 1. Linear gradient text (red → blue, horizontal)
    let linear_text = SceneNode::new(NodeContent::Styled {
        style: Box::new(
            VisualStyle::new()
                .fill(Paint::Linear(LinearGradient {
                    start: Vec2::new(0.0, 0.5),
                    end: Vec2::new(1.0, 0.5),
                    stops: vec![
                        ColorStop::new(0.0, Vec4::new(1.0, 0.2, 0.2, 1.0)), // Red
                        ColorStop::new(1.0, Vec4::new(0.2, 0.4, 1.0, 1.0)), // Blue
                    ],
                }))
                .text(TextContent::new("GRADIENT", 48.0)),
        ),
    });
    let linear_text_id = scene.add_node(root, linear_text);
    scene.get_node_mut(linear_text_id).unwrap().bounds = Rect::new(x_start, y, width, height);

    // 2. Rainbow gradient text (multi-stop)
    let rainbow_text = SceneNode::new(NodeContent::Styled {
        style: Box::new(
            VisualStyle::new()
                .fill(Paint::Linear(LinearGradient {
                    start: Vec2::new(0.0, 0.5),
                    end: Vec2::new(1.0, 0.5),
                    stops: vec![
                        ColorStop::new(0.0, Vec4::new(1.0, 0.0, 0.0, 1.0)), // Red
                        ColorStop::new(0.25, Vec4::new(1.0, 1.0, 0.0, 1.0)), // Yellow
                        ColorStop::new(0.5, Vec4::new(0.0, 1.0, 0.0, 1.0)), // Green
                        ColorStop::new(0.75, Vec4::new(0.0, 0.5, 1.0, 1.0)), // Blue
                        ColorStop::new(1.0, Vec4::new(0.8, 0.0, 1.0, 1.0)), // Purple
                    ],
                }))
                .text(TextContent::new("RAINBOW", 48.0)),
        ),
    });
    let rainbow_text_id = scene.add_node(root, rainbow_text);
    scene.get_node_mut(rainbow_text_id).unwrap().bounds =
        Rect::new(x_start + spacing, y, width, height);

    // 3. Radial gradient text (center burst)
    let radial_text = SceneNode::new(NodeContent::Styled {
        style: Box::new(
            VisualStyle::new()
                .fill(Paint::Radial(RadialGradient {
                    center: Vec2::new(0.5, 0.5),
                    radius: 0.8,
                    stops: vec![
                        ColorStop::new(0.0, Vec4::new(1.0, 1.0, 0.0, 1.0)), // Yellow center
                        ColorStop::new(1.0, Vec4::new(1.0, 0.3, 0.0, 1.0)), // Orange edge
                    ],
                }))
                .text(TextContent::new("RADIAL", 48.0)),
        ),
    });
    let radial_text_id = scene.add_node(root, radial_text);
    scene.get_node_mut(radial_text_id).unwrap().bounds =
        Rect::new(x_start + spacing * 2.0, y, width, height);

    // 4. Angular gradient text with stroke and shadow (THE ULTIMATE SHOWSTOPPER!)
    let ultimate_text = SceneNode::new(NodeContent::Styled {
        style: Box::new(
            VisualStyle::new()
                .fill(Paint::Angular(AngularGradient {
                    center: Vec2::new(0.5, 0.5),
                    angle: 0.0,
                    stops: vec![
                        ColorStop::new(0.0, Vec4::new(1.0, 0.0, 0.5, 1.0)), // Pink
                        ColorStop::new(0.33, Vec4::new(0.5, 0.0, 1.0, 1.0)), // Purple
                        ColorStop::new(0.66, Vec4::new(0.0, 0.8, 1.0, 1.0)), // Cyan
                        ColorStop::new(1.0, Vec4::new(1.0, 0.0, 0.5, 1.0)), // Pink (wrap)
                    ],
                }))
                .stroke(StrokeStyle::solid(
                    Paint::Solid(Vec4::new(0.1, 0.1, 0.1, 0.8)),
                    2.0,
                    StrokeAlign::Center,
                ))
                .drop_shadow(Vec2::new(3.0, 3.0), 8.0, Vec4::new(0.0, 0.0, 0.0, 0.5))
                .text(TextContent::new("PHASE 2!", 48.0)),
        ),
    });
    let ultimate_text_id = scene.add_node(root, ultimate_text);
    scene.get_node_mut(ultimate_text_id).unwrap().bounds =
        Rect::new(x_start + spacing * 3.0, y, width, height);
}

fn main() {
    env_logger::init();
    plat_core::run::<VisualTestPhase2>().expect("Failed to run Phase 2 visual test");
}
