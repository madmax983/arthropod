/// A single elevation level in the MD3 scale.
///
/// Material Design 3 uses tonal elevation (surface tint overlay)
/// rather than traditional shadow-based elevation.
#[derive(Debug, Clone, Copy)]
pub struct ElevationLevel {
    /// Opacity of the surface tint overlay (0.0 = none, 1.0 = full)
    pub tint_opacity: f32,
    /// Shadow offset in logical pixels
    pub shadow_offset: f32,
}

/// MD3 elevation scale — six tonal elevation levels (0-5).
///
/// Level 0 is the base surface with no tint or shadow.
/// Higher levels add progressively more tint and shadow.
#[derive(Debug, Clone, Copy)]
pub struct ElevationScale {
    pub level0: ElevationLevel,
    pub level1: ElevationLevel,
    pub level2: ElevationLevel,
    pub level3: ElevationLevel,
    pub level4: ElevationLevel,
    pub level5: ElevationLevel,
}

impl Default for ElevationScale {
    fn default() -> Self {
        Self {
            level0: ElevationLevel {
                tint_opacity: 0.0,
                shadow_offset: 0.0,
            },
            level1: ElevationLevel {
                tint_opacity: 0.05,
                shadow_offset: 1.0,
            },
            level2: ElevationLevel {
                tint_opacity: 0.08,
                shadow_offset: 3.0,
            },
            level3: ElevationLevel {
                tint_opacity: 0.11,
                shadow_offset: 6.0,
            },
            level4: ElevationLevel {
                tint_opacity: 0.12,
                shadow_offset: 8.0,
            },
            level5: ElevationLevel {
                tint_opacity: 0.14,
                shadow_offset: 12.0,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_elevation_scale() {
        let scale = ElevationScale::default();
        assert_eq!(scale.level0.tint_opacity, 0.0);
        assert_eq!(scale.level0.shadow_offset, 0.0);
        assert_eq!(scale.level5.tint_opacity, 0.14);
        assert_eq!(scale.level5.shadow_offset, 12.0);
    }

    #[test]
    fn test_elevation_scale_monotonically_increasing() {
        let scale = ElevationScale::default();
        let levels = [
            scale.level0,
            scale.level1,
            scale.level2,
            scale.level3,
            scale.level4,
            scale.level5,
        ];
        for i in 1..levels.len() {
            assert!(
                levels[i].tint_opacity >= levels[i - 1].tint_opacity,
                "Tint opacity should increase: level{} ({}) < level{} ({})",
                i - 1,
                levels[i - 1].tint_opacity,
                i,
                levels[i].tint_opacity
            );
            assert!(
                levels[i].shadow_offset >= levels[i - 1].shadow_offset,
                "Shadow offset should increase: level{} ({}) < level{} ({})",
                i - 1,
                levels[i - 1].shadow_offset,
                i,
                levels[i].shadow_offset
            );
        }
    }
}
