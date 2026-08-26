use proc_macro2::TokenStream as TokenStream2;
use quote::{ToTokens, TokenStreamExt, quote};
use rstml::node::{KeyedAttribute, Node, NodeAttribute, NodeBlock};
use syn::{Ident, Lit, spanned::Spanned};

use crate::trace_tailwind::add_to_tailwind;

use super::commons::{
    extract_spread_block, generate_debug_class_name, is_component_name, is_default_block,
    parse_block_of_statements, take_block_or_literal_expr, unwrap_block_if_single,
};
use super::component::{convert_child_to_component, convert_to_component};

const HTML_ATTR_FORMAT_ERROR: &str =
    "in html node. Expected key=\"value\", key={value}, key={}, {value} or {..value} attribute.";

pub(super) fn convert_node(node: &Node, convert_to_dom_node: bool) -> TokenStream2 {
    let parent_span = node.span();

    let element = match node {
        Node::Fragment(_) => {
            emit_error!(parent_span, "Fragments not supported");
            return TokenStream2::new();
        }
        Node::Element(element) => element,
        _ => {
            emit_error!(parent_span, "Expected Element");
            return TokenStream2::new();
        }
    };

    let node_name = element.name().to_string();

    if is_component_name(&node_name) {
        return convert_to_component(node);
    }

    let mut out_attr = Vec::new();
    let mut out_spread_attrs = Vec::new();
    let mut out_child = Vec::new();
    let mut class_values = Vec::new();

    let mut push_attr = |name, value, is_str_literal| {
        push_attribute(
            name,
            value,
            is_str_literal,
            &mut out_attr,
            &mut class_values,
        )
    };

    for attr_item in element.attributes() {
        let span = attr_item.span();
        match attr_item {
            // Key-value attribute
            NodeAttribute::Attribute(KeyedAttribute {
                key,
                possible_value,
            }) => {
                let matches = take_block_or_literal_expr(possible_value, HTML_ATTR_FORMAT_ERROR);
                match matches {
                    (Some(value), None) => {
                        let outcome = if is_default_block(&value.block) {
                            quote! { vertigo::AttrValue::Static("") }
                        } else {
                            unwrap_block_if_single(&value.block)
                        };
                        push_attr(key.to_string(), outcome, false)
                    }
                    (None, Some(lit)) => push_attr(
                        key.to_string(),
                        lit.to_token_stream(),
                        matches!(lit.lit, Lit::Str(_)),
                    ),
                    _ => (),
                }
            }
            // Try to use attribute value as key if no key provided
            NodeAttribute::Block(block) => {
                if let NodeBlock::ValidBlock(block) = block {
                    match block.stmts.len() {
                        0 => emit_error!(span, "Empty block {}", HTML_ATTR_FORMAT_ERROR),
                        _ => {
                            if let Some(new_block) =
                                extract_spread_block(block, |inmost_value| quote! { #inmost_value })
                            {
                                out_spread_attrs.push(quote! {
                                    .add_attr_group({
                                        #new_block
                                    })
                                });
                                continue;
                            }

                            let (key, value, _) = parse_block_of_statements(block);
                            push_attr(key.to_string(), quote! { #value }, false)
                        }
                    }
                } else {
                    emit_error!(span, "Invalid block {}", HTML_ATTR_FORMAT_ERROR);
                    continue;
                }
            }
        }
    }

    // Generate code glueing class= values with tw= values
    if !class_values.is_empty() {
        if let [(class_value, is_str_literal)] = class_values.as_slice() {
            out_attr.push(if *is_str_literal {
                quote! { .attr_static("class", #class_value) }
            } else {
                quote! { .attr("class", #class_value) }
            });
        } else {
            let mut output = quote! {};
            for (class_value, is_str_literal) in class_values {
                let class_value = as_static_attr_value(class_value, is_str_literal);
                output.append_all(quote! { vertigo::AttrValue::from(#class_value), });
            }
            out_attr.push(quote! {
                .attr("class", vertigo::AttrValue::combine(vec![#output]))
            });
        }
    }

    if let Some(children) = node.children() {
        for child in children {
            child_to_tokens(child, &mut out_child);
        }
    }

    let dom_element = quote! {
        vertigo::DomElement::new(#node_name)
        #(#out_attr)*
        #(#out_spread_attrs)*
        #(#out_child)*
    };

    if convert_to_dom_node {
        quote! {
            vertigo::DomNode::from(
                #dom_element
            )
        }
    } else {
        dom_element
    }
}

/// A string literal in the source is a `&'static str`, and saying so spares the runtime two
/// heap allocations per attribute - `AttrValue::from(&str)` cannot tell a literal from a
/// borrow of a local, so it has to copy into an `Rc<String>`. Only the macro knows.
fn as_static_attr_value(value: TokenStream2, is_str_literal: bool) -> TokenStream2 {
    if is_str_literal {
        quote! { vertigo::AttrValue::Static(#value) }
    } else {
        value
    }
}

fn push_attribute(
    name: String,
    value: TokenStream2,
    is_str_literal: bool,
    out_attr: &mut Vec<TokenStream2>,
    class_values: &mut Vec<(TokenStream2, bool)>,
) {
    // Store used class name for tailwind bundler
    if name.as_str() == "tw" {
        let span = value.span();
        let output = match add_to_tailwind(value) {
            Ok(output) => output,
            Err(err) => {
                emit_error!(span, err);
                return;
            }
        };
        class_values.push((output, is_str_literal));
        return;
    }

    if name.as_str() == "class" {
        class_values.push((value, is_str_literal));
        return;
    }

    let method_str = match name.as_str() {
        "hook_key_down" | "on_blur" | "on_change" | "on_change_file" | "on_click"
        | "on_dropfile" | "on_input" | "on_intersect" | "on_key_down" | "on_load"
        | "on_mouse_down" | "on_mouse_enter" | "on_mouse_leave" | "on_mouse_up" | "on_submit" => {
            &name
        }

        "form" => "on_submit",

        "css" => {
            let class_name = generate_debug_class_name(&value);
            out_attr.push(quote! { .css_with_class_name(#value, #class_name) });
            return;
        }

        _ => {
            // A literal has a `&'static str` value and a name known here, so it can skip
            // `AttrValue` entirely - see `DomElement::attr_static`.
            out_attr.push(if is_str_literal {
                quote! { .attr_static(#name, #value) }
            } else {
                quote! { .attr(#name, #value) }
            });
            return;
        }
    };

    let method = Ident::new(method_str, value.span());

    out_attr.push(quote! {
        .#method(#value)
    });
}

fn child_to_tokens(child: &Node, out_child: &mut Vec<TokenStream2>) {
    match child {
        Node::Text(txt) => {
            out_child.push(quote! { .child(vertigo::DomText::new(#txt)) });
        }
        Node::Element(element) => {
            let tag_name = element.name().to_string();
            if is_component_name(&tag_name) {
                out_child.push(convert_child_to_component(child))
            } else {
                let node_ready = convert_node(child, false);
                out_child.push(quote! { .child(#node_ready) });
            }
        }
        Node::Block(block) => {
            if let NodeBlock::ValidBlock(block) = block {
                if block.stmts.is_empty() {
                    emit_warning!(block.span(), "Missing expression");
                } else {
                    let spread_block = extract_spread_block(block, |value| {
                        quote! {
                            #value
                                .into_iter()
                                .map(|item| vertigo::EmbedDom::embed(item))
                                .collect::<Vec<_>>()
                        }
                    });

                    let new_block = if let Some(new_block) = spread_block {
                        quote! { .children({ #new_block }) }
                    } else {
                        let value = unwrap_block_if_single(block);
                        quote! { .child(vertigo::EmbedDom::embed(#value)) }
                    };

                    out_child.push(new_block)
                }
            } else {
                emit_error!(block.span(), "Invalid block");
            }
        }
        node => {
            emit_error!(
                node.span(),
                "Unsupported node type {} as a child",
                node.r#type()
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::quote;

    fn run_push_attribute(
        name: &str,
        value: TokenStream2,
    ) -> (Vec<TokenStream2>, Vec<(TokenStream2, bool)>) {
        run_push_attribute_lit(name, value, false)
    }

    fn run_push_attribute_lit(
        name: &str,
        value: TokenStream2,
        is_str_literal: bool,
    ) -> (Vec<TokenStream2>, Vec<(TokenStream2, bool)>) {
        let mut out_attr = Vec::new();
        let mut class_values = Vec::new();
        push_attribute(
            name.to_string(),
            value,
            is_str_literal,
            &mut out_attr,
            &mut class_values,
        );
        (out_attr, class_values)
    }

    fn without_whitespace(tokens: &TokenStream2) -> String {
        tokens
            .to_string()
            .chars()
            .filter(|c| !c.is_whitespace())
            .collect()
    }

    /// A literal in the source has a `&'static str` value, and only the macro knows that -
    /// `AttrValue::from(&str)` cannot tell a literal from a borrow of a local.
    ///
    /// It routes to `attr_static`, which skips `AttrValue` altogether. That matters for more
    /// than the two allocations it saves: the enum has four reactive variants, and reaching
    /// them from a shared non-generic function keeps `Computed`, `Value` and their drop glue
    /// linked into applications that use no reactive attributes at all. Measured on the
    /// js-framework-benchmark entry, going through `AttrValue` here cost 12kB of wasm.
    #[test]
    fn test_string_literal_takes_the_static_path() {
        let (out_attr, _) = run_push_attribute_lit("id", quote! { "main" }, true);
        let emitted = without_whitespace(&out_attr[0]);
        assert!(
            emitted.contains(".attr_static(\"id\",\"main\")"),
            "expected the static path, got: {emitted}"
        );
    }

    /// A class is held back until the end - `tw=` and `class=` are glued together first - so
    /// the literal flag has to travel with it.
    #[test]
    fn test_string_literal_class_keeps_its_literal_flag() {
        let (out_attr, class_values) = run_push_attribute_lit("class", quote! { "row" }, true);
        assert!(out_attr.is_empty());
        let (value, is_str_literal) = &class_values[0];
        assert!(is_str_literal, "a literal class must be marked as one");
        assert_eq!(without_whitespace(value), "\"row\"");
    }

    /// Anything that is not a literal has to keep going through `Into<AttrValue>`, because
    /// its lifetime is unknown here.
    #[test]
    fn test_non_literal_is_left_alone() {
        let (out_attr, _) = run_push_attribute("id", quote! { some_variable });
        let emitted = without_whitespace(&out_attr[0]);
        assert!(
            !emitted.contains("attr_static") && !emitted.contains("AttrValue::Static"),
            "a non-literal must not be claimed as static: {emitted}"
        );
    }

    #[test]
    fn test_push_attribute_plain_attr() {
        let (out_attr, class_values) = run_push_attribute("id", quote! { "main" });
        assert_eq!(out_attr.len(), 1);
        // Token streams have spaces, e.g. `. attr (` - compare without whitespace
        let s: String = out_attr[0]
            .to_string()
            .chars()
            .filter(|c| !c.is_whitespace())
            .collect();
        assert!(s.contains(".attr("), "Expected .attr call, got: {s}");
        assert!(s.contains("\"id\""), "Expected attribute name, got: {s}");
        assert!(class_values.is_empty());
    }

    #[test]
    fn test_push_attribute_event_handlers() {
        let event_attrs = [
            "on_click",
            "on_change",
            "on_input",
            "on_intersect",
            "on_blur",
            "on_submit",
            "on_key_down",
            "on_load",
            "on_mouse_down",
            "on_mouse_enter",
            "on_mouse_leave",
            "on_mouse_up",
            "on_dropfile",
            "hook_key_down",
        ];
        for attr in event_attrs {
            let (out_attr, class_values) = run_push_attribute(attr, quote! { my_handler });
            assert_eq!(out_attr.len(), 1, "Expected 1 attr entry for {attr}");
            let s = out_attr[0].to_string();
            assert!(s.contains(attr), "Expected method name {attr} in: {s}");
            assert!(
                class_values.is_empty(),
                "Unexpected class_values for {attr}"
            );
        }
    }

    #[test]
    fn test_push_attribute_form_alias() {
        // "form" is an alias for on_submit
        let (out_attr, _) = run_push_attribute("form", quote! { my_handler });
        assert_eq!(out_attr.len(), 1);
        let s = out_attr[0].to_string();
        assert!(
            s.contains("on_submit"),
            "Expected on_submit alias, got: {s}"
        );
    }

    #[test]
    fn test_push_attribute_class() {
        // "class" values go into class_values, not out_attr
        let (out_attr, class_values) = run_push_attribute("class", quote! { "my-class" });
        assert!(
            out_attr.is_empty(),
            "class should not go to out_attr directly"
        );
        assert_eq!(class_values.len(), 1);
    }

    #[test]
    fn test_push_attribute_css() {
        // "css" emits .css_with_class_name and does NOT populate class_values
        let (out_attr, class_values) = run_push_attribute("css", quote! { my_css_value });
        assert_eq!(out_attr.len(), 1);
        let s = out_attr[0].to_string();
        assert!(
            s.contains("css_with_class_name"),
            "Expected css_with_class_name, got: {s}"
        );
        assert!(class_values.is_empty());
    }

    #[test]
    fn test_push_attribute_multiple_classes_accumulated() {
        let mut out_attr = Vec::new();
        let mut class_values = Vec::new();
        push_attribute(
            "class".to_string(),
            quote! { "foo" },
            true,
            &mut out_attr,
            &mut class_values,
        );
        push_attribute(
            "class".to_string(),
            quote! { "bar" },
            true,
            &mut out_attr,
            &mut class_values,
        );
        // Both should accumulate in class_values
        assert_eq!(class_values.len(), 2);
        assert!(out_attr.is_empty());
    }
}
