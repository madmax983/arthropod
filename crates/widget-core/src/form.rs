//! Form widget - container for input fields with validation and submission

use crate::{Widget, WidgetContext};
use render_engine::{NodeId, NodeContent, Color};
use layout_engine::FlexDirection;
use std::collections::HashMap;
use std::sync::Arc;

/// Form data (field name -> field value)
pub type FormData = HashMap<String, String>;

/// Form submit callback type
pub type SubmitCallback = Arc<dyn Fn(FormData) -> Result<(), String> + Send + Sync>;

/// Form widget that aggregates child input fields
///
/// # Example
///
/// ```no_run
/// use widget_core::{Form, TextInput};
/// use flux_state::{Runtime, Signal};
///
/// let runtime = Runtime::new();
/// let name = Signal::new(runtime.clone(), String::new());
/// let email = Signal::new(runtime.clone(), String::new());
///
/// let form = Form::new()
///     .field("name", TextInput::new(name))
///     .field("email", TextInput::new(email))
///     .on_submit(|data| {
///         println!("Submitted: {:?}", data);
///         Ok(())
///     });
/// ```
pub struct Form {
    fields: Vec<(String, Box<dyn Widget>)>,
    on_submit: Option<SubmitCallback>,
    gap: f32,
    padding: f32,
}

impl Form {
    /// Create a new form
    pub fn new() -> Self {
        Self {
            fields: Vec::new(),
            on_submit: None,
            gap: 12.0,
            padding: 16.0,
        }
    }

    /// Add a field to the form
    pub fn field(mut self, name: impl Into<String>, widget: impl Widget + 'static) -> Self {
        self.fields.push((name.into(), Box::new(widget)));
        self
    }

    /// Set submit callback
    pub fn on_submit<F>(mut self, callback: F) -> Self
    where
        F: Fn(FormData) -> Result<(), String> + Send + Sync + 'static,
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

impl Default for Form {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for Form {
    fn build(&self, ctx: &mut WidgetContext) -> NodeId {
        // Create form container
        let form_node = ctx.create_node(
            ctx.root(),
            NodeContent::Rect {
                color: Color::rgba(0.95, 0.95, 0.95, 1.0), // Light gray background
            },
        );

        // Build all fields and collect their IDs
        let mut field_mapping = HashMap::new();
        let mut field_ids = Vec::new();

        for (field_name, field_widget) in &self.fields {
            let field_id = field_widget.build(ctx);
            field_ids.push(field_id);
            field_mapping.insert(field_name.clone(), field_id);
        }

        // Re-parent fields to form
        let root_id = ctx.root();
        for field_id in &field_ids {
            ctx.reparent_node(*field_id, root_id, form_node);
        }

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
        ctx.add_form_state(
            form_node,
            field_mapping,
            self.on_submit.clone(),
        );

        // Initial validation
        ctx.revalidate_form(form_node);

        form_node
    }
}
