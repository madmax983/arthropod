use super::runtime::StoryRuntime;
use arthropod_ecs::SceneNodeRef;
use bevy_ecs::prelude::*;
use render_engine::{NodeContent, Scene, SceneNode, TextContent, VisualStyle};

/// Marker component for an entity that should render the current story state
#[derive(Component, Default)]
pub struct NarrativeGenerator;

/// System to update the visual representation of the story
pub fn story_view_system(
    runtime: Res<StoryRuntime>,
    query: Query<&SceneNodeRef, With<NarrativeGenerator>>,
    mut scene: ResMut<Scene>,
) {
    // Only update if the story state has changed
    if !runtime.is_changed() {
        return;
    }

    let text = runtime.current_text();
    let choices = runtime.current_choices();

    for node_ref in query.iter() {
        let root_id = node_ref.0;

        // 1. Clear existing children
        // We clone the children list first to avoid borrow issues while mutating the scene
        let children_to_remove = if let Some(node) = scene.get_node(root_id) {
            node.children.clone()
        } else {
            continue;
        };

        for child_id in children_to_remove {
            scene.remove_node(child_id);
        }

        // 2. Add Passage Text
        let mut text_node = SceneNode::new(NodeContent::Styled {
            style: Box::new(
                VisualStyle::new()
                    .text(TextContent::new(text, 16.0))
                    .solid_fill(render_engine::Vec4::ZERO),
            ),
        });
        // Set some bounds for layout/hit testing (though TUI ignores this usually)
        text_node.bounds = plat_core::Rect::new(0.0, 0.0, 800.0, 600.0);
        scene.add_node(root_id, text_node);

        // 3. Add Choices
        for (i, choice) in choices.iter().enumerate() {
            let choice_text = format!("\n[{}] {}", i + 1, choice.text);
            let choice_node = SceneNode::new(NodeContent::Styled {
                style: Box::new(
                    VisualStyle::new()
                        .text(TextContent::new(choice_text, 14.0))
                        .solid_fill(render_engine::Vec4::ZERO),
                ),
            });
            scene.add_node(root_id, choice_node);
        }
    }
}
