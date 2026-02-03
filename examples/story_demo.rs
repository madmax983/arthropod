#[cfg(feature = "nova")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use arthropod::prelude::*;
    use arthropod::experimental::story::{register_story, NarrativeGenerator};

    println!("Starting Story Demo (Nova Feature Enabled)");

    // Create headless app for demo purposes
    let mut app = App::new_headless()?;

    // Register story system
    register_story(&mut app);

    // Spawn the generator component
    // We attach it to the root node entity, but any entity would do
    let root_id = app.world().resource::<Scene>().root();
    app.spawn(root_id)
        .insert(NarrativeGenerator);

    println!("Updating app to generate story...");
    // Run update loop once
    app.update();

    // Verify results
    let scene = app.world().resource::<Scene>();
    let root = scene.root();
    let root_node = scene.get_node(root).unwrap();

    // Check children
    if !root_node.children.is_empty() {
        println!("SUCCESS: Narrative generated!");

        // Find the text node
        let mut found = false;
        for &child_id in &root_node.children {
if let Some(text) = scene.get_node(child_id).and_then(|child| match &child.content {
    NodeContent::Text { text, .. } => Some(text),
    _ => None,
}) {
    println!("Story says: \"{}\"", text);
    found = true;
}
             if let Some(child) = scene.get_node(child_id) {
                 if let NodeContent::Text { text, .. } = &child.content {
                     println!("Story says: \"{}\"", text);
                     found = true;
                 }
             }
        }

        if !found {
            println!("FAILURE: Child found but not text.");
        }
    } else {
        println!("FAILURE: No narrative generated (no children on root).");
    }

    Ok(())
}

#[cfg(not(feature = "nova"))]
fn main() {
    eprintln!("Error: This example requires the 'nova' feature.");
    eprintln!("Try running with:");
    eprintln!("    cargo run --example story_demo --features nova");
    std::process::exit(1);
}
