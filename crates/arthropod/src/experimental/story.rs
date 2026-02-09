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
                        "Once upon a time...".to_string(),
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
