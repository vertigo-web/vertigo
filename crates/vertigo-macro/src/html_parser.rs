use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;
use syn::{Expr, __private::ToTokens};
use syn_rsx::{parse, Node, NodeType};

pub(crate) fn dom_inner(input: TokenStream) -> TokenStream {
    let nodes = match parse(input) {
        Ok(nodes) => nodes,
        Err(err) => return err.to_compile_error().into(),
    };

    let mut dom_nodes = Vec::new();

    for node in nodes {
        let tokens = convert_node(node, true);
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
        let node = convert_node(node, false);
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

/// Strips expression from excessive brackets (only once)
fn strip_brackets(expr: Expr) -> TokenStream2 {
    let expr_block = if let Expr::Block(expr) = expr.clone() {
        let mut stmts = expr.block.stmts;

        let first = stmts.pop();

        if stmts.is_empty() {
            first.map(|first| first.to_token_stream())
        } else {
            None
        }
    } else {
        None
    };

    if let Some(expr_block) = expr_block {
        return expr_block;
    }

    expr.to_token_stream()
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

fn convert_to_component(node: Node) -> TokenStream2 {
    let component = node.name;
    let attributes = node
        .attributes
        .into_iter()
        .filter_map(|attr_node| {
            let span = get_span(&attr_node);
            if let (Some(key), Some(value)) = (attr_node.name, attr_node.value) {
                let value = strip_brackets(value);

                if value.to_string() == "{}" {
                    Some(quote! { #key: Default::default(), })
                } else if value.to_string().starts_with('&') {
                    Some(quote! { #key: (#value).clone(), })
                } else {
                    Some(quote! { #key: (#value).into(), })
                }
            } else {
                emit_error!(span, "Expected key=value attribute");
                None
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

fn convert_child_to_component(node: Node) -> TokenStream2 {
    let cmp_stream = convert_to_component(node);
    quote! {
        .child(#cmp_stream)
    }
}

fn check_ident(name: &str) -> bool {
    for char in name.chars() {
        if char.is_ascii_alphanumeric() || char == '_' {
            //ok
        } else {
            return false;
        }
    }

    true
}

fn convert_node(node: Node, convert_to_dom_node: bool) -> TokenStream2 {
    let parent_span = get_span(&node);

    if node.node_type == NodeType::Fragment {
        emit_error!(parent_span, "Fragments not supported");
    }

    if node.node_type != NodeType::Element {
        emit_error!(parent_span, "Expected Element");
    }

    let node_name = node.name_as_string().unwrap_or_default();

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

    for attr_item in node.attributes {
        let span = attr_item.name_span().unwrap_or(parent_span);
        if attr_item.node_type == NodeType::Block {
            let Some(value) = extract_value(attr_item, parent_span) else {
                continue;
            };
            let name = value.to_string();

            if !check_ident(name.as_str()) {
                emit_error!(span, "Invalid ident_name");
            } else {
                push_attr(name, value);
            }
        } else if attr_item.node_type == NodeType::Attribute {
            let Some(name) = attr_item.name_as_string() else {
                emit_error!(span, "Missing attribute name");
                continue;
            };
            let Some(value) = extract_value(attr_item, parent_span) else {
                continue;
            };
            push_attr(name, value);
        } else {
            emit_error!(span, "Invalid attribute type");
        }
    }

    for child in node.children {
        match &child.node_type {
            NodeType::Text => {
                let Some(child_value) = extract_value(child, parent_span) else {
                    continue;
                };
                out_child.push(quote! {
                    .child(vertigo::DomText::new(#child_value))
                });
            }
            NodeType::Element => match child.name_as_string() {
                Some(tag_name) if is_component_name(&tag_name) => {
                    out_child.push(convert_child_to_component(child))
                }
                _ => {
                    let node_ready = convert_node(child, false);

                    out_child.push(quote! {
                        .child(#node_ready)
                    });
                }
            },
            NodeType::Block => {
                let Some(block) = extract_value(child, parent_span) else {
                    continue;
                };

                out_child.push(quote! {
                    .child(vertigo::EmbedDom::embed(#block))
                });
            }
            node_type => {
                emit_error!(
                    child.name_span().unwrap_or(parent_span),
                    "Unsupported {} node as a child",
                    node_type
                );
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

fn extract_value(node: Node, fallback_span: Span) -> Option<TokenStream2> {
    match node.value {
        Some(value) => Some(strip_brackets(value)),
        None => {
            let span = node.name_span().unwrap_or(fallback_span);
            emit_error!(span, "Missing attribute value");
            None
        }
    }
}

fn get_span(node: &Node) -> Span {
    node.name_span().unwrap_or_else(Span::call_site)
}
