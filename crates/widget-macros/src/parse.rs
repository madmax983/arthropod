//! Attribute parsing utilities for widget derive macros

use proc_macro2::{Ident, TokenStream};
use syn::{Attribute, DeriveInput, Expr, Field, Fields, Lit, Meta, Token, Variant};

// ============================================================================
// Struct-Level Configuration
// ============================================================================

/// Struct-level configuration from `#[widget_macro(...)]`
#[derive(Debug, Default)]
pub struct WidgetMacroConfig {
    /// Override macro name (default: snake_case of struct name)
    pub name: Option<String>,
    /// Optional alias for the macro
    pub alias: Option<String>,
    /// Constructor function to call (default: "new")
    pub constructor: String,
    /// Positional argument type (when constructor arg differs from field type)
    /// e.g., "`Signal<String>`" for TextInput where constructor takes Signal but stores ReadSignal/WriteSignal
    pub positional_type: Option<String>,
    /// Method to add children (default: "child" for arrays, "field" for named_children)
    pub child_method: Option<String>,
    /// Skip generating Widget::build() impl (use when struct has custom impl)
    pub skip_impl: bool,
}

impl WidgetMacroConfig {
    pub fn from_attrs(attrs: &[Attribute]) -> syn::Result<Self> {
        let mut config = Self {
            constructor: "new".to_string(),
            ..Default::default()
        };

        for attr in attrs {
            // Support both #[widget(...)] and #[widget_macro(...)] for backwards compat
            if attr.path().is_ident("widget") || attr.path().is_ident("widget_macro") {
                attr.parse_nested_meta(|meta| {
                    if meta.path.is_ident("name") {
                        let _: Token![=] = meta.input.parse()?;
                        let lit: Lit = meta.input.parse()?;
                        if let Lit::Str(s) = lit {
                            config.name = Some(s.value());
                        }
                    } else if meta.path.is_ident("alias") {
                        let _: Token![=] = meta.input.parse()?;
                        let lit: Lit = meta.input.parse()?;
                        if let Lit::Str(s) = lit {
                            config.alias = Some(s.value());
                        }
                    } else if meta.path.is_ident("constructor") {
                        let _: Token![=] = meta.input.parse()?;
                        let lit: Lit = meta.input.parse()?;
                        if let Lit::Str(s) = lit {
                            config.constructor = s.value();
                        }
                    } else if meta.path.is_ident("positional_type") {
                        let _: Token![=] = meta.input.parse()?;
                        let lit: Lit = meta.input.parse()?;
                        if let Lit::Str(s) = lit {
                            config.positional_type = Some(s.value());
                        }
                    } else if meta.path.is_ident("child_method") {
                        let _: Token![=] = meta.input.parse()?;
                        let lit: Lit = meta.input.parse()?;
                        if let Lit::Str(s) = lit {
                            config.child_method = Some(s.value());
                        }
                    } else if meta.path.is_ident("skip_impl") {
                        config.skip_impl = true;
                    }
                    Ok(())
                })?;
            }
        }

        Ok(config)
    }
}

/// Struct-level layout configuration from `#[layout(...)]`
#[derive(Debug, Default, Clone)]
pub struct LayoutConfig {
    /// Flex direction: Row, Column, RowReverse, ColumnReverse
    pub direction: Option<String>,
    /// Gap between children
    pub gap: Option<f32>,
    /// Uniform padding
    pub padding: Option<f32>,
    /// Individual padding values
    pub padding_left: Option<f32>,
    pub padding_right: Option<f32>,
    pub padding_top: Option<f32>,
    pub padding_bottom: Option<f32>,
    /// Justify content: FlexStart, FlexEnd, Center, SpaceBetween, SpaceAround, SpaceEvenly
    pub justify: Option<String>,
    /// Align items: FlexStart, FlexEnd, Center, Stretch, Baseline
    pub align: Option<String>,
    /// Align content (for multi-line)
    pub align_content: Option<String>,
    /// Flex wrap: NoWrap, Wrap, WrapReverse
    pub wrap: Option<String>,
    /// Flex grow
    pub flex: Option<f32>,
    /// Flex shrink
    pub flex_shrink: Option<f32>,
    /// Flex basis
    pub flex_basis: Option<String>,
    /// Align self (overrides parent's align_items)
    pub align_self: Option<String>,
    /// Width
    pub width: Option<String>,
    /// Height
    pub height: Option<String>,
    /// Min width
    pub min_width: Option<String>,
    /// Min height
    pub min_height: Option<String>,
    /// Max width
    pub max_width: Option<String>,
    /// Max height
    pub max_height: Option<String>,
}

impl LayoutConfig {
    pub fn from_attr(attr: &Attribute) -> syn::Result<Self> {
        let mut config = Self::default();

        attr.parse_nested_meta(|meta| {
            let path_str = meta
                .path
                .get_ident()
                .map(|i| i.to_string())
                .unwrap_or_default();

            match path_str.as_str() {
                "direction" => {
                    let _: Token![=] = meta.input.parse()?;
                    config.direction = Some(parse_ident_or_string(&meta)?);
                }
                "gap" => {
                    let _: Token![=] = meta.input.parse()?;
                    config.gap = Some(parse_float(&meta)?);
                }
                "padding" => {
                    let _: Token![=] = meta.input.parse()?;
                    config.padding = Some(parse_float(&meta)?);
                }
                "padding_left" => {
                    let _: Token![=] = meta.input.parse()?;
                    config.padding_left = Some(parse_float(&meta)?);
                }
                "padding_right" => {
                    let _: Token![=] = meta.input.parse()?;
                    config.padding_right = Some(parse_float(&meta)?);
                }
                "padding_top" => {
                    let _: Token![=] = meta.input.parse()?;
                    config.padding_top = Some(parse_float(&meta)?);
                }
                "padding_bottom" => {
                    let _: Token![=] = meta.input.parse()?;
                    config.padding_bottom = Some(parse_float(&meta)?);
                }
                "justify" => {
                    let _: Token![=] = meta.input.parse()?;
                    config.justify = Some(parse_ident_or_string(&meta)?);
                }
                "align" => {
                    let _: Token![=] = meta.input.parse()?;
                    config.align = Some(parse_ident_or_string(&meta)?);
                }
                "align_content" => {
                    let _: Token![=] = meta.input.parse()?;
                    config.align_content = Some(parse_ident_or_string(&meta)?);
                }
                "wrap" => {
                    let _: Token![=] = meta.input.parse()?;
                    config.wrap = Some(parse_ident_or_string(&meta)?);
                }
                "flex" => {
                    let _: Token![=] = meta.input.parse()?;
                    config.flex = Some(parse_float(&meta)?);
                }
                "flex_shrink" => {
                    let _: Token![=] = meta.input.parse()?;
                    config.flex_shrink = Some(parse_float(&meta)?);
                }
                "flex_basis" => {
                    let _: Token![=] = meta.input.parse()?;
                    config.flex_basis = Some(parse_ident_or_string(&meta)?);
                }
                "align_self" => {
                    let _: Token![=] = meta.input.parse()?;
                    config.align_self = Some(parse_ident_or_string(&meta)?);
                }
                "width" => {
                    let _: Token![=] = meta.input.parse()?;
                    config.width = Some(parse_dimension(&meta)?);
                }
                "height" => {
                    let _: Token![=] = meta.input.parse()?;
                    config.height = Some(parse_dimension(&meta)?);
                }
                "min_width" => {
                    let _: Token![=] = meta.input.parse()?;
                    config.min_width = Some(parse_dimension(&meta)?);
                }
                "min_height" => {
                    let _: Token![=] = meta.input.parse()?;
                    config.min_height = Some(parse_dimension(&meta)?);
                }
                "max_width" => {
                    let _: Token![=] = meta.input.parse()?;
                    config.max_width = Some(parse_dimension(&meta)?);
                }
                "max_height" => {
                    let _: Token![=] = meta.input.parse()?;
                    config.max_height = Some(parse_dimension(&meta)?);
                }
                _ => {}
            }
            Ok(())
        })?;

        Ok(config)
    }

    pub fn is_empty(&self) -> bool {
        self.direction.is_none()
            && self.gap.is_none()
            && self.padding.is_none()
            && self.padding_left.is_none()
            && self.padding_right.is_none()
            && self.padding_top.is_none()
            && self.padding_bottom.is_none()
            && self.justify.is_none()
            && self.align.is_none()
            && self.flex.is_none()
    }
}

/// Struct-level style configuration from `#[style(...)]`
#[derive(Debug, Default, Clone)]
pub struct StyleConfig {
    /// Background color
    pub background: Option<String>,
    /// Border color
    pub border_color: Option<String>,
    /// Border width
    pub border_width: Option<f32>,
    /// Corner radius
    pub corner_radius: Option<f32>,
    /// Shadow elevation
    pub shadow: Option<f32>,
    /// Opacity
    pub opacity: Option<f32>,
}

impl StyleConfig {
    pub fn from_attr(attr: &Attribute) -> syn::Result<Self> {
        let mut config = Self::default();

        attr.parse_nested_meta(|meta| {
            let path_str = meta
                .path
                .get_ident()
                .map(|i| i.to_string())
                .unwrap_or_default();

            match path_str.as_str() {
                "background" => {
                    let _: Token![=] = meta.input.parse()?;
                    config.background = Some(parse_ident_or_string(&meta)?);
                }
                "border_color" => {
                    let _: Token![=] = meta.input.parse()?;
                    config.border_color = Some(parse_ident_or_string(&meta)?);
                }
                "border_width" => {
                    let _: Token![=] = meta.input.parse()?;
                    config.border_width = Some(parse_float(&meta)?);
                }
                "corner_radius" => {
                    let _: Token![=] = meta.input.parse()?;
                    config.corner_radius = Some(parse_float(&meta)?);
                }
                "shadow" => {
                    let _: Token![=] = meta.input.parse()?;
                    config.shadow = Some(parse_float(&meta)?);
                }
                "opacity" => {
                    let _: Token![=] = meta.input.parse()?;
                    config.opacity = Some(parse_float(&meta)?);
                }
                _ => {}
            }
            Ok(())
        })?;

        Ok(config)
    }

    pub fn is_empty(&self) -> bool {
        self.background.is_none()
            && self.border_color.is_none()
            && self.border_width.is_none()
            && self.corner_radius.is_none()
            && self.shadow.is_none()
            && self.opacity.is_none()
    }
}

// ============================================================================
// Field-Level Configuration
// ============================================================================

/// Field configuration from field-level attributes
#[derive(Debug)]
pub struct FieldConfig {
    /// Field name
    pub name: Ident,
    /// Field kind (how it appears in macro)
    pub kind: FieldKind,
    /// Slot-specific layout (for child widgets)
    pub slot_layout: Option<LayoutConfig>,
}

#[derive(Debug)]
pub enum FieldKind {
    /// First positional argument: `macro!("value")`
    Positional {
        /// Supports reactive: `macro!(@signal)`
        reactive: bool,
    },
    /// Single child widget: `#[widget(child)]`
    Child,
    /// Multiple children (WidgetTuple): `#[widget(children)]`
    Children,
    /// Named children: `macro! { "name" => widget, ... }`
    /// Used for Form-like patterns where children have string keys
    NamedChildren,
    /// Named parameter: `macro!(param: value)`
    Param {
        /// Default value expression (reserved for future use)
        #[allow(dead_code)]
        default: Option<TokenStream>,
        /// Setter method name (default: field name)
        setter: Option<String>,
    },
    /// Boolean flag: `macro!(flag_name)`
    Flag,
    /// Method flag: calls a parameterless method like `.primary()`
    MethodFlag {
        /// Method names to expose as flags
        methods: Vec<String>,
    },
    /// Callback: `macro!(on_event: || {})`
    Callback,
    /// Internal field (not exposed in macro)
    Internal,
}

impl FieldConfig {
    pub fn from_field(field: &Field) -> syn::Result<Self> {
        let name = field
            .ident
            .clone()
            .ok_or_else(|| syn::Error::new_spanned(field, "Tuple struct fields not supported"))?;
        let kind = Self::parse_field_kind(&field.attrs)?;
        let slot_layout = Self::parse_slot_layout(&field.attrs)?;

        Ok(Self {
            name,
            kind,
            slot_layout,
        })
    }

    fn parse_field_kind(attrs: &[Attribute]) -> syn::Result<FieldKind> {
        for attr in attrs {
            // #[positional] or #[positional(reactive)]
            if attr.path().is_ident("positional") {
                let reactive = match &attr.meta {
                    Meta::Path(_) => false,
                    Meta::List(list) => {
                        let content = list.tokens.to_string();
                        content.contains("reactive")
                    }
                    _ => false,
                };
                return Ok(FieldKind::Positional { reactive });
            }

            // #[widget(child)] or #[widget(children)] or #[widget(named_children)]
            if attr.path().is_ident("widget") {
                if let Meta::List(list) = &attr.meta {
                    let content = list.tokens.to_string();
                    if content.contains("named_children") {
                        return Ok(FieldKind::NamedChildren);
                    } else if content.contains("children") {
                        return Ok(FieldKind::Children);
                    } else if content.contains("child") {
                        return Ok(FieldKind::Child);
                    }
                }
            }

            // Legacy #[children] (for backwards compatibility)
            if attr.path().is_ident("children") {
                return Ok(FieldKind::Children);
            }

            // #[scaffold] or #[named_children] standalone attribute
            if attr.path().is_ident("scaffold") || attr.path().is_ident("named_children") {
                return Ok(FieldKind::NamedChildren);
            }

            // #[param] or #[param(default = X)] or #[param(setter = "name")]
            if attr.path().is_ident("param") {
                let mut default = None;
                let mut setter = None;

                // Parse nested attributes if present
                let _ = attr.parse_nested_meta(|meta| {
                    if meta.path.is_ident("default") {
                        let _: Token![=] = meta.input.parse()?;
                        let expr: Expr = meta.input.parse()?;
                        default = Some(quote::quote!(#expr));
                    } else if meta.path.is_ident("setter") {
                        let _: Token![=] = meta.input.parse()?;
                        let lit: Lit = meta.input.parse()?;
                        if let Lit::Str(s) = lit {
                            setter = Some(s.value());
                        }
                    }
                    Ok(())
                });

                return Ok(FieldKind::Param { default, setter });
            }

            // #[flag]
            if attr.path().is_ident("flag") {
                return Ok(FieldKind::Flag);
            }

            // #[method_flag(primary, secondary)]
            if attr.path().is_ident("method_flag") {
                let mut methods = Vec::new();

                if let Meta::List(list) = &attr.meta {
                    let content = list.tokens.to_string();
                    for method in content.split(',') {
                        let method = method.trim();
                        if !method.is_empty() {
                            methods.push(method.to_string());
                        }
                    }
                }

                return Ok(FieldKind::MethodFlag { methods });
            }

            // #[callback]
            if attr.path().is_ident("callback") {
                return Ok(FieldKind::Callback);
            }
        }

        // No recognized attribute = internal field
        Ok(FieldKind::Internal)
    }

    fn parse_slot_layout(attrs: &[Attribute]) -> syn::Result<Option<LayoutConfig>> {
        for attr in attrs {
            if attr.path().is_ident("layout") {
                return Ok(Some(LayoutConfig::from_attr(attr)?));
            }
        }
        Ok(None)
    }
}

// ============================================================================
// Complete Widget Configuration
// ============================================================================

/// Complete widget configuration parsed from all attributes
#[derive(Debug)]
pub struct WidgetConfig {
    /// Macro configuration
    pub macro_config: WidgetMacroConfig,
    /// Layout configuration
    pub layout: LayoutConfig,
    /// Style configuration
    pub style: StyleConfig,
    /// All fields
    pub fields: Vec<FieldConfig>,
}

impl WidgetConfig {
    pub fn from_input(input: &DeriveInput) -> syn::Result<Self> {
        let macro_config = WidgetMacroConfig::from_attrs(&input.attrs)?;
        let layout = Self::parse_layout(&input.attrs)?;
        let style = Self::parse_style(&input.attrs)?;
        let fields = parse_struct_fields(input)?;

        Ok(Self {
            macro_config,
            layout,
            style,
            fields,
        })
    }

    fn parse_layout(attrs: &[Attribute]) -> syn::Result<LayoutConfig> {
        for attr in attrs {
            if attr.path().is_ident("layout") {
                return LayoutConfig::from_attr(attr);
            }
        }
        Ok(LayoutConfig::default())
    }

    fn parse_style(attrs: &[Attribute]) -> syn::Result<StyleConfig> {
        for attr in attrs {
            if attr.path().is_ident("style") {
                return StyleConfig::from_attr(attr);
            }
        }
        Ok(StyleConfig::default())
    }

    /// Check if this is a container widget (has children)
    pub fn has_children(&self) -> bool {
        self.fields
            .iter()
            .any(|f| matches!(f.kind, FieldKind::Children | FieldKind::Child))
    }

    /// Check if this has named children (Form-like pattern)
    pub fn has_named_children(&self) -> bool {
        self.fields
            .iter()
            .any(|f| matches!(f.kind, FieldKind::NamedChildren))
    }

    /// Check if this has a positional field
    pub fn has_positional(&self) -> bool {
        self.fields
            .iter()
            .any(|f| matches!(f.kind, FieldKind::Positional { .. }))
    }

    /// Check if this has a positional type override (for Signal-like patterns)
    pub fn has_positional_type(&self) -> bool {
        self.macro_config.positional_type.is_some()
    }

    /// Get all child fields
    #[allow(dead_code)]
    pub fn child_fields(&self) -> Vec<&FieldConfig> {
        self.fields
            .iter()
            .filter(|f| matches!(f.kind, FieldKind::Child | FieldKind::Children))
            .collect()
    }
}

/// Parse all fields from a struct
pub fn parse_struct_fields(input: &DeriveInput) -> syn::Result<Vec<FieldConfig>> {
    let fields = match &input.data {
        syn::Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => &fields.named,
            _ => {
                return Err(syn::Error::new_spanned(
                    input,
                    "Only named struct fields are supported",
                ))
            }
        },
        _ => {
            return Err(syn::Error::new_spanned(
                input,
                "Widget can only be derived for structs",
            ))
        }
    };

    fields.iter().map(FieldConfig::from_field).collect()
}

// ============================================================================
// Enum Configuration (for WidgetEnum)
// ============================================================================

/// Configuration for an enum variant
#[derive(Debug)]
pub struct VariantConfig {
    /// Variant name
    pub name: Ident,
    /// Whether this variant is a flag (usable as bare identifier)
    pub is_flag: bool,
}

impl VariantConfig {
    pub fn from_variant(variant: &Variant) -> Self {
        let is_flag = variant
            .attrs
            .iter()
            .any(|attr| attr.path().is_ident("flag"));

        Self {
            name: variant.ident.clone(),
            is_flag,
        }
    }
}

/// Parse all variants from an enum
pub fn parse_enum_variants(input: &DeriveInput) -> syn::Result<Vec<VariantConfig>> {
    match &input.data {
        syn::Data::Enum(data) => Ok(data
            .variants
            .iter()
            .map(VariantConfig::from_variant)
            .collect()),
        _ => Err(syn::Error::new_spanned(
            input,
            "WidgetEnum can only be derived for enums",
        )),
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

fn parse_float(meta: &syn::meta::ParseNestedMeta) -> syn::Result<f32> {
    let lit: Lit = meta.input.parse()?;
    match lit {
        Lit::Float(f) => Ok(f.base10_parse()?),
        Lit::Int(i) => Ok(i.base10_parse::<i64>()? as f32),
        _ => Err(meta.error("expected float literal")),
    }
}

fn parse_ident_or_string(meta: &syn::meta::ParseNestedMeta) -> syn::Result<String> {
    // Try to parse as identifier first
    if let Ok(ident) = meta.input.parse::<Ident>() {
        return Ok(ident.to_string());
    }
    // Fall back to string literal
    let lit: Lit = meta.input.parse()?;
    match lit {
        Lit::Str(s) => Ok(s.value()),
        _ => Err(meta.error("expected identifier or string literal")),
    }
}

fn parse_dimension(meta: &syn::meta::ParseNestedMeta) -> syn::Result<String> {
    // Could be: 100.0, "100px", "50%", Auto, etc.
    if let Ok(ident) = meta.input.parse::<Ident>() {
        return Ok(ident.to_string());
    }
    let lit: Lit = meta.input.parse()?;
    match lit {
        Lit::Float(f) => Ok(f.to_string()),
        Lit::Int(i) => Ok(i.to_string()),
        Lit::Str(s) => Ok(s.value()),
        _ => Err(meta.error("expected dimension value")),
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn test_parse_widget_macro_config() {
        let input: DeriveInput = parse_quote! {
            #[widget_macro(name = "btn", alias = "button")]
            struct Button {
                text: String,
            }
        };

        let config = WidgetMacroConfig::from_attrs(&input.attrs).unwrap();
        assert_eq!(config.name, Some("btn".to_string()));
        assert_eq!(config.alias, Some("button".to_string()));
    }

    #[test]
    fn test_parse_layout_config() {
        let input: DeriveInput = parse_quote! {
            #[layout(direction = Row, gap = 10.0, padding = 16.0, justify = Center)]
            struct Container {
                children: Vec<Widget>,
            }
        };

        let config = WidgetConfig::from_input(&input).unwrap();
        assert_eq!(config.layout.direction, Some("Row".to_string()));
        assert_eq!(config.layout.gap, Some(10.0));
        assert_eq!(config.layout.padding, Some(16.0));
        assert_eq!(config.layout.justify, Some("Center".to_string()));
    }

    #[test]
    fn test_parse_style_config() {
        let input: DeriveInput = parse_quote! {
            #[style(background = White, corner_radius = 8.0, shadow = 2.0)]
            struct Card {
                children: Vec<Widget>,
            }
        };

        let config = WidgetConfig::from_input(&input).unwrap();
        assert_eq!(config.style.background, Some("White".to_string()));
        assert_eq!(config.style.corner_radius, Some(8.0));
        assert_eq!(config.style.shadow, Some(2.0));
    }

    #[test]
    fn test_parse_widget_child() {
        let input: DeriveInput = parse_quote! {
            struct FormField {
                #[widget(child)]
                label: Label,
                #[widget(child)]
                input: TextInput,
            }
        };

        let fields = parse_struct_fields(&input).unwrap();
        assert!(matches!(fields[0].kind, FieldKind::Child));
        assert!(matches!(fields[1].kind, FieldKind::Child));
    }

    #[test]
    fn test_parse_widget_children() {
        let input: DeriveInput = parse_quote! {
            struct Column {
                #[widget(children)]
                children: C,
            }
        };

        let fields = parse_struct_fields(&input).unwrap();
        assert!(matches!(fields[0].kind, FieldKind::Children));
    }

    #[test]
    fn test_parse_slot_layout() {
        let input: DeriveInput = parse_quote! {
            struct FormField {
                #[widget(child)]
                label: Label,
                #[widget(child)]
                #[layout(flex = 1.0, align_self = Center)]
                input: TextInput,
            }
        };

        let fields = parse_struct_fields(&input).unwrap();
        assert!(fields[0].slot_layout.is_none());
        let slot_layout = fields[1].slot_layout.as_ref().unwrap();
        assert_eq!(slot_layout.flex, Some(1.0));
        assert_eq!(slot_layout.align_self, Some("Center".to_string()));
    }

    #[test]
    fn test_parse_positional_field() {
        let input: DeriveInput = parse_quote! {
            struct Text {
                #[positional]
                content: String,
            }
        };

        let fields = parse_struct_fields(&input).unwrap();
        assert!(matches!(
            fields[0].kind,
            FieldKind::Positional { reactive: false }
        ));
    }

    #[test]
    fn test_parse_positional_reactive_field() {
        let input: DeriveInput = parse_quote! {
            struct Text {
                #[positional(reactive)]
                content: TextContent,
            }
        };

        let fields = parse_struct_fields(&input).unwrap();
        assert!(matches!(
            fields[0].kind,
            FieldKind::Positional { reactive: true }
        ));
    }

    #[test]
    fn test_parse_param_with_default() {
        let input: DeriveInput = parse_quote! {
            struct Text {
                #[param(default = 16.0)]
                font_size: f32,
            }
        };

        let fields = parse_struct_fields(&input).unwrap();
        assert!(matches!(
            &fields[0].kind,
            FieldKind::Param {
                default: Some(_),
                ..
            }
        ));
    }

    #[test]
    fn test_parse_flag_field() {
        let input: DeriveInput = parse_quote! {
            struct Button {
                #[flag]
                disabled: bool,
            }
        };

        let fields = parse_struct_fields(&input).unwrap();
        assert!(matches!(fields[0].kind, FieldKind::Flag));
    }

    #[test]
    fn test_parse_callback_field() {
        let input: DeriveInput = parse_quote! {
            struct Button {
                #[callback]
                on_click: Option<Arc<dyn Fn()>>,
            }
        };

        let fields = parse_struct_fields(&input).unwrap();
        assert!(matches!(fields[0].kind, FieldKind::Callback));
    }

    #[test]
    fn test_complete_widget_config() {
        let input: DeriveInput = parse_quote! {
            #[widget_macro(name = "form_field")]
            #[layout(direction = Row, gap = 8.0)]
            #[style(background = White)]
            struct FormField {
                #[widget(child)]
                label: Label,

                #[widget(child)]
                #[layout(flex = 1.0)]
                input: TextInput,

                #[param]
                spacing: f32,
            }
        };

        let config = WidgetConfig::from_input(&input).unwrap();
        assert_eq!(config.macro_config.name, Some("form_field".to_string()));
        assert_eq!(config.layout.direction, Some("Row".to_string()));
        assert_eq!(config.layout.gap, Some(8.0));
        assert_eq!(config.style.background, Some("White".to_string()));
        assert!(config.has_children());
        assert!(!config.has_positional());
        assert_eq!(config.child_fields().len(), 2);
    }

    #[test]
    fn test_parse_enum_variants() {
        let input: DeriveInput = parse_quote! {
            enum ButtonStyle {
                Default,
                #[flag]
                Primary,
                #[flag]
                Secondary,
            }
        };

        let variants = parse_enum_variants(&input).unwrap();
        assert_eq!(variants.len(), 3);
        assert!(!variants[0].is_flag);
        assert!(variants[1].is_flag);
        assert!(variants[2].is_flag);
    }
}
