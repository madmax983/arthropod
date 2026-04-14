//! Material Design 3 component wrappers.
//!
//! MD3-styled widgets that implement the [`Widget`](widget_core::Widget)
//! trait directly, applying Material Design 3 colors, typography, and shape
//! from [`MaterialTheme`](crate::theme::MaterialTheme).

mod button;
mod card;
mod checkbox;
mod divider;
mod form;
mod icon;
mod progress_bar;
mod text_input;

pub use button::{MaterialButton, MaterialButtonVariant};
pub use card::{CardVariant, MaterialCard};
pub use checkbox::MaterialCheckbox;
pub use divider::{DividerVariant, MaterialDivider};
pub use form::MaterialForm;
pub use icon::MaterialIcon;
pub use progress_bar::MaterialProgressBar;
pub use text_input::{MaterialTextInput, TextInputVariant};
