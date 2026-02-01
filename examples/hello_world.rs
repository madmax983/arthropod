use arthropod::prelude::*;
use widget_core::{btn, col, txt};

fn main() -> Result<(), AppError> {
    App::run("Hello Arthropod", 400, 300, |_ctx| {
        col!(
            [
                txt!("Hello, World!", size: 24.0),
                btn!("Click Me", primary, on_click: || println!("Button clicked!")),
            ],
            gap: 20.0,
            padding: 20.0
        )
    })
}
