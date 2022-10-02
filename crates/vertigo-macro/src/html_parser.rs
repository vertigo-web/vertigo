use syn::{Expr, __private::ToTokens};
use syn_rsx::{parse, Node, NodeType};
use proc_macro::{TokenStream};
use proc_macro2::{TokenStream as TokenStream2, Span};

fn find_attribute(span: Span, attributes: &[Node], attribute: &'static str) -> Result<Expr, ()> {
    for attr_item in attributes {
        assert_eq!(attr_item.node_type, NodeType::Attribute);

        let name = attr_item.name_as_string().unwrap();
        let value = attr_item.value.clone().unwrap();

        if name == attribute {
            return Ok(value);
        }
    }

    emit_error!(span, format!("Expected attribute '{attribute}'"));
    Err(())
}

fn convert_expresion_without_brackets(expr: Expr) -> TokenStream2 {
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

fn convert_node(node: Node) -> Result<TokenStream2, ()> {
    assert_eq!(node.node_type, NodeType::Element);
    let node_name = node.name_as_string().unwrap();
    // let span = node.name_span().unwrap();

    if node_name == "text" {
        let span = node.name_span().unwrap();

        let computed = find_attribute(span, &node.attributes, "computed")?;
        let computed = convert_expresion_without_brackets(computed);

        return Ok(quote!{
            vertigo::DomText::new_computed(#computed)
        });
    }

    let mut out_attr = Vec::new();
    let mut out_child = Vec::new();

    for attr_item in node.attributes {
        assert_eq!(attr_item.node_type, NodeType::Attribute);

        let name = attr_item.name_as_string().unwrap();
        let value = attr_item.value.unwrap();

        let value = convert_expresion_without_brackets(value);
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
        } else if name == "css" {
            out_attr.push(quote!{
                .css(#value.into())
            })
        } else {
            out_attr.push(quote!{
                .attr(#name, #value.into())
            })
        }
    }

    for child in node.children {
        if child.node_type == NodeType::Text {
            let child_value = child.value.unwrap();
            out_child.push(quote! {
                .child(vertigo::DomText::new(#child_value))
            });
        } else if child.node_type == NodeType::Element {
            let node_ready = convert_node(child)?;

            out_child.push(quote! {
                .child(#node_ready)
            });
        } else if child.node_type == NodeType::Block {
            let block = child.value.unwrap();
            let block = convert_expresion_without_brackets(block);

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

    Ok(quote! {
        vertigo::DomElement::new(#node_name)
        #(#out_attr)*
        #(#out_child)*
    })
}

pub fn dom_inner(input: TokenStream) -> TokenStream {
    let mut nodes = parse(input).unwrap();

    let nodes_len = nodes.len();
    let last = nodes.pop();

    if !nodes.is_empty() {
        panic!("exactly one node was expected - received = {nodes_len}");
    }

    if let Some(last) = last {
        return match convert_node(last) {
            Ok(result) => result.into(),
            _ => {
                quote! {}.into()
            }
        };
    }

    panic!("exactly one node was expected - received = {nodes_len}");
}
