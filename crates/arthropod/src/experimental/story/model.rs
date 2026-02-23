use std::collections::HashMap;

/// Unique identifier for a passage in the story
pub type PassageId = String;

/// A choice that links to another passage
#[derive(Debug, Clone)]
pub struct Choice {
    /// The text displayed to the user
    pub text: String,
    /// The ID of the target passage
    pub target: PassageId,
}

impl Choice {
    pub fn new(text: impl Into<String>, target: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            target: target.into(),
        }
    }
}

/// A single node in the narrative graph
#[derive(Debug, Clone)]
pub struct Passage {
    pub id: PassageId,
    /// The main text content of the passage
    pub text: String,
    /// Available choices (edges)
    pub choices: Vec<Choice>,
}

impl Passage {
    pub fn new(id: impl Into<String>, text: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            text: text.into(),
            choices: Vec::new(),
        }
    }

    pub fn add_choice(mut self, text: impl Into<String>, target: impl Into<String>) -> Self {
        self.choices.push(Choice::new(text, target));
        self
    }
}

/// The complete narrative graph
#[derive(Debug, Clone, Default)]
pub struct Story {
    pub start_node: PassageId,
    pub nodes: HashMap<PassageId, Passage>,
}

impl Story {
    pub fn new(start_node: impl Into<String>) -> Self {
        Self {
            start_node: start_node.into(),
            nodes: HashMap::new(),
        }
    }

    pub fn add_passage(&mut self, passage: Passage) {
        self.nodes.insert(passage.id.clone(), passage);
    }

    pub fn get_passage(&self, id: &str) -> Option<&Passage> {
        self.nodes.get(id)
    }

    /// Create the default "Crystal Spire" demo story
    pub fn demo() -> Self {
        let mut story = Story::new("start");

        story.add_passage(
            Passage::new("start", "# The Crystal Spire\n\nThe wind howls around the **obsidian** peaks. Below, the valley is shrouded in a **thick, purple mist**.\n\nYou stand at the precipice.")
                .add_choice("Descend into the mist", "mist")
                .add_choice("Climb towards the spire", "spire")
        );

        story.add_passage(
            Passage::new("mist", "## The Mist\n\nThe mist is cold and tastes of copper. Shadows move in the periphery.\n\nYou hear a voice.")
                .add_choice("Call out", "voice_call")
                .add_choice("Hide", "voice_hide")
                .add_choice("Return to the precipice", "start")
        );

        story.add_passage(
            Passage::new("spire", "## The Spire\n\nThe climb is treacherous. The twin suns beat down on your back.\n\nYou reach the Guardian's Gate.")
                .add_choice("Challenge the Guardian", "guardian_fight")
                .add_choice("Offer a tribute", "guardian_tribute")
                .add_choice("Return to the precipice", "start")
        );

        story.add_passage(
            Passage::new("voice_call", "You call out into the mist. The shadows stop moving.\n\nThen, they all turn towards you at once.\n\n**THE END**")
                .add_choice("Restart", "start")
        );

        story.add_passage(
            Passage::new("voice_hide", "You crouch behind a rock. The shadows pass by, whispering ancient secrets.\n\nYou learn the Word of Power.\n\n**THE END (Success)**")
                .add_choice("Restart", "start")
        );

        story.add_passage(
            Passage::new("guardian_fight", "The Guardian laughs. His armor deflects your blow effortlessly.\n\nYou are cast down.\n\n**THE END**")
                .add_choice("Restart", "start")
        );

        story.add_passage(
            Passage::new("guardian_tribute", "You offer your last ration. The Guardian nods solemnly and steps aside.\n\nThe Spire is yours.\n\n**THE END (Success)**")
                .add_choice("Restart", "start")
        );

        story
    }
}
