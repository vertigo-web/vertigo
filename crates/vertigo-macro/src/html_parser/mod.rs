use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use rstml::parse;

mod commons;
mod component;
mod group_attrs;
mod node;
use node::convert_node;

pub(crate) fn dom_inner(input: TokenStream) -> TokenStream2 {
    let nodes = match parse(input) {
        Ok(nodes) => nodes,
        Err(err) => return err.to_compile_error(),
    };

    let mut dom_nodes = Vec::new();

    for node in nodes {
        let tokens = convert_node(&node, true);
        dom_nodes.push(tokens);
    }

    if dom_nodes.is_empty() {
        emit_call_site_error!("Empty input");
        return quote! {};
    }

    if dom_nodes.len() == 1 {
        let Some(last) = dom_nodes.pop() else {
            emit_call_site_error!("Empty input");
            return quote! {};
        };

        return quote! {
            vertigo::DomNode::from(#last)
        };
    }

    quote! {
        vertigo::DomNode::from(vertigo::DomComment::dom_fragment(vec!(
            #(#dom_nodes,)*
        )))
    }
}

pub(crate) fn dom_element_inner(input: TokenStream) -> TokenStream2 {
    let nodes = match parse(input) {
        Ok(nodes) => nodes,
        Err(err) => return err.to_compile_error(),
    };

    let mut modes_dom = Vec::new();

    for node in nodes {
        let node = convert_node(&node, false);
        modes_dom.push(node);
    }

    if modes_dom.len() != 1 {
        emit_call_site_error!("This macro supports only one DomElement as root");
        return quote! {};
    }

    modes_dom.pop().unwrap_or_default()
}
