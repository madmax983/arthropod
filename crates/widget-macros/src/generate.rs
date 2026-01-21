//! Code generation utilities for widget macros

use heck::ToSnakeCase;
use proc_macro2::{Ident, Span, TokenStream};
use quote::{format_ident, quote};

use syn::DeriveInput;

use crate::parse::{
    FieldConfig, FieldKind, LayoutConfig, StyleConfig, WidgetConfig, WidgetMacroConfig,
};

/// Generate the macro name from struct name or override
pub fn macro_name(struct_name: &Ident, override_name: Option<&str>) -> Ident {
    match override_name {
        Some(name) => Ident::new(name, Span::call_site()),
        None => Ident::new(&struct_name.to_string().to_snake_case(), Span::call_site()),
    }
}

/// Generate the helper macro name (prefixed with __)
pub fn helper_macro_name(macro_name: &Ident) -> Ident {
    format_ident!("__{}_apply", macro_name)
}

/// Information needed to generate flag arms from external enum types
#[derive(Debug, Clone)]
pub struct ExternalEnumFlag {
    /// Flag name (snake_case, e.g., "primary")
    pub flag_name: Ident,
    /// Enum variant path (e.g., ButtonStyle::Primary)
    pub variant_path: TokenStream,
    /// Setter method name
    pub setter: Ident,
}

/// Generate the main macro rules
pub fn generate_main_macro(
    macro_name: &Ident,
    struct_name: &Ident,
    fields: &[FieldConfig],
    macro_config: &WidgetMacroConfig,
) -> TokenStream {
    let helper_name = helper_macro_name(macro_name);
    let constructor_ident = Ident::new(&macro_config.constructor, Span::call_site());

    // Find positional field
    let positional = fields
        .iter()
        .find(|f| matches!(f.kind, FieldKind::Positional { .. } | FieldKind::Children));

    // Check for named children (Form pattern)
    let has_named_children = fields
        .iter()
        .any(|f| matches!(f.kind, FieldKind::NamedChildren));

    // Check if positional supports reactive
    let supports_reactive = positional
        .map(|p| matches!(p.kind, FieldKind::Positional { reactive: true }))
        .unwrap_or(false);

    // Check if this is a children-based macro
    let is_container = positional
        .map(|p| matches!(p.kind, FieldKind::Children))
        .unwrap_or(false);

    // Check if we have positional_type (constructor takes positional arg but no #[positional] field)
    let has_positional_type = macro_config.positional_type.is_some();

    // Get child method (default: "child" for arrays, "field" for named_children)
    let child_method_name = macro_config
        .child_method
        .as_deref()
        .unwrap_or(if has_named_children { "field" } else { "child" });
    let _child_method = Ident::new(child_method_name, Span::call_site());

    // Determine if we have an effective positional (either from field or positional_type)
    let has_effective_positional = positional.is_some() || has_positional_type;

    // Check for actual positional field (not just positional_type)
    let has_positional_field = positional
        .map(|p| matches!(p.kind, FieldKind::Positional { .. }))
        .unwrap_or(false);

    if has_positional_field && has_named_children {
        // Scaffold pattern: positional body + named slots
        // Syntax: scaffold!(body, [("slot", widget), ("slot2", widget2)]; params)
        // Uses tuple construction for type-safety (no builder loop)
        quote! {
            #[macro_export]
            macro_rules! #macro_name {
                // Positional + named children array
                ($pos:expr, [$(($slot_name:expr, $slot_widget:expr)),* $(,)?]) => {{
                    $crate::#struct_name::#constructor_ident($pos, ($(($slot_name, $slot_widget),)*))
                }};

                // Positional + named children + options
                ($pos:expr, [$(($slot_name:expr, $slot_widget:expr)),* $(,)?], $($rest:tt)*) => {{
                    let widget = $crate::#struct_name::#constructor_ident($pos, ($(($slot_name, $slot_widget),)*));
                    $crate::#helper_name!(widget, $($rest)*)
                }};

                // Positional only (no slots) - empty tuple
                ($pos:expr) => {
                    $crate::#struct_name::#constructor_ident($pos, ())
                };

                // Positional + params (no slots)
                ($pos:expr, $($rest:tt)*) => {{
                    let widget = $crate::#struct_name::#constructor_ident($pos, ());
                    $crate::#helper_name!(widget, $($rest)*)
                }};
            }
        }
    } else if has_named_children {
        // Named children macro: array of (name, widget) tuples (for Form-like widgets)
        // Uses tuple construction for type-safety (no builder loop)
        quote! {
            #[macro_export]
            macro_rules! #macro_name {
                // Named children array only
                ([$(($field_name:expr, $field_widget:expr)),* $(,)?]) => {{
                    $crate::#struct_name::#constructor_ident(($(($field_name, $field_widget),)*))
                }};

                // Named children + options
                ([$(($field_name:expr, $field_widget:expr)),* $(,)?], $($rest:tt)*) => {{
                    let widget = $crate::#struct_name::#constructor_ident(($(($field_name, $field_widget),)*));
                    $crate::#helper_name!(widget, $($rest)*)
                }};
            }
        }
    } else if is_container {
        // Container macro: children array -> tuple construction (one-shot, type-safe)
        // Cannot use builder loop because each .child() call changes the type
        quote! {
            #[macro_export]
            macro_rules! #macro_name {
                // Children only - construct tuple directly
                ([$($children:expr),* $(,)?]) => {{
                    $crate::#struct_name::#constructor_ident(($($children,)*))
                }};

                // Children + params/flags
                ([$($children:expr),* $(,)?], $($rest:tt)*) => {{
                    let widget = $crate::#struct_name::#constructor_ident(($($children,)*));
                    $crate::#helper_name!(widget, $($rest)*)
                }};
            }
        }
    } else if supports_reactive {
        // Display widget with reactive support
        quote! {
            #[macro_export]
            macro_rules! #macro_name {
                // Reactive: @signal
                (@$signal:expr) => {
                    $crate::#struct_name::reactive($signal)
                };

                // Reactive + params
                (@$signal:expr, $($rest:tt)*) => {{
                    let widget = $crate::#struct_name::reactive($signal);
                    $crate::#helper_name!(widget, $($rest)*)
                }};

                // Static: positional only
                ($pos:expr) => {
                    $crate::#struct_name::#constructor_ident($pos)
                };

                // Static + params/flags
                ($pos:expr, $($rest:tt)*) => {{
                    let widget = $crate::#struct_name::#constructor_ident($pos);
                    $crate::#helper_name!(widget, $($rest)*)
                }};
            }
        }
    } else if has_effective_positional {
        // Display widget with positional arg (either from field or positional_type)
        quote! {
            #[macro_export]
            macro_rules! #macro_name {
                // Positional only
                ($pos:expr) => {
                    $crate::#struct_name::#constructor_ident($pos)
                };

                // Positional + params/flags
                ($pos:expr, $($rest:tt)*) => {{
                    let widget = $crate::#struct_name::#constructor_ident($pos);
                    $crate::#helper_name!(widget, $($rest)*)
                }};
            }
        }
    } else {
        // Widget with no positional arg (only params/flags)
        quote! {
            #[macro_export]
            macro_rules! #macro_name {
                // No positional args, just params/flags
                ($($rest:tt)*) => {{
                    let widget = $crate::#struct_name::#constructor_ident();
                    $crate::#helper_name!(widget, $($rest)*)
                }};
            }
        }
    }
}

/// Generate the helper macro for applying params/flags
pub fn generate_helper_macro(
    macro_name: &Ident,
    fields: &[FieldConfig],
    enum_flags: &[ExternalEnumFlag],
) -> TokenStream {
    let helper_name = helper_macro_name(macro_name);

    // Generate arms for each field type
    let mut arms = Vec::new();

    // Base cases: empty rest
    arms.push(quote! {
        ($w:expr,) => { $w };
    });
    arms.push(quote! {
        ($w:expr) => { $w };
    });

    // Stray comma handling
    arms.push(quote! {
        ($w:expr, , $($rest:tt)*) => {
            $crate::#helper_name!($w, $($rest)*)
        };
    });

    // Generate flag arms (bool fields marked with #[flag])
    for field in fields {
        if let FieldKind::Flag = &field.kind {
            let flag_name = &field.name;
            // For flags, we just call the setter with true
            arms.push(quote! {
                ($w:expr, #flag_name $($rest:tt)*) => {
                    $crate::#helper_name!($w.#flag_name(true), $($rest)*)
                };
            });
        }
    }

    // Generate method flag arms (calls parameterless methods like .primary())
    for field in fields {
        if let FieldKind::MethodFlag { methods } = &field.kind {
            for method_name in methods {
                let method_ident = Ident::new(method_name, Span::call_site());
                arms.push(quote! {
                    ($w:expr, #method_ident $($rest:tt)*) => {
                        $crate::#helper_name!($w.#method_ident(), $($rest)*)
                    };
                });
            }
        }
    }

    // Generate external enum flag arms
    for enum_flag in enum_flags {
        let flag_name = &enum_flag.flag_name;
        let variant_path = &enum_flag.variant_path;
        let setter = &enum_flag.setter;

        arms.push(quote! {
            ($w:expr, #flag_name $($rest:tt)*) => {
                $crate::#helper_name!($w.#setter($crate::#variant_path), $($rest)*)
            };
        });
    }

    // Generate named parameter arms
    for field in fields {
        if let FieldKind::Param { setter, .. } = &field.kind {
            let field_name = &field.name;
            let setter_name = setter
                .as_ref()
                .map(|s| Ident::new(s, Span::call_site()))
                .unwrap_or_else(|| field_name.clone());

            // Use setter name as the macro parameter name (e.g., "size" not "font_size")
            let param_name = &setter_name;

            arms.push(quote! {
                ($w:expr, #param_name: $v:expr $(, $($rest:tt)*)?) => {
                    $crate::#helper_name!($w.#setter_name($v), $($($rest)*)?)
                };
            });
        }
    }

    // Generate callback arms
    for field in fields {
        if let FieldKind::Callback = &field.kind {
            let field_name = &field.name;
            arms.push(quote! {
                ($w:expr, #field_name: $cb:expr $(, $($rest:tt)*)?) => {
                    $crate::#helper_name!($w.#field_name($cb), $($($rest)*)?)
                };
            });
        }
    }

    quote! {
        #[macro_export]
        #[doc(hidden)]
        macro_rules! #helper_name {
            #(#arms)*
        }
    }
}

/// Generate both main and helper macros
pub fn generate_widget_macros(
    struct_name: &Ident,
    macro_name: &Ident,
    fields: &[FieldConfig],
    macro_config: &WidgetMacroConfig,
    enum_flags: &[ExternalEnumFlag],
) -> TokenStream {
    let main_macro = generate_main_macro(macro_name, struct_name, fields, macro_config);
    let helper_macro = generate_helper_macro(macro_name, fields, enum_flags);

    quote! {
        #main_macro
        #helper_macro
    }
}

/// Generate a complete set of macros including alias if provided
pub fn generate_with_alias(
    struct_name: &Ident,
    macro_name: &Ident,
    alias: Option<&str>,
    fields: &[FieldConfig],
    macro_config: &WidgetMacroConfig,
    enum_flags: &[ExternalEnumFlag],
) -> TokenStream {
    let main_macros =
        generate_widget_macros(struct_name, macro_name, fields, macro_config, enum_flags);

    // If there's an alias, generate a forwarding macro
    let alias_macro = alias.map(|alias_name| {
        let alias_ident = Ident::new(alias_name, Span::call_site());
        quote! {
            #[macro_export]
            macro_rules! #alias_ident {
                ($($tt:tt)*) => {
                    $crate::#macro_name!($($tt)*)
                };
            }
        }
    });

    quote! {
        #main_macros
        #alias_macro
    }
}

// ============================================================================
// Widget::build() Implementation Generation
// ============================================================================

/// Generate the Widget::build() implementation
pub fn generate_widget_impl(
    struct_name: &Ident,
    config: &WidgetConfig,
    input: &DeriveInput,
) -> TokenStream {
    let layout_setup = generate_layout_setup(&config.layout);
    let style_setup = generate_style_setup(&config.style);
    let child_building = generate_child_building(&config.fields);

    // Extract generics
    let generics = &input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    // Determine node content based on style config
    let node_content = if config.style.background.is_some() || config.style.corner_radius.is_some()
    {
        quote! { render_engine::NodeContent::Rect { color: __background_color } }
    } else {
        quote! { render_engine::NodeContent::Group }
    };

    // Use crate:: paths since this derive is used from within widget-core
    // For external usage, we'd need to make this configurable
    quote! {
        impl #impl_generics crate::Widget for #struct_name #ty_generics #where_clause {
            fn build(&self, ctx: &mut crate::WidgetContext) -> render_engine::NodeId {
                // Setup background color from style
                #style_setup

                // Create the node
                let __parent = ctx.root();
                let __node = render_engine::SceneNode::new(#node_content);
                let __node_id = ctx.scene_mut().add_node(__parent, __node);

                // Apply layout style
                #layout_setup
                ctx.set_layout_style(__node_id, __layout_style);

                // Build children
                #child_building

                __node_id
            }
        }
    }
}

/// Generate layout style setup code from LayoutConfig
fn generate_layout_setup(layout: &LayoutConfig) -> TokenStream {
    let mut setters = Vec::new();

    // Direction
    if let Some(dir) = &layout.direction {
        let dir_variant = match dir.as_str() {
            "Row" => quote! { layout_engine::FlexDirection::Row },
            "RowReverse" => quote! { layout_engine::FlexDirection::RowReverse },
            "Column" => quote! { layout_engine::FlexDirection::Column },
            "ColumnReverse" => quote! { layout_engine::FlexDirection::ColumnReverse },
            _ => quote! { layout_engine::FlexDirection::Column },
        };
        setters.push(quote! { __layout_style.direction = #dir_variant; });
    }

    // Gap
    if let Some(gap) = layout.gap {
        setters.push(quote! { __layout_style.gap = #gap; });
    }

    // Padding
    if let Some(padding) = layout.padding {
        setters.push(quote! {
            __layout_style.padding_left = #padding;
            __layout_style.padding_right = #padding;
            __layout_style.padding_top = #padding;
            __layout_style.padding_bottom = #padding;
        });
    }
    if let Some(v) = layout.padding_left {
        setters.push(quote! { __layout_style.padding_left = #v; });
    }
    if let Some(v) = layout.padding_right {
        setters.push(quote! { __layout_style.padding_right = #v; });
    }
    if let Some(v) = layout.padding_top {
        setters.push(quote! { __layout_style.padding_top = #v; });
    }
    if let Some(v) = layout.padding_bottom {
        setters.push(quote! { __layout_style.padding_bottom = #v; });
    }

    // Justify content
    if let Some(justify) = &layout.justify {
        let variant = match justify.as_str() {
            "FlexStart" => quote! { layout_engine::JustifyContent::FlexStart },
            "FlexEnd" => quote! { layout_engine::JustifyContent::FlexEnd },
            "Center" => quote! { layout_engine::JustifyContent::Center },
            "SpaceBetween" => quote! { layout_engine::JustifyContent::SpaceBetween },
            "SpaceAround" => quote! { layout_engine::JustifyContent::SpaceAround },
            "SpaceEvenly" => quote! { layout_engine::JustifyContent::SpaceEvenly },
            _ => quote! { layout_engine::JustifyContent::FlexStart },
        };
        setters.push(quote! { __layout_style.justify_content = #variant; });
    }

    // Align items
    if let Some(align) = &layout.align {
        let variant = match align.as_str() {
            "FlexStart" => quote! { layout_engine::AlignItems::FlexStart },
            "FlexEnd" => quote! { layout_engine::AlignItems::FlexEnd },
            "Center" => quote! { layout_engine::AlignItems::Center },
            "Stretch" => quote! { layout_engine::AlignItems::Stretch },
            "Baseline" => quote! { layout_engine::AlignItems::Baseline },
            _ => quote! { layout_engine::AlignItems::Stretch },
        };
        setters.push(quote! { __layout_style.align_items = #variant; });
    }

    // Flex grow
    if let Some(flex) = layout.flex {
        setters.push(quote! { __layout_style.flex_grow = #flex; });
    }

    // Flex shrink
    if let Some(flex_shrink) = layout.flex_shrink {
        setters.push(quote! { __layout_style.flex_shrink = #flex_shrink; });
    }

    quote! {
        let mut __layout_style = layout_engine::FlexStyle::default();
        #(#setters)*
    }
}

/// Generate style setup code from StyleConfig
fn generate_style_setup(style: &StyleConfig) -> TokenStream {
    let background_color = if let Some(bg) = &style.background {
        // Parse color name or use default - use render_engine::Color type
        match bg.as_str() {
            "White" => quote! { render_engine::Color::WHITE },
            "Black" => quote! { render_engine::Color::BLACK },
            "Red" => quote! { render_engine::Color::rgba(1.0, 0.0, 0.0, 1.0) },
            "Green" => quote! { render_engine::Color::rgba(0.0, 1.0, 0.0, 1.0) },
            "Blue" => quote! { render_engine::Color::rgba(0.0, 0.0, 1.0, 1.0) },
            "Transparent" => quote! { render_engine::Color::rgba(0.0, 0.0, 0.0, 0.0) },
            _ => {
                // Assume it's a variable name or expression
                let ident = Ident::new(bg, Span::call_site());
                quote! { #ident }
            }
        }
    } else {
        quote! { render_engine::Color::rgba(0.0, 0.0, 0.0, 0.0) }
    };

    let mut style_setters = Vec::new();

    if style.opacity.is_some() {
        let opacity = style.opacity.unwrap();
        style_setters.push(quote! {
            if let Some(node) = ctx.scene_mut().get_node_mut(__node_id) {
                node.opacity = #opacity;
            }
        });
    }

    quote! {
        let __background_color = #background_color;
        #(#style_setters)*
    }
}

/// Generate child building code from fields
fn generate_child_building(fields: &[FieldConfig]) -> TokenStream {
    let mut child_builders = Vec::new();

    for field in fields {
        let field_name = &field.name;

        match &field.kind {
            FieldKind::Child => {
                // Single child widget
                let slot_layout = generate_slot_layout(&field.slot_layout);
                child_builders.push(quote! {
                    {
                        let __child_id = self.#field_name.build(ctx);
                        ctx.reparent_to(__child_id, __node_id);
                        #slot_layout
                    }
                });
            }
            FieldKind::Children => {
                // Multiple children (WidgetTuple)
                child_builders.push(quote! {
                    self.#field_name.build_all(ctx, __node_id);
                });
            }
            _ => {}
        }
    }

    quote! {
        #(#child_builders)*
    }
}

/// Generate slot-specific layout application
fn generate_slot_layout(slot_layout: &Option<LayoutConfig>) -> TokenStream {
    match slot_layout {
        Some(layout) if !layout.is_empty() => {
            let layout_setup = generate_layout_setup(layout);
            quote! {
                {
                    #layout_setup
                    ctx.set_layout_style(__child_id, __layout_style);
                }
            }
        }
        _ => quote! {},
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_macro_name_from_struct() {
        let struct_name = Ident::new("TextInput", Span::call_site());
        let name = macro_name(&struct_name, None);
        assert_eq!(name.to_string(), "text_input");
    }

    #[test]
    fn test_macro_name_override() {
        let struct_name = Ident::new("TextInput", Span::call_site());
        let name = macro_name(&struct_name, Some("input"));
        assert_eq!(name.to_string(), "input");
    }

    #[test]
    fn test_helper_macro_name() {
        let name = Ident::new("btn", Span::call_site());
        let helper = helper_macro_name(&name);
        assert_eq!(helper.to_string(), "__btn_apply");
    }
}
