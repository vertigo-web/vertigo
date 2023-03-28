use syn::{Expr, __private::ToTokens};
use syn_rsx::{parse, Node, NodeType};
use proc_macro::{TokenStream};
use proc_macro2::{TokenStream as TokenStream2, Span};

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
    name.chars().next().map(char::is_uppercase).unwrap_or_default()
}

fn convert_to_component(node: Node) -> TokenStream2 {
    let component = node.name;
    let attributes = node.attributes.into_iter()
        .filter_map(|attr_node| {
            let span = attr_node.name_span().unwrap();
            if let (Some(key), Some(value)) = (attr_node.name, attr_node.value) {
                let value = strip_brackets(value);

                if value.to_string().starts_with('&') {
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

fn convert_node(node: Node, convert_to_dom_node: bool) -> Result<TokenStream2, ()> {
    assert_eq!(node.node_type, NodeType::Element);
    let node_name = node.name_as_string().unwrap();
    // let span = node.name_span().unwrap();

    if is_component_name(&node_name) {
        return Ok(convert_to_component(node))
    }

    let mut out_attr = Vec::new();
    let mut out_child = Vec::new();

    let mut push_attr = |name: String, value: TokenStream2| {
        if name == "on_click" {
            out_attr.push(quote!{
                .on_click(#value)
            })
        } else if name == "on_mouse_enter" {
            out_attr.push(quote!{
                .on_mouse_enter(#value)
            })
        } else if name == "on_mouse_leave" {
            out_attr.push(quote!{
                .on_mouse_leave(#value)
            })
        } else if name == "on_input" {
            out_attr.push(quote!{
                .on_input(#value)
            })
        } else if name == "on_key_down" {
            out_attr.push(quote!{
                .on_key_down(#value)
            })
        } else if name == "on_dropfile" {
            out_attr.push(quote!{
                .on_dropfile(#value)
            })
        } else if name == "hook_key_down" {
            out_attr.push(quote!{
                .hook_key_down(#value)
            })
        } else if name == "on_load" {
            out_attr.push(quote!{
                .on_load(#value)
            })
        } else if name == "css" {
            out_attr.push(quote!{
                .css(#value)
            })
        } else {
            out_attr.push(quote!{
                .attr(#name, #value)
            })
        }
    };

    for attr_item in node.attributes {
        if attr_item.node_type == NodeType::Block {
            let value = strip_brackets(attr_item.value.unwrap());
            let name = value.to_string();

            if !check_ident(name.as_str()) {
                return Err(());
            }

            push_attr(name, value);

        } else if attr_item.node_type == NodeType::Attribute {
            let name = attr_item.name_as_string().unwrap();
            let value = attr_item.value.unwrap();
            let value = strip_brackets(value);
            push_attr(name, value);

        } else {
            return Err(());
        }
    }

    for child in node.children {
        if child.node_type == NodeType::Text {
            let child_value = child.value.unwrap();
            out_child.push(quote! {
                .child(vertigo::DomText::new(#child_value))
            });
        } else if child.node_type == NodeType::Element {
            match child.name_as_string() {
                Some(tag_name) if is_component_name(&tag_name) => {
                    out_child.push(convert_child_to_component(child))
                }
                _ => {
                    let node_ready = convert_node(child, false)?;

                    out_child.push(quote! {
                        .child(#node_ready)
                    });
                }
            }
        } else if child.node_type == NodeType::Block {
            let block = child.value.unwrap();
            let block = strip_brackets(block);

            out_child.push(quote! {
                .child(vertigo::EmbedDom::embed(#block))
            });
        } else {
            let span = child.name_span();

            match span {
                Some(span) => {
                    emit_error!(span, "no support for the node".to_string());
                },
                None => {
                    panic!("the span element was expected");
                }
            }
            return Err(());
        }
    }

    if convert_to_dom_node {
        Ok(quote! {
            vertigo::DomNode::from(
                vertigo::DomElement::new(#node_name)
                #(#out_attr)*
                #(#out_child)*
            )
        })
    } else {
        Ok(quote! {
            vertigo::DomElement::new(#node_name)
            #(#out_attr)*
            #(#out_child)*
        })
    }
}

pub fn dom_inner(input: TokenStream) -> TokenStream {
    let nodes = parse(input).unwrap();

    let mut modes_dom = Vec::new();

    for node in nodes {
        let Ok(node) = convert_node(node, true) else {
            return quote! {}.into();
        };

        modes_dom.push(node);
    }

    if modes_dom.len() == 1 {
        let last = modes_dom.pop().unwrap();

        return quote! {
            vertigo::DomNode::from(#last)
        }.into();
    }

    if modes_dom.is_empty() {
        panic!("node / nodes expected");
    }

    quote! {
        vertigo::DomNode::from(vertigo::DomComment::dom_fragment(vec!(
            #(#modes_dom,)*
        )))
    }.into()
}


pub fn dom_element_inner(input: TokenStream) -> TokenStream {
    let nodes = parse(input).unwrap();

    let mut modes_dom = Vec::new();

    for node in nodes {
        let Ok(node) = convert_node(node, false) else {
            return quote! {}.into();
        };

        modes_dom.push(node);
    }

    if modes_dom.len() == 1 {
        let last = modes_dom.pop().unwrap();

        return quote! {
            #last
        }.into();
    }

    emit_error!(Span::call_site(), "This macro supports only one DomElement as root".to_string());
    quote!{}.into()

}
