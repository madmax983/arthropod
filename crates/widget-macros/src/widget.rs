//! Unified Widget derive macro implementation
//!
//! Auto-detects whether the widget is a display widget (has #[positional])
//! or a container widget (has #[children]) and generates the appropriate macro.
//!
//! This derive generates:
//! 1. Declarative macros for widget instantiation (txt!, btn!, col!, etc.)
//! 2. Widget::build() implementation for scene construction

use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse2, DeriveInput};

use crate::generate::{generate_widget_impl, generate_with_alias, macro_name, ExternalEnumFlag};
use crate::parse::{FieldKind, WidgetConfig};

/// Main derive implementation for Widget
pub fn derive(input: TokenStream) -> TokenStream {
    let input: DeriveInput = match parse2(input) {
        Ok(input) => input,
        Err(e) => return e.to_compile_error(),
    };

    match derive_impl(&input) {
        Ok(tokens) => tokens,
        Err(e) => e.to_compile_error(),
    }
}

fn derive_impl(input: &DeriveInput) -> syn::Result<TokenStream> {
    let struct_name = &input.ident;

    // Parse complete widget configuration
    let config = WidgetConfig::from_input(input)?;

    // Detect widget type based on attributes
    let has_positional = config.has_positional();
    let has_children = config.has_children();
    let has_named_children = config.has_named_children();
    let has_positional_type = config.has_positional_type();

    // positional_type acts as a virtual positional field when no #[positional] field exists
    let effective_positional = has_positional || has_positional_type;

    // Validate: cannot have both positional and array children (confusing syntax)
    // BUT: positional + named_children is allowed (Scaffold pattern)
    if effective_positional && has_children {
        return Err(syn::Error::new_spanned(
            input,
            "Widget cannot have both #[positional]/positional_type and #[widget(children)] - use #[scaffold] for slot-based patterns",
        ));
    }

    // Validate: cannot have both array children and named_children
    if has_children && has_named_children {
        return Err(syn::Error::new_spanned(
            input,
            "Widget cannot have both #[widget(children)] and #[scaffold] fields",
        ));
    }

    // For widgets with layout/style but no positional/children, allow them
    // (they might be composition widgets using #[widget(child)])
    let is_layout_widget = !config.layout.is_empty() || !config.style.is_empty();

    if !effective_positional && !has_children && !has_named_children && !is_layout_widget {
        return Err(syn::Error::new_spanned(
            input,
            "Widget requires either #[positional], positional_type, #[widget(children)], #[scaffold], or #[layout]/[style] attributes",
        ));
    }

    // Validate positional count
    if has_positional {
        let positional_count = config
            .fields
            .iter()
            .filter(|f| matches!(f.kind, FieldKind::Positional { .. }))
            .count();
        if positional_count > 1 {
            return Err(syn::Error::new_spanned(
                input,
                "Widget cannot have more than one #[positional] field",
            ));
        }
    }

    // Validate children count
    let children_count = config
        .fields
        .iter()
        .filter(|f| matches!(f.kind, FieldKind::Children))
        .count();
    if children_count > 1 {
        return Err(syn::Error::new_spanned(
            input,
            "Widget cannot have more than one #[widget(children)] field",
        ));
    }

    // Validate named_children count
    let named_children_count = config
        .fields
        .iter()
        .filter(|f| matches!(f.kind, FieldKind::NamedChildren))
        .count();
    if named_children_count > 1 {
        return Err(syn::Error::new_spanned(
            input,
            "Widget cannot have more than one #[scaffold] field",
        ));
    }

    // Generate macro name
    let name = macro_name(struct_name, config.macro_config.name.as_deref());

    // No external enum flags for now
    let enum_flags: Vec<ExternalEnumFlag> = Vec::new();

    // Generate the macros
    let macros = generate_with_alias(
        struct_name,
        &name,
        config.macro_config.alias.as_deref(),
        &config.fields,
        &config.macro_config,
        &enum_flags,
    );

    // Generate Widget::build() impl if we have layout/style configuration
    // Skip if skip_impl is set (for widgets with custom impl)
    let widget_impl = if !config.macro_config.skip_impl
        && (is_layout_widget || has_children || has_named_children)
    {
        generate_widget_impl(struct_name, &config, input)
    } else {
        quote! {}
    };

    Ok(quote! {
        #macros
        #widget_impl
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn test_display_widget_detected() {
        let input: DeriveInput = parse_quote! {
            #[widget(name = "txt")]
            pub struct Text {
                #[positional]
                content: String,

                #[param]
                size: f32,
            }
        };

        let result = derive_impl(&input);
        assert!(result.is_ok(), "Failed: {:?}", result.err());

        let tokens = result.unwrap().to_string();
        assert!(tokens.contains("macro_rules ! txt"));
    }

    #[test]
    fn test_container_widget_detected() {
        // Use generic C: WidgetTuple pattern for type-safe children
        let input: DeriveInput = parse_quote! {
            #[widget(name = "stack")]
            pub struct Stack<C: WidgetTuple> {
                #[children]
                children: C,

                #[param]
                gap: f32,
            }
        };

        let result = derive_impl(&input);
        assert!(result.is_ok(), "Failed: {:?}", result.err());

        let tokens = result.unwrap().to_string();
        assert!(tokens.contains("macro_rules ! stack"));
        assert!(tokens.contains("$ children"));
    }

    #[test]
    fn test_error_both_positional_and_array_children() {
        // Positional + array children is forbidden (confusing)
        let input: DeriveInput = parse_quote! {
            pub struct Invalid<C: WidgetTuple> {
                #[positional]
                text: String,
                #[children]
                children: C,
            }
        };

        let result = derive_impl(&input);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("cannot have both"));
    }

    #[test]
    fn test_error_neither_positional_nor_children() {
        let input: DeriveInput = parse_quote! {
            pub struct Invalid {
                #[param]
                value: i32,
            }
        };

        let result = derive_impl(&input);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("requires either"));
    }

    #[test]
    fn test_with_reactive() {
        let input: DeriveInput = parse_quote! {
            pub struct Text {
                #[positional(reactive)]
                content: TextContent,
            }
        };

        let result = derive_impl(&input);
        assert!(result.is_ok());

        let tokens = result.unwrap().to_string();
        assert!(tokens.contains("@ $ signal"));
    }

    #[test]
    fn test_with_flags_and_params() {
        let input: DeriveInput = parse_quote! {
            #[widget(name = "btn")]
            pub struct Button {
                #[positional]
                text: String,

                #[flag]
                disabled: bool,

                #[param]
                padding: f32,

                #[callback]
                on_click: Option<Callback>,
            }
        };

        let result = derive_impl(&input);
        assert!(result.is_ok());

        let tokens = result.unwrap().to_string();
        assert!(tokens.contains("disabled"));
        assert!(tokens.contains("padding"));
        assert!(tokens.contains("on_click"));
    }

    #[test]
    fn test_with_alias() {
        let input: DeriveInput = parse_quote! {
            #[widget(name = "btn", alias = "button")]
            pub struct Button {
                #[positional]
                text: String,
            }
        };

        let result = derive_impl(&input);
        assert!(result.is_ok());

        let tokens = result.unwrap().to_string();
        assert!(tokens.contains("macro_rules ! btn"));
        assert!(tokens.contains("macro_rules ! button"));
    }

    #[test]
    fn test_method_flag() {
        let input: DeriveInput = parse_quote! {
            pub struct Button {
                #[positional]
                text: String,

                #[method_flag(primary, secondary)]
                style: ButtonStyle,
            }
        };

        let result = derive_impl(&input);
        assert!(result.is_ok());

        let tokens = result.unwrap().to_string();
        assert!(tokens.contains("primary"));
        assert!(tokens.contains("secondary"));
    }

    #[test]
    fn test_scaffold_pattern_positional_plus_named_children() {
        // Scaffold pattern: positional body + named slots (allowed!)
        // Uses generics for type-safe children
        let input: DeriveInput = parse_quote! {
            #[widget(name = "scaffold", skip_impl)]
            pub struct Scaffold<B: Widget, S: NamedWidgetTuple> {
                #[positional]
                body: B,

                #[scaffold]
                slots: S,

                #[param]
                show_drawer: bool,
            }
        };

        let result = derive_impl(&input);
        assert!(
            result.is_ok(),
            "Scaffold pattern should be allowed: {:?}",
            result.err()
        );

        let tokens = result.unwrap().to_string();
        assert!(tokens.contains("macro_rules ! scaffold"));
    }

    #[test]
    fn test_named_children_widget() {
        // Pure named children widget (like Form)
        // Uses generics for type-safe children
        let input: DeriveInput = parse_quote! {
            #[widget(name = "form", skip_impl)]
            pub struct Form<F: NamedWidgetTuple> {
                #[scaffold]
                fields: F,

                #[callback]
                on_submit: Option<SubmitCallback>,

                #[param]
                gap: f32,
            }
        };

        let result = derive_impl(&input);
        assert!(
            result.is_ok(),
            "Named children widget should work: {:?}",
            result.err()
        );

        let tokens = result.unwrap().to_string();
        assert!(tokens.contains("macro_rules ! form"));
        // Should have the "name" => widget pattern
        assert!(tokens.contains("field_name"));
    }
}
