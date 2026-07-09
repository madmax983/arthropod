use widget_macros::Widget;

#[derive(Widget)]
#[style(corner_radius = true)]
pub struct InvalidDimension {
    #[positional]
    pub text: String,
}

fn main() {}
