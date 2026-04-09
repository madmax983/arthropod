use serde::{Deserialize, Serialize};

/// Defines independent border radii for the four corners of a rectangle.
///
/// Used to create pills, circles, or asymmetric rounded containers.
/// In CSS, this corresponds to `border-radius: <top-left> <top-right> <bottom-right> <bottom-left>;`.
///
/// ## Examples
/// ```
/// use style_engine::CornerRadii;
/// // Create a uniform pill shape (e.g. for a button)
/// let uniform = CornerRadii::uniform(999.0);
///
/// // Create a chat bubble tail effect (sharp bottom-right corner)
/// let chat_bubble = CornerRadii::new(12.0, 12.0, 0.0, 12.0);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[allow(missing_docs)]
pub struct CornerRadii {
    pub top_left: f32,
    pub top_right: f32,
    pub bottom_right: f32,
    pub bottom_left: f32,
}

impl CornerRadii {
    /// Create corner radii with individual values for each corner
    pub fn new(top_left: f32, top_right: f32, bottom_right: f32, bottom_left: f32) -> Self {
        Self {
            top_left,
            top_right,
            bottom_right,
            bottom_left,
        }
    }

    /// Create corner radii with all corners having the same radius
    pub fn uniform(radius: f32) -> Self {
        Self {
            top_left: radius,
            top_right: radius,
            bottom_right: radius,
            bottom_left: radius,
        }
    }

    /// Zero corner radii (sharp corners)
    pub const ZERO: Self = Self {
        top_left: 0.0,
        top_right: 0.0,
        bottom_right: 0.0,
        bottom_left: 0.0,
    };

    /// Check if all corners have the same radius
    pub fn is_uniform(&self) -> bool {
        self.top_left == self.top_right
            && self.top_right == self.bottom_right
            && self.bottom_right == self.bottom_left
    }

    /// Check if all corners are zero (sharp corners)
    pub fn is_zero(&self) -> bool {
        self.top_left == 0.0
            && self.top_right == 0.0
            && self.bottom_right == 0.0
            && self.bottom_left == 0.0
    }

    /// Convert to array [top_left, top_right, bottom_right, bottom_left]
    pub fn to_array(&self) -> [f32; 4] {
        [
            self.top_left,
            self.top_right,
            self.bottom_right,
            self.bottom_left,
        ]
    }
}

impl From<f32> for CornerRadii {
    fn from(radius: f32) -> Self {
        Self::uniform(radius)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uniform_corners() {
        let radii = CornerRadii::uniform(8.0);
        assert_eq!(radii.top_left, 8.0);
        assert_eq!(radii.top_right, 8.0);
        assert_eq!(radii.bottom_right, 8.0);
        assert_eq!(radii.bottom_left, 8.0);
        assert!(radii.is_uniform(), "uniform(8.0) should be uniform");
    }

    #[test]
    fn test_zero_corners() {
        let radii = CornerRadii::ZERO;
        assert!(radii.is_zero(), "ZERO should be zero");
        assert_eq!(radii.to_array(), [0.0, 0.0, 0.0, 0.0]);
    }

    #[test]
    fn test_per_corner_radii() {
        let radii = CornerRadii {
            top_left: 8.0,
            top_right: 4.0,
            bottom_right: 2.0,
            bottom_left: 1.0,
        };
        assert!(
            !radii.is_uniform(),
            "per-corner radii should not be uniform"
        );
        assert_eq!(radii.to_array(), [8.0, 4.0, 2.0, 1.0]);
    }

    #[test]
    fn test_serde_roundtrip() {
        let original = CornerRadii {
            top_left: 8.0,
            top_right: 4.0,
            bottom_right: 2.0,
            bottom_left: 1.0,
        };

        let json = serde_json::to_string(&original).expect("serialize failed");
        let deserialized: CornerRadii = serde_json::from_str(&json).expect("deserialize failed");

        assert_eq!(
            original, deserialized,
            "serde roundtrip should preserve data"
        );
    }
}
