use widget_macros::Widget;

#[derive(Widget)]
#[style(background = 123)]
pub struct InvalidIdent {
    #[positional]
    pub text: String,
}

fn main() {}
