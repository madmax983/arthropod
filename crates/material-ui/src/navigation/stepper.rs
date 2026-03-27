//! MD3 Stepper widget -- multi-step progress indicator.

use crate::theme::MaterialTheme;
use glam::Vec4;
use layout_engine::{FlexAlign, FlexDirection, FlexJustifyContent, FlexStyle};
use render_engine::node::NodeContent;
use render_engine::{NodeId, TextContent, VisualStyle};
use widget_core::WidgetContext;
use widget_core::widget_trait::Widget;

// ---------------------------------------------------------------------------
// MD3 fallback colors
// ---------------------------------------------------------------------------

/// MD3 primary (#6750A4) -- active/completed step circle and label.
const FALLBACK_PRIMARY: Vec4 = Vec4::new(0.404, 0.314, 0.643, 1.0);

/// MD3 on_surface (#1D1B20) -- completed step label.
const FALLBACK_ON_SURFACE: Vec4 = Vec4::new(0.114, 0.106, 0.125, 1.0);

/// MD3 on_surface_variant (#49454F) -- disabled step label.
const FALLBACK_ON_SURFACE_VARIANT: Vec4 = Vec4::new(0.286, 0.271, 0.310, 1.0);

/// MD3 surface_variant (#E7E0EB) -- connector lines.
const FALLBACK_SURFACE_VARIANT: Vec4 = Vec4::new(0.906, 0.878, 0.925, 1.0);

/// MD3 outline (#79747E) -- disabled step circle outline.
const FALLBACK_OUTLINE: Vec4 = Vec4::new(0.475, 0.455, 0.494, 1.0);

/// MD3 error (#B3261E) -- error step circle and label.
const FALLBACK_ERROR: Vec4 = Vec4::new(0.702, 0.149, 0.118, 1.0);

/// White color for step number text inside filled circles.
const WHITE: Vec4 = Vec4::new(1.0, 1.0, 1.0, 1.0);

// ---------------------------------------------------------------------------
// Dimension constants (dp)
// ---------------------------------------------------------------------------

/// Circle diameter for step indicators.
const CIRCLE_SIZE: f32 = 24.0;

/// Font size for step number inside circle.
const NUMBER_FONT_SIZE: f32 = 12.0;

/// Font size for step label.
const LABEL_FONT_SIZE: f32 = 14.0;

/// Connector line thickness.
const CONNECTOR_THICKNESS: f32 = 1.0;

/// Connector line length (horizontal).
const CONNECTOR_LENGTH: f32 = 32.0;

/// Gap between circle and label.
const CIRCLE_LABEL_GAP: f32 = 8.0;

/// State of an individual step in a [`Stepper`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum StepState {
    /// The step is currently active (in progress).
    Active,
    /// The step has been completed successfully.
    Completed,
    /// The step encountered an error.
    Error,
    /// The step is not yet reached (default).
    #[default]
    Disabled,
}

/// A single step definition for use in a [`Stepper`].
///
/// # Example
///
/// ```rust,no_run
/// use material_ui::navigation::Step;
///
/// let step = Step::new("Shipping").completed();
/// ```
pub struct Step {
    /// Display label for the step.
    pub label: String,
    /// Current state of the step.
    pub state: StepState,
}

impl Step {
    /// Create a new step with the given label (default state: [`StepState::Disabled`]).
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            state: StepState::Disabled,
        }
    }

    /// Mark the step as active.
    pub fn active(mut self) -> Self {
        self.state = StepState::Active;
        self
    }

    /// Mark the step as completed.
    pub fn completed(mut self) -> Self {
        self.state = StepState::Completed;
        self
    }

    /// Mark the step as having an error.
    pub fn error(mut self) -> Self {
        self.state = StepState::Error;
        self
    }
}

/// MD3 Stepper -- multi-step progress indicator.
///
/// Renders steps connected by lines. Each step shows a circle with the step
/// number (or a checkmark for completed steps) and a label. Supports both
/// horizontal and vertical orientations.
///
/// # Example
///
/// ```rust,no_run
/// use material_ui::navigation::{Step, Stepper};
///
/// let stepper = Stepper::new(vec![
///     Step::new("Cart").completed(),
///     Step::new("Shipping").active(),
///     Step::new("Payment"),
///     Step::new("Review"),
/// ]);
/// ```
pub struct Stepper {
    steps: Vec<Step>,
    orientation: FlexDirection,
}

impl Stepper {
    /// Create a new horizontal stepper with the given steps.
    pub fn new(steps: Vec<Step>) -> Self {
        Self {
            steps,
            orientation: FlexDirection::Row,
        }
    }

    /// Set the stepper orientation to vertical (column layout).
    pub fn vertical(mut self) -> Self {
        self.orientation = FlexDirection::Column;
        self
    }
}

impl Widget for Stepper {
    fn build(&self, ctx: &mut WidgetContext) -> NodeId {
        // -- Resolve theme colors --
        let theme = ctx.get_extension::<MaterialTheme>().cloned();
        let primary = theme
            .as_ref()
            .map(|t| t.color.primary)
            .unwrap_or(FALLBACK_PRIMARY);
        let on_surface = theme
            .as_ref()
            .map(|t| t.color.on_surface)
            .unwrap_or(FALLBACK_ON_SURFACE);
        let on_surface_variant = theme
            .as_ref()
            .map(|t| t.color.on_surface_variant)
            .unwrap_or(FALLBACK_ON_SURFACE_VARIANT);
        let surface_variant = theme
            .as_ref()
            .map(|t| t.color.surface_variant)
            .unwrap_or(FALLBACK_SURFACE_VARIANT);
        let outline = theme
            .as_ref()
            .map(|t| t.color.outline)
            .unwrap_or(FALLBACK_OUTLINE);
        let error = theme
            .as_ref()
            .map(|t| t.color.error)
            .unwrap_or(FALLBACK_ERROR);

        let is_vertical = self.orientation == FlexDirection::Column;

        // -- Root container --
        let root = ctx.create_node(ctx.root(), NodeContent::Empty);
        ctx.set_layout_style(
            root,
            FlexStyle {
                direction: self.orientation,
                align_items: FlexAlign::Center,
                ..Default::default()
            },
        );

        for (i, step) in self.steps.iter().enumerate() {
            // -- Build step indicator --
            let step_container = ctx.create_node(root, NodeContent::Empty);
            let step_direction = if is_vertical {
                FlexDirection::Row
            } else {
                FlexDirection::Column
            };
            ctx.set_layout_style(
                step_container,
                FlexStyle {
                    direction: step_direction,
                    align_items: FlexAlign::Center,
                    gap: CIRCLE_LABEL_GAP,
                    ..Default::default()
                },
            );

            // Determine colors based on step state
            let (circle_fill, circle_text_color, label_color, circle_text) = match step.state {
                StepState::Active => (primary, WHITE, primary, format!("{}", i + 1)),
                StepState::Completed => (primary, WHITE, on_surface, "\u{2713}".to_owned()),
                StepState::Error => (error, WHITE, error, "!".to_owned()),
                StepState::Disabled => (outline, WHITE, on_surface_variant, format!("{}", i + 1)),
            };

            // Circle node (filled background with centered number/icon)
            let circle_container = ctx.create_node(step_container, NodeContent::Empty);
            ctx.set_layout_style(
                circle_container,
                FlexStyle {
                    width: Some(CIRCLE_SIZE),
                    height: Some(CIRCLE_SIZE),
                    justify_content: FlexJustifyContent::Center,
                    align_items: FlexAlign::Center,
                    ..Default::default()
                },
            );

            // Circle background
            let circle_bg_style = VisualStyle::new()
                .solid_fill(circle_fill)
                .corner_radius(CIRCLE_SIZE / 2.0);
            let circle_bg = ctx.create_node(
                circle_container,
                NodeContent::Styled {
                    style: Box::new(circle_bg_style),
                },
            );
            ctx.set_layout_style(
                circle_bg,
                FlexStyle {
                    width: Some(CIRCLE_SIZE),
                    height: Some(CIRCLE_SIZE),
                    justify_content: FlexJustifyContent::Center,
                    align_items: FlexAlign::Center,
                    ..Default::default()
                },
            );

            // Step number/icon text
            let number_style = VisualStyle::new()
                .solid_fill(circle_text_color)
                .text(TextContent::new(circle_text, NUMBER_FONT_SIZE));
            ctx.create_node(
                circle_bg,
                NodeContent::Styled {
                    style: Box::new(number_style),
                },
            );

            // Label text
            let label_style = VisualStyle::new()
                .solid_fill(label_color)
                .text(TextContent::new(step.label.clone(), LABEL_FONT_SIZE));
            ctx.create_node(
                step_container,
                NodeContent::Styled {
                    style: Box::new(label_style),
                },
            );

            // -- Connector line between steps (not after the last) --
            if i < self.steps.len() - 1 {
                let connector_style = VisualStyle::new().solid_fill(surface_variant);
                let connector = ctx.create_node(
                    root,
                    NodeContent::Styled {
                        style: Box::new(connector_style),
                    },
                );

                let (w, h) = if is_vertical {
                    (CONNECTOR_THICKNESS, CONNECTOR_LENGTH)
                } else {
                    (CONNECTOR_LENGTH, CONNECTOR_THICKNESS)
                };

                ctx.set_layout_style(
                    connector,
                    FlexStyle {
                        width: Some(w),
                        height: Some(h),
                        ..Default::default()
                    },
                );
            }
        }

        root
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use render_engine::Paint;

    #[test]
    fn test_stepper_builds_steps() {
        let mut ctx = WidgetContext::new_test();
        let stepper = Stepper::new(vec![
            Step::new("Step 1"),
            Step::new("Step 2"),
            Step::new("Step 3"),
        ]);
        let root_id = stepper.build(&mut ctx);

        let root_node = ctx.scene().get_node(root_id).unwrap();
        // 3 step containers + 2 connectors = 5 children
        assert_eq!(
            root_node.children.len(),
            5,
            "3 steps + 2 connectors = 5 children, got {}",
            root_node.children.len()
        );
    }

    #[test]
    fn test_stepper_active_completed_error_styling() {
        let mut ctx = WidgetContext::new_test();
        let stepper = Stepper::new(vec![
            Step::new("Done").completed(),
            Step::new("Current").active(),
            Step::new("Failed").error(),
            Step::new("Later"), // Disabled (default)
        ]);
        let root_id = stepper.build(&mut ctx);

        let scene = ctx.scene();
        let root_node = scene.get_node(root_id).unwrap();

        // Helper: extract the circle background fill color from a step container
        // Step container structure: [circle_container, label_text]
        // circle_container -> [circle_bg -> [number_text]]
        let get_circle_fill = |step_index: usize| -> Vec4 {
            // root children: step0, conn, step1, conn, step2, conn, step3
            let child_index = step_index * 2; // steps at even indices, connectors at odd
            let step_container_id = root_node.children[child_index];
            let step_container = scene.get_node(step_container_id).unwrap();
            let circle_container_id = step_container.children[0];
            let circle_container = scene.get_node(circle_container_id).unwrap();
            let circle_bg_id = circle_container.children[0];
            let circle_bg = scene.get_node(circle_bg_id).unwrap();

            if let NodeContent::Styled { ref style } = circle_bg.content
                && let Some(Paint::Solid(color)) = style.fills.first()
            {
                *color
            } else {
                panic!("Circle background should be Styled with solid fill");
            }
        };

        // Completed step: primary color circle
        let completed_color = get_circle_fill(0);
        assert!(
            (completed_color.x - FALLBACK_PRIMARY.x).abs() < 0.01,
            "Completed step should have primary circle, got {completed_color:?}"
        );

        // Active step: primary color circle
        let active_color = get_circle_fill(1);
        assert!(
            (active_color.x - FALLBACK_PRIMARY.x).abs() < 0.01,
            "Active step should have primary circle, got {active_color:?}"
        );

        // Error step: error color circle
        let error_color = get_circle_fill(2);
        assert!(
            (error_color.x - FALLBACK_ERROR.x).abs() < 0.01,
            "Error step should have error circle, got {error_color:?}"
        );

        // Disabled step: outline color circle
        let disabled_color = get_circle_fill(3);
        assert!(
            (disabled_color.x - FALLBACK_OUTLINE.x).abs() < 0.01,
            "Disabled step should have outline circle, got {disabled_color:?}"
        );
    }

    #[test]
    fn test_stepper_vertical_layout() {
        let mut ctx = WidgetContext::new_test();
        let stepper = Stepper::new(vec![Step::new("A").active(), Step::new("B")]).vertical();

        assert_eq!(
            stepper.orientation,
            FlexDirection::Column,
            "Vertical stepper should have Column orientation"
        );

        let root_id = stepper.build(&mut ctx);

        let root_node = ctx.scene().get_node(root_id).unwrap();
        // 2 steps + 1 connector = 3 children
        assert_eq!(
            root_node.children.len(),
            3,
            "2 steps + 1 connector = 3 children, got {}",
            root_node.children.len()
        );
    }

    #[test]
    fn test_stepper_connector_lines() {
        let mut ctx = WidgetContext::new_test();
        let stepper = Stepper::new(vec![
            Step::new("First").completed(),
            Step::new("Second").active(),
        ]);
        let root_id = stepper.build(&mut ctx);

        let scene = ctx.scene();
        let root_node = scene.get_node(root_id).unwrap();

        // Connector is the second child (index 1) -- between step 0 and step 1
        let connector_id = root_node.children[1];
        let connector_node = scene.get_node(connector_id).unwrap();

        if let NodeContent::Styled { ref style } = connector_node.content {
            assert!(!style.fills.is_empty(), "Connector should have a fill");
            if let Paint::Solid(color) = &style.fills[0] {
                assert!(
                    (color.x - FALLBACK_SURFACE_VARIANT.x).abs() < 0.01
                        && (color.y - FALLBACK_SURFACE_VARIANT.y).abs() < 0.01,
                    "Connector should use surface_variant color, got {color:?}"
                );
            } else {
                panic!("Connector fill should be solid");
            }
        } else {
            panic!("Connector should be Styled content");
        }
    }
}
