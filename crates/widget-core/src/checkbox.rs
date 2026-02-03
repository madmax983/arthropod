//! Checkbox widget - boolean toggle

use crate::{Text, Widget, WidgetContext};
use flux_state::{Effect, ReadSignal, Signal, WriteSignal};
use layout_engine::{FlexDirection, FlexStyle};
use render_engine::{Color, NodeContent, NodeId};
use std::sync::Arc;

/// Checkbox widget
pub struct Checkbox {
    read_signal: ReadSignal<bool>,
    write_signal: WriteSignal<bool>,
    label: Option<String>,
    disabled: bool,
}

impl Checkbox {
    pub fn new(signal: Signal<bool>) -> Self {
        let (read_signal, write_signal) = signal.split();
        Self {
            read_signal,
            write_signal,
            label: None,
            disabled: false,
        }
    }

    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

impl Widget for Checkbox {
    fn build(&self, ctx: &mut WidgetContext) -> NodeId {
        let runtime = self.read_signal.runtime().clone();
        let read = self.read_signal.clone();
        let write = self.write_signal.clone();

        // Colors
        let checked_color = Color::rgba(0.0, 0.47, 0.84, 1.0); // Blue
        let unchecked_color = Color::rgba(0.9, 0.9, 0.9, 1.0); // Light Gray
        let checkmark_color = Color::WHITE;
        let transparent = Color::rgba(0.0, 0.0, 0.0, 0.0);

        // Create reactive signals for visual state
        // Box Background
        let box_bg = Signal::new(runtime.clone(), unchecked_color);
        let (box_bg_read, box_bg_write) = box_bg.split();

        // Checkmark Color
        let check_fg = Signal::new(runtime.clone(), transparent);
        let (check_fg_read, check_fg_write) = check_fg.split();

        // Effect to sync state
        let read_clone = read.clone();
        Effect::new(runtime.clone(), move || {
            let is_checked = read_clone.get();
            if is_checked {
                box_bg_write.set(checked_color);
                check_fg_write.set(checkmark_color);
            } else {
                box_bg_write.set(unchecked_color);
                check_fg_write.set(transparent);
            }
        });

        // 1. Create Box Node
        // Initial values
        let initial_checked = read.get_untracked();
        let initial_bg = if initial_checked { checked_color } else { unchecked_color };
        let initial_fg = if initial_checked { checkmark_color } else { transparent };

        let box_node = ctx.create_node(
            ctx.root(),
            NodeContent::RoundedRect {
                color: initial_bg,
                corner_radius: 4.0,
            },
        );
        ctx.set_layout_style(box_node, FlexStyle {
             width: Some(20.0),
             height: Some(20.0),
             padding_left: 4.0, // Approximation for centering
             padding_top: 2.0,
             ..Default::default()
        });
        ctx.add_reactive_color_state(box_node, box_bg_read);

        // 2. Create Checkmark Node
        let check_node = ctx.create_node(
            box_node, // Child of box
            NodeContent::Text {
                text: "✓".to_string(),
                font_size: 14.0,
                color: initial_fg,
            }
        );
        ctx.add_reactive_color_state(check_node, check_fg_read);

        // 3. Add Interaction
        if !self.disabled {
            let write_clone = write.clone();
            ctx.add_clickable(box_node, Arc::new(move || {
                write_clone.update(|b| *b = !*b);
            }));
            ctx.add_hover_state(box_node);
        }

        // 4. Handle Label (Wrap in Row if needed)
        if let Some(label_text) = &self.label {
            let row_node = ctx.create_node(ctx.root(), NodeContent::Empty);
            ctx.set_layout_style(row_node, FlexStyle {
                direction: FlexDirection::Row,
                gap: 8.0,
                ..Default::default()
            });

            // Reparent box to row
            ctx.reparent_to(box_node, row_node);

            // Create label
            let label_widget = Text::new(label_text.clone());
            let label_id = label_widget.build(ctx);
            ctx.reparent_to(label_id, row_node);

            // Also make label clickable to toggle
             if !self.disabled {
                let write_clone = write.clone();
                ctx.add_clickable(label_id, Arc::new(move || {
                    write_clone.update(|b| *b = !*b);
                }));
            }

            row_node
        } else {
            box_node
        }
    }
}
