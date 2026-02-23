// Nova Story Engine
//
// Provides interactive narrative generation and runtime.

pub mod model;
pub mod runtime;
pub mod system;

pub use self::model::*;
pub use self::runtime::*;
pub use self::system::*;

/// Register story engine components and systems
pub fn register_story(app: &mut crate::App) {
    app.world_mut().init_resource::<StoryRuntime>();
    app.add_update_system(system::story_view_system);
}
