use arthropod::experimental::story::{NarrativeGenerator, register_story};
use arthropod::prelude::*;

fn main() -> Result<(), AppError> {
    println!("I want to generate a story!");

    // This looks like valid code!
    let mut app = App::new_headless()?; // Assuming this exists, based on story_demo
    register_story(&mut app);

    // I expect this to do something
    let _id = app
        .spawn(app.world().resource::<render_engine::Scene>().root())
        .insert(NarrativeGenerator)
        .id();

    app.update();

    println!("App updated. Did anything happen?");
    Ok(())
}
