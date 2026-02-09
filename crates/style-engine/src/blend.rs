use serde::{Deserialize, Serialize};

/// Blend mode for layer blending (19 Figma variants)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum BlendMode {
    /// Normal blending (default)
    #[default]
    Normal,
    /// Darken blending
    Darken,
    /// Multiply blending
    Multiply,
    /// Color burn blending
    ColorBurn,
    /// Lighten blending
    Lighten,
    /// Screen blending
    Screen,
    /// Color dodge blending
    ColorDodge,
    /// Overlay blending
    Overlay,
    /// Soft light blending
    SoftLight,
    /// Hard light blending
    HardLight,
    /// Difference blending
    Difference,
    /// Exclusion blending
    Exclusion,
    /// Hue blending
    Hue,
    /// Saturation blending
    Saturation,
    /// Color blending
    Color,
    /// Luminosity blending
    Luminosity,
    /// Linear burn blending
    LinearBurn,
    /// Linear dodge blending
    LinearDodge,
    /// Pass through (for groups)
    PassThrough,
}

impl BlendMode {
    /// Convert blend mode to 5-bit flag for shader
    pub fn to_flag_bits(&self) -> u8 {
        match self {
            Self::Normal => 0,
            Self::Darken => 1,
            Self::Multiply => 2,
            Self::ColorBurn => 3,
            Self::Lighten => 4,
            Self::Screen => 5,
            Self::ColorDodge => 6,
            Self::Overlay => 7,
            Self::SoftLight => 8,
            Self::HardLight => 9,
            Self::Difference => 10,
            Self::Exclusion => 11,
            Self::Hue => 12,
            Self::Saturation => 13,
            Self::Color => 14,
            Self::Luminosity => 15,
            Self::LinearBurn => 16,
            Self::LinearDodge => 17,
            Self::PassThrough => 18,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn test_default_is_normal() {
        assert_eq!(BlendMode::default(), BlendMode::Normal);
    }

    #[test]
    fn test_all_19_variants_exist() {
        // List all 19 Figma blend modes
        let modes = vec![
            BlendMode::Normal,
            BlendMode::Darken,
            BlendMode::Multiply,
            BlendMode::ColorBurn,
            BlendMode::Lighten,
            BlendMode::Screen,
            BlendMode::ColorDodge,
            BlendMode::Overlay,
            BlendMode::SoftLight,
            BlendMode::HardLight,
            BlendMode::Difference,
            BlendMode::Exclusion,
            BlendMode::Hue,
            BlendMode::Saturation,
            BlendMode::Color,
            BlendMode::Luminosity,
            BlendMode::LinearBurn,
            BlendMode::LinearDodge,
            BlendMode::PassThrough,
        ];
        assert_eq!(modes.len(), 19, "Should have exactly 19 blend modes");
    }

    #[test]
    fn test_to_flag_bits_unique() {
        // Verify each mode maps to a unique 5-bit value (0-31)
        let modes = vec![
            BlendMode::Normal,
            BlendMode::Darken,
            BlendMode::Multiply,
            BlendMode::ColorBurn,
            BlendMode::Lighten,
            BlendMode::Screen,
            BlendMode::ColorDodge,
            BlendMode::Overlay,
            BlendMode::SoftLight,
            BlendMode::HardLight,
            BlendMode::Difference,
            BlendMode::Exclusion,
            BlendMode::Hue,
            BlendMode::Saturation,
            BlendMode::Color,
            BlendMode::Luminosity,
            BlendMode::LinearBurn,
            BlendMode::LinearDodge,
            BlendMode::PassThrough,
        ];

        let mut bits_set = HashSet::new();
        for mode in modes {
            let bits = mode.to_flag_bits();
            assert!(bits < 32, "Flag bits should be 5-bit (0-31)");
            assert!(
                bits_set.insert(bits),
                "Each blend mode should have unique flag bits"
            );
        }
    }

    #[test]
    fn test_serde_roundtrip() {
        let original = BlendMode::Multiply;
        let json = serde_json::to_string(&original).expect("serialize failed");
        let deserialized: BlendMode = serde_json::from_str(&json).expect("deserialize failed");
        assert_eq!(
            original, deserialized,
            "serde roundtrip should preserve data"
        );
    }
}
