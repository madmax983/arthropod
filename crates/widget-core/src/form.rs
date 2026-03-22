//! Form widget - container for input fields with validation and submission

pub use crate::form_state::{FormData, SubmitCallback};
use crate::{NamedWidgetTuple, Widget, WidgetContext};
use layout_engine::FlexDirection;
use render_engine::{Color, NodeContent, NodeId};
use std::sync::Arc;

/// Form widget that aggregates child input fields with validation and submission
///
/// Forms collect data from child widgets (like `TextInput`) identified by string keys.
/// When submitted, the `on_submit` callback receives a snapshot of all field values.
///
/// # Data Handling
///
/// The `on_submit` callback receives [`FormData`], which is an alias for `HashMap<String, String>`.
/// Keys correspond to the names provided in the form definition.
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
/// // Create a form with named fields and a submission handler
/// let my_form = form!([
///     ("name", TextInput::new(name)),
///     ("email", TextInput::new(email)),
/// ],
/// gap: 16.0,
/// on_submit: |data| {
///     // data is FormData (HashMap<String, String>)
///     let name_val = data.get("name").map(|s| s.as_str()).unwrap_or("");
///     let email_val = data.get("email").map(|s| s.as_str()).unwrap_or("");
///
///     println!("Submitting: {} <{}>", name_val, email_val);
///
///     // Return Ok(()) on success, or Err(String) to display a form-level error
///     if name_val.is_empty() {
///         Err("Name is required".to_string())
///     } else {
///         Ok(())
///     }
/// });
/// ```
///
/// # Builder Usage
///
/// ```no_run
/// use widget_core::{Form, TextInput};
/// use flux_state::{Runtime, Signal};
///
/// let runtime = Runtime::new();
/// let name = Signal::new(runtime.clone(), String::new());
/// let email = Signal::new(runtime.clone(), String::new());
///
/// let form = Form::new((
///     ("name", TextInput::new(name)),
///     ("email", TextInput::new(email)),
/// ))
/// .padding(20.0)
/// .on_submit(|data| {
///     println!("Data: {:?}", data);
///     Ok(())
/// });
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
    ///
    /// The callback receives a [`FormData`] map containing the current values of all fields.
    /// It should return `Ok(())` if submission succeeds, or `Err(String)` with an error message.
    pub fn on_submit<Cb>(mut self, callback: Cb) -> Self
    where
        Cb: Fn(FormData) -> Result<(), String> + Send + Sync + 'static,
    {
        self.on_submit = Some(Arc::new(callback));
        self
    }

    /// Set gap between fields (vertical spacing)
    pub fn gap(mut self, gap: f32) -> Self {
        self.gap = gap;
        self
    }

    /// Set padding around the form
    pub fn padding(mut self, padding: f32) -> Self {
        self.padding = padding;
        self
    }
}

impl<F: NamedWidgetTuple> Widget for Form<F> {
    fn build(&self, ctx: &mut WidgetContext) -> NodeId {
        // Get background from design tokens
        let bg_color = {
            let tokens = ctx.design_tokens();
            match tokens {
                Some(t) => t.surface_secondary.as_color(),
                None => glam::Vec4::new(0.95, 0.95, 0.95, 1.0), // Fallback light gray
            }
        };

        // Create form container with themed background
        let form_node = ctx.create_node(
            ctx.root(),
            NodeContent::Styled {
                style: Box::new(render_engine::VisualStyle::new().solid_fill(
                    Color::rgba(bg_color.x, bg_color.y, bg_color.z, bg_color.w).as_vec4(),
                )),
            },
        );

        // Register background color for theming tests
        ctx.set_background_color(form_node, bg_color);

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

    // Fields with colon syntax: "name": widget
    ([$($name:literal : $widget:expr),* $(,)?]) => {{
        $crate::Form::new(($(($name, $widget),)*))
    }};

    // Fields + params
    ([$(($name:expr, $widget:expr)),* $(,)?], $($rest:tt)*) => {{
        let widget = $crate::Form::new(($(($name, $widget),)*));
        $crate::__form_apply!(widget, $($rest)*)
    }};

    // Fields with colon syntax + params
    ([$($name:literal : $widget:expr),* $(,)?], $($rest:tt)*) => {{
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

    // Catch-all for unknown properties
    ($w:expr, $unknown:ident: $v:expr $(, $($rest:tt)*)?) => {
        compile_error!(concat!("Unknown property or flag: ", stringify!($unknown), ": ", stringify!($v)))
    };
    ($w:expr, $unknown:ident $(, $($rest:tt)*)?) => {
        compile_error!(concat!("Unknown property or flag: ", stringify!($unknown)))
    };
}
