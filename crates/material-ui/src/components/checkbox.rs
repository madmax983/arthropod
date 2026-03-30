//! MD3 MaterialCheckbox -- rounded checkbox with indeterminate state.

use crate::theme::MaterialTheme;
use flux_state::Signal;
use glam::Vec4;
use layout_engine::{FlexAlign, FlexDirection, FlexJustifyContent, FlexStyle};
use render_engine::node::NodeContent;
use render_engine::{NodeId, Paint, StrokeStyle, TextContent, VisualStyle};
use std::sync::Arc;
use style_engine::StrokeAlign;
use widget_core::Widget;
use widget_core::WidgetContext;

// ---------------------------------------------------------------------------
// MD3 fallback colors
// ---------------------------------------------------------------------------

/// MD3 primary (#6750A4)
const FALLBACK_PRIMARY: Vec4 = Vec4::new(0.404, 0.314, 0.643, 1.0);

/// MD3 on_primary (white)
const FALLBACK_ON_PRIMARY: Vec4 = Vec4::new(1.0, 1.0, 1.0, 1.0);

/// MD3 on_surface (#1D1B20)
const FALLBACK_ON_SURFACE: Vec4 = Vec4::new(0.114, 0.106, 0.125, 1.0);

/// MD3 outline (#79747E)
const FALLBACK_OUTLINE: Vec4 = Vec4::new(0.475, 0.455, 0.494, 1.0);

/// Transparent fill
const TRANSPARENT: Vec4 = Vec4::new(0.0, 0.0, 0.0, 0.0);

/// Disabled content opacity multiplier.
const DISABLED_ALPHA: f32 = 0.38;

/// Checkbox box size in dp.
const BOX_SIZE: f32 = 18.0;

/// Checkbox corner radius (MD3).
const BOX_CORNER_RADIUS: f32 = 2.0;

/// Check-mark / indeterminate-mark font size.
const MARK_SIZE: f32 = 14.0;

/// Outline border weight for unchecked state.
const OUTLINE_WEIGHT: f32 = 2.0;

/// Gap between box and label.
const LABEL_GAP: f32 = 12.0;

// ---------------------------------------------------------------------------
// MaterialCheckbox
// ---------------------------------------------------------------------------

/// MD3 Checkbox with checked, unchecked, and indeterminate states.
///
/// - **Checked**: primary background with on_primary check mark.
/// - **Unchecked**: transparent background with 2dp outline.
/// - **Indeterminate**: primary background with on_primary dash mark.
///
/// # Example
///
/// ```rust,no_run
/// use flux_state::{Runtime, Signal};
/// use material_ui::components::MaterialCheckbox;
///
/// let runtime = Runtime::new();
/// let agree = Signal::new(runtime, false);
///
/// let checkbox = MaterialCheckbox::new(agree)
///     .label("I agree to the terms");
/// ```
pub struct MaterialCheckbox {
    signal: Signal<bool>,
    label: Option<String>,
    indeterminate: bool,
    disabled: bool,
}

impl MaterialCheckbox {
    /// Create a new checkbox wired to a reactive boolean signal.
    pub fn new(signal: Signal<bool>) -> Self {
        Self {
            signal,
            label: None,
            indeterminate: false,
            disabled: false,
        }
    }

    /// Set a text label displayed to the right of the checkbox.
    pub fn label(mut self, text: impl Into<String>) -> Self {
        self.label = Some(text.into());
        self
    }

    /// Put the checkbox in indeterminate state (shows dash instead of check).
    pub fn indeterminate(mut self) -> Self {
        self.indeterminate = true;
        self
    }

    /// Disable interaction.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

impl Widget for MaterialCheckbox {
    fn build(&self, ctx: &mut WidgetContext) -> NodeId {
        let (read, write) = self.signal.clone().split();
        let is_checked = read.get_untracked();

        // -- Resolve theme colors --
        let theme = ctx.get_extension::<MaterialTheme>().cloned();
        let primary = theme
            .as_ref()
            .map(|t| t.color.primary)
            .unwrap_or(FALLBACK_PRIMARY);
        let on_primary = theme
            .as_ref()
            .map(|t| t.color.on_primary)
            .unwrap_or(FALLBACK_ON_PRIMARY);
        let on_surface = theme
            .as_ref()
            .map(|t| t.color.on_surface)
            .unwrap_or(FALLBACK_ON_SURFACE);
        let outline = theme
            .as_ref()
            .map(|t| t.color.outline)
            .unwrap_or(FALLBACK_OUTLINE);

        // -- Root row container --
        let root = ctx.create_node(ctx.root(), NodeContent::Empty);
        ctx.set_layout_style(
            root,
            FlexStyle {
                direction: FlexDirection::Row,
                gap: LABEL_GAP,
                align_items: FlexAlign::Center,
                ..Default::default()
            },
        );

        // -- Box styling --
        let filled = is_checked || self.indeterminate;
        let box_style = if filled {
            let fill = if self.disabled {
                Vec4::new(primary.x, primary.y, primary.z, DISABLED_ALPHA)
            } else {
                primary
            };
            VisualStyle::new()
                .solid_fill(fill)
                .corner_radius(BOX_CORNER_RADIUS)
        } else {
            let border_color = if self.disabled {
                Vec4::new(outline.x, outline.y, outline.z, DISABLED_ALPHA)
            } else {
                outline
            };
            VisualStyle::new()
                .solid_fill(TRANSPARENT)
                .corner_radius(BOX_CORNER_RADIUS)
                .stroke(StrokeStyle::solid(
                    Paint::solid(border_color),
                    OUTLINE_WEIGHT,
                    StrokeAlign::Inside,
                ))
        };

        let checkbox_box = ctx.create_node(
            root,
            NodeContent::Styled {
                style: Box::new(box_style),
            },
        );
        ctx.set_layout_style(
            checkbox_box,
            FlexStyle {
                width: Some(BOX_SIZE),
                height: Some(BOX_SIZE),
                justify_content: FlexJustifyContent::Center,
                align_items: FlexAlign::Center,
                ..Default::default()
            },
        );

        // -- Check / indeterminate mark --
        if filled {
            let mark_text = if self.indeterminate {
                "\u{2014}" // em-dash
            } else {
                "\u{2713}" // check mark
            };
            let mark_color = if self.disabled {
                Vec4::new(on_primary.x, on_primary.y, on_primary.z, DISABLED_ALPHA)
            } else {
                on_primary
            };
            let mark_style = VisualStyle::new()
                .solid_fill(mark_color)
                .text(TextContent::new(mark_text.to_string(), MARK_SIZE));
            let mark_node = ctx.create_node(
                checkbox_box,
                NodeContent::Styled {
                    style: Box::new(mark_style),
                },
            );
            let _ = mark_node;
        }

        // -- Click handler --
        if !self.disabled {
            let write_clone = write.clone();
            ctx.add_clickable(
                checkbox_box,
                Arc::new(move || {
                    write_clone.update(|v| *v = !*v);
                }),
            );
        }

        // -- Optional label --
        if let Some(ref label_text) = self.label {
            let label_color = if self.disabled {
                Vec4::new(on_surface.x, on_surface.y, on_surface.z, DISABLED_ALPHA)
            } else {
                on_surface
            };
            let label_style = VisualStyle::new()
                .solid_fill(label_color)
                .text(TextContent::new(label_text.clone(), MARK_SIZE));
            let label_node = ctx.create_node(
                root,
                NodeContent::Styled {
                    style: Box::new(label_style),
                },
            );
            let _ = label_node;

            // Also make label clickable
            if !self.disabled {
                let write_clone = write.clone();
                ctx.add_clickable(
                    label_node,
                    Arc::new(move || {
                        write_clone.update(|v| *v = !*v);
                    }),
                );
            }
        }

        root
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use flux_state::Runtime;
    use render_engine::Paint;

    #[test]
    fn test_unchecked_checkbox() {
        let runtime = Runtime::new();
        let signal = Signal::new(runtime, false);

        let mut ctx = WidgetContext::new_test();
        let checkbox = MaterialCheckbox::new(signal);
        let root_id = checkbox.build(&mut ctx);

        let root_node = ctx.scene().get_node(root_id).unwrap();
        assert!(
            !root_node.children.is_empty(),
            "Checkbox should have the box child"
        );

        // The box should have a stroke (unchecked = outline border)
        let box_id = root_node.children[0];
        let box_node = ctx.scene().get_node(box_id).unwrap();
        if let NodeContent::Styled { ref style } = box_node.content {
            assert!(
                style.stroke.is_some(),
                "Unchecked checkbox should have an outline stroke"
            );
            // Fill should be transparent
            if let Paint::Solid(color) = &style.fills[0] {
                assert!(
                    color.w < 0.01,
                    "Unchecked checkbox fill should be transparent, got alpha={}",
                    color.w
                );
            }
        } else {
            panic!("Checkbox box should be Styled content");
        }
    }

    #[test]
    fn test_checked_styling() {
        let runtime = Runtime::new();
        let signal = Signal::new(runtime, true);

        let mut ctx = WidgetContext::new_test();
        let checkbox = MaterialCheckbox::new(signal);
        let root_id = checkbox.build(&mut ctx);

        let root_node = ctx.scene().get_node(root_id).unwrap();
        let box_id = root_node.children[0];
        let box_node = ctx.scene().get_node(box_id).unwrap();
        if let NodeContent::Styled { ref style } = box_node.content {
            // Checked = primary fill, no stroke
            assert!(
                style.stroke.is_none(),
                "Checked checkbox should not have a stroke"
            );
            if let Paint::Solid(color) = &style.fills[0] {
                assert!(
                    (color.x - FALLBACK_PRIMARY.x).abs() < 0.01,
                    "Checked checkbox should use primary fill, got {color:?}"
                );
            }
        } else {
            panic!("Checkbox box should be Styled content");
        }

        // Should have a check-mark child inside the box
        assert!(
            !box_node.children.is_empty(),
            "Checked checkbox box should have a mark child"
        );
    }

    #[test]
    fn test_indeterminate() {
        let runtime = Runtime::new();
        let signal = Signal::new(runtime, false);

        let mut ctx = WidgetContext::new_test();
        let checkbox = MaterialCheckbox::new(signal).indeterminate();
        let root_id = checkbox.build(&mut ctx);

        let root_node = ctx.scene().get_node(root_id).unwrap();
        let box_id = root_node.children[0];
        let box_node = ctx.scene().get_node(box_id).unwrap();
        if let NodeContent::Styled { ref style } = box_node.content {
            // Indeterminate = primary fill (same as checked)
            if let Paint::Solid(color) = &style.fills[0] {
                assert!(
                    (color.x - FALLBACK_PRIMARY.x).abs() < 0.01,
                    "Indeterminate checkbox should use primary fill, got {color:?}"
                );
            }
        } else {
            panic!("Checkbox box should be Styled content");
        }

        // Should have a mark child
        assert!(
            !box_node.children.is_empty(),
            "Indeterminate checkbox should have a mark child"
        );
    }

    #[test]
    fn test_with_label() {
        let runtime = Runtime::new();
        let signal = Signal::new(runtime, false);

        let mut ctx = WidgetContext::new_test();
        let checkbox = MaterialCheckbox::new(signal).label("Agree");
        let root_id = checkbox.build(&mut ctx);

        let root_node = ctx.scene().get_node(root_id).unwrap();
        // Should have box + label = 2 children
        assert!(
            root_node.children.len() >= 2,
            "Checkbox with label should have at least 2 children, got {}",
            root_node.children.len()
        );
    }
}
