//! MD3 MaterialForm -- column of labeled fields with MD3 spacing.

use crate::theme::MaterialTheme;
use glam::Vec4;
use layout_engine::{FlexDirection, FlexStyle};
use render_engine::node::NodeContent;
use render_engine::{NodeId, TextContent, VisualStyle};
use std::sync::Arc;
use widget_core::WidgetContext;
use widget_core::widget_trait::Widget;

// ---------------------------------------------------------------------------
// MD3 fallback colors
// ---------------------------------------------------------------------------

/// MD3 surface (#FEF7FF)
const FALLBACK_SURFACE: Vec4 = Vec4::new(0.996, 0.969, 1.0, 1.0);

/// MD3 on_surface (#1D1B20)
const FALLBACK_ON_SURFACE: Vec4 = Vec4::new(0.114, 0.106, 0.125, 1.0);

/// Default field gap in dp (MD3 recommended).
const DEFAULT_GAP: f32 = 24.0;

/// Default padding in dp.
const DEFAULT_PADDING: f32 = 24.0;

/// Label font size (title_small in MD3).
const LABEL_SIZE: f32 = 14.0;

/// Inner gap between label and field widget.
const LABEL_FIELD_GAP: f32 = 8.0;

// ---------------------------------------------------------------------------
// MaterialForm
// ---------------------------------------------------------------------------

/// MD3 Form -- column layout of labeled fields with surface background.
///
/// Builds a column with 24dp gap between fields and 24dp padding.
/// Each field entry is rendered as a label followed by the field widget.
///
/// # Example
///
/// ```rust,no_run
/// use flux_state::{Runtime, Signal};
/// use material_ui::components::{MaterialForm, MaterialTextInput};
///
/// let runtime = Runtime::new();
/// let name = Signal::new(runtime.clone(), String::new());
/// let email = Signal::new(runtime, String::new());
///
/// let form = MaterialForm::new()
///     .field("Name", MaterialTextInput::new(name))
///     .field("Email", MaterialTextInput::new(email))
///     .on_submit(|| println!("Submitted!"));
/// ```
pub struct MaterialForm {
    fields: Vec<(String, Box<dyn Widget>)>,
    on_submit: Option<Arc<dyn Fn() + Send + Sync>>,
    gap: f32,
    padding: f32,
}

impl MaterialForm {
    /// Create a new empty form.
    pub fn new() -> Self {
        Self {
            fields: Vec::new(),
            on_submit: None,
            gap: DEFAULT_GAP,
            padding: DEFAULT_PADDING,
        }
    }

    /// Add a labeled field to the form.
    pub fn field(mut self, name: impl Into<String>, widget: impl Widget + 'static) -> Self {
        self.fields.push((name.into(), Box::new(widget)));
        self
    }

    /// Set the callback invoked when the form is submitted.
    pub fn on_submit(mut self, f: impl Fn() + Send + Sync + 'static) -> Self {
        self.on_submit = Some(Arc::new(f));
        self
    }

    /// Override the gap between fields in dp (default: 24.0).
    pub fn gap(mut self, gap: f32) -> Self {
        self.gap = gap;
        self
    }

    /// Override the form padding in dp (default: 24.0).
    pub fn padding(mut self, p: f32) -> Self {
        self.padding = p;
        self
    }
}

impl Default for MaterialForm {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for MaterialForm {
    fn build(&self, ctx: &mut WidgetContext) -> NodeId {
        // -- Resolve theme colors --
        let theme = ctx.get_extension::<MaterialTheme>().cloned();
        let surface = theme
            .as_ref()
            .map(|t| t.color.surface)
            .unwrap_or(FALLBACK_SURFACE);
        let on_surface = theme
            .as_ref()
            .map(|t| t.color.on_surface)
            .unwrap_or(FALLBACK_ON_SURFACE);

        // -- Form container (surface bg, column layout) --
        let form_style = VisualStyle::new().solid_fill(surface);
        let form_container = ctx.create_node(
            ctx.root(),
            NodeContent::Styled {
                style: Box::new(form_style),
            },
        );
        ctx.set_layout_style(
            form_container,
            FlexStyle {
                direction: FlexDirection::Column,
                gap: self.gap,
                padding_left: self.padding,
                padding_right: self.padding,
                padding_top: self.padding,
                padding_bottom: self.padding,
                ..Default::default()
            },
        );

        // -- Build each field as a label + widget sub-column --
        for (field_name, field_widget) in &self.fields {
            // Field wrapper column (label + widget)
            let field_column = ctx.create_node(form_container, NodeContent::Empty);
            ctx.set_layout_style(
                field_column,
                FlexStyle {
                    direction: FlexDirection::Column,
                    gap: LABEL_FIELD_GAP,
                    ..Default::default()
                },
            );

            // Label text
            let label_style = VisualStyle::new()
                .solid_fill(on_surface)
                .text(TextContent::new(field_name.clone(), LABEL_SIZE));
            let label_node = ctx.create_node(
                field_column,
                NodeContent::Styled {
                    style: Box::new(label_style),
                },
            );
            let _ = label_node;

            // Field widget
            let widget_id = field_widget.build(ctx);
            ctx.reparent_to(widget_id, field_column);
        }

        form_container
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use render_engine::Paint;

    #[test]
    fn test_form_builds_fields() {
        let mut ctx = WidgetContext::new_test();
        let form = MaterialForm::new()
            .field("Name", widget_core::Text::new("test"))
            .field("Email", widget_core::Text::new("test"));
        let root_id = form.build(&mut ctx);

        let node = ctx.scene().get_node(root_id).unwrap();
        // Should have 2 field sub-columns
        assert_eq!(
            node.children.len(),
            2,
            "Form with 2 fields should have 2 children, got {}",
            node.children.len()
        );

        // Check surface background
        if let NodeContent::Styled { ref style } = node.content
            && let Paint::Solid(color) = &style.fills[0]
        {
            assert!(
                (color.x - FALLBACK_SURFACE.x).abs() < 0.01,
                "Form should use surface fill, got {color:?}"
            );
        }
    }

    #[test]
    fn test_form_with_submit() {
        let form = MaterialForm::new()
            .field("Name", widget_core::Text::new("test"))
            .on_submit(|| {});
        assert!(form.on_submit.is_some(), "Form should have on_submit");
    }

    #[test]
    fn test_form_custom_spacing() {
        let form = MaterialForm::new().gap(12.0).padding(32.0);
        assert!(
            (form.gap - 12.0).abs() < f32::EPSILON,
            "Custom gap should be 12.0, got {}",
            form.gap
        );
        assert!(
            (form.padding - 32.0).abs() < f32::EPSILON,
            "Custom padding should be 32.0, got {}",
            form.padding
        );
    }
}
