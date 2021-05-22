use pest::{Parser, iterators::Pair};

use proc_macro2::{TokenStream, Ident, Span};
use syn::{Expr, parse_str};

#[derive(Parser)]
#[grammar = "html.pest"]
pub struct HtmlParser {
    call_site: Span,
    children: Vec<TokenStream>,
}

impl HtmlParser {
    pub fn new(call_site: Span) -> Self {
        Self {
            call_site,
            children: Vec::new(),
        }
    }

    pub fn parse_stream(call_site: Span, input: &str, is_root: bool) -> TokenStream {
        match HtmlParser::parse(Rule::html, input) {
            Ok(pairs) => {
                let mut parser = Self::new(call_site);
                for pair in pairs {
                    // emit_warning!(call_site, "HTML: parse_stream debug: {:?}", pair);
                    match pair.as_rule() {
                        Rule::html => { },

                        Rule::el_normal |
                        Rule::el_void |
                        Rule::el_void_xml |
                        Rule::el_raw_text => {
                            let child = parser.generate_node_element(pair, is_root);
                            parser.children.push(child);
                        },

                        Rule::el_vcomponent => parser.children.push(parser.generate_vcomponent(pair, false)),
                        Rule::el_vcomponent_val => parser.children.push(parser.generate_vcomponent(pair, true)),

                        Rule::node_text => {
                            emit_warning!(call_site, "HTML: Plain text can't be a root node");
                        },

                        Rule::EOI => { }

                        _ => {
                            emit_warning!(call_site, "HTML: unhandler root pair: {:?}", pair);
                        }
                    }
                }
                let children = parser.children;
                quote! { #(#children)* }
            },
            Err(e) => {
                emit_error!(call_site, "HTML Parsing fatal error: {}", e);
                quote! { }
            },
        }
    }

    fn generate_node_element(&mut self, pair: Pair<Rule>, is_root: bool) -> TokenStream {
        let mut tag_name = "";

        // FIXME:
        //
        // children are regular elements added into HTML
        // children_lists are { ..something } ones (it's a list of lists)
        //
        // Regular children will always render first no matter how placed in HTML, so
        //    <div> "foo" { ..vec1 } "bar" { ..vec2 } {value} </div>
        // will render as:
        //    <div> "foo" "bar" {value} { ..vec1 } { ..vec2 } </div>

        let mut attrs = Vec::new();
        let mut children = Vec::new();
        let mut children_lists = Vec::new();

        for pair in pair.into_inner() {
            // emit_warning!(self.call_site, "HTML: generate_node_element debug: {:?}", pair);
            match pair.as_rule() {
                Rule::el_name => tag_name = pair.as_str(),
                Rule::el_void_name => tag_name = pair.as_str(),
                Rule::el_raw_text_name => tag_name = pair.as_str(),

                Rule::regular_attr => attrs.push(self.generate_regular_attr(pair)),
                Rule::css_attr => attrs.push(self.generate_expression_attr(pair, Some("css"))),
                Rule::onclick_attr => attrs.push(self.generate_expression_attr(pair, Some("on_click"))),
                Rule::oninput_attr => attrs.push(self.generate_expression_attr(pair, Some("on_input"))),
                Rule::onmouseenter_attr => attrs.push(self.generate_expression_attr(pair, Some("on_mouse_enter"))),
                Rule::onmouseleave_attr => attrs.push(self.generate_expression_attr(pair, Some("on_mouse_leave"))),
                Rule::onkeydown_attr => attrs.push(self.generate_expression_attr(pair, Some("on_key_down"))),
                Rule::expression_attr => attrs.push(self.generate_expression_attr(pair, None)),

                Rule::el_vcomponent => children.push(pair),
                Rule::el_vcomponent_val => children.push(pair),
                Rule::el_normal => children.push(pair),
                Rule::el_void => children.push(pair),
                Rule::el_void_xml => children.push(pair),
                Rule::el_raw_text => children.push(pair),
                Rule::el_raw_text_content => children.push(pair),
                Rule::el_velement => children.push(pair),
                Rule::node_text => children.push(pair),
                Rule::expression => children.push(pair),

                Rule::el_normal_start => {
                    for tag_pair in pair.into_inner() {
                        // TODO: Refactor: These repeated variants should be taken into some separate function
                        match tag_pair.as_rule() {
                            Rule::el_name => tag_name = tag_pair.as_str(),
                            Rule::el_void_name => tag_name = tag_pair.as_str(),
                            Rule::el_raw_text_name => tag_name = tag_pair.as_str(),

                            Rule::regular_attr => attrs.push(self.generate_regular_attr(tag_pair)),
                            Rule::css_attr => attrs.push(self.generate_expression_attr(tag_pair, Some("css"))),
                            Rule::onclick_attr => attrs.push(self.generate_expression_attr(tag_pair, Some("on_click"))),
                            Rule::oninput_attr => attrs.push(self.generate_expression_attr(tag_pair, Some("on_input"))),
                            Rule::onmouseenter_attr => attrs.push(self.generate_expression_attr(tag_pair, Some("on_mouse_enter"))),
                            Rule::onmouseleave_attr => attrs.push(self.generate_expression_attr(tag_pair, Some("on_mouse_leave"))),
                            Rule::onkeydown_attr => attrs.push(self.generate_expression_attr(tag_pair, Some("on_key_down"))),
                            Rule::expression_attr => attrs.push(self.generate_expression_attr(tag_pair, None)),

                            _ => {
                                emit_error!(self.call_site, "HTML: unhandled tag_pair in generate_node_element: {:?}", tag_pair);
                            }
                        }
                    }
                }
                Rule::el_normal_end => {},

                Rule::children => children_lists.push(pair),

                _ => {
                    emit_error!(self.call_site, "HTML: unhandled pair in generate_node_element: {:?}", pair);
                }
            }
        }

        let mut generated_children = Vec::new();
        for pair in children.into_iter() {
            match pair.as_rule() {
                Rule::el_vcomponent => generated_children.push(self.generate_vcomponent(pair, false)),
                Rule::el_vcomponent_val => generated_children.push(self.generate_vcomponent(pair, true)),
                Rule::el_normal => generated_children.push(self.generate_node_element(pair, false)),
                Rule::el_void => generated_children.push(self.generate_node_element(pair, false)),
                Rule::el_void_xml => generated_children.push(self.generate_node_element(pair, false)),
                Rule::el_raw_text => generated_children.push(self.generate_node_element(pair, false)),
                Rule::el_raw_text_content => {
                    if let Some(ts) = self.generate_text(pair) {
                        generated_children.push(ts)
                    }
                },
                Rule::el_velement => generated_children.push(self.generate_velement(pair)),
                Rule::node_text => if let Some(ts) = self.generate_text(pair) {
                    generated_children.push(ts)
                }
                Rule::expression => generated_children.push(self.generate_expression(pair)),
                _ => {
                    emit_error!(self.call_site, "HTML: unhandled child pair in generate_node_element: {:?}", pair);
                }
            }
        }

        let mut generated_children_lists = Vec::new();
        for pair in children_lists {
            generated_children_lists.push(self.generate_children(pair))
        }

        let builder = if is_root {
            quote! { vertigo::VDomElement::new }
        } else {
            quote! { vertigo::VDomNode::node }
        };

        if generated_children_lists.is_empty() {
            quote! {
                #builder(
                    #tag_name,
                    vec![#(#attrs),*],
                    vec![#(#generated_children),*],
                )
            }
        } else {
            quote! {
                #builder(
                    #tag_name,
                    vec![#(#attrs),*],
                    {
                        let mut children = vec![#(#generated_children),*];
                        #(children.extend(#generated_children_lists);)*
                        children
                    }
                )
            }
        }
    }

    fn generate_vcomponent(&self, pair: Pair<Rule>, value: bool) -> TokenStream {
        let mut render_func = None::<Expr>;
        let mut data_expr = None::<Expr>;

        for pair in pair.into_inner() {
            match pair.as_rule() {
                Rule::vcomp_render_func => {
                    let value = pair.into_inner().next().unwrap().as_str();
                    render_func = parse_str(value).map_err(|e| {
                        emit_error!(self.call_site, "Error while parsing `{}`: {}", value, e);
                        e
                    }).ok();
                },
                Rule::vcomp_data_attr => {
                    let value = pair.into_inner().next().unwrap().as_str();
                    data_expr = parse_str(value).map_err(|e| {
                        emit_error!(self.call_site, "Error while parsing `{}`: {}", value, e);
                        e
                    }).ok();
                }
                _ => {
                    emit_warning!(self.call_site, "HTML: unhandler pair in generate_component: {:?}", pair);
                }
            }
        }

        let builder = if value {
            quote! { vertigo::VDomComponent::from_value }
        } else {
            quote! { vertigo::VDomComponent::new }
        };

        if let Some(render_func) = render_func {
            if let Some(data_expr) = data_expr {
                quote! { vertigo::VDomNode::component(#builder(#data_expr.clone(), #render_func)) }
            } else {
                emit_warning!(self.call_site, "HTML: Component don't have data attribute");
                quote! { }
            }
        } else {
            emit_warning!(self.call_site, "HTML: Component don't have render function defined");
            quote! { }
        }
    }

    fn generate_velement(&self, pair: Pair<Rule>) -> TokenStream {
        let velem_value = pair.into_inner().next().unwrap();
        match velem_value.as_rule() {
            Rule::attr_expression_value => {
                let value = velem_value.as_str();
                let expr: Expr = parse_str(value).unwrap_or_else(|e| panic!("Error while parsing `{}`: {}", value, e));
                return quote! { #expr }
            }
            _ => {
                emit_warning!(self.call_site, "HTML: unhandler pair in generate_velement (2): {:?}", velem_value);
            }
        };
        quote! { }
    }

    fn generate_text(&self, pair: Pair<Rule>) -> Option<TokenStream> {
        match pair.as_rule() {
            Rule::node_text |
            Rule::el_raw_text_content => {
                let trimmed = pair.as_str().trim();
                let unquoted = trimmed.trim_matches('"');
                if unquoted.len() + 2 != trimmed.len() {
                    emit_error!(self.call_site, "Please double-quote strings in your HTML to avoid problems with formatting");
                }
                Some(quote! { vertigo::VDomNode::text(#unquoted) })
            },
            _ => {
                emit_warning!(self.call_site, "HTML: unhandler pair in generate_text: {:?}", pair);
                None
            }
        }
    }

    fn generate_expression(&mut self, pair: Pair<Rule>) -> TokenStream {
        // emit_warning!(self.call_site, "Parsing expresion {:?}", pair);
        let pair = pair.into_inner().next().unwrap();
        match pair.as_rule() {
            Rule::expression_value => {
                let value = pair.as_str();
                let expr: Expr = parse_str(value).unwrap_or_else(|e| panic!("Error while parsing `{}`: {}", value, e));
                quote! { vertigo_html::Embed::embed((#expr)) }
            },
            _ => {
                emit_warning!(self.call_site, "HTML: unhandler pair in generate_expression: {:?}", pair);
                quote! { }
            }
        }
    }

    fn generate_children(&self, pair: Pair<Rule>) -> TokenStream {
        let pair = pair.into_inner().next().unwrap();
        match pair.as_rule() {
            Rule::expression_value => {
                let value = pair.as_str();
                let expr: Expr = parse_str(value).unwrap_or_else(|e| panic!("Error while parsing `{}`: {}", value, e));
                quote! { (#expr).into_iter().map(|e| vertigo_html::Embed::embed(e)) }
            },
            _ => {
                emit_warning!(self.call_site, "HTML: unhandler pair in generate_children: {:?}", pair);
                quote! { }
            }
        }
    }

    fn generate_regular_attr(&self, pair: Pair<Rule>) -> TokenStream {
        match pair.as_rule() {
            Rule::regular_attr => {
                let mut inner = pair.into_inner();
                let key = inner.next().unwrap().as_str().replace(' ', "");
                let value = inner.next().unwrap().as_str();
                quote! { vertigo::node_attr::attr(#key, #value) }
            }
            _ => {
                emit_warning!(self.call_site, "HTML: unhandler pair in generate_regular_attr: {:?}", pair);
                quote! { }
            }
        }
    }

    fn generate_expression_attr(&self, pair: Pair<Rule>, attr_key_opt: Option<&str>) -> TokenStream {
        let mut pair = pair.into_inner();

        // Use vertigo attr if provided, otherwise read custom attr from grammar
        let attr_key = attr_key_opt.unwrap_or_else(|| pair.next().unwrap().as_str()).replace(' ', "");

        let expression_val = pair.next().unwrap();

        match expression_val.as_rule() {
            Rule::attr_expression_value => {
                let value = expression_val.as_str();
                let expr: Expr = parse_str(value).unwrap_or_else(|e| panic!("Error while parsing `{}`: {}", value, e));
                if attr_key_opt.is_some() {
                    // Vertigo attribute
                    let attr_key = Ident::new(&attr_key, self.call_site);
                    return quote! { vertigo::node_attr::#attr_key((#expr)) }
                } else {
                    // Custom attribute
                    return quote! { vertigo::node_attr::attr(#attr_key, (#expr)) }
                }
            },
            _ => {
                emit_warning!(self.call_site, "HTML: unhandler pair in generate_expression_attr, attr_key {}: {:?}", attr_key, expression_val);
            }
        };
        quote! { }
    }
}
