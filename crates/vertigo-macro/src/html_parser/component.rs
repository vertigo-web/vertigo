use proc_macro2::Punct;
use proc_macro2::TokenStream as TokenStream2;
use quote::{ToTokens, quote};
use rstml::node::{
    KeyedAttribute, KeyedAttributeValue, Node, NodeAttribute, NodeBlock, NodeName, NodeNameFragment,
};
use std::collections::BTreeMap;
use syn::punctuated::Punctuated;
use syn::{ExprPath, spanned::Spanned};

use crate::{
    component::get_group_attrs_replace_method_name, trace_tailwind::add_to_tailwind,
    utils::release_build,
};

use super::commons::{
    dereferenced_assignment, extract_spread_block, is_default_block, parse_block_of_statements,
    take_block_or_literal_expr,
};
use super::group_attrs::convert_group_attrs;

pub(super) const COMPONENT_ATTR_FORMAT_ERROR: &str = "in component. Expected key=\"value\", key={value}, key={}, group:key=\"value\", group:key={value} {value} or {..value} attribute.";

pub(super) fn convert_child_to_component(node: &Node) -> TokenStream2 {
    let cmp_stream = convert_to_component(node);
    quote! {
        .child(#cmp_stream)
    }
}

pub(super) fn convert_to_component(node: &Node) -> TokenStream2 {
    let element = match node {
        Node::Element(el) => el,
        _ => {
            emit_error!(node.span(), "Can't convert to component");
            return quote! {};
        }
    };
    let constructor_name = element.name();
    let component_name_string = constructor_name.to_string();

    let mut groupped_attrs = BTreeMap::<String, BTreeMap<String, KeyedAttributeValue>>::new();
    let mut spread_attrs = Vec::new();

    let attributes = element
        .attributes()
        .iter()
        .filter_map(|attr_node| {
            attribute_to_tokens(attr_node, &mut spread_attrs, &mut groupped_attrs)
        })
        .collect::<Vec<_>>();

    let mut children_stream = quote! {};
    if let Some(children) = node.children()
        && !children.is_empty()
    {
        let mut stmts = Vec::new();
        let mut closure_arg = None;

        // Pre-scan: if any child component has no explicit attributes, generate a
        // zero-arg lazy closure (fn() -> Vec<DomNode>) so children render inside the
        // parent's mount() — after the parent has pushed its context.
        let has_no_attr_component = children.iter().any(|child| {
            if let Node::Element(el) = child {
                super::commons::is_component_name(&el.name().to_string())
                    && el.attributes().is_empty()
                    && el.children.is_empty()
            } else {
                false
            }
        });

        for (i, child) in children.iter().enumerate() {
            if i == 0 {
                let maybe_text = match child {
                    Node::Text(txt) => Some((txt.value.value(), txt.span())),
                    Node::RawText(txt) => Some((txt.to_token_stream().to_string(), txt.span())),
                    _ => None,
                };

                if let Some((ts, span)) = maybe_text {
                    let ts_no_spaces = ts.replace(" ", "");
                    if ts_no_spaces.starts_with('|') && ts_no_spaces.ends_with('|') {
                        let arg = ts_no_spaces[1..ts_no_spaces.len() - 1].trim();
                        if !arg.is_empty() {
                            let arg_ident = syn::Ident::new(arg, span);
                            closure_arg = Some(arg_ident);
                            continue;
                        }
                    }
                }
            }

            match child {
                Node::Text(txt) => {
                    stmts.push(quote! { __children.push(vertigo::EmbedDom::embed(vertigo::DomText::new(#txt))); });
                }
                Node::Element(element) => {
                    let tag_name = element.name().to_string();
                    if super::commons::is_component_name(&tag_name) {
                        let cmp = convert_to_component(child);
                        stmts.push(quote! { __children.push(vertigo::EmbedDom::embed(#cmp)); });
                    } else {
                        let node_ready = super::node::convert_node(child, false);
                        stmts.push(
                            quote! { __children.push(vertigo::EmbedDom::embed(#node_ready)); },
                        );
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
                                }
                            });

                            if let Some(new_block) = spread_block {
                                stmts.push(quote! {
                                    __children.extend(
                                        #new_block
                                            .into_iter()
                                            .map(|item| vertigo::EmbedDom::embed(item))
                                    );
                                });
                            } else {
                                let value = super::commons::unwrap_block_if_single(block);
                                stmts.push(
                                    quote! { __children.push(vertigo::EmbedDom::embed(#value)); },
                                );
                            }
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

        if let Some(arg) = closure_arg {
            children_stream = quote! {
                children: |#arg| {
                    let mut __children = Vec::new();
                    #(#stmts)*
                    __children
                },
            };
        } else if has_no_attr_component && !stmts.is_empty() {
            children_stream = quote! {
                children: || {
                    let mut __children = Vec::new();
                    #(#stmts)*
                    __children
                },
            };
        } else if !stmts.is_empty() {
            children_stream = quote! {
                children: {
                    let mut __children = Vec::new();
                    #(#stmts)*
                    __children
                },
            };
        }
    }

    let mut grouped_attrs_stream = quote! {};

    for (group, attrs) in groupped_attrs {
        convert_group_attrs(&group, &attrs, constructor_name).to_tokens(&mut grouped_attrs_stream);
    }

    let debug_info = quote! {
        match &cmp {
            vertigo::DomNode::Node { node } => {
                node.add_attr("v-component", #component_name_string);
            }
            _ => {}
        };
    };

    let effective_debug_info = if release_build() {
        quote! {
            #[cfg(test)]
            #debug_info
        }
    } else {
        debug_info
    };

    quote! {
        {
            let cmp = #constructor_name {
                #(#attributes)*
                #children_stream
            };
            let cmp = cmp.into_component()
                #grouped_attrs_stream
                #(#spread_attrs)*
            ;
            let cmp = cmp.mount();
            #effective_debug_info
            cmp
        }
    }
}

fn attribute_to_tokens(
    attr_node: &NodeAttribute,
    spread_attrs: &mut Vec<TokenStream2>,
    groupped_attrs: &mut BTreeMap<String, BTreeMap<String, KeyedAttributeValue>>,
) -> Option<TokenStream2> {
    let span = attr_node.span();
    match attr_node {
        // Key-value attribute
        NodeAttribute::Attribute(KeyedAttribute {
            key,
            possible_value,
        }) => {
            match key {
                // Regular attribute name
                NodeName::Path(key) => {
                    regular_attribute_to_tokens(key, possible_value, spread_attrs)
                }
                // Attribute name prefixed by group name and colon
                NodeName::Punctuated(p) => {
                    punctuated_attribute_to_tokens(p, possible_value, groupped_attrs)
                }
                _ => {
                    emit_error!(
                        key.span(),
                        "Invalid attribute key {}",
                        COMPONENT_ATTR_FORMAT_ERROR
                    );
                    None
                }
            }
        }
        // Try to use attribute value as key if no key provided
        NodeAttribute::Block(block) => {
            if let NodeBlock::ValidBlock(block) = block {
                if block.stmts.is_empty() {
                    emit_error!(span, "Empty block {}", COMPONENT_ATTR_FORMAT_ERROR);
                    None
                } else {
                    let (key, value, method) = parse_block_of_statements(block);
                    Some(quote! { #key: #value.#method, })
                }
            } else {
                emit_error!(span, "Invalid block {}", COMPONENT_ATTR_FORMAT_ERROR);
                None
            }
        }
    }
}

fn regular_attribute_to_tokens(
    key: &ExprPath,
    possible_value: &KeyedAttributeValue,
    spread_attrs: &mut Vec<TokenStream2>,
) -> Option<TokenStream2> {
    let matches = take_block_or_literal_expr(possible_value, COMPONENT_ATTR_FORMAT_ERROR);
    match matches {
        (Some(value), None) => {
            // If attribute is 'tw' then register tailwind classes
            if key.to_token_stream().to_string() == "tw"
                && let Err(err) = add_to_tailwind(value.to_token_stream())
            {
                emit_error!(value.span(), err);
                return None;
            }

            if is_default_block(&value.block) {
                Some(quote! { #key: Default::default(), })
            } else if let Some(new_block) =
                extract_spread_block(&value.block, |inmost_value| quote! { #inmost_value })
            {
                let replace_method_name = get_group_attrs_replace_method_name(key);
                spread_attrs.push(quote! {
                    .#replace_method_name({
                        #new_block
                    })
                });
                None
            } else {
                dereferenced_assignment(key.to_token_stream(), value)
            }
        }
        (None, Some(lit)) => {
            // If attribute is 'tw' then register tailwind classes
            if key.to_token_stream().to_string() == "tw"
                && let Err(err) = add_to_tailwind(lit.to_token_stream())
            {
                emit_error!(lit.span(), err);
                return None;
            }
            Some(quote! { #key: #lit.into(), })
        }
        _ => None,
    }
}

fn punctuated_attribute_to_tokens(
    p: &Punctuated<NodeNameFragment, Punct>,
    possible_value: &KeyedAttributeValue,
    groupped_attrs: &mut BTreeMap<String, BTreeMap<String, KeyedAttributeValue>>,
) -> Option<TokenStream2> {
    let mut i = p.pairs();
    let group = i.next();
    let have_colon = group.is_some_and(|pair| pair.punct().is_some_and(|p| p.as_char() == ':'));

    if p.len() > 1 && have_colon {
        // Strip colon group name
        let group = group.map(|p| *p.value());
        // Convert whole punctuated to string without spacing
        let key_str = p.to_token_stream().to_string().replace(' ', "");
        let key = key_str
            .trim_start_matches(&format!("{}:", group.to_token_stream()))
            .to_string();

        // If value name is 'tw' then register tailwind classes
        if key == "tw"
            && let Err(err) = add_to_tailwind(possible_value.to_token_stream())
        {
            emit_error!(possible_value.span(), err);
            return None;
        };

        match group {
            Some(group) => {
                let group_entry = groupped_attrs.entry(group.to_string()).or_default();
                group_entry.insert(key, possible_value.clone());
            }
            _ => {
                emit_error!(
                    key.span(),
                    "Invalid punctuated attribute key {}",
                    COMPONENT_ATTR_FORMAT_ERROR
                );
            }
        }
        None
    } else {
        // No colon, add regular attribute
        let key = p.to_token_stream();
        let matches = take_block_or_literal_expr(possible_value, COMPONENT_ATTR_FORMAT_ERROR);
        match matches {
            (Some(value), None) => {
                if is_default_block(&value.block) {
                    Some(quote! { #key: Default::default(), })
                } else {
                    dereferenced_assignment(key, value)
                }
            }
            (None, Some(lit)) => Some(quote! { #key: #lit.into(), }),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::quote;
    use rstml::parse2;
    use std::error::Error;

    fn format_token_stream(tokens: TokenStream2) -> Result<String, Box<dyn Error>> {
        let file = syn::parse2::<syn::File>(quote! {
            fn dummy() {
                #tokens
            }
        })?;
        Ok(prettyplease::unparse(&file))
    }

    #[test]
    fn test_convert_to_component_simple() -> Result<(), Box<dyn Error>> {
        let input = quote! { <MyComponent /> };
        let nodes = parse2(input)?;
        let node = &nodes[0];
        let result = convert_to_component(node);

        let expected = quote! {
            {
                let cmp = MyComponent {};
                let cmp = cmp.into_component();
                let cmp = cmp.mount();
                match &cmp {
                    vertigo::DomNode::Node { node } => {
                        node.add_attr("v-component", "MyComponent");
                    }
                    _ => {}
                };
                cmp
            }
        };

        pretty_assertions::assert_eq!(format_token_stream(result)?, format_token_stream(expected)?);
        Ok(())
    }

    #[test]
    fn test_convert_to_component_with_attributes() -> Result<(), Box<dyn Error>> {
        let input = quote! { <MyComponent attr1="val1" attr2={42} /> };
        let nodes = parse2(input)?;
        let node = &nodes[0];
        let result = convert_to_component(node);

        let expected = quote! {
            {
                let cmp = MyComponent {
                    attr1: "val1".into(),
                    attr2: { 42 }.into(),
                };
                let cmp = cmp.into_component();
                let cmp = cmp.mount();
                match &cmp {
                    vertigo::DomNode::Node { node } => {
                        node.add_attr("v-component", "MyComponent");
                    }
                    _ => {}
                };
                cmp
            }
        };

        pretty_assertions::assert_eq!(format_token_stream(result)?, format_token_stream(expected)?);
        Ok(())
    }

    #[test]
    fn test_convert_to_component_with_grouped_attributes() -> Result<(), Box<dyn Error>> {
        let input = quote! { <MyComponent css:color="red" css:margin={10} /> };
        let nodes = parse2(input)?;
        let node = &nodes[0];
        let result = convert_to_component(node);

        // Grouped attributes are handled via convert_group_attrs which is called for each group.
        // It results in .group_{group}_push(...) calls.
        // Literals are handled directly, while blocks use attr_to_group_variant.
        let expected = quote! {
            {
                let cmp = MyComponent {};
                let cmp = cmp.into_component()
                    .group_css_push("color".into(), "red".into())
                    .group_css_push("margin".into(), { 10 }.into());
                let cmp = cmp.mount();
                match &cmp {
                    vertigo::DomNode::Node { node } => {
                        node.add_attr("v-component", "MyComponent");
                    }
                    _ => {}
                };
                cmp
            }
        };

        pretty_assertions::assert_eq!(format_token_stream(result)?, format_token_stream(expected)?);
        Ok(())
    }

    #[test]
    fn test_convert_to_component_with_block_attribute() -> Result<(), Box<dyn Error>> {
        let input = quote! { <MyComponent { some_value } /> };
        let nodes = parse2(input)?;
        let node = &nodes[0];
        let result = convert_to_component(node);

        // parse_block_of_statements for { some_value } results in (some_value, some_value, into())
        let expected = quote! {
            {
                let cmp = MyComponent {
                    some_value: some_value.into(),
                };
                let cmp = cmp.into_component();
                let cmp = cmp.mount();
                match &cmp {
                    vertigo::DomNode::Node { node } => {
                        node.add_attr("v-component", "MyComponent");
                    }
                    _ => {}
                };
                cmp
            }
        };

        pretty_assertions::assert_eq!(format_token_stream(result)?, format_token_stream(expected)?);
        Ok(())
    }

    #[test]
    fn test_convert_to_component_with_tw_attribute() -> Result<(), Box<dyn Error>> {
        let input = quote! { <MyComponent tw="text-red-500" /> };
        let nodes = parse2(input)?;
        let node = &nodes[0];
        let result = convert_to_component(node);

        let expected = quote! {
            {
                let cmp = MyComponent {
                    tw: "text-red-500".into(),
                };
                let cmp = cmp.into_component();
                let cmp = cmp.mount();
                match &cmp {
                    vertigo::DomNode::Node { node } => {
                        node.add_attr("v-component", "MyComponent");
                    }
                    _ => {}
                };
                cmp
            }
        };

        pretty_assertions::assert_eq!(format_token_stream(result)?, format_token_stream(expected)?);
        Ok(())
    }

    #[test]
    fn test_convert_to_component_with_spread_attribute() -> Result<(), Box<dyn Error>> {
        let input = quote! { <MyComponent some_attr={..spread_val} /> };
        let nodes = parse2(input)?;
        let node = &nodes[0];
        let result = convert_to_component(node);

        let expected = quote! {
            {
                let cmp = MyComponent {};
                let cmp = cmp.into_component()
                    .group_some_attr_replace({
                        spread_val
                    });
                let cmp = cmp.mount();
                match &cmp {
                    vertigo::DomNode::Node { node } => {
                        node.add_attr("v-component", "MyComponent");
                    }
                    _ => {}
                };
                cmp
            }
        };

        pretty_assertions::assert_eq!(format_token_stream(result)?, format_token_stream(expected)?);
        Ok(())
    }

    #[test]
    fn test_convert_to_component_with_default_attribute() -> Result<(), Box<dyn Error>> {
        let input = quote! { <MyComponent some_attr={ Default::default() } /> };
        let nodes = parse2(input)?;
        let node = &nodes[0];
        let result = convert_to_component(node);

        let expected = quote! {
            {
                let cmp = MyComponent {
                    some_attr: Default::default(),
                };
                let cmp = cmp.into_component();
                let cmp = cmp.mount();
                match &cmp {
                    vertigo::DomNode::Node { node } => {
                        node.add_attr("v-component", "MyComponent");
                    }
                    _ => {}
                };
                cmp
            }
        };

        pretty_assertions::assert_eq!(format_token_stream(result)?, format_token_stream(expected)?);
        Ok(())
    }
}
