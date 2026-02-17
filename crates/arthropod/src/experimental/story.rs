use bevy_ecs::prelude::*;
use render_engine::{Color, NodeContent, Scene, SceneNode};

/// Component for generating narrative elements
#[derive(Component, Default)]
pub struct NarrativeGenerator;

/// System to generate narrative content
///
/// This system looks for `NarrativeGenerator` components and spawns a "story" (a text node)
/// in the scene if one doesn't exist yet.
pub fn generate_narrative(
    query: Query<Entity, Added<NarrativeGenerator>>,
    mut scene: ResMut<Scene>,
) {
    for _entity in query.iter() {
        println!("NarrativeGenerator: Generating story...");

        // Create a text node for the story
        let node = SceneNode::new(NodeContent::Styled {
            style: Box::new(
                render_engine::VisualStyle::new()
                    .solid_fill(Color::BLACK.as_vec4())
                    .text(render_engine::TextContent::new(
                        "# The Crystal Spire\n\n## Chapter 1: The Descent\n\nThe wind howled around the **obsidian** peaks, carrying whispers of ancient magic. Below, the valley was shrouded in a **thick, purple mist**.\n\n## Chapter 2: The Guardian\n\nA lone figure stood at the gate, clad in shimmering armor that reflected the **dying light** of the twin suns. \"None shall pass,\" the Guardian intoned, voice resonating like thunder.\n\n## Chapter 3: The Choice\n\nWould you **fight** or would you **flee**? The destiny of the realm hung in the balance.",
                        32.0,
                    )),
            ),
        });

        // Add to scene root
        let root = scene.root();
        scene.add_node(root, node);
    }
}

/// Register story systems with the application
pub fn register_story(app: &mut crate::App) {
    app.add_update_system(generate_narrative);
}
