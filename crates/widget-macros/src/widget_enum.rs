//! WidgetEnum derive macro implementation

use heck::ToSnakeCase;
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::{parse2, DeriveInput};

use crate::parse::parse_enum_variants;

/// Main derive implementation for WidgetEnum
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
    let enum_name = &input.ident;
    let variants = parse_enum_variants(input)?;

    // Filter to only flag variants
    let flag_variants: Vec<_> = variants.iter().filter(|v| v.is_flag).collect();

    if flag_variants.is_empty() {
        // No flags to generate, just return empty impl
        return Ok(quote! {});
    }

    // Generate a helper function that returns the flag info for use in widget macros
    // This creates a compile-time constant with the flag information
    let flag_info: Vec<_> = flag_variants
        .iter()
        .map(|v| {
            let variant_name = &v.name;
            let flag_name_str = variant_name.to_string().to_snake_case();
            quote! {
                (#flag_name_str, #enum_name::#variant_name)
            }
        })
        .collect();

    // Generate the flag constants
    let flag_consts: Vec<_> = flag_variants
        .iter()
        .map(|v| {
            let variant_name = &v.name;
            let const_name = Ident::new(
                &format!("{}_FLAG", variant_name.to_string().to_uppercase()),
                Span::call_site(),
            );
            let flag_name_str = variant_name.to_string().to_snake_case();

            quote! {
                /// Flag name for use in macros
                pub const #const_name: &'static str = #flag_name_str;
            }
        })
        .collect();

    Ok(quote! {
        impl #enum_name {
            #(#flag_consts)*

            /// Get all flag variants as (flag_name, variant) pairs
            pub fn flag_variants() -> &'static [(&'static str, Self)] {
                &[#(#flag_info),*]
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn test_basic_widget_enum() {
        let input: DeriveInput = parse_quote! {
            #[derive(Clone, Copy)]
            pub enum ButtonStyle {
                Default,
                #[flag]
                Primary,
                #[flag]
                Secondary,
            }
        };

        let result = derive_impl(&input);
        assert!(result.is_ok(), "Failed: {:?}", result.err());

        let tokens = result.unwrap().to_string();
        // Should have flag constants
        assert!(tokens.contains("PRIMARY_FLAG"));
        assert!(tokens.contains("SECONDARY_FLAG"));
        // Should have flag_variants function
        assert!(tokens.contains("flag_variants"));
    }

    #[test]
    fn test_no_flags() {
        let input: DeriveInput = parse_quote! {
            pub enum ButtonStyle {
                Default,
                Primary,
                Secondary,
            }
        };

        let result = derive_impl(&input);
        assert!(result.is_ok());

        let tokens = result.unwrap().to_string();
        // Should be empty if no flags
        assert!(tokens.is_empty() || !tokens.contains("flag_variants"));
    }

    #[test]
    fn test_single_flag() {
        let input: DeriveInput = parse_quote! {
            pub enum Theme {
                Light,
                #[flag]
                Dark,
            }
        };

        let result = derive_impl(&input);
        assert!(result.is_ok());

        let tokens = result.unwrap().to_string();
        assert!(tokens.contains("DARK_FLAG"));
        assert!(tokens.contains("dark"));
    }

    #[test]
    fn test_error_not_enum() {
        let input: DeriveInput = parse_quote! {
            pub struct NotAnEnum {
                field: i32,
            }
        };

        let result = derive_impl(&input);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("can only be derived for enums"));
    }
}
