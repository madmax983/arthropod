//! MD3 Tabs widget -- horizontal tab selection with active indicator.

use crate::theme::MaterialTheme;
use flux_state::{Effect, Signal};
use glam::Vec4;
use layout_engine::{FlexAlign, FlexDirection, FlexJustifyContent, FlexStyle};
use render_engine::node::NodeContent;
use render_engine::{Color, NodeId, TextContent, VisualStyle};
use std::sync::Arc;
use widget_core::Widget;
use widget_core::WidgetContext;

// ---------------------------------------------------------------------------
// MD3 fallback colors
// ---------------------------------------------------------------------------

/// MD3 primary (#6750A4) -- selected tab text & indicator.
const FALLBACK_PRIMARY: Vec4 = Vec4::new(0.404, 0.314, 0.643, 1.0);

/// MD3 on_surface (#1D1B20) -- selected tab text (secondary variant).
const FALLBACK_ON_SURFACE: Vec4 = Vec4::new(0.114, 0.106, 0.125, 1.0);

/// MD3 on_surface_variant (#49454F) -- unselected tab text.
const FALLBACK_ON_SURFACE_VARIANT: Vec4 = Vec4::new(0.286, 0.271, 0.310, 1.0);

/// Surface color (transparent -- tabs sit on top of the app bar or surface).
const FALLBACK_SURFACE: Vec4 = Vec4::new(0.996, 0.969, 1.0, 1.0);

// ---------------------------------------------------------------------------
// Dimension constants (dp)
// ---------------------------------------------------------------------------

/// Overall tab bar height.
const TAB_BAR_HEIGHT: f32 = 48.0;

/// Active tab indicator height.
const INDICATOR_HEIGHT: f32 = 3.0;

/// Default label font size.
const LABEL_FONT_SIZE: f32 = 14.0;

/// Disabled alpha multiplier.
const DISABLED_ALPHA: f32 = 0.38;

/// A single tab definition.
///
/// # Example
///
/// ```rust,no_run
/// use material_ui::navigation::Tab;
///
/// let tab = Tab::new("Overview").disabled();
/// ```
pub struct Tab {
    /// Display label.
    pub label: String,
    /// Whether the tab is disabled.
    pub disabled: bool,
}

impl Tab {
    /// Create a new enabled tab.
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            disabled: false,
        }
    }

    /// Mark the tab as disabled (not clickable, reduced opacity).
    pub fn disabled(mut self) -> Self {
        self.disabled = true;
        self
    }
}

/// MD3 Tabs widget -- horizontal tab selection bar.
///
/// Renders a row of tab labels with a colored indicator under the selected
/// tab. Selection state is driven by a [`Signal<usize>`] so the component
/// is fully reactive.
///
/// # Example
///
/// ```rust,no_run
/// use flux_state::{Runtime, Signal};
/// use material_ui::navigation::{Tab, Tabs};
///
/// let runtime = Runtime::new();
/// let selected = Signal::new(runtime, 0usize);
///
/// let tabs = Tabs::new(
///     vec![Tab::new("All"), Tab::new("Unread"), Tab::new("Flagged")],
///     selected,
/// );
/// ```
pub struct Tabs {
    tabs: Vec<Tab>,
    signal: Signal<usize>,
    secondary: bool,
}

impl Tabs {
    /// Create a new tab bar.
    ///
    /// # Arguments
    ///
    /// * `tabs` - The ordered list of tabs to display.
    /// * `signal` - A signal holding the currently selected tab index.
    pub fn new(tabs: Vec<Tab>, signal: Signal<usize>) -> Self {
        Self {
            tabs,
            signal,
            secondary: false,
        }
    }

    /// Use the secondary style (on_surface text color for selected instead of primary).
    pub fn secondary(mut self) -> Self {
        self.secondary = true;
        self
    }
}

impl Widget for Tabs {
    fn build(&self, ctx: &mut WidgetContext) -> NodeId {
        let (read, write) = self.signal.clone().split();
        let runtime = read.runtime().clone();

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
        let surface = theme
            .as_ref()
            .map(|t| t.color.surface)
            .unwrap_or(FALLBACK_SURFACE);

        let selected_text_color = if self.secondary { on_surface } else { primary };
        let indicator_color = primary;

        let current_index = read.get_untracked();

        // -- Root row container --
        let root = ctx.create_node(
            ctx.root(),
            NodeContent::Styled {
                style: Box::new(VisualStyle::new().solid_fill(surface)),
            },
        );
        ctx.set_layout_style(
            root,
            FlexStyle {
                direction: FlexDirection::Row,
                height: Some(TAB_BAR_HEIGHT),
                ..Default::default()
            },
        );

        // Track label + indicator node ids for reactive effect
        let mut label_ids: Vec<NodeId> = Vec::new();
        let mut indicator_ids: Vec<NodeId> = Vec::new();
        let mut disabled_flags: Vec<bool> = Vec::new();

        for (i, tab) in self.tabs.iter().enumerate() {
            let is_selected = i == current_index;
            let is_disabled = tab.disabled;

            disabled_flags.push(is_disabled);

            // Determine colors
            let text_color = if is_disabled {
                Vec4::new(
                    on_surface_variant.x,
                    on_surface_variant.y,
                    on_surface_variant.z,
                    DISABLED_ALPHA,
                )
            } else if is_selected {
                selected_text_color
            } else {
                on_surface_variant
            };

            let ind_color = if is_selected && !is_disabled {
                indicator_color
            } else {
                // Transparent indicator for unselected/disabled
                Vec4::new(0.0, 0.0, 0.0, 0.0)
            };

            // Tab cell: Column with [label area, indicator]
            let tab_cell = ctx.create_node(root, NodeContent::Empty);
            ctx.set_layout_style(
                tab_cell,
                FlexStyle {
                    direction: FlexDirection::Column,
                    flex_grow: 1.0,
                    height: Some(TAB_BAR_HEIGHT),
                    ..Default::default()
                },
            );

            // Label container (centered)
            let label_container = ctx.create_node(tab_cell, NodeContent::Empty);
            ctx.set_layout_style(
                label_container,
                FlexStyle {
                    flex_grow: 1.0,
                    justify_content: FlexJustifyContent::Center,
                    align_items: FlexAlign::Center,
                    ..Default::default()
                },
            );

            // Label text node
            let label_style = VisualStyle::new()
                .solid_fill(text_color)
                .text(TextContent::new(tab.label.clone(), LABEL_FONT_SIZE));

            let label_node = ctx.create_node(
                label_container,
                NodeContent::Styled {
                    style: Box::new(label_style),
                },
            );
            label_ids.push(label_node);

            // Indicator bar
            let indicator_style = VisualStyle::new().solid_fill(ind_color);
            let indicator_node = ctx.create_node(
                tab_cell,
                NodeContent::Styled {
                    style: Box::new(indicator_style),
                },
            );
            ctx.set_layout_style(
                indicator_node,
                FlexStyle {
                    height: Some(INDICATOR_HEIGHT),
                    ..Default::default()
                },
            );
            indicator_ids.push(indicator_node);

            // Click handler (unless disabled)
            if !is_disabled {
                let w = write.clone();
                let idx = i;
                ctx.add_clickable(
                    tab_cell,
                    Arc::new(move || {
                        w.set(idx);
                    }),
                );
            }

            // Register reactive color on label and indicator so effect can update them
            let label_color_signal = Signal::new(
                runtime.clone(),
                Color::rgba(text_color.x, text_color.y, text_color.z, text_color.w),
            );
            let (label_cr, label_cw) = label_color_signal.split();
            ctx.add_reactive_color_state(label_node, label_cr);

            let ind_color_signal = Signal::new(
                runtime.clone(),
                Color::rgba(ind_color.x, ind_color.y, ind_color.z, ind_color.w),
            );
            let (ind_cr, ind_cw) = ind_color_signal.split();
            ctx.add_reactive_color_state(indicator_node, ind_cr);

            // Store write signals for the effect
            // We need an effect per-tab that watches the selection signal
            let read_for_effect = read.clone();
            let tab_index = i;
            let tab_disabled = is_disabled;
            let sel_color = selected_text_color;
            let unsel_color = on_surface_variant;
            let ind_primary = indicator_color;
            let effect = Effect::new(runtime.clone(), move || {
                let current = read_for_effect.get();
                let selected = current == tab_index;

                if tab_disabled {
                    let disabled_color =
                        Vec4::new(unsel_color.x, unsel_color.y, unsel_color.z, DISABLED_ALPHA);
                    label_cw.set(Color::rgba(
                        disabled_color.x,
                        disabled_color.y,
                        disabled_color.z,
                        disabled_color.w,
                    ));
                    ind_cw.set(Color::rgba(0.0, 0.0, 0.0, 0.0));
                } else if selected {
                    label_cw.set(Color::rgba(
                        sel_color.x,
                        sel_color.y,
                        sel_color.z,
                        sel_color.w,
                    ));
                    ind_cw.set(Color::rgba(
                        ind_primary.x,
                        ind_primary.y,
                        ind_primary.z,
                        ind_primary.w,
                    ));
                } else {
                    label_cw.set(Color::rgba(
                        unsel_color.x,
                        unsel_color.y,
                        unsel_color.z,
                        unsel_color.w,
                    ));
                    ind_cw.set(Color::rgba(0.0, 0.0, 0.0, 0.0));
                }
            });
            ctx.store_effect(effect);
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
    fn test_tabs_builds_correct_tab_count() {
        let runtime = Runtime::new();
        let selected = Signal::new(runtime, 0usize);

        let mut ctx = WidgetContext::new_test();
        let tabs = Tabs::new(
            vec![Tab::new("One"), Tab::new("Two"), Tab::new("Three")],
            selected,
        );
        let root_id = tabs.build(&mut ctx);

        let root_node = ctx.scene().get_node(root_id).unwrap();
        assert_eq!(
            root_node.children.len(),
            3,
            "Tabs with 3 items should have 3 tab cell children, got {}",
            root_node.children.len()
        );
    }

    #[test]
    fn test_tabs_selected_tab_styling() {
        let runtime = Runtime::new();
        // Select the second tab (index 1)
        let selected = Signal::new(runtime, 1usize);

        let mut ctx = WidgetContext::new_test();
        let tabs = Tabs::new(vec![Tab::new("A"), Tab::new("B"), Tab::new("C")], selected);
        let root_id = tabs.build(&mut ctx);

        let scene = ctx.scene();
        let root_node = scene.get_node(root_id).unwrap();

        // Each tab cell is a Column: [label_container, indicator]
        // Check indicator of the selected tab (index 1) has primary color
        let selected_cell_id = root_node.children[1];
        let selected_cell = scene.get_node(selected_cell_id).unwrap();
        let indicator_id = *selected_cell.children.last().unwrap();
        let indicator_node = scene.get_node(indicator_id).unwrap();

        if let NodeContent::Styled { ref style } = indicator_node.content {
            assert!(
                !style.fills.is_empty(),
                "Selected indicator should have fill"
            );
            if let Paint::Solid(color) = &style.fills[0] {
                assert!(
                    color.w > 0.5,
                    "Selected indicator should be visible (alpha > 0.5), got {}",
                    color.w
                );
                assert!(
                    (color.x - FALLBACK_PRIMARY.x).abs() < 0.01
                        && (color.y - FALLBACK_PRIMARY.y).abs() < 0.01,
                    "Selected indicator should match primary color, got {color:?}"
                );
            } else {
                panic!("Indicator fill should be solid");
            }
        } else {
            panic!("Indicator should be Styled content");
        }

        // Check that the unselected tab (index 0) indicator is transparent
        let unselected_cell_id = root_node.children[0];
        let unselected_cell = scene.get_node(unselected_cell_id).unwrap();
        let unselected_ind_id = *unselected_cell.children.last().unwrap();
        let unselected_ind = scene.get_node(unselected_ind_id).unwrap();

        if let NodeContent::Styled { ref style } = unselected_ind.content
            && let Some(Paint::Solid(color)) = style.fills.first()
        {
            assert!(
                color.w < 0.01,
                "Unselected indicator should be transparent, got alpha={}",
                color.w
            );
        }
    }

    #[test]
    fn test_tabs_click_changes_signal() {
        let runtime = Runtime::new();
        let selected = Signal::new(runtime, 0usize);
        let observer = selected.clone();
        let (obs_read, _) = observer.split();

        let mut ctx = WidgetContext::new_test();
        let tabs = Tabs::new(vec![Tab::new("X"), Tab::new("Y")], selected);
        let root_id = tabs.build(&mut ctx);

        assert_eq!(obs_read.get_untracked(), 0, "Should start at tab 0");

        // Click the second tab cell
        let second_tab_id = ctx.scene().get_node(root_id).unwrap().children[1];
        ctx.trigger_click(second_tab_id);

        assert_eq!(
            obs_read.get_untracked(),
            1,
            "After clicking second tab, signal should be 1"
        );
    }

    #[test]
    fn test_tabs_disabled_tab() {
        let runtime = Runtime::new();
        let selected = Signal::new(runtime, 0usize);
        let observer = selected.clone();
        let (obs_read, _) = observer.split();

        let mut ctx = WidgetContext::new_test();
        let tabs = Tabs::new(
            vec![Tab::new("Enabled"), Tab::new("Locked").disabled()],
            selected,
        );
        let root_id = tabs.build(&mut ctx);

        // Disabled tab cell should not be clickable
        let disabled_tab_id = ctx.scene().get_node(root_id).unwrap().children[1];
        assert!(
            !ctx.has_clickable(disabled_tab_id),
            "Disabled tab should not be clickable"
        );

        // Clicking disabled should not change signal
        ctx.trigger_click(disabled_tab_id);
        assert_eq!(
            obs_read.get_untracked(),
            0,
            "Signal should remain 0 after clicking disabled tab"
        );

        // Verify disabled label has reduced alpha
        let disabled_cell = ctx.scene().get_node(disabled_tab_id).unwrap();
        let label_container_id = disabled_cell.children[0];
        let label_container = ctx.scene().get_node(label_container_id).unwrap();
        let label_id = label_container.children[0];
        let label_node = ctx.scene().get_node(label_id).unwrap();

        if let NodeContent::Styled { ref style } = label_node.content
            && let Some(Paint::Solid(color)) = style.fills.first()
        {
            assert!(
                (color.w - DISABLED_ALPHA).abs() < 0.01,
                "Disabled tab label should have alpha={DISABLED_ALPHA}, got {}",
                color.w
            );
        }
    }

    #[test]
    fn test_tabs_secondary_variant() {
        let runtime = Runtime::new();
        let selected = Signal::new(runtime, 0usize);

        let mut ctx = WidgetContext::new_test();
        let tabs = Tabs::new(vec![Tab::new("Sec1"), Tab::new("Sec2")], selected).secondary();

        assert!(tabs.secondary, "Secondary flag should be set");

        let root_id = tabs.build(&mut ctx);
        assert!(
            ctx.scene().get_node(root_id).is_some(),
            "Secondary tabs should build successfully"
        );
    }
}
