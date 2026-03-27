//! MD3 Accordion widget -- expandable section with header and collapsible content.

use crate::theme::MaterialTheme;
use flux_state::{Effect, ReadSignal, Signal, WriteSignal};
use glam::Vec4;
use layout_engine::{FlexAlign, FlexDirection, FlexStyle};
use render_engine::node::NodeContent;
use render_engine::{NodeId, Paint, StrokeStyle, TextContent, VisualStyle};
use std::sync::Arc;
use style_engine::StrokeAlign;
use widget_core::WidgetContext;
use widget_core::widget_trait::Widget;

// ---------------------------------------------------------------------------
// MD3 fallback colors
// ---------------------------------------------------------------------------

/// MD3 surface (#FEF7FF) -- accordion background.
const FALLBACK_SURFACE: Vec4 = Vec4::new(0.996, 0.969, 1.0, 1.0);

/// MD3 on-surface (#1D1B20) -- header text color.
const FALLBACK_ON_SURFACE: Vec4 = Vec4::new(0.114, 0.106, 0.125, 1.0);

/// MD3 outline (#79747E) -- divider line color.
const FALLBACK_OUTLINE: Vec4 = Vec4::new(0.475, 0.455, 0.494, 1.0);

// ---------------------------------------------------------------------------
// Dimension constants (dp)
// ---------------------------------------------------------------------------

/// Header row height in dp.
const HEADER_HEIGHT: f32 = 48.0;

/// Header horizontal padding in dp.
const HEADER_PADDING: f32 = 16.0;

/// Header text font size in dp.
const HEADER_FONT_SIZE: f32 = 16.0;

/// Expand indicator font size in dp.
const INDICATOR_FONT_SIZE: f32 = 14.0;

/// Divider line weight in dp.
const DIVIDER_WEIGHT: f32 = 1.0;

/// Divider height (1dp line).
const DIVIDER_HEIGHT: f32 = 1.0;

/// Disabled content opacity multiplier.
const DISABLED_ALPHA: f32 = 0.38;

/// Expand indicator when expanded.
const INDICATOR_EXPANDED: &str = "\u{25BC}"; // "▼"

/// Expand indicator when collapsed.
const INDICATOR_COLLAPSED: &str = "\u{25B8}"; // "▸"

// ---------------------------------------------------------------------------
// Accordion
// ---------------------------------------------------------------------------

/// MD3 Accordion -- expandable section with header and collapsible content.
///
/// Displays a header row with text and an expand/collapse indicator. Clicking
/// the header toggles the `expanded` signal. The content widget is always
/// built (future: visibility toggle via scene node API).
///
/// # Structure
///
/// 1. **Root column** -- surface background
///    - **Header row** -- 48dp, 16dp padding, on_surface text + indicator
///    - **Divider** -- 1dp outline-colored line
///    - **Content** -- child widget (always built)
///
/// # Example
///
/// ```rust,no_run
/// use flux_state::{Runtime, Signal};
/// use material_ui::surfaces::Accordion;
/// use widget_core::Text;
///
/// let runtime = Runtime::new();
/// let expanded = Signal::new(runtime, false);
///
/// let accordion = Accordion::new("Settings", Text::new("Content here"), expanded);
/// ```
pub struct Accordion {
    header: String,
    content: Box<dyn Widget>,
    read_signal: ReadSignal<bool>,
    write_signal: WriteSignal<bool>,
    disabled: bool,
}

impl Accordion {
    /// Create a new accordion with a header label, content widget, and
    /// expansion state signal.
    ///
    /// The signal is split internally: clicking the header toggles the write
    /// side; the read side drives the expand/collapse indicator.
    pub fn new(
        header: impl Into<String>,
        content: impl Widget + 'static,
        expanded: Signal<bool>,
    ) -> Self {
        let (read_signal, write_signal) = expanded.split();
        Self {
            header: header.into(),
            content: Box::new(content),
            read_signal,
            write_signal,
            disabled: false,
        }
    }

    /// Disable interaction. The header renders with reduced opacity and
    /// ignores click events. The expand indicator still reflects the current
    /// signal state.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

impl Widget for Accordion {
    fn build(&self, ctx: &mut WidgetContext) -> NodeId {
        let runtime = self.read_signal.runtime().clone();
        let read = self.read_signal.clone();
        let write = self.write_signal.clone();

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
        let outline = theme
            .as_ref()
            .map(|t| t.color.outline)
            .unwrap_or(FALLBACK_OUTLINE);

        // -- Disabled text color --
        let text_color = if self.disabled {
            Vec4::new(on_surface.x, on_surface.y, on_surface.z, DISABLED_ALPHA)
        } else {
            on_surface
        };

        // -- 1. Root column container (surface background) --
        let root_style = VisualStyle::new().solid_fill(surface);
        let root = ctx.create_node(
            ctx.root(),
            NodeContent::Styled {
                style: Box::new(root_style),
            },
        );
        ctx.set_layout_style(
            root,
            FlexStyle {
                direction: FlexDirection::Column,
                ..Default::default()
            },
        );

        // -- 2. Header row --
        let header_row = ctx.create_node(root, NodeContent::Empty);
        ctx.set_layout_style(
            header_row,
            FlexStyle {
                direction: FlexDirection::Row,
                height: Some(HEADER_HEIGHT),
                padding_left: HEADER_PADDING,
                padding_right: HEADER_PADDING,
                align_items: FlexAlign::Center,
                gap: 8.0,
                ..Default::default()
            },
        );

        // -- 2a. Expand/collapse indicator --
        let is_expanded = read.get_untracked();
        let indicator_text = if is_expanded {
            INDICATOR_EXPANDED
        } else {
            INDICATOR_COLLAPSED
        };
        let indicator_style = VisualStyle::new()
            .solid_fill(text_color)
            .text(TextContent::new(
                indicator_text.to_string(),
                INDICATOR_FONT_SIZE,
            ));

        ctx.create_node(
            header_row,
            NodeContent::Styled {
                style: Box::new(indicator_style),
            },
        );

        // -- 2b. Header text --
        let header_style = VisualStyle::new()
            .solid_fill(text_color)
            .text(TextContent::new(self.header.clone(), HEADER_FONT_SIZE));

        ctx.create_node(
            header_row,
            NodeContent::Styled {
                style: Box::new(header_style),
            },
        );

        // -- 2c. Click handler on header row (toggle expanded state) --
        if !self.disabled {
            let write_clone = write.clone();
            ctx.add_clickable(
                header_row,
                Arc::new(move || {
                    write_clone.update(|b| *b = !*b);
                }),
            );
        }

        // -- 3. Divider (1dp outline-colored line) --
        let divider_style = VisualStyle::new().stroke(StrokeStyle::solid(
            Paint::solid(outline),
            DIVIDER_WEIGHT,
            StrokeAlign::Inside,
        ));
        let divider = ctx.create_node(
            root,
            NodeContent::Styled {
                style: Box::new(divider_style),
            },
        );
        ctx.set_layout_style(
            divider,
            FlexStyle {
                height: Some(DIVIDER_HEIGHT),
                ..Default::default()
            },
        );

        // -- 4. Content (always built; future: visibility toggle) --
        let content_id = self.content.build(ctx);
        ctx.reparent_to(content_id, root);

        // -- 5. Reactive effect to track expansion state --
        // Currently a no-op effect that reads the signal to establish the
        // reactive dependency. When scene node visibility is available, this
        // effect will show/hide the content node.
        let _content_node = content_id;
        let read_for_effect = read.clone();
        let effect = Effect::new(runtime, move || {
            let _is_expanded = read_for_effect.get();
            // Future: toggle content node visibility based on _is_expanded
        });
        ctx.store_effect(effect);

        root
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use flux_state::Runtime;

    #[test]
    fn test_accordion_builds_header_and_content() {
        let runtime = Runtime::new();
        let expanded = Signal::new(runtime, false);

        let mut ctx = WidgetContext::new_test();
        let accordion = Accordion::new("Section", widget_core::Text::new("Body text"), expanded);
        let root_id = accordion.build(&mut ctx);

        let scene = ctx.scene();
        let root_node = scene.get_node(root_id).unwrap();

        // Root should have: header row + divider + content = 3 children
        assert_eq!(
            root_node.children.len(),
            3,
            "Accordion should have 3 children (header + divider + content), got {}",
            root_node.children.len()
        );

        // First child should be the header row (Empty node with children)
        let header_row_id = root_node.children[0];
        let header_row = scene.get_node(header_row_id).unwrap();
        assert!(
            !header_row.children.is_empty(),
            "Header row should have children (indicator + text)"
        );
    }

    #[test]
    fn test_accordion_click_toggles_signal() {
        let runtime = Runtime::new();
        let expanded = Signal::new(runtime, false);
        let observer = expanded.clone();
        let (read, _) = observer.split();

        let mut ctx = WidgetContext::new_test();
        let accordion = Accordion::new("Toggle Me", widget_core::Text::new("Content"), expanded);
        let root_id = accordion.build(&mut ctx);

        // Header row is the first child of root
        let scene = ctx.scene();
        let root_node = scene.get_node(root_id).unwrap();
        let header_row_id = root_node.children[0];

        // Initially collapsed
        assert!(!read.get_untracked(), "Accordion should start collapsed");

        // Click the header to expand
        ctx.trigger_click(header_row_id);
        assert!(
            read.get_untracked(),
            "Accordion should be expanded after click"
        );

        // Click again to collapse
        ctx.trigger_click(header_row_id);
        assert!(
            !read.get_untracked(),
            "Accordion should be collapsed after second click"
        );
    }

    #[test]
    fn test_accordion_disabled_not_clickable() {
        let runtime = Runtime::new();
        let expanded = Signal::new(runtime, false);
        let observer = expanded.clone();
        let (read, _) = observer.split();

        let mut ctx = WidgetContext::new_test();
        let accordion =
            Accordion::new("Disabled", widget_core::Text::new("Content"), expanded).disabled(true);
        let root_id = accordion.build(&mut ctx);

        let scene = ctx.scene();
        let root_node = scene.get_node(root_id).unwrap();
        let header_row_id = root_node.children[0];

        assert!(
            !ctx.has_clickable(header_row_id),
            "Disabled accordion header should not be clickable"
        );

        // State should not change
        assert!(
            !read.get_untracked(),
            "Disabled accordion should remain collapsed"
        );
    }

    #[test]
    fn test_accordion_expand_indicator() {
        // Test collapsed indicator
        let runtime = Runtime::new();
        let collapsed_signal = Signal::new(runtime.clone(), false);

        let mut ctx = WidgetContext::new_test();
        let accordion = Accordion::new(
            "Collapsed",
            widget_core::Text::new("Content"),
            collapsed_signal,
        );
        let root_id = accordion.build(&mut ctx);

        let scene = ctx.scene();
        let root_node = scene.get_node(root_id).unwrap();
        let header_row_id = root_node.children[0];
        let header_row = scene.get_node(header_row_id).unwrap();

        // First child of header row is the indicator
        let indicator_id = header_row.children[0];
        let indicator_node = scene.get_node(indicator_id).unwrap();
        if let NodeContent::Styled { ref style } = indicator_node.content {
            let text = style
                .text
                .as_ref()
                .expect("Indicator should have TextContent");
            assert_eq!(
                text.text, INDICATOR_COLLAPSED,
                "Collapsed indicator should show '{}', got '{}'",
                INDICATOR_COLLAPSED, text.text
            );
        } else {
            panic!("Indicator should be Styled content");
        }

        // Test expanded indicator
        let expanded_signal = Signal::new(runtime, true);

        let mut ctx2 = WidgetContext::new_test();
        let accordion2 = Accordion::new(
            "Expanded",
            widget_core::Text::new("Content"),
            expanded_signal,
        );
        let root_id2 = accordion2.build(&mut ctx2);

        let scene2 = ctx2.scene();
        let root_node2 = scene2.get_node(root_id2).unwrap();
        let header_row_id2 = root_node2.children[0];
        let header_row2 = scene2.get_node(header_row_id2).unwrap();
        let indicator_id2 = header_row2.children[0];
        let indicator_node2 = scene2.get_node(indicator_id2).unwrap();

        if let NodeContent::Styled { ref style } = indicator_node2.content {
            let text = style
                .text
                .as_ref()
                .expect("Indicator should have TextContent");
            assert_eq!(
                text.text, INDICATOR_EXPANDED,
                "Expanded indicator should show '{}', got '{}'",
                INDICATOR_EXPANDED, text.text
            );
        } else {
            panic!("Indicator should be Styled content");
        }
    }

    #[test]
    fn test_accordion_header_text() {
        let runtime = Runtime::new();
        let expanded = Signal::new(runtime, false);

        let mut ctx = WidgetContext::new_test();
        let accordion = Accordion::new("My Section", widget_core::Text::new("Content"), expanded);
        let root_id = accordion.build(&mut ctx);

        let scene = ctx.scene();
        let root_node = scene.get_node(root_id).unwrap();
        let header_row_id = root_node.children[0];
        let header_row = scene.get_node(header_row_id).unwrap();

        // Second child of header row is the header text
        assert!(
            header_row.children.len() >= 2,
            "Header row should have at least 2 children (indicator + text), got {}",
            header_row.children.len()
        );

        let header_text_id = header_row.children[1];
        let header_text_node = scene.get_node(header_text_id).unwrap();
        if let NodeContent::Styled { ref style } = header_text_node.content {
            let text = style
                .text
                .as_ref()
                .expect("Header text node should have TextContent");
            assert_eq!(
                text.text, "My Section",
                "Header text should be 'My Section', got '{}'",
                text.text
            );
            assert!(
                (text.font_size - HEADER_FONT_SIZE).abs() < 0.01,
                "Header font size should be {HEADER_FONT_SIZE}, got {}",
                text.font_size
            );
        } else {
            panic!("Header text should be Styled content");
        }
    }

    #[test]
    fn test_accordion_has_surface_background() {
        let runtime = Runtime::new();
        let expanded = Signal::new(runtime, false);

        let mut ctx = WidgetContext::new_test();
        let accordion = Accordion::new("Section", widget_core::Text::new("Content"), expanded);
        let root_id = accordion.build(&mut ctx);

        let scene = ctx.scene();
        let node = scene.get_node(root_id).unwrap();

        if let NodeContent::Styled { ref style } = node.content {
            assert!(!style.fills.is_empty(), "Accordion should have a fill");
            if let Paint::Solid(color) = &style.fills[0] {
                assert!(
                    (color.x - FALLBACK_SURFACE.x).abs() < 0.01
                        && (color.y - FALLBACK_SURFACE.y).abs() < 0.01
                        && (color.z - FALLBACK_SURFACE.z).abs() < 0.01,
                    "Accordion background should match surface color, got {color:?}"
                );
            } else {
                panic!("Accordion fill should be solid");
            }
        } else {
            panic!("Accordion root should be Styled content");
        }
    }

    #[test]
    fn test_accordion_has_divider() {
        let runtime = Runtime::new();
        let expanded = Signal::new(runtime, false);

        let mut ctx = WidgetContext::new_test();
        let accordion = Accordion::new("Section", widget_core::Text::new("Content"), expanded);
        let root_id = accordion.build(&mut ctx);

        let scene = ctx.scene();
        let root_node = scene.get_node(root_id).unwrap();

        // Second child should be the divider
        let divider_id = root_node.children[1];
        let divider_node = scene.get_node(divider_id).unwrap();

        if let NodeContent::Styled { ref style } = divider_node.content {
            assert!(style.stroke.is_some(), "Divider should have a stroke");
            let stroke = style.stroke.as_ref().unwrap();
            assert!(
                (stroke.weight - DIVIDER_WEIGHT).abs() < 0.01,
                "Divider stroke weight should be {DIVIDER_WEIGHT}, got {}",
                stroke.weight
            );
        } else {
            panic!("Divider should be Styled content");
        }
    }

    #[test]
    fn test_accordion_with_theme() {
        let runtime = Runtime::new();
        let expanded = Signal::new(runtime, true);

        let mut ctx = WidgetContext::new_test();
        let theme = MaterialTheme::from_seed(Vec4::new(0.4, 0.2, 0.8, 1.0));
        ctx.set_extension(theme);

        let accordion = Accordion::new("Themed", widget_core::Text::new("Content"), expanded);
        let root_id = accordion.build(&mut ctx);

        assert!(
            ctx.scene().get_node(root_id).is_some(),
            "Accordion with theme should build successfully"
        );
    }
}
