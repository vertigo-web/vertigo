
use pest::{Parser, iterators::Pair};

use proc_macro2::{TokenStream, Ident, Span};
use syn::{Expr, parse_str};

#[derive(Parser)]
#[grammar = "grammar.pest"]
pub struct HtmlParser;

impl HtmlParser {
    pub fn parse_stream(call_site: Span, input: &str, is_root: bool) -> TokenStream {
        match HtmlParser::parse(Rule::html, input) {
            Ok(pairs) => {
                let mut children = Vec::new();
                for pair in pairs {
                    match pair.as_rule() {
                        Rule::html => { },
                        Rule::root_node => children.push(HtmlParser::generate_node_element(call_site, pair.into_inner().next().unwrap(), is_root)),
                        Rule::node_text => {
                            emit_warning!(call_site, "HTML: Plain text can't be a root node");
                        },
                        Rule::EOI => { }
                        _ => {
                            emit_warning!(call_site, "HTML: unhandler pair: {:?}", pair);
                        }
                    }
                }
                let output = quote! { #(#children)* };
                // emit_warning!(call_site, "HTML: output: {}", output);
                output
            },
            Err(e) => {
                emit_error!(call_site, "HTML Parsing fatal error: {}", e);
                quote! { }
            },
        }
    }

    fn generate_node_element(call_site: Span, pair: Pair<Rule>, is_root: bool) -> TokenStream {
        let mut tag_name = "";

        // FIXME:
        //
        // children are regular elements added into HTML
        // children_lists are { ..something } ones (it's a list of lists)
        //
        // Regular children will always render first no matter how places in HTML, so
        //    <div> "foo" { ..vec1 } "bar" { ..vec2 } {value} </div>
        // will render as:
        //    <div> "foo" "bar" {value} { ..vec1 } { ..vec2 } </div>

        let mut children = Vec::new();
        let mut children_lists = Vec::new();

        for pair in pair.into_inner() {
            // emit_warning!(call_site, "HTML: generate_node_element debug: {:?}", pair);
            match pair.as_rule() {
                Rule::el_name => tag_name = pair.as_str(),
                Rule::regular_attr => children.push(HtmlParser::generate_regular_attr(call_site, pair)),
                Rule::css_attr => children.push(HtmlParser::generate_css_attr(call_site, pair)),
                Rule::onclick_attr => children.push(HtmlParser::generate_onclick_attr(call_site, pair)),
                Rule::el_vcomponent => children.push(HtmlParser::generate_vcomponent(call_site, pair)),
                Rule::el_velement => children.push(HtmlParser::generate_velement(call_site, pair)),
                Rule::el_normal => children.push(HtmlParser::generate_node_element(call_site, pair, false)),
                Rule::node_text => children.push(HtmlParser::generate_text(call_site, pair)),
                Rule::expression => children.push(HtmlParser::generate_expression(call_site, pair)),
                Rule::children => children_lists.push(HtmlParser::generate_children(call_site, pair)),
                Rule::el_normal_end => {},
                _ => {
                    emit_warning!(call_site, "HTML: unhandler pair in generate_node_element: {:?}", pair);
                }
            }
        }

        let builder = if is_root {
            quote! { build_node }
        } else {
            quote! { node }
        };

        if children_lists.is_empty() {
            quote! {
                vertigo::node_attr::#builder(
                    #tag_name,
                    vec![#(#children),*]
                )
            }
        } else {
            quote! {
                vertigo::node_attr::#builder(
                    #tag_name,
                    {
                        let mut children = vec![#(#children),*];
                        #(children.extend(#children_lists);)*
                        children
                    }
                )
            }
        }
    }

    fn generate_vcomponent(call_site: Span, pair: Pair<Rule>) -> TokenStream {
        let mut render_func = None::<Expr>;
        let mut data_expr = None::<Expr>;

        for pair in pair.into_inner() {
            match pair.as_rule() {
                Rule::vcomp_render_func => {
                    let value = pair.into_inner().next().unwrap().as_str();
                    render_func = Some(parse_str(value).unwrap_or_else(|e| {
                        emit_error!(call_site, "Error while parsing `{}`: {}", value, e);
                        Expr::__Nonexhaustive
                    }));
                },
                Rule::vcomp_data_attr => {
                    let value = pair.into_inner().next().unwrap().as_str();
                    data_expr = Some(parse_str(value).unwrap_or_else(|e| {
                        emit_error!(call_site, "Error while parsing `{}`: {}", value, e);
                        Expr::__Nonexhaustive
                    }));
                }
                _ => {
                    emit_warning!(call_site, "HTML: unhandler pair in generate_component: {:?}", pair);
                }
            }
        }

        if let Some(render_func) = render_func {
            if let Some(data_expr) = data_expr {
                quote! { vertigo::node_attr::component(#data_expr, #render_func) }
            } else {
                emit_warning!(call_site, "HTML: Component don't have data attribute");
                quote! { }
            }
        } else {
            emit_warning!(call_site, "HTML: Component don't have render function defined");
            quote! { }
        }
    }

    fn generate_velement(call_site: Span, pair: Pair<Rule>) -> TokenStream {
        let velem_value = pair.into_inner().next().unwrap();
        match velem_value.as_rule() {
            Rule::attr_expression_value => {
                let value = velem_value.as_str();
                let expr: Expr = parse_str(value).unwrap_or_else(|e| {
                    emit_error!(call_site, "Error while parsing `{}`: {}", value, e);
                    Expr::__Nonexhaustive
                });
                return quote! { #expr }
            }
            _ => {
                emit_warning!(call_site, "HTML: unhandler pair in generate_velement (2): {:?}", velem_value);
            }
        };
        quote! { }
    }

    fn generate_text(call_site: Span, pair: Pair<Rule>) -> TokenStream {
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

    fn generate_expression(call_site: Span, pair: Pair<Rule>) -> TokenStream {
        // emit_warning!(call_site, "Parsing expresion {:?}", pair);
        let pair = pair.into_inner().next().unwrap();
        match pair.as_rule() {
            Rule::expression_value => {
                let value = pair.as_str();
                let expr: Expr = parse_str(value).unwrap_or_else(|e| {
                    emit_error!(call_site, "Error while parsing `{}`: {}", value, e);
                    Expr::__Nonexhaustive
                });
                quote! { #expr .embed() }
            },
            _ => {
                emit_warning!(call_site, "HTML: unhandler pair in generate_expression: {:?}", pair);
                quote! { }
            }
        }
    }

    fn generate_children(call_site: Span, pair: Pair<Rule>) -> TokenStream {
        let pair = pair.into_inner().next().unwrap();
        match pair.as_rule() {
            Rule::expression_value => {
                let value = pair.as_str();
                let expr: Expr = parse_str(value).unwrap_or_else(|e| {
                    emit_error!(call_site, "Error while parsing `{}`: {}", value, e);
                    Expr::__Nonexhaustive
                });
                quote! { #expr }
            },
            _ => {
                emit_warning!(call_site, "HTML: unhandler pair in generate_expression: {:?}", pair);
                quote! { }
            }
        }
    }

    fn generate_regular_attr(call_site: Span, pair: Pair<Rule>) -> TokenStream {
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

    fn generate_css_attr(call_site: Span, pair: Pair<Rule>) -> TokenStream {
        let expression_val = pair.into_inner().next().unwrap();
        match expression_val.as_rule() {
            Rule::attr_expression_value => {
                let value = expression_val.as_str();
                let expr: Expr = parse_str(value).unwrap_or_else(|e| {
                    emit_error!(call_site, "Error while parsing `{}`: {}", value, e);
                    Expr::__Nonexhaustive
                });
                return quote! { vertigo::node_attr::css(#expr) }
            }
            _ => {
                emit_warning!(call_site, "HTML: unhandler pair in generate_css_attr (2): {:?}", expression_val);
            }
        };
        quote! { }
    }

    fn generate_onclick_attr(call_site: Span, pair: Pair<Rule>) -> TokenStream {
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
