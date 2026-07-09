use widget_macros::Widget;

#[derive(Widget)]
#[layout(gap = "not_a_float")]
pub struct InvalidFloat {
    #[positional]
    pub text: String,
}

fn main() {}
