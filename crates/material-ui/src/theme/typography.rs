/// Font weight for MD3 type scale.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FontWeight {
    /// 400 weight
    Regular,
    /// 500 weight
    Medium,
    /// 700 weight
    Bold,
}

/// A single text style in the MD3 type scale.
#[derive(Debug, Clone, Copy)]
pub struct TextStyle {
    pub font_size: f32,
    pub line_height: f32,
    pub font_weight: FontWeight,
    pub letter_spacing: f32,
}

/// MD3 typography scale with all 15 slots.
///
/// Covers display, headline, title, body, and label categories,
/// each with large/medium/small variants.
#[derive(Debug, Clone)]
pub struct TypographyScale {
    pub display_large: TextStyle,
    pub display_medium: TextStyle,
    pub display_small: TextStyle,
    pub headline_large: TextStyle,
    pub headline_medium: TextStyle,
    pub headline_small: TextStyle,
    pub title_large: TextStyle,
    pub title_medium: TextStyle,
    pub title_small: TextStyle,
    pub body_large: TextStyle,
    pub body_medium: TextStyle,
    pub body_small: TextStyle,
    pub label_large: TextStyle,
    pub label_medium: TextStyle,
    pub label_small: TextStyle,
}

impl Default for TypographyScale {
    fn default() -> Self {
        Self {
            display_large: TextStyle {
                font_size: 57.0,
                line_height: 64.0,
                font_weight: FontWeight::Regular,
                letter_spacing: -0.25,
            },
            display_medium: TextStyle {
                font_size: 45.0,
                line_height: 52.0,
                font_weight: FontWeight::Regular,
                letter_spacing: 0.0,
            },
            display_small: TextStyle {
                font_size: 36.0,
                line_height: 44.0,
                font_weight: FontWeight::Regular,
                letter_spacing: 0.0,
            },
            headline_large: TextStyle {
                font_size: 32.0,
                line_height: 40.0,
                font_weight: FontWeight::Regular,
                letter_spacing: 0.0,
            },
            headline_medium: TextStyle {
                font_size: 28.0,
                line_height: 36.0,
                font_weight: FontWeight::Regular,
                letter_spacing: 0.0,
            },
            headline_small: TextStyle {
                font_size: 24.0,
                line_height: 32.0,
                font_weight: FontWeight::Regular,
                letter_spacing: 0.0,
            },
            title_large: TextStyle {
                font_size: 22.0,
                line_height: 28.0,
                font_weight: FontWeight::Regular,
                letter_spacing: 0.0,
            },
            title_medium: TextStyle {
                font_size: 16.0,
                line_height: 24.0,
                font_weight: FontWeight::Medium,
                letter_spacing: 0.15,
            },
            title_small: TextStyle {
                font_size: 14.0,
                line_height: 20.0,
                font_weight: FontWeight::Medium,
                letter_spacing: 0.1,
            },
            body_large: TextStyle {
                font_size: 16.0,
                line_height: 24.0,
                font_weight: FontWeight::Regular,
                letter_spacing: 0.5,
            },
            body_medium: TextStyle {
                font_size: 14.0,
                line_height: 20.0,
                font_weight: FontWeight::Regular,
                letter_spacing: 0.25,
            },
            body_small: TextStyle {
                font_size: 12.0,
                line_height: 16.0,
                font_weight: FontWeight::Regular,
                letter_spacing: 0.4,
            },
            label_large: TextStyle {
                font_size: 14.0,
                line_height: 20.0,
                font_weight: FontWeight::Medium,
                letter_spacing: 0.1,
            },
            label_medium: TextStyle {
                font_size: 12.0,
                line_height: 16.0,
                font_weight: FontWeight::Medium,
                letter_spacing: 0.5,
            },
            label_small: TextStyle {
                font_size: 11.0,
                line_height: 16.0,
                font_weight: FontWeight::Medium,
                letter_spacing: 0.5,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_typography_scale() {
        let scale = TypographyScale::default();
        // Display large should be the biggest
        assert_eq!(scale.display_large.font_size, 57.0);
        // Label small should be the smallest
        assert_eq!(scale.label_small.font_size, 11.0);
        // Title medium uses Medium weight
        assert_eq!(scale.title_medium.font_weight, FontWeight::Medium);
        // Body uses Regular weight
        assert_eq!(scale.body_large.font_weight, FontWeight::Regular);
    }
}
