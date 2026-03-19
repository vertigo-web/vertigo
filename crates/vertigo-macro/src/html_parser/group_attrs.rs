use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use rstml::node::{KeyedAttributeValue, NodeName};
use std::collections::BTreeMap;
use syn::{Ident, spanned::Spanned};

use crate::component::get_group_attrs_push_method_name;

use super::commons::{
    dereference_maybe, generate_debug_class_name, is_default_block, take_block_or_literal_expr,
    unwrap_block_if_single,
};
use super::component::COMPONENT_ATTR_FORMAT_ERROR;

pub(super) fn convert_group_attrs(
    group: &str,
    attrs: &BTreeMap<String, KeyedAttributeValue>,
    constructor_name: &NodeName,
) -> TokenStream2 {
    let group = Ident::new(group, constructor_name.span());
    let group_method = get_group_attrs_push_method_name(&group);
    let mut attrs_stream = Vec::new();

    for (key, possible_value) in attrs {
        let matches = take_block_or_literal_expr(possible_value, COMPONENT_ATTR_FORMAT_ERROR);
        let key_value_pair = match matches {
            (Some(value), None) => {
                if is_default_block(&value.block) {
                    Some(quote! { #key.into(), vertigo::AttrGroupValue::AttrValue("".into()) })
                } else {
                    let value = dereference_maybe(&value.block)
                        .unwrap_or_else(|| unwrap_block_if_single(&value.block));
                    let variant = attr_to_group_variant(key, value);
                    Some(quote! { .#group_method(#key.into(), #variant) })
                }
            }
            (None, Some(lit)) => Some(quote! { .#group_method(#key.into(), #lit.into()) }),
            _ => None,
        };

        attrs_stream.push(key_value_pair);
    }

    quote! { #(#attrs_stream)* }
}

fn attr_to_group_variant(key: impl AsRef<str>, value: TokenStream2) -> TokenStream2 {
    match key.as_ref() {
        "css" => {
            let debug_class_name = generate_debug_class_name(&value);
            quote! { vertigo::AttrGroupValue::css(#value, #debug_class_name) }
        }
        "hook_key_down" => quote! { vertigo::AttrGroupValue::hook_key_down(#value) },
        "on_blur" => quote! { vertigo::AttrGroupValue::on_blur(#value) },
        "on_change" => quote! { vertigo::AttrGroupValue::on_change(#value) },
        "on_click" => quote! { vertigo::AttrGroupValue::on_click(#value) },
        "on_dropfile" => quote! { vertigo::AttrGroupValue::on_dropfile(#value) },
        "on_input" => quote! { vertigo::AttrGroupValue::on_input(#value) },
        "on_key_down" => quote! { vertigo::AttrGroupValue::on_key_down(#value) },
        "on_load" => quote! { vertigo::AttrGroupValue::on_load(#value) },
        "on_mouse_down" => quote! { vertigo::AttrGroupValue::on_mouse_down(#value) },
        "on_mouse_enter" => quote! { vertigo::AttrGroupValue::on_mouse_enter(#value) },
        "on_mouse_leave" => quote! { vertigo::AttrGroupValue::on_mouse_leave(#value) },
        "on_mouse_up" => quote! { vertigo::AttrGroupValue::on_mouse_up(#value) },
        "on_submit" | "form" => quote! { vertigo::AttrGroupValue::on_submit(#value) },
        _ => quote! { {#value}.into() },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::quote;

    fn tokens_eq(a: &TokenStream2, b: &TokenStream2) -> bool {
        a.to_string() == b.to_string()
    }

    #[test]
    fn test_attr_to_group_variant_css() {
        let val = quote! { my_css };
        let result = attr_to_group_variant("css", val);
        let s = result.to_string();
        assert!(
            s.contains("AttrGroupValue :: css"),
            "Expected css variant, got: {s}"
        );
        assert!(s.contains("my_css"), "Expected value token, got: {s}");
    }

    #[test]
    fn test_attr_to_group_variant_fallback() {
        let val = quote! { some_value };
        let result = attr_to_group_variant("unknown_attr", val.clone());
        let expected = quote! { { some_value }.into() };
        assert!(
            tokens_eq(&result, &expected),
            "Expected fallback into(), got: {}",
            result
        );
    }
}
