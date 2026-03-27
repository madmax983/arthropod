//! Integration tests for MaterialTheme.
//!
//! These test cross-module behavior: theme generation -> design token conversion,
//! typography/shape/elevation MD3 spec compliance (full slot coverage), and
//! WidgetContext extension storage round-trip.
//!
//! Unit tests for individual sub-modules live in their respective files.

use glam::Vec4;
use material_ui::theme::{ElevationScale, FontWeight, MaterialTheme, ShapeScale, TypographyScale};

// ---------------------------------------------------------------------------
// Cross-module integration: from_seed -> all subsystems populated
// ---------------------------------------------------------------------------

#[test]
fn test_material_theme_from_seed_populates_all_subsystems() {
    let theme = MaterialTheme::from_seed(Vec4::new(0.4, 0.3, 0.9, 1.0));

    // Color subsystem accessible
    assert_eq!(theme.color.primary.w, 1.0);
    assert_eq!(theme.color.surface.w, 1.0);
    assert_eq!(theme.color.error.w, 1.0);

    // Typography subsystem accessible
    assert!(theme.typography.display_large.font_size > 0.0);
    assert!(theme.typography.label_small.font_size > 0.0);

    // Shape subsystem accessible
    assert!(theme.shape.full > theme.shape.none);

    // Elevation subsystem accessible
    assert!(theme.elevation.level5.shadow_offset > theme.elevation.level0.shadow_offset);
}

// ---------------------------------------------------------------------------
// Cross-module integration: to_design_tokens mapping correctness
// ---------------------------------------------------------------------------

#[test]
fn test_to_design_tokens_maps_color_roles_correctly() {
    let theme = MaterialTheme::from_seed(Vec4::new(0.4, 0.3, 0.9, 1.0));
    let tokens = theme.to_design_tokens();

    // Surface should map from MD3 surface (as_color returns Vec4)
    let surface_color = tokens.surface_primary.as_color();
    assert!(
        surface_color.w > 0.0,
        "surface_primary should have a non-zero alpha"
    );

    // Accent should be primary
    assert_eq!(tokens.accent, theme.color.primary);

    // Text tokens should map from on_surface roles
    assert_eq!(tokens.text_primary, theme.color.on_surface);
    assert_eq!(tokens.text_secondary, theme.color.on_surface_variant);
    assert_eq!(tokens.text_tertiary, theme.color.outline);

    // Accent hover/pressed should map from primary container roles
    assert_eq!(tokens.accent_hover, theme.color.primary_container);
    assert_eq!(tokens.accent_pressed, theme.color.on_primary_container);
}

#[test]
fn test_to_design_tokens_spacing_follows_8px_scale() {
    let theme = MaterialTheme::from_seed(Vec4::new(0.4, 0.3, 0.9, 1.0));
    let tokens = theme.to_design_tokens();

    assert_eq!(tokens.space_xs, 4.0);
    assert_eq!(tokens.space_sm, 8.0);
    assert_eq!(tokens.space_md, 16.0);
    assert_eq!(tokens.space_lg, 24.0);
    assert_eq!(tokens.space_xl, 32.0);
    assert_eq!(tokens.space_2xl, 48.0);
}

#[test]
fn test_to_design_tokens_radius_maps_from_shape_scale() {
    let theme = MaterialTheme::from_seed(Vec4::new(0.4, 0.3, 0.9, 1.0));
    let tokens = theme.to_design_tokens();

    assert_eq!(tokens.radius_sm, theme.shape.extra_small);
    assert_eq!(tokens.radius_md, theme.shape.small);
    assert_eq!(tokens.radius_lg, theme.shape.medium);
    assert_eq!(tokens.radius_xl, theme.shape.large);
}

// ---------------------------------------------------------------------------
// MD3 typography spec compliance (all 15 slots)
// ---------------------------------------------------------------------------

#[test]
fn test_typography_display_slots_match_md3_spec() {
    let typo = TypographyScale::default();

    assert_eq!(typo.display_large.font_size, 57.0);
    assert_eq!(typo.display_large.line_height, 64.0);
    assert_eq!(typo.display_large.font_weight, FontWeight::Regular);

    assert_eq!(typo.display_medium.font_size, 45.0);
    assert_eq!(typo.display_medium.line_height, 52.0);

    assert_eq!(typo.display_small.font_size, 36.0);
    assert_eq!(typo.display_small.line_height, 44.0);
}

#[test]
fn test_typography_headline_slots_match_md3_spec() {
    let typo = TypographyScale::default();

    assert_eq!(typo.headline_large.font_size, 32.0);
    assert_eq!(typo.headline_large.line_height, 40.0);

    assert_eq!(typo.headline_medium.font_size, 28.0);
    assert_eq!(typo.headline_medium.line_height, 36.0);

    assert_eq!(typo.headline_small.font_size, 24.0);
    assert_eq!(typo.headline_small.line_height, 32.0);
}

#[test]
fn test_typography_title_slots_match_md3_spec() {
    let typo = TypographyScale::default();

    assert_eq!(typo.title_large.font_size, 22.0);
    assert_eq!(typo.title_large.line_height, 28.0);

    assert_eq!(typo.title_medium.font_size, 16.0);
    assert_eq!(typo.title_medium.line_height, 24.0);
    assert_eq!(typo.title_medium.font_weight, FontWeight::Medium);

    assert_eq!(typo.title_small.font_size, 14.0);
    assert_eq!(typo.title_small.line_height, 20.0);
    assert_eq!(typo.title_small.font_weight, FontWeight::Medium);
}

#[test]
fn test_typography_body_slots_match_md3_spec() {
    let typo = TypographyScale::default();

    assert_eq!(typo.body_large.font_size, 16.0);
    assert_eq!(typo.body_large.line_height, 24.0);
    assert_eq!(typo.body_large.font_weight, FontWeight::Regular);

    assert_eq!(typo.body_medium.font_size, 14.0);
    assert_eq!(typo.body_medium.line_height, 20.0);
    assert_eq!(typo.body_medium.font_weight, FontWeight::Regular);

    assert_eq!(typo.body_small.font_size, 12.0);
    assert_eq!(typo.body_small.line_height, 16.0);
}

#[test]
fn test_typography_label_slots_match_md3_spec() {
    let typo = TypographyScale::default();

    assert_eq!(typo.label_large.font_size, 14.0);
    assert_eq!(typo.label_large.line_height, 20.0);
    assert_eq!(typo.label_large.font_weight, FontWeight::Medium);

    assert_eq!(typo.label_medium.font_size, 12.0);
    assert_eq!(typo.label_medium.line_height, 16.0);
    assert_eq!(typo.label_medium.font_weight, FontWeight::Medium);

    assert_eq!(typo.label_small.font_size, 11.0);
    assert_eq!(typo.label_small.line_height, 16.0);
    assert_eq!(typo.label_small.font_weight, FontWeight::Medium);
}

// ---------------------------------------------------------------------------
// MD3 shape spec compliance
// ---------------------------------------------------------------------------

#[test]
fn test_shape_defaults_match_md3_spec() {
    let shape = ShapeScale::default();
    assert_eq!(shape.none, 0.0);
    assert_eq!(shape.extra_small, 4.0);
    assert_eq!(shape.small, 8.0);
    assert_eq!(shape.medium, 12.0);
    assert_eq!(shape.large, 16.0);
    assert_eq!(shape.extra_large, 28.0);
    assert_eq!(shape.full, 9999.0);
}

// ---------------------------------------------------------------------------
// MD3 elevation spec compliance
// ---------------------------------------------------------------------------

#[test]
fn test_elevation_defaults_tint_and_shadow() {
    let elevation = ElevationScale::default();

    // Level 0: no tint, no shadow
    assert_eq!(elevation.level0.tint_opacity, 0.0);
    assert_eq!(elevation.level0.shadow_offset, 0.0);

    // Higher levels have strictly more tint and shadow than level 0
    assert!(elevation.level1.tint_opacity > 0.0);
    assert!(elevation.level1.shadow_offset > 0.0);

    // Level ordering: tint and shadow increase monotonically
    assert!(elevation.level3.tint_opacity > elevation.level1.tint_opacity);
    assert!(elevation.level5.shadow_offset > elevation.level3.shadow_offset);
}

// ---------------------------------------------------------------------------
// Light vs dark theme integration
// ---------------------------------------------------------------------------

#[test]
fn test_light_and_dark_themes_surface_contrast() {
    let seed = Vec4::new(0.4, 0.3, 0.9, 1.0);
    let light = MaterialTheme::from_seed_light(seed);
    let dark = MaterialTheme::from_seed_dark(seed);

    let surface_diff = (light.color.surface - dark.color.surface).length();
    assert!(
        surface_diff > 0.5,
        "Light/dark surfaces should differ significantly, got {surface_diff}"
    );
}

#[test]
fn test_light_and_dark_themes_primary_contrast() {
    let seed = Vec4::new(0.4, 0.3, 0.9, 1.0);
    let light = MaterialTheme::from_seed_light(seed);
    let dark = MaterialTheme::from_seed_dark(seed);

    let primary_diff = (light.color.primary - dark.color.primary).length();
    assert!(
        primary_diff > 0.2,
        "Light/dark primaries should differ, got {primary_diff}"
    );
}

#[test]
fn test_light_and_dark_themes_share_shape_and_typography() {
    let seed = Vec4::new(0.4, 0.3, 0.9, 1.0);
    let light = MaterialTheme::from_seed_light(seed);
    let dark = MaterialTheme::from_seed_dark(seed);

    // Shape and typography should be identical across light/dark
    assert_eq!(light.shape.medium, dark.shape.medium);
    assert_eq!(
        light.typography.body_medium.font_size,
        dark.typography.body_medium.font_size
    );
}

// ---------------------------------------------------------------------------
// WidgetContext extension round-trip (cross-crate integration)
// ---------------------------------------------------------------------------

#[test]
fn test_theme_stored_in_widget_context_extensions() {
    let theme = MaterialTheme::from_seed(Vec4::new(0.4, 0.3, 0.9, 1.0));
    let mut ctx = widget_core::WidgetContext::new_test();
    ctx.set_extension(theme.clone());

    let retrieved = ctx
        .get_extension::<MaterialTheme>()
        .expect("MaterialTheme should be retrievable from extensions");
    assert_eq!(retrieved.color.primary, theme.color.primary);
    assert_eq!(retrieved.shape.medium, 12.0);
    assert_eq!(
        retrieved.typography.body_medium.font_size,
        theme.typography.body_medium.font_size
    );
}

#[test]
fn test_theme_design_tokens_set_in_widget_context() {
    let theme = MaterialTheme::from_seed(Vec4::new(0.4, 0.3, 0.9, 1.0));
    let tokens = theme.to_design_tokens();

    let mut ctx = widget_core::WidgetContext::new_test();
    ctx.set_design_tokens(tokens.clone());

    let retrieved = ctx.design_tokens().expect("tokens should be set");
    assert_eq!(retrieved.accent, theme.color.primary);
    assert_eq!(retrieved.space_md, 16.0);
}
