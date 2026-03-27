//! MD3 FAB (Floating Action Button) widget.
//!
//! A prominent button for the primary action on a screen.
//! Supports three sizes (small, regular, large) and an extended variant
//! with an icon + text label.

use crate::theme::MaterialTheme;
use glam::Vec4;
use layout_engine::{FlexAlign, FlexDirection, FlexJustifyContent, FlexStyle};
use render_engine::NodeId;
use render_engine::node::NodeContent;
use std::sync::Arc;
use widget_core::widget_trait::Widget;
use widget_core::{Text, WidgetContext};

// ---------------------------------------------------------------------------
// MD3 fallback colors (used when no MaterialTheme is provided)
// ---------------------------------------------------------------------------

/// MD3 primary_container (#EADDFF)
const FALLBACK_PRIMARY_CONTAINER: Vec4 = Vec4::new(0.918, 0.867, 1.0, 1.0);

/// MD3 on_primary_container (#21005D)
const FALLBACK_ON_PRIMARY_CONTAINER: Vec4 = Vec4::new(0.129, 0.0, 0.365, 1.0);

/// Disabled content opacity multiplier.
const DISABLED_ALPHA: f32 = 0.38;

// ---------------------------------------------------------------------------
// FAB sizes (dp)
// ---------------------------------------------------------------------------

/// Small FAB: 40dp with 12dp corner radius.
const SMALL_SIZE: f32 = 40.0;
const SMALL_CORNER_RADIUS: f32 = 12.0;

/// Regular FAB: 56dp with 16dp corner radius.
const REGULAR_SIZE: f32 = 56.0;
const REGULAR_CORNER_RADIUS: f32 = 16.0;

/// Large FAB: 96dp with 28dp corner radius.
const LARGE_SIZE: f32 = 96.0;
const LARGE_CORNER_RADIUS: f32 = 28.0;

/// Horizontal padding for extended FAB (dp).
const EXTENDED_PADDING_H: f32 = 16.0;

/// Gap between icon and label in extended FAB (dp).
const EXTENDED_GAP: f32 = 12.0;

/// FAB size variants following MD3 specifications.
///
/// Each variant maps to a specific dp size and corner radius.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FabSize {
    /// Small FAB: 40x40dp, corner radius 12dp.
    Small,
    /// Regular FAB: 56x56dp, corner radius 16dp (default).
    #[default]
    Regular,
    /// Large FAB: 96x96dp, corner radius 28dp.
    Large,
}

impl FabSize {
    /// Returns the side length in dp.
    fn dp(self) -> f32 {
        match self {
            Self::Small => SMALL_SIZE,
            Self::Regular => REGULAR_SIZE,
            Self::Large => LARGE_SIZE,
        }
    }

    /// Returns the corner radius in dp.
    fn corner_radius(self) -> f32 {
        match self {
            Self::Small => SMALL_CORNER_RADIUS,
            Self::Regular => REGULAR_CORNER_RADIUS,
            Self::Large => LARGE_CORNER_RADIUS,
        }
    }
}

/// MD3 Floating Action Button (FAB).
///
/// A prominent button for the primary action on a screen. Renders as a
/// rounded square container with a centered icon. When a label is set,
/// it becomes an "extended" FAB with the icon on the left and text on
/// the right.
///
/// # Example
///
/// ```rust,no_run
/// use material_ui::inputs::Fab;
///
/// // Standard FAB with icon
/// let fab = Fab::new("+")
///     .on_click(|| println!("Create!"));
///
/// // Extended FAB with label
/// let extended = Fab::new("+")
///     .label("Create")
///     .on_click(|| println!("Create!"));
///
/// // Small FAB
/// let small = Fab::new("+").small();
///
/// // Large FAB
/// let large = Fab::new("+").large();
/// ```
pub struct Fab {
    icon: String,
    label: Option<String>,
    size: FabSize,
    on_click: Option<Arc<dyn Fn() + Send + Sync>>,
    disabled: bool,
}

impl Fab {
    /// Create a new FAB with the given icon text.
    ///
    /// The icon is rendered as text for now (e.g., "+" or an emoji).
    /// A proper icon system will replace this in a future phase.
    pub fn new(icon: impl Into<String>) -> Self {
        Self {
            icon: icon.into(),
            label: None,
            size: FabSize::default(),
            on_click: None,
            disabled: false,
        }
    }

    /// Set a text label, turning this into an "extended" FAB.
    ///
    /// Extended FABs display the icon on the left and label on the right
    /// in a row layout. The width auto-expands to fit the content.
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Set the FAB size to small (40dp).
    pub fn small(mut self) -> Self {
        self.size = FabSize::Small;
        self
    }

    /// Set the FAB size to large (96dp).
    pub fn large(mut self) -> Self {
        self.size = FabSize::Large;
        self
    }

    /// Set the click handler.
    pub fn on_click(mut self, f: impl Fn() + Send + Sync + 'static) -> Self {
        self.on_click = Some(Arc::new(f));
        self
    }

    /// Disable interaction. The FAB renders with reduced opacity and
    /// ignores click events.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

impl Widget for Fab {
    fn build(&self, ctx: &mut WidgetContext) -> NodeId {
        // -- Resolve theme colors --
        let theme = ctx.get_extension::<MaterialTheme>().cloned();
        let primary_container = theme
            .as_ref()
            .map(|t| t.color.primary_container)
            .unwrap_or(FALLBACK_PRIMARY_CONTAINER);
        let on_primary_container = theme
            .as_ref()
            .map(|t| t.color.on_primary_container)
            .unwrap_or(FALLBACK_ON_PRIMARY_CONTAINER);

        let size_dp = self.size.dp();
        let corner_radius = self.size.corner_radius();
        let is_extended = self.label.is_some();

        // -- Determine colors (reduce alpha when disabled) --
        let bg_color = if self.disabled {
            Vec4::new(
                primary_container.x,
                primary_container.y,
                primary_container.z,
                DISABLED_ALPHA,
            )
        } else {
            primary_container
        };

        let content_color = if self.disabled {
            Vec4::new(
                on_primary_container.x,
                on_primary_container.y,
                on_primary_container.z,
                DISABLED_ALPHA,
            )
        } else {
            on_primary_container
        };

        // -- Build container --
        let container_style = render_engine::VisualStyle::new()
            .solid_fill(bg_color)
            .corner_radius(corner_radius);

        let container = ctx.create_node(
            ctx.root(),
            NodeContent::Styled {
                style: Box::new(container_style),
            },
        );

        if is_extended {
            // Extended FAB: row layout, min height = size, auto width
            ctx.set_layout_style(
                container,
                FlexStyle {
                    direction: FlexDirection::Row,
                    height: Some(size_dp),
                    padding_left: EXTENDED_PADDING_H,
                    padding_right: EXTENDED_PADDING_H,
                    gap: EXTENDED_GAP,
                    justify_content: FlexJustifyContent::Center,
                    align_items: FlexAlign::Center,
                    ..Default::default()
                },
            );
        } else {
            // Standard FAB: square container
            ctx.set_layout_style(
                container,
                FlexStyle {
                    width: Some(size_dp),
                    height: Some(size_dp),
                    justify_content: FlexJustifyContent::Center,
                    align_items: FlexAlign::Center,
                    ..Default::default()
                },
            );
        }

        // -- Icon text --
        let icon_widget = Text::new(self.icon.clone()).color(render_engine::Color::rgba(
            content_color.x,
            content_color.y,
            content_color.z,
            content_color.w,
        ));
        let icon_id = icon_widget.build(ctx);
        ctx.reparent_to(icon_id, container);

        // -- Label text (extended FAB only) --
        if let Some(ref label_text) = self.label {
            let label_widget = Text::new(label_text.clone()).color(render_engine::Color::rgba(
                content_color.x,
                content_color.y,
                content_color.z,
                content_color.w,
            ));
            let label_id = label_widget.build(ctx);
            ctx.reparent_to(label_id, container);
        }

        // -- Click handler --
        if !self.disabled
            && let Some(ref callback) = self.on_click
        {
            let cb = Arc::clone(callback);
            ctx.add_clickable(container, Arc::new(move || cb()));
        }

        container
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, Ordering};

    #[test]
    fn test_fab_default_regular_size() {
        let mut ctx = WidgetContext::new_test();
        let fab = Fab::new("+");
        let root_id = fab.build(&mut ctx);

        // Node should exist
        assert!(
            ctx.scene().get_node(root_id).is_some(),
            "FAB root node should exist in scene"
        );

        // Should be styled with primary_container fallback color
        let node = ctx.scene().get_node(root_id).unwrap();
        if let NodeContent::Styled { ref style } = node.content {
            assert!(!style.fills.is_empty(), "FAB should have a fill");
            if let render_engine::Paint::Solid(color) = &style.fills[0] {
                assert!(
                    (color.x - FALLBACK_PRIMARY_CONTAINER.x).abs() < 0.01,
                    "FAB fill should match primary_container fallback"
                );
            }
            // Corner radius should be 16dp (regular)
            assert!(
                (style.corner_radii.top_left - REGULAR_CORNER_RADIUS).abs() < 0.01,
                "Regular FAB corner radius should be {}dp",
                REGULAR_CORNER_RADIUS
            );
        } else {
            panic!("FAB container should be Styled content");
        }
    }

    #[test]
    fn test_fab_small_size() {
        let mut ctx = WidgetContext::new_test();
        let fab = Fab::new("+").small();
        let root_id = fab.build(&mut ctx);

        let node = ctx.scene().get_node(root_id).unwrap();
        if let NodeContent::Styled { ref style } = node.content {
            assert!(
                (style.corner_radii.top_left - SMALL_CORNER_RADIUS).abs() < 0.01,
                "Small FAB corner radius should be {}dp",
                SMALL_CORNER_RADIUS
            );
        } else {
            panic!("FAB container should be Styled content");
        }
    }

    #[test]
    fn test_fab_large_size() {
        let mut ctx = WidgetContext::new_test();
        let fab = Fab::new("+").large();
        let root_id = fab.build(&mut ctx);

        let node = ctx.scene().get_node(root_id).unwrap();
        if let NodeContent::Styled { ref style } = node.content {
            assert!(
                (style.corner_radii.top_left - LARGE_CORNER_RADIUS).abs() < 0.01,
                "Large FAB corner radius should be {}dp",
                LARGE_CORNER_RADIUS
            );
        } else {
            panic!("FAB container should be Styled content");
        }
    }

    #[test]
    fn test_fab_extended_with_label() {
        let mut ctx = WidgetContext::new_test();
        let fab = Fab::new("+").label("Create");
        let root_id = fab.build(&mut ctx);

        let root_node = ctx.scene().get_node(root_id).unwrap();

        // Extended FAB should have 2 children: icon + label
        assert_eq!(
            root_node.children.len(),
            2,
            "Extended FAB should have 2 children (icon + label), got {}",
            root_node.children.len()
        );
    }

    #[test]
    fn test_fab_on_click() {
        let clicked = Arc::new(AtomicBool::new(false));
        let clicked_clone = Arc::clone(&clicked);

        let mut ctx = WidgetContext::new_test();
        let fab = Fab::new("+").on_click(move || {
            clicked_clone.store(true, Ordering::SeqCst);
        });
        let root_id = fab.build(&mut ctx);

        // Should be clickable
        assert!(
            ctx.has_clickable(root_id),
            "FAB with on_click should be clickable"
        );

        // Clicking should fire callback
        ctx.trigger_click(root_id);
        assert!(
            clicked.load(Ordering::SeqCst),
            "FAB click callback should have been invoked"
        );
    }

    #[test]
    fn test_fab_disabled() {
        let clicked = Arc::new(AtomicBool::new(false));
        let clicked_clone = Arc::clone(&clicked);

        let mut ctx = WidgetContext::new_test();
        let fab = Fab::new("+").disabled(true).on_click(move || {
            clicked_clone.store(true, Ordering::SeqCst);
        });
        let root_id = fab.build(&mut ctx);

        // Should NOT be clickable
        assert!(
            !ctx.has_clickable(root_id),
            "Disabled FAB should not be clickable"
        );
    }

    #[test]
    fn test_fab_with_theme() {
        let mut ctx = WidgetContext::new_test();
        let theme = MaterialTheme::from_seed(Vec4::new(0.4, 0.2, 0.8, 1.0));
        ctx.set_extension(theme);

        let fab = Fab::new("+");
        let root_id = fab.build(&mut ctx);

        assert!(
            ctx.scene().get_node(root_id).is_some(),
            "FAB with theme should build successfully"
        );
    }

    #[test]
    fn test_fab_icon_child_exists() {
        let mut ctx = WidgetContext::new_test();
        let fab = Fab::new("+");
        let root_id = fab.build(&mut ctx);

        let root_node = ctx.scene().get_node(root_id).unwrap();

        // Standard FAB should have 1 child (the icon)
        assert_eq!(
            root_node.children.len(),
            1,
            "Standard FAB should have 1 child (icon), got {}",
            root_node.children.len()
        );
    }

    #[test]
    fn test_fab_disabled_reduced_alpha() {
        let mut ctx = WidgetContext::new_test();
        let fab = Fab::new("+").disabled(true);
        let root_id = fab.build(&mut ctx);

        let node = ctx.scene().get_node(root_id).unwrap();
        if let NodeContent::Styled { ref style } = node.content {
            if let render_engine::Paint::Solid(color) = &style.fills[0] {
                assert!(
                    (color.w - DISABLED_ALPHA).abs() < 0.01,
                    "Disabled FAB should have reduced alpha ({}), got {}",
                    DISABLED_ALPHA,
                    color.w
                );
            }
        } else {
            panic!("FAB container should be Styled content");
        }
    }
}
