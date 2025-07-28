use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{quote, ToTokens, TokenStreamExt};
use rstml::{
    node::{
        KVAttributeValue, KeyedAttribute, KeyedAttributeValue, Node, NodeAttribute, NodeBlock,
        NodeName,
    },
    parse,
};
use std::collections::BTreeMap;
use syn::{spanned::Spanned, Expr, ExprBlock, ExprLit, Ident, Stmt};

use crate::{
    component::get_group_attrs_method_name, trace_tailwind::add_to_tailwind, utils::release_build,
};

const HTML_ATTR_FORMAT_ERROR: &str =
    "in html node. Expected key=\"value\", key={value}, key={}, {value} or {..value} attribute.";
const COMPONENT_ATTR_FORMAT_ERROR: &str =
    "in component. Expected key=\"value\", key={value}, key={}, group:key=\"value\", group:key={value} or {value} attribute.";

pub(crate) fn dom_inner(input: TokenStream) -> TokenStream {
    let nodes = match parse(input) {
        Ok(nodes) => nodes,
        Err(err) => return err.to_compile_error().into(),
    };

    let mut dom_nodes = Vec::new();

    for node in nodes {
        let tokens = convert_node(&node, true);
        dom_nodes.push(tokens);
    }

    if dom_nodes.len() == 1 {
        let last = dom_nodes.pop().unwrap();

        return quote! {
            vertigo::DomNode::from(#last)
        }
        .into();
    }

    if dom_nodes.is_empty() {
        emit_error!(Span::call_site(), "Empty input");
    }

    quote! {
        vertigo::DomNode::from(vertigo::DomComment::dom_fragment(vec!(
            #(#dom_nodes,)*
        )))
    }
    .into()
}

pub(crate) fn dom_element_inner(input: TokenStream) -> TokenStream {
    let nodes = match parse(input) {
        Ok(nodes) => nodes,
        Err(err) => return err.to_compile_error().into(),
    };

    let mut modes_dom = Vec::new();

    for node in nodes {
        let node = convert_node(&node, false);
        modes_dom.push(node);
    }

    if modes_dom.len() != 1 {
        emit_error!(
            Span::call_site(),
            "This macro supports only one DomElement as root".to_string()
        );
        return TokenStream::default();
    }

    modes_dom.pop().unwrap_or_default().into()
}

fn convert_to_component(node: &Node) -> TokenStream2 {
    let element = match node {
        Node::Element(el) => el,
        _ => {
            emit_error!(node.span(), "Can't convert to component");
            return quote! {};
        }
    };
    let constructor_name = element.name();
    let component_name_string = constructor_name.to_string();

    let mut grouped_attrs = BTreeMap::<String, BTreeMap<String, KeyedAttributeValue>>::new();

    let attributes = element
        .attributes()
        .iter()
        .filter_map(|attr_node| {
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
                            let matches = take_block_or_literal_expr(
                                possible_value,
                                COMPONENT_ATTR_FORMAT_ERROR,
                            );
                            match matches {
                                (Some(value), None) => {
                                    if value.block.stmts.is_empty()
                                        || value.block.stmts[0].to_token_stream().to_string()
                                            == "Default :: default()"
                                    {
                                        Some(quote! { #key: Default::default(), })
                                    } else if let Some(Stmt::Expr(Expr::Reference(inner), _)) =
                                        value.block.stmts.last()
                                    {
                                        let value = &inner.expr;
                                        Some(quote! { #key: #value.clone(), })
                                    } else {
                                        Some(quote! { #key: #value.into(), })
                                    }
                                }
                                (None, Some(lit)) => Some(quote! { #key: #lit.into(), }),
                                _ => None,
                            }
                        }
                        // Attribute name prefixed by group name and colon
                        NodeName::Punctuated(p) => {
                            let mut i = p.pairs();
                            let group = i.next();
                            if p.len() > 1
                                && group
                                    .filter(|pair| {
                                        pair.punct().filter(|p| p.as_char() == ':').is_some()
                                    })
                                    .is_some()
                            {
                                // Strip colon group name
                                let group = group.map(|p| *p.value());
                                // Convert whole punctuated to string without spacing
                                let key_str = p.to_token_stream().to_string().replace(' ', "");
                                let key = key_str
                                    .trim_start_matches(&format!("{}:", group.to_token_stream()))
                                    .to_string();
                                match group {
                                    Some(group) => {
                                        let group_entry =
                                            grouped_attrs.entry(group.to_string()).or_default();
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
                                let matches = take_block_or_literal_expr(
                                    possible_value,
                                    COMPONENT_ATTR_FORMAT_ERROR,
                                );
                                match matches {
                                    (Some(value), None) => {
                                        if value.block.stmts.is_empty()
                                            || value.block.stmts[0].to_token_stream().to_string()
                                                == "Default :: default()"
                                        {
                                            Some(quote! { #key: Default::default(), })
                                        } else if let Some(Stmt::Expr(Expr::Reference(inner), _)) =
                                            value.block.stmts.last()
                                        {
                                            let value = &inner.expr;
                                            Some(quote! { #key: #value.clone(), })
                                        } else {
                                            Some(quote! { #key: #value.into(), })
                                        }
                                    }
                                    (None, Some(lit)) => Some(quote! { #key: #lit.into(), }),
                                    _ => None,
                                }
                            }
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
                        } else if let Some(Stmt::Expr(Expr::Reference(inner), _)) =
                            block.stmts.last()
                        {
                            let value = &inner.expr;
                            Some(quote! { #value: {#value}.clone(), })
                        } else {
                            let key = block.stmts.last();
                            Some(quote! { #key: #block.into(), })
                        }
                    } else {
                        emit_error!(span, "Invalid block {}", COMPONENT_ATTR_FORMAT_ERROR);
                        None
                    }
                }
            }
        })
        .collect::<Vec<_>>();

    let mut grouped_attrs_stream = quote! {};

    for (group, attrs) in grouped_attrs {
        let group = Ident::new(&group, constructor_name.span());
        let group_method = get_group_attrs_method_name(&group);
        let mut attrs_stream = Vec::new();
        for (key, possible_value) in attrs {
            let matches = take_block_or_literal_expr(&possible_value, COMPONENT_ATTR_FORMAT_ERROR);
            let key_value_pair = match matches {
                (Some(value), None) => {
                    if value.block.stmts.is_empty()
                        || value.block.stmts[0].to_token_stream().to_string()
                            == "Default :: default()"
                    {
                        Some(quote! { #key.into(), vertigo::AttrGroupValue::AttrValue("".into()) })
                    } else {
                        let value = if let Some(Stmt::Expr(Expr::Reference(inner), _)) =
                            value.block.stmts.last()
                        {
                            inner.expr.to_token_stream()
                        } else if value.block.stmts.len() == 1 {
                            value.block.stmts.last().unwrap().to_token_stream()
                        } else {
                            value.to_token_stream()
                        };
                        let value = match key.as_str() {
                            "css" => {
                                let debug_class_name = generate_debug_class_name(&value);
                                quote! {
                                    vertigo::AttrGroupValue::css(#value, #debug_class_name)
                                }
                            }
                            "hook_key_down" => {
                                quote! { vertigo::AttrGroupValue::hook_key_down(#value) }
                            }
                            "on_blur" => quote! { vertigo::AttrGroupValue::on_blur(#value) },
                            "on_change" => quote! { vertigo::AttrGroupValue::on_change(#value) },
                            "on_click" => quote! { vertigo::AttrGroupValue::on_click(#value) },
                            "on_dropfile" => {
                                quote! { vertigo::AttrGroupValue::on_dropfile(#value) }
                            }
                            "on_input" => quote! { vertigo::AttrGroupValue::on_input(#value) },
                            "on_key_down" => {
                                quote! { vertigo::AttrGroupValue::on_key_down(#value) }
                            }
                            "on_load" => quote! { vertigo::AttrGroupValue::on_load(#value) },
                            "on_mouse_down" => {
                                quote! { vertigo::AttrGroupValue::on_mouse_down(#value) }
                            }
                            "on_mouse_enter" => {
                                quote! { vertigo::AttrGroupValue::on_mouse_enter(#value) }
                            }
                            "on_mouse_leave" => {
                                quote! { vertigo::AttrGroupValue::on_mouse_leave(#value) }
                            }
                            "on_mouse_up" => {
                                quote! { vertigo::AttrGroupValue::on_mouse_up(#value) }
                            }
                            "on_submit" | "form" => {
                                quote! { vertigo::AttrGroupValue::on_submit(#value) }
                            }
                            "vertigo-suspense" => {
                                quote! { vertigo::AttrGroupValue::suspense(#value) }
                            }
                            _ => quote! { {#value}.into() },
                        };
                        Some(quote! { .#group_method(#key.into(), #value) })
                    }
                }
                (None, Some(lit)) => Some(quote! { .#group_method(#key.into(), #lit.into()) }),
                _ => None,
            };

            attrs_stream.push(key_value_pair);
        }
        quote! {
            #(#attrs_stream)*
        }
        .to_tokens(&mut grouped_attrs_stream);
    }

    let debug_info = if release_build() {
        quote! {}
    } else {
        quote! {
            match &cmp {
                vertigo::DomNode::Node { node } => {
                    node.add_attr("v-component", #component_name_string);
                }
                _ => {}
            };
        }
    };

    quote! {
        {
            let cmp = #constructor_name {
                #(#attributes)*
            };
            let cmp = cmp.into_component()
                #grouped_attrs_stream
            ;
            let cmp = cmp.mount();
            #debug_info
            cmp
        }
    }
}

fn convert_node(node: &Node, convert_to_dom_node: bool) -> TokenStream2 {
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

    let mut push_attr = |name: String, value: TokenStream2| {
        // Store used class name for tailwind bundler
        if name.as_str() == "tw" {
            let output = add_to_tailwind(value);
            class_values.push(quote! { vertigo::TwClass::from(#output).to_class_value() });
            return;
        }

        if name.as_str() == "class" {
            class_values.push(quote! { #value.to_string() });
            return;
        }

        let method_str = match name.as_str() {
            "hook_key_down" | "on_blur" | "on_change" | "on_click" | "on_dropfile" | "on_input"
            | "on_key_down" | "on_load" | "on_mouse_down" | "on_mouse_enter" | "on_mouse_leave"
            | "on_mouse_up" | "on_submit" => &name,

            "form" => "on_submit",

            "css" => {
                let class_name = generate_debug_class_name(&value);
                out_attr.push(quote! {
                    .css_with_class_name(#value, #class_name)
                });
                return;
            }

            "vertigo-suspense" => {
                out_attr.push(quote! {
                    .suspense(Some(#value))
                });
                return;
            }

            _ => {
                out_attr.push(quote! {
                    .attr(#name, #value)
                });
                return;
            }
        };

        let method = Ident::new(method_str, value.span());

        out_attr.push(quote! {
            .#method(#value)
        });
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
                        if value.block.stmts.is_empty()
                            || value.block.stmts[0].to_token_stream().to_string()
                                == "Default :: default()"
                        {
                            push_attr(
                                key.to_string(),
                                quote! { vertigo::dom::attr_value::AttrValue::String(Default::default()) },
                            )
                        } else if value.block.stmts.len() == 1 {
                            let value = value.block.stmts.first().unwrap();
                            push_attr(key.to_string(), value.to_token_stream())
                        } else {
                            push_attr(key.to_string(), value.to_token_stream())
                        }
                    }
                    (None, Some(lit)) => push_attr(key.to_string(), lit.to_token_stream()),
                    _ => (),
                }
            }
            // Try to use attribute value as key if no key provided
            NodeAttribute::Block(block) => {
                if let NodeBlock::ValidBlock(block) = block {
                    match block.stmts.len() {
                        0 => emit_error!(span, "Empty block {}", HTML_ATTR_FORMAT_ERROR),
                        n => {
                            // First detect if there is a spread operator used at the end of the block
                            // which is basically a Range without start
                            let value = block.stmts.last();

                            if let Some(Stmt::Expr(Expr::Range(range), _)) = value {
                                if range.start.is_none() {
                                    if let Some(value) = &range.end {
                                        let mut block = block.clone();
                                        // Remove the statement with spread operator and prepare modified one
                                        block.stmts.pop();

                                        // Prepare new block based on if spread operator was used
                                        let mut new_block = quote! {};
                                        for stmt in block.stmts {
                                            new_block.append_all(stmt.to_token_stream());
                                        }
                                        new_block.append_all(quote! {
                                            #value
                                        });

                                        out_spread_attrs.push(quote! {
                                            .add_attr_group({
                                                #new_block
                                            })
                                        });
                                        continue;
                                    }
                                }
                            }

                            if let Some(Stmt::Expr(Expr::Reference(inner), _)) = block.stmts.last()
                            {
                                let value = &inner.expr;
                                push_attr(
                                    value.to_token_stream().to_string(),
                                    value.to_token_stream(),
                                )
                            } else if n == 1 {
                                let value = block.stmts.last();
                                push_attr(
                                    value.to_token_stream().to_string(),
                                    value.to_token_stream(),
                                )
                            } else {
                                let key = block.stmts.last();
                                push_attr(
                                    key.to_token_stream().to_string(),
                                    block.to_token_stream(),
                                )
                            }
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
        let mut output = quote! {};
        for class_value in class_values {
            output.append_all(quote! { #class_value, });
        }
        out_attr.push(quote! {
            .attr("class", {
                [#output].join(" ")
            })
        });
    }

    if let Some(children) = node.children() {
        for child in children {
            match child {
                Node::Text(txt) => {
                    out_child.push(quote! {
                        .child(vertigo::DomText::new(#txt))
                    });
                }
                Node::Element(element) => {
                    let tag_name = element.name().to_string();
                    if is_component_name(&tag_name) {
                        out_child.push(convert_child_to_component(child))
                    } else {
                        let node_ready = convert_node(child, false);

                        out_child.push(quote! {
                            .child(#node_ready)
                        });
                    }
                }
                Node::Block(block) => {
                    match block {
                        NodeBlock::ValidBlock(block) => {
                            match block.stmts.len() {
                                0 => emit_warning!(block.span(), "Missing expression"),
                                n => {
                                    // First detect if there is a spread operator used at the end of the block
                                    // which is basically a Range without start
                                    let value = block.stmts.last();
                                    let mut block = block.clone();
                                    let mut spread_statement = None;

                                    if let Some(Stmt::Expr(Expr::Range(range), _)) = value {
                                        if range.start.is_none() {
                                            if let Some(value) = &range.end {
                                                // Remove the statement with spread operator and prepare modified one
                                                block.stmts.pop();
                                                spread_statement = Some(quote! {
                                                    #value
                                                        .into_iter()
                                                        .map(|item| vertigo::EmbedDom::embed(item))
                                                        .collect::<Vec<_>>()
                                                });
                                            }
                                        }
                                    }

                                    // Prepare new block based on if spread operator was used
                                    let new_block = if let Some(spread_statement) = spread_statement
                                    {
                                        let mut new_block = quote! {};
                                        for stmt in block.stmts {
                                            new_block.append_all(stmt.to_token_stream());
                                        }
                                        new_block.append_all(spread_statement);

                                        quote! { .children({ #new_block }) }
                                    } else if n == 1 {
                                        let stmt = block.stmts.first().unwrap();
                                        quote! { .child(vertigo::EmbedDom::embed(#stmt)) }
                                    } else {
                                        quote! { .child(vertigo::EmbedDom::embed(#block)) }
                                    };

                                    out_child.push(new_block)
                                }
                            }
                        }
                        NodeBlock::Invalid(invalid) => {
                            emit_error!(invalid.span(), "Invalid block");
                        }
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

/// Out of attribute value takes ExprBlock or ExprLit and ignores everything else
fn take_block_or_literal_expr<'a>(
    expr: &'a KeyedAttributeValue,
    msg: &str,
) -> (Option<&'a ExprBlock>, Option<&'a ExprLit>) {
    match expr {
        KeyedAttributeValue::Binding(_fn_binding) => {
            emit_error!(expr.span(), "Invalid attribute binding {}", msg);
            (None, None)
        }
        KeyedAttributeValue::Value(attribute_value_expr) => match &attribute_value_expr.value {
            KVAttributeValue::Expr(expr) => {
                match expr {
                    Expr::Block(block) => (Some(block), None),
                    Expr::Lit(lit) => (None, Some(lit)),
                    // TODO: Possibly others needs to be supported,
                    // so this function should in fact return Option<Expr> in future.
                    _ => {
                        emit_error!(expr.span(), "Invalid attribute {}", msg);
                        (None, None)
                    }
                }
            }
            KVAttributeValue::InvalidBraced(invalid) => {
                emit_error!(invalid.span(), "Invalid attribute braces {}", msg);
                (None, None)
            }
        },
        KeyedAttributeValue::None => {
            emit_error!(expr.span(), "Missing attribute value {}", msg);
            (None, None)
        }
    }
}

/// Tags starting with uppercase letter are considered components
fn is_component_name(name: &str) -> bool {
    name.split("::")
        .last()
        .unwrap_or_default()
        .chars()
        .next()
        .map(char::is_uppercase)
        .unwrap_or_default()
}

fn convert_child_to_component(node: &Node) -> TokenStream2 {
    let cmp_stream = convert_to_component(node);
    quote! {
        .child(#cmp_stream)
    }
}

fn generate_debug_class_name(value: &TokenStream2) -> TokenStream2 {
    if release_build() {
        quote! { None }
    } else {
        let debug_class_name = value
            .to_string()
            .chars()
            .filter(|c| !c.is_whitespace())
            .collect::<String>();

        quote! { Some(#debug_class_name.to_string()) }
    }
}
