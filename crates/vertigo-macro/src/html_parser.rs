use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{quote, ToTokens, TokenStreamExt};
use rstml::{
    node::{KVAttributeValue, KeyedAttribute, KeyedAttributeValue, Node, NodeAttribute, NodeBlock},
    parse,
};
use syn::{spanned::Spanned, Expr, ExprBlock, ExprLit, Stmt};

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
    let component = element.name();
    let attributes = element
        .attributes()
        .iter()
        .filter_map(|attr_node| {
            let span = attr_node.span();
            match attr_node {
                // Key-value attribute
                NodeAttribute::Attribute(KeyedAttribute { key, possible_value }) => {
                    let matches = take_block_or_literal_expr(possible_value);

                    match matches {
                        (Some(value), None) => {
                            if value.block.stmts.is_empty() || value.block.stmts[0].to_token_stream().to_string() == "Default :: default()" {
                                Some(quote! { #key: Default::default(), })
                            } else if let Some(Stmt::Expr(Expr::Reference(inner), _)) = value.block.stmts.last() {
                                let value = &inner.expr;
                                Some(quote! { #key: #value.clone(), })
                            } else {
                                Some(quote! { #key: #value.into(), })
                            }
                        }
                        (None, Some(lit)) => {
                            Some(quote! { #key: #lit.into(), })
                        }
                        _ => {
                            None
                        }
                    }
                },
                // Try to use attribute value as key if no key provided
                NodeAttribute::Block(block) => {
                    if let NodeBlock::ValidBlock(block) = block {
                        if block.stmts.is_empty() {
                            emit_error!(span, "Expected key={} attribute - can't omit attribute name when providing default value");
                            None
                        } else if let Some(Stmt::Expr(Expr::Reference(inner), _)) = block.stmts.last() {
                            let value = &inner.expr;
                            Some(quote! { #value: {#value}.clone(), })
                        } else {
                            let key = block.stmts.last();
                            Some(quote! { #key: #block.into(), })
                        }
                    } else {
                        emit_error!(span, "Expected key=value attribute");
                        None
                    }
                }
            }
        })
        .collect::<Vec<_>>();
    quote! {
        {
            let cmp = #component {
                #(#attributes)*
            };
            cmp.mount()
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
    let mut out_child = Vec::new();

    let mut push_attr = |name: String, value: TokenStream2| {
        if name == "on_click" {
            out_attr.push(quote! {
                .on_click(#value)
            })
        } else if node_name == "form" && name == "on_submit" {
            out_attr.push(quote! {
                .on_submit(#value)
            })
        } else if name == "on_mouse_down" {
            out_attr.push(quote! {
                .on_mouse_down(#value)
            })
        } else if name == "on_mouse_up" {
            out_attr.push(quote! {
                .on_mouse_up(#value)
            })
        } else if name == "on_mouse_enter" {
            out_attr.push(quote! {
                .on_mouse_enter(#value)
            })
        } else if name == "on_mouse_leave" {
            out_attr.push(quote! {
                .on_mouse_leave(#value)
            })
        } else if name == "on_input" {
            out_attr.push(quote! {
                .on_input(#value)
            })
        } else if name == "on_change" {
            out_attr.push(quote! {
                .on_change(#value)
            })
        } else if name == "on_blur" {
            out_attr.push(quote! {
                .on_blur(#value)
            })
        } else if name == "on_key_down" {
            out_attr.push(quote! {
                .on_key_down(#value)
            })
        } else if name == "on_dropfile" {
            out_attr.push(quote! {
                .on_dropfile(#value)
            })
        } else if name == "hook_key_down" {
            out_attr.push(quote! {
                .hook_key_down(#value)
            })
        } else if name == "on_load" {
            out_attr.push(quote! {
                .on_load(#value)
            })
        } else if name == "css" {
            out_attr.push(quote! {
                .css(#value)
            })
        } else if name == "vertigo-suspense" {
            out_attr.push(quote! {
                .suspense(Some(#value))
            })
        } else {
            out_attr.push(quote! {
                .attr(#name, #value)
            })
        }
    };

    for attr_item in element.attributes() {
        let span = attr_item.span();
        match attr_item {
            // Key-value attribute
            NodeAttribute::Attribute(KeyedAttribute {
                key,
                possible_value,
            }) => {
                let matches = take_block_or_literal_expr(possible_value);
                match matches {
                    (Some(value), None) => {
                        if value.block.stmts.is_empty()
                            || value.block.stmts[0].to_token_stream().to_string()
                                == "Default :: default()"
                        {
                            push_attr(
                                key.to_string(),
                                quote! { vertigo::dom::attr_value::AttrValue::String(String::new()) },
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
                    if block.stmts.is_empty() {
                        emit_error!(span, "Expected key={} attribute - can't omit attribute name when providing default value");
                    } else if let Some(Stmt::Expr(Expr::Reference(inner), _)) = block.stmts.last() {
                        let value = &inner.expr;
                        push_attr(value.to_token_stream().to_string(), value.to_token_stream())
                    } else if block.stmts.len() == 1 {
                        let value = block.stmts.last();
                        push_attr(value.to_token_stream().to_string(), value.to_token_stream())
                    } else {
                        let key = block.stmts.last();
                        push_attr(key.to_token_stream().to_string(), block.to_token_stream())
                    }
                } else {
                    emit_error!(span, "Expected key=value attribute");
                    continue;
                }
            }
        }
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

    if convert_to_dom_node {
        quote! {
            vertigo::DomNode::from(
                vertigo::DomElement::new(#node_name)
                #(#out_attr)*
                #(#out_child)*
            )
        }
    } else {
        quote! {
            vertigo::DomElement::new(#node_name)
            #(#out_attr)*
            #(#out_child)*
        }
    }
}

/// Out of attribute value takes ExprBlock or ExprLit and ignores everything else
fn take_block_or_literal_expr(
    expr: &KeyedAttributeValue,
) -> (Option<&ExprBlock>, Option<&ExprLit>) {
    match expr {
        KeyedAttributeValue::Binding(_fn_binding) => {
            emit_error!(expr.span(), "Invalid attr - binding");
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
                        emit_error!(expr.span(), "Invalid attr - invalid block type {:?}", expr);
                        (None, None)
                    }
                }
            }
            KVAttributeValue::InvalidBraced(invalid) => {
                emit_error!(invalid.span(), "Invalid attr - invalid braces");
                (None, None)
            }
        },
        KeyedAttributeValue::None => {
            emit_error!(expr.span(), "Invalid attr - none");
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
