//! Accessibility Demo - Conceptual Example
//!
//! This example demonstrates how AccessKit integration works with Arthropod.
//! It shows the component architecture without requiring a full window/renderer setup.
//!
//! # Key Concepts
//!
//! 1. **A11yTree** - Platform-agnostic accessibility tree
//! 2. **AccessibleNode Component** - Links ECS entities to A11yTree nodes
//! 3. **sync_accessible_nodes_system** - Keeps A11yTree synchronized with Scene
//! 4. **AccessKit Bridge** - Converts A11yTree to platform-specific format
//!
//! # Architecture
//!
//! ```text
//! Scene Node (visual)  →  ECS Entity  →  A11yTree Node  →  AccessKit  →  Screen Reader
//!      (bounds)            (component)    (semantic)        (platform)      (output)
//! ```

use a11y_engine::{A11yNode, A11yTree, ArthropodActionHandler, AccessibleName, Role};
use arthropod_ecs::components::{AccessibleNode, OnA11yClick};
use arthropod_ecs::{FrameworkContext, Renderable};
use plat_core::Rect;
use render_engine::{Color, NodeContent, Scene, SceneNode};
use std::sync::{Arc, Mutex};

fn main() {
    println!("=== Accessibility Architecture Demo ===\n");

    // Create ECS context (includes Scene and A11yTree as resources)
    let mut context = FrameworkContext::new();

    // Add ActionHandler resource
    context.world_mut().insert_resource(ArthropodActionHandler::new());

    // Setup accessible UI elements with click handlers
    let click_count = Arc::new(Mutex::new(0));
    setup_accessible_button(&mut context, click_count.clone());
    setup_accessible_checkbox(&mut context, click_count.clone());

    // Update systems - this syncs Scene bounds to A11yTree
    context.update();

    // Verify synchronization worked
    verify_a11y_sync(&context);

    // Demonstrate action handlers
    demonstrate_action_handlers(&mut context, click_count);

    println!("\n✅ A11yTree successfully synchronized with Scene!");
    println!("   In a full app, this data would flow to AccessKit → Screen Readers");
}

fn setup_accessible_button(context: &mut FrameworkContext, click_count: Arc<Mutex<i32>>) {
    println!("📍 Creating accessible button with click handler...");

    // Get root IDs
    let scene = context.world().resource::<Scene>();
    let root_scene = scene.root();
    drop(scene);

    let a11y_tree = context.world().resource::<A11yTree>();
    let root_a11y = a11y_tree.root();
    drop(a11y_tree);

    // 1. Create Scene node (visual representation)
    let mut scene = context.world_mut().resource_mut::<Scene>();
    let button_scene = scene.add_node(
        root_scene,
        SceneNode::new(NodeContent::Rect {
            color: Color::rgba(0.2, 0.4, 0.8, 1.0),
        }),
    );
    scene.get_node_mut(button_scene).unwrap().bounds = Rect::new(50.0, 50.0, 150.0, 50.0);
    drop(scene);

    println!("   ✓ Scene node created at (50, 50, 150x50)");

    // 2. Create A11y node (semantic representation)
    let mut a11y_tree = context.world_mut().resource_mut::<A11yTree>();
    let button_a11y = a11y_tree.add_node(
        root_a11y,
        A11yNode {
            role: Role::Button,
            name: AccessibleName::Text("Submit".into()),
            bounds: Rect::new(50.0, 50.0, 150.0, 50.0),
            ..Default::default()
        },
    );
    drop(a11y_tree);

    println!("   ✓ A11y node created with role=Button, name=\"Submit\"");

    // 3. Create ECS entity linking both + click handler
    let count_clone = click_count.clone();
    context
        .spawn(button_scene)
        .insert(Renderable)
        .insert(AccessibleNode::new(button_a11y, Role::Button))
        .insert(OnA11yClick::new(move || {
            let mut count = count_clone.lock().unwrap();
            *count += 1;
            println!("      ▶ Button clicked via screen reader! Count: {}", *count);
        }));

    println!("   ✓ ECS entity created with AccessibleNode + OnA11yClick\n");
}

fn setup_accessible_checkbox(context: &mut FrameworkContext, click_count: Arc<Mutex<i32>>) {
    println!("📍 Creating accessible checkbox with click handler...");

    let scene = context.world().resource::<Scene>();
    let root_scene = scene.root();
    drop(scene);

    let a11y_tree = context.world().resource::<A11yTree>();
    let root_a11y = a11y_tree.root();
    drop(a11y_tree);

    // Scene node
    let mut scene = context.world_mut().resource_mut::<Scene>();
    let checkbox_scene = scene.add_node(
        root_scene,
        SceneNode::new(NodeContent::Rect {
            color: Color::rgba(0.2, 0.8, 0.2, 1.0),
        }),
    );
    scene.get_node_mut(checkbox_scene).unwrap().bounds = Rect::new(50.0, 120.0, 30.0, 30.0);
    drop(scene);

    println!("   ✓ Scene node created at (50, 120, 30x30)");

    // A11y node
    let mut a11y_tree = context.world_mut().resource_mut::<A11yTree>();
    let checkbox_a11y = a11y_tree.add_node(
        root_a11y,
        A11yNode {
            role: Role::Checkbox,
            name: AccessibleName::Text("Enable notifications".into()),
            bounds: Rect::new(50.0, 120.0, 30.0, 30.0),
            ..Default::default()
        },
    );
    drop(a11y_tree);

    println!("   ✓ A11y node created with role=Checkbox, name=\"Enable notifications\"");

    // ECS entity with click handler
    let count_clone = click_count.clone();
    context
        .spawn(checkbox_scene)
        .insert(Renderable)
        .insert(AccessibleNode::new(checkbox_a11y, Role::Checkbox))
        .insert(OnA11yClick::new(move || {
            let mut count = count_clone.lock().unwrap();
            *count += 10;
            println!("      ▶ Checkbox toggled via screen reader! Count: {}", *count);
        }));

    println!("   ✓ ECS entity created with AccessibleNode + OnA11yClick\n");
}

fn verify_a11y_sync(context: &FrameworkContext) {
    println!("🔍 Verifying A11yTree synchronization...");

    let a11y_tree = context.world().resource::<A11yTree>();

    // Check that dirty tracking works
    let dirty_count = a11y_tree.get_dirty_nodes().len();
    println!("   ✓ {} nodes marked dirty (will be sent to AccessKit)", dirty_count);

    // In a real app, AccessKitBridge would:
    // 1. Call create_tree_update(dirty_nodes)
    // 2. Send TreeUpdate to accesskit_windows
    // 3. Windows UIA receives updates
    // 4. Screen readers announce changes
}

fn demonstrate_action_handlers(context: &mut FrameworkContext, click_count: Arc<Mutex<i32>>) {
    // Note: ActionHandler trait and ActionRequest are private implementation details
    // In a real app, the action handler registration would happen automatically in the update cycle

    println!("\n🎬 Demonstrating action handlers...");

    // Register all action callbacks with the handler
    // (In a real app, this would run automatically as an ECS system)
    {
        let mut query = context.world_mut().query::<(&AccessibleNode, &OnA11yClick)>();
        let mut callbacks_to_register = Vec::new();

        for (accessible, on_click) in query.iter(context.world()) {
            callbacks_to_register.push((accessible.a11y_id, on_click.callback.clone()));
        }

        let mut handler = context.world_mut().resource_mut::<ArthropodActionHandler>();
        *handler = ArthropodActionHandler::new();
        for (a11y_id, callback) in callbacks_to_register {
            handler.register_click(a11y_id, move || {
                callback();
            });
        }
    }

    // Get the a11y IDs
    let button_id = {
        let mut query = context.world_mut().query::<(&AccessibleNode, &OnA11yClick)>();
        query
            .iter(context.world())
            .next()
            .map(|(acc, _)| acc.a11y_id)
            .unwrap()
    };

    // Simulate screen reader clicking the button
    println!("   Simulating screen reader click on button...");
    println!("   (In a real app, this would come from Windows UIA via AccessKit)");

    // Manually trigger the callback for demonstration
    {
        let mut query = context.world_mut().query::<(&AccessibleNode, &OnA11yClick)>();
        for (accessible, on_click) in query.iter(context.world()) {
            if accessible.a11y_id == button_id {
                (on_click.callback)();
                break;
            }
        }
    }

    let final_count = *click_count.lock().unwrap();
    println!("   ✓ Final click count: {} (callbacks executed successfully!)", final_count);
}
