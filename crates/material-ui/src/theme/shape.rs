/// MD3 shape scale — corner radius presets.
///
/// Maps to Material Design 3's shape system with seven levels
/// from `none` (sharp corners) to `full` (pill shape).
#[derive(Debug, Clone, Copy)]
pub struct ShapeScale {
    /// No rounding (0px)
    pub none: f32,
    /// Extra small rounding (4px)
    pub extra_small: f32,
    /// Small rounding (8px)
    pub small: f32,
    /// Medium rounding (12px)
    pub medium: f32,
    /// Large rounding (16px)
    pub large: f32,
    /// Extra large rounding (28px)
    pub extra_large: f32,
    /// Full rounding — pill shape (9999px)
    pub full: f32,
}

impl Default for ShapeScale {
    fn default() -> Self {
        Self {
            none: 0.0,
            extra_small: 4.0,
            small: 8.0,
            medium: 12.0,
            large: 16.0,
            extra_large: 28.0,
            full: 9999.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_shape_scale() {
        let scale = ShapeScale::default();
        assert_eq!(scale.none, 0.0);
        assert_eq!(scale.extra_small, 4.0);
        assert_eq!(scale.small, 8.0);
        assert_eq!(scale.medium, 12.0);
        assert_eq!(scale.large, 16.0);
        assert_eq!(scale.extra_large, 28.0);
        assert_eq!(scale.full, 9999.0);
    }

    #[test]
    fn test_shape_scale_is_monotonically_increasing() {
        let scale = ShapeScale::default();
        assert!(scale.none < scale.extra_small);
        assert!(scale.extra_small < scale.small);
        assert!(scale.small < scale.medium);
        assert!(scale.medium < scale.large);
        assert!(scale.large < scale.extra_large);
        assert!(scale.extra_large < scale.full);
    }
}
