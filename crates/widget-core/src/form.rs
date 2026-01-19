//! Form widget - container for input fields with validation and submission

use crate::{NamedWidgetTuple, Widget, WidgetContext};
use layout_engine::FlexDirection;
use render_engine::{Color, NodeContent, NodeId};
use std::collections::HashMap;
use std::sync::Arc;

/// Form data (field name -> field value)
pub type FormData = HashMap<String, String>;

/// Form submit callback type
pub type SubmitCallback = Arc<dyn Fn(FormData) -> Result<(), String> + Send + Sync>;

/// Form widget that aggregates child input fields with validation and submission
///
/// Use the `form!` macro for declarative construction:
/// ```no_run
/// use widget_core::form;
/// use widget_core::TextInput;
/// use flux_state::{Runtime, Signal};
///
/// let runtime = Runtime::new();
/// let name = Signal::new(runtime.clone(), String::new());
/// let email = Signal::new(runtime.clone(), String::new());
///
/// // Form with tuple-based fields (compile-time typed)
/// let form1 = form!([
///     ("name", TextInput::new(name)),
///     ("email", TextInput::new(email)),
/// ]);
///
/// // Form with options
/// let runtime2 = Runtime::new();
/// let username = Signal::new(runtime2.clone(), String::new());
/// let user_email = Signal::new(runtime2.clone(), String::new());
///
/// let form2 = form!([
///     ("username", TextInput::new(username)),
///     ("email", TextInput::new(user_email)),
/// ], gap: 16.0, padding: 20.0);
/// ```
///
/// Or use the builder pattern directly:
/// ```no_run
/// use widget_core::Form;
/// use widget_core::TextInput;
/// use flux_state::{Runtime, Signal};
///
/// let runtime = Runtime::new();
/// let name = Signal::new(runtime.clone(), String::new());
/// let email = Signal::new(runtime.clone(), String::new());
///
/// let form = Form::new((
///     ("name", TextInput::new(name)),
///     ("email", TextInput::new(email)),
/// ));
/// ```
pub struct Form<F: NamedWidgetTuple> {
    fields: F,
    on_submit: Option<SubmitCallback>,
    gap: f32,
    padding: f32,
}

impl<F: NamedWidgetTuple> Form<F> {
    /// Create a new form with named fields
    pub fn new(fields: F) -> Self {
        Self {
            fields,
            on_submit: None,
            gap: 12.0,
            padding: 16.0,
        }
    }

    /// Set submit callback
    pub fn on_submit<Cb>(mut self, callback: Cb) -> Self
    where
        Cb: Fn(FormData) -> Result<(), String> + Send + Sync + 'static,
    {
        self.on_submit = Some(Arc::new(callback));
        self
    }

    /// Set gap between fields
    pub fn gap(mut self, gap: f32) -> Self {
        self.gap = gap;
        self
    }

    /// Set padding
    pub fn padding(mut self, padding: f32) -> Self {
        self.padding = padding;
        self
    }
}

impl<F: NamedWidgetTuple> Widget for Form<F> {
    fn build(&self, ctx: &mut WidgetContext) -> NodeId {
        // Create form container
        let form_node = ctx.create_node(
            ctx.root(),
            NodeContent::Rect {
                color: Color::rgba(0.95, 0.95, 0.95, 1.0), // Light gray background
            },
        );

        // Build all fields using NamedWidgetTuple trait
        let field_mapping = self.fields.build_all_named(ctx, form_node);

        // Configure layout (vertical column)
        let layout_style = layout_engine::FlexStyle {
            direction: FlexDirection::Column,
            gap: self.gap,
            padding_left: self.padding,
            padding_right: self.padding,
            padding_top: self.padding,
            padding_bottom: self.padding,
            ..Default::default()
        };

        ctx.set_layout_style(form_node, layout_style);

        // Add form state tracking
        ctx.add_form_state(form_node, field_mapping, self.on_submit.clone());

        // Initial validation
        ctx.revalidate_form(form_node);

        form_node
    }
}

/// Create a form with named fields
///
/// # Example
///
/// ```no_run
/// use widget_core::{form, TextInput};
/// use flux_state::{Runtime, Signal};
///
/// let runtime = Runtime::new();
/// let name = Signal::new(runtime.clone(), String::new());
/// let email = Signal::new(runtime.clone(), String::new());
///
/// // Basic form
/// let form1 = form!([
///     ("name", TextInput::new(name)),
///     ("email", TextInput::new(email)),
/// ]);
/// ```
#[macro_export]
macro_rules! form {
    // Fields only - wrap as tuple of pairs
    ([$(($name:expr, $widget:expr)),* $(,)?]) => {{
        $crate::Form::new(($(($name, $widget),)*))
    }};

    // Fields + params
    ([$(($name:expr, $widget:expr)),* $(,)?], $($rest:tt)*) => {{
        let widget = $crate::Form::new(($(($name, $widget),)*));
        $crate::__form_apply!(widget, $($rest)*)
    }};
}

/// Helper macro for applying form! parameters
#[macro_export]
#[doc(hidden)]
macro_rules! __form_apply {
    ($w:expr,) => { $w };
    ($w:expr) => { $w };
    ($w:expr, , $($rest:tt)*) => { $crate::__form_apply!($w, $($rest)*) };

    // Named parameters
    ($w:expr, gap: $v:expr $(, $($rest:tt)*)?) => {
        $crate::__form_apply!($w.gap($v), $($($rest)*)?)
    };
    ($w:expr, padding: $v:expr $(, $($rest:tt)*)?) => {
        $crate::__form_apply!($w.padding($v), $($($rest)*)?)
    };
    ($w:expr, on_submit: $v:expr $(, $($rest:tt)*)?) => {
        $crate::__form_apply!($w.on_submit($v), $($($rest)*)?)
    };
}
