//! Nova Story Engine
//!
//! Provides an interactive narrative generation and playback runtime.

/// Data structures representing the story graph, passages, and choices.
pub mod model;
/// State machine for executing and tracking story progress.
pub mod runtime;
/// ECS systems that sync the active story passage into visual UI elements.
pub mod system;

pub use self::model::*;
pub use self::runtime::*;
pub use self::system::*;

/// Register story engine components and systems
pub fn register_story(app: &mut crate::App) {
    app.world_mut().init_resource::<StoryRuntime>();
    app.add_update_system(system::story_view_system);
}
