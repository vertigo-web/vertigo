
use pest::{Parser, iterators::Pair};

use proc_macro2::{TokenStream, Ident, Span};
use syn::{Expr, parse_str};

#[derive(Parser)]
#[grammar = "grammar.pest"]
pub struct HtmlParser;

impl HtmlParser {
    pub fn parse_stream(call_site: Span, input: &str) -> TokenStream {
        match HtmlParser::parse(Rule::html, input) {
            Ok(pairs) => {
                let mut children = Vec::new();
                for pair in pairs {
                    match pair.as_rule() {
                        Rule::html => { },
                        Rule::root_node => children.push(HtmlParser::generate_node_element(call_site, pair, true)),
                        Rule::node_text => {
                            emit_warning!(call_site, "HTML: Plaing text can't me a root node");
                        },
                        Rule::EOI => { }
                        _ => {
                            emit_warning!(call_site, "HTML: unhandler pair: {:?}", pair);
                        }
                    }
                }
                let output = quote! { #(#children)* };
                emit_warning!(call_site, "HTML: output: {}", output);
                output
            },
            Err(e) => {
                emit_error!(call_site, "HTML Parsing fatal error: {}", e);
                quote! { }
            },
        }
    }

    pub fn generate_node_element(call_site: Span, pair: Pair<Rule>, root: bool) -> TokenStream {
        let mut tag_name = "";
        let mut children = Vec::new();

        for pair in pair.into_inner() {
            match pair.as_rule() {
                Rule::el_name => tag_name = pair.as_str(),
                Rule::regular_attr => children.push(HtmlParser::generate_regular_attr(call_site, pair)),
                Rule::css_attr => children.push(HtmlParser::generate_css_attr(call_site, pair)),
                Rule::onclick_attr => children.push(HtmlParser::generate_onclick_attr(call_site, pair)),
                Rule::node_element => children.push(HtmlParser::generate_node_element(call_site, pair, false)),
                Rule::node_text => children.push(HtmlParser::generate_text(call_site, pair)),
                Rule::expression => children.push(HtmlParser::generate_expression(call_site, pair)),
                Rule::el_normal_end => {},
                _ => {
                    emit_warning!(call_site, "HTML: unhandler pair in generate_node_element: {:?}", pair);
                }
            }
        }

        let builder = if root {
            quote! { build_node }
        } else {
            quote! { node }
        };

        quote! {
            vertigo::node_attr::#builder(#tag_name, vec![#(#children),*])
        }
    }

    pub fn generate_text(call_site: Span, pair: Pair<Rule>) -> TokenStream {
        match pair.as_rule() {
            Rule::node_text => {
                let content = pair.as_str();
                quote! { vertigo::node_attr::text(#content) }
            },
            _ => {
                emit_warning!(call_site, "HTML: unhandler pair in generate_text: {:?}", pair);
                quote! { }
            }
        }
    }

    pub fn generate_expression(call_site: Span, pair: Pair<Rule>) -> TokenStream {
        // emit_warning!(call_site, "Parsing expresion {:?}", pair);
        let pair = pair.into_inner().next().unwrap();
        match pair.as_rule() {
            Rule::expression_value => {
                let value = pair.as_str();
                let expr: Expr = parse_str(value).unwrap_or_else(|e| {
                    emit_error!(call_site, "Error while parsing `{}`: {}", value, e);
                    Expr::__Nonexhaustive
                });
                quote! { vertigo::node_attr::text(#expr .to_string()) }
            },
            _ => {
                emit_warning!(call_site, "HTML: unhandler pair in generate_expression: {:?}", pair);
                quote! { }
            }
        }
    }

    pub fn generate_regular_attr(call_site: Span, pair: Pair<Rule>) -> TokenStream {
        match pair.as_rule() {
            Rule::regular_attr => {
                let mut inner = pair.into_inner();
                let key = inner.next().unwrap().as_str();
                let value = inner.next().unwrap().as_str();
                quote! { vertigo::node_attr::attr(#key, #value) }
            }
            _ => {
                emit_warning!(call_site, "HTML: unhandler pair in generate_regular_attr: {:?}", pair);
                quote! { }
            }
        }
    }

    pub fn generate_css_attr(call_site: Span, pair: Pair<Rule>) -> TokenStream {
        let expression_val = pair.into_inner().next().unwrap();
        match expression_val.as_rule() {
            Rule::attr_expression_value => {
                let value = expression_val.as_str();
                let func_name: Ident = Ident::new(value, call_site);
                return quote! { vertigo::node_attr::css(#func_name ()) }
            }
            _ => {
                emit_warning!(call_site, "HTML: unhandler pair in generate_css_attr (2): {:?}", expression_val);
            }
        };
        quote! { }
    }

    pub fn generate_onclick_attr(call_site: Span, pair: Pair<Rule>) -> TokenStream {
        let expression_val = pair.into_inner().next().unwrap();
        match expression_val.as_rule() {
            Rule::attr_expression_value => {
                let value = expression_val.as_str();
                let expr: Expr = parse_str(value).unwrap_or_else(|e| {
                    emit_error!(call_site, "Error while parsing `{}`: {}", value, e);
                    Expr::__Nonexhaustive
                });
                return quote! { vertigo::node_attr::on_click(#expr) }
            },
            _ => {
                emit_warning!(call_site, "HTML: unhandler pair in generate_onclick_attr: {:?}", expression_val);
            }
        };
        quote! { }
    }
}


/*
<div css="asdf">

<Component data={f} render={render_cell} />

<Component render_cell data={f} />

*/