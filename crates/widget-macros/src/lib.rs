//! Widget Macros - Procedural derive macros for automatic widget macro generation
//!
//! This crate provides derive macros that automatically generate declarative macros
//! for widget instantiation:
//!
//! - `Widget` - Universal derive for all widget types (auto-detects pattern)
//! - `WidgetEnum` - For enum types used as flags
//!
//! # Example
//!
//! ```ignore
//! use widget_macros::Widget;
//!
//! #[derive(Widget)]
//! #[widget(name = "btn")]
//! pub struct Button {
//!     #[positional]
//!     text: String,
//!
//!     #[param(default = 12.0)]
//!     padding: f32,
//!
//!     #[flag]
//!     disabled: bool,
//!
//!     #[callback]
//!     on_click: Option<Arc<dyn Fn() + Send + Sync>>,
//! }
//!
//! // Generated macro usage:
//! // btn!("Click Me")
//! // btn!("Click Me", disabled, padding: 20.0)
//! ```

use proc_macro::TokenStream;

mod generate;
mod parse;
mod widget;
mod widget_enum;

/// Universal derive macro for widgets
///
/// Generates both declarative macros for widget instantiation AND the
/// `Widget::build()` implementation for scene construction.
///
/// Automatically detects the widget pattern based on field attributes:
/// - Has `#[positional]` → Display widget (leaf widget)
/// - Has `#[children]` → Container widget (layout widget)
/// - Has `#[scaffold]` → Scaffold widget (positional body + named slots)
///
/// # Struct-Level Attributes
///
/// - `#[widget(name = "x")]` - Override macro name (default: snake_case of struct name)
/// - `#[widget(name = "x", alias = "y")]` - Add an alias macro
/// - `#[widget(constructor = "new")]` - Override constructor (default: "new")
/// - `#[widget(skip_impl)]` - Skip generating Widget::build() impl
/// - `#[layout(...)]` - Configure flexbox layout (see Layout Attributes below)
/// - `#[style(...)]` - Configure visual styling (see Style Attributes below)
///
/// # Layout Attributes `#[layout(...)]`
///
/// - `direction = Row|Column|RowReverse|ColumnReverse` - Flex direction
/// - `gap = 10.0` - Gap between children
/// - `padding = 16.0` - Uniform padding
/// - `padding_left/right/top/bottom = 8.0` - Individual padding
/// - `justify = FlexStart|FlexEnd|Center|SpaceBetween|SpaceAround|SpaceEvenly`
/// - `align = FlexStart|FlexEnd|Center|Stretch|Baseline` - Align items
/// - `flex = 1.0` - Flex grow factor (for slot layouts)
///
/// # Style Attributes `#[style(...)]`
///
/// - `background = White|Black|Red|Green|Blue|Transparent|<var>` - Background color
/// - `corner_radius = 8.0` - Corner radius
/// - `shadow = 2.0` - Shadow elevation
/// - `opacity = 0.5` - Opacity
///
/// # Field-Level Attributes
///
/// - `#[positional]` - First positional argument
/// - `#[positional(reactive)]` - Positional field with `@signal` support. This enables the `txt!(@read)` syntax in the generated macro.
/// - `#[children]` - Multiple children (uses WidgetTuple trait)
/// - `#[scaffold]` - Named slots for scaffold pattern: `"slot" => widget`
/// - `#[layout(...)]` - Slot-specific layout (applied to child's layout style)
/// - `#[param]` - Named parameter
/// - `#[param(default = X)]` - Named parameter with default value
/// - `#[param(setter = "name")]` - Use different setter method name
/// - `#[flag]` - Boolean flag (bare identifier in macro)
/// - `#[method_flag(a, b)]` - Method-based flags (calls `.a()`, `.b()`)
/// - `#[callback]` - Callback parameter
///
/// # Examples
///
/// ## Display Widget (leaf)
/// ```ignore
/// #[derive(Widget)]
/// #[widget(name = "txt")]
/// pub struct Text {
///     // Mark the content as reactive to enable special macro syntax
///     #[positional(reactive)]
///     content: TextContent,
///
///     #[param(setter = "size")]
///     font_size: f32,
/// }
///
/// // Usage:
/// // txt!("Hello")                  // Static string
/// // txt!(@read_signal, size: 20.0) // Reactive signal (via @ syntax)
/// ```
///
/// ## Container Widget with Layout
/// ```ignore
/// #[derive(Widget)]
/// #[widget(name = "col")]
/// #[layout(direction = Column, gap = 8.0, padding = 16.0)]
/// #[style(background = White, corner_radius = 8.0)]
/// pub struct Column<C: WidgetTuple> {
///     #[children]
///     children: C,
/// }
///
/// // Usage: col!([child1, child2], gap: 10.0)
/// ```
///
/// ## Scaffold Widget (positional body + named slots)
/// ```ignore
/// #[derive(Widget)]
/// #[widget(name = "scaffold", skip_impl)]
/// pub struct Scaffold<B: Widget, S: NamedWidgetTuple> {
///     #[positional]
///     body: B,
///
///     #[scaffold]
///     slots: S,  // Type-safe tuple of named slots
/// }
///
/// // Usage: scaffold!(body; "app_bar" => header, "fab" => button)
/// ```
#[proc_macro_derive(
    Widget,
    attributes(
        widget,
        layout,
        style,
        positional,
        children,
        scaffold,
        param,
        flag,
        method_flag,
        callback
    )
)]
pub fn derive_widget(input: TokenStream) -> TokenStream {
    widget::derive(input.into()).into()
}

// Re-export under old name for backwards compatibility
#[doc(hidden)]
#[proc_macro_derive(
    WidgetMacro,
    attributes(
        widget_macro,
        layout,
        style,
        widget,
        positional,
        children,
        named_children,
        param,
        flag,
        method_flag,
        callback
    )
)]
pub fn derive_widget_macro_compat(input: TokenStream) -> TokenStream {
    widget::derive(input.into()).into()
}

/// Derive macro for enum types used as flags in widget macros
///
/// Allows enum variants to be used as bare identifiers in generated macros.
///
/// # Variant Attributes
///
/// - `#[flag]` - Makes this variant usable as a bare identifier
/// - `#[default]` - Marks the default variant (use with `#[derive(Default)]`)
///
/// # Example
///
/// ```ignore
/// #[derive(Default, Clone, Copy, WidgetEnum)]
/// pub enum ButtonStyle {
///     #[default]
///     Default,
///     #[flag]
///     Primary,
///     #[flag]
///     Secondary,
/// }
///
/// // In generated macro:
/// btn!("Click", primary)      // Sets style to ButtonStyle::Primary
/// btn!("Click", secondary)    // Sets style to ButtonStyle::Secondary
/// ```
#[proc_macro_derive(WidgetEnum, attributes(flag))]
pub fn derive_widget_enum(input: TokenStream) -> TokenStream {
    widget_enum::derive(input.into()).into()
}
