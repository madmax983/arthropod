use super::model::{Choice, PassageId, Story};
use bevy_ecs::prelude::Resource;

/// Runtime state for the Story Engine
#[derive(Resource)]
pub struct StoryRuntime {
    /// The static story data
    pub story: Story,
    /// The ID of the currently active passage
    pub current_passage: PassageId,
    /// History of visited passages
    pub history: Vec<PassageId>,
}

impl Default for StoryRuntime {
    fn default() -> Self {
        // Load the demo story by default if no story is provided
        let story = Story::demo();
        let start = story.start_node.clone();
        Self {
            story,
            current_passage: start,
            history: Vec::new(),
        }
    }
}

impl StoryRuntime {
    /// Create a new runtime for the given story
    pub fn new(story: Story) -> Self {
        let start = story.start_node.clone();
        Self {
            story,
            current_passage: start,
            history: Vec::new(),
        }
    }

    /// Get the text of the current passage
    pub fn current_text(&self) -> &str {
        self.story
            .nodes
            .get(&self.current_passage)
            .map(|p| p.text.as_str())
            .unwrap_or("Error: Passage not found.")
    }

    /// Get the available choices for the current passage
    pub fn current_choices(&self) -> &[Choice] {
        self.story
            .nodes
            .get(&self.current_passage)
            .map(|p| p.choices.as_slice())
            .unwrap_or(&[])
    }

    /// Advance the story by selecting a choice index (0-based)
    pub fn choose(&mut self, index: usize) -> Result<(), String> {
        // We need to clone the target ID to avoid borrowing issues
        let target = {
            let choices = self.current_choices();
            if index >= choices.len() {
                return Err(format!("Invalid choice index: {}", index));
            }
            choices[index].target.clone()
        };

        self.history.push(self.current_passage.clone());
        self.current_passage = target;
        Ok(())
    }
}
