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
    /// The size of the font in logical pixels.
    pub font_size: f32,
    /// The line height (leading) in logical pixels.
    pub line_height: f32,
    /// The font weight (Regular, Medium, or Bold).
    pub font_weight: FontWeight,
    /// The letter spacing (tracking) in logical pixels.
    pub letter_spacing: f32,
}

/// MD3 typography scale with all 15 slots.
///
/// Covers display, headline, title, body, and label categories,
/// each with large/medium/small variants.
///
/// # Examples
///
/// ```
/// use material_ui::theme::TypographyScale;
///
/// let typography = TypographyScale::default();
/// assert_eq!(typography.body_medium.font_size, 14.0);
/// assert_eq!(typography.display_large.font_size, 57.0);
/// ```
#[derive(Debug, Clone)]
pub struct TypographyScale {
    /// The largest text on the screen, reserved for short, important text or numerals.
    pub display_large: TextStyle,
    /// Medium-sized display text.
    pub display_medium: TextStyle,
    /// Small-sized display text.
    pub display_small: TextStyle,
    /// Large headline text, best for short, high-emphasis text on smaller screens.
    pub headline_large: TextStyle,
    /// Medium-sized headline text.
    pub headline_medium: TextStyle,
    /// Small-sized headline text.
    pub headline_small: TextStyle,
    /// Large title text, used for medium-emphasis text that remains relatively short.
    pub title_large: TextStyle,
    /// Medium-sized title text.
    pub title_medium: TextStyle,
    /// Small-sized title text.
    pub title_small: TextStyle,
    /// Large body text, used for long passages of text.
    pub body_large: TextStyle,
    /// Medium body text, the default for standard text reading.
    pub body_medium: TextStyle,
    /// Small body text.
    pub body_small: TextStyle,
    /// Large label text, used for text inside components or for very small text in the content body.
    pub label_large: TextStyle,
    /// Medium label text.
    pub label_medium: TextStyle,
    /// Small label text, the smallest readable text.
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
