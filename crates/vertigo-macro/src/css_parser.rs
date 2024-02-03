use itertools::Itertools;
use pest::{iterators::Pair, Parser};
use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;
use std::collections::HashMap;
use syn::{parse_str, Expr, ExprLit, Ident, Lit};

#[derive(Parser)]
#[grammar = "css.pest"]
pub struct CssParser {
    call_site: Span,
    children: Vec<String>,
    params: ParamsEnumerator,
}

impl CssParser {
    pub fn new(call_site: Span) -> Self {
        Self {
            call_site,
            children: Vec::new(),
            params: ParamsEnumerator::default(),
        }
    }

    pub fn parse_stream(call_site: Span, input: &str) -> (TokenStream2, bool) {
        match CssParser::parse(Rule::css_block, input) {
            Ok(pairs) => {
                let mut parser = Self::new(call_site);
                for pair in pairs {
                    // emit_warning!(call_site, "Pair: {:?}", pair);
                    let new_children = parser.generate_rule(pair);
                    parser.children.extend(new_children);
                }

                let css_output = parser.children.join("\n");

                let params = parser.params.into_hashmap();

                if params.is_empty() {
                    let css_output = css_output
                        .replace("{{", "{")
                        .replace("}}", "}");
                    (quote! { #css_output }, false)
                } else {
                    let params_stream: TokenStream2 = params
                        .into_iter()
                        .map(|(param_expr, param_key)| {
                            // let ident = Ident::new(&p, call_site);
                            let data_expr: Option<Expr> = parse_str(&param_expr)
                                .map_err(|e| {
                                    emit_error!(call_site, "Error while parsing `{}`: {}", param_expr, e);
                                    e
                                })
                                .ok();
                            if let Some(data_expr) = data_expr {
                                let param_ident = Ident::new(&param_key, call_site);
                                quote! { #param_ident=#data_expr, }
                            } else {
                                quote! {}
                            }
                        })
                        .collect();
                    (quote! { format!(#css_output, #params_stream) }, true)
                }
            }
            Err(e) => {
                emit_error!(call_site, "CSS Parsing fatal error: {}", e);
                (quote! {}, false)
            }
        }
    }

    fn generate_rule(&mut self, pair: Pair<Rule>) -> Vec<String> {
        let mut children = Vec::new();
        match pair.as_rule() {
            Rule::unknown_rule => {
                let child = self.generate_unknown_rule(pair);
                children.push(child)
            }
            Rule::animation_rule => {
                let child = self.generate_animation_rule(pair);
                children.push(child);
            }
            Rule::media_rules => {
                let child = self.generate_media_rules(pair);
                children.push(child);
            }
            Rule::sub_rule => {
                let child = self.generate_sub_rule(pair);
                children.push(child);
            }
            _ => (),
        }
        children
    }

    fn generate_unknown_rule(&mut self, pair: Pair<Rule>) -> String {
        let mut pairs = pair.into_inner();
        let rule_ident = pairs.next().unwrap();

        let ident_str = rule_ident.as_str();
        let mut value_strs = Vec::new();

        for value in pairs {
            let value_str = match value.as_rule() {
                Rule::color_value => value.as_str().to_string(),
                Rule::unquoted_value => value.as_str().to_string(),
                Rule::quoted_value => value.as_str().to_string(),
                Rule::expression => {
                    self.params.insert(
                        value.into_inner().next().unwrap().as_str().to_string()
                    )
                }
                _ => {
                    emit_warning!(self.call_site, "CSS: unhandler value in generate_unknown_rule: {:?}", value);
                    "".to_string()
                }
            };
            value_strs.push(value_str);
        }

        format!("{ident_str}: {};", value_strs.join(" ").replace("} px", "}px"))
    }

    fn generate_animation_rule(&mut self, pair: Pair<Rule>) -> String {
        let pairs = pair.into_inner();

        let mut value_strs = Vec::new();

        for value in pairs {
            let value_str = match value.as_rule() {
                Rule::animation_params_fallback => value.as_str().to_string(),
                Rule::frames => {
                    let frames_strs = value
                        .into_inner()
                        .map(|frame| {
                            let mut frame_children = frame.into_inner();
                            let step = frame_children.next().unwrap().as_str();
                            let frame_rules = frame_children
                                .map(|rule| {
                                    // emit_warning!(call_site, "{:?}", rule);
                                    self.generate_unknown_rule(rule)
                                })
                                .collect::<Vec<_>>()
                                .join("\n");
                            format!("{step} {{{{ {frame_rules} }}}}")
                        })
                        .collect::<Vec<_>>()
                        .join("\n");
                    format!("{{{{ {frames_strs} }}}}")
                }
                _ => {
                    emit_warning!(self.call_site, "CSS: unhandled value in generate_animation_rule: {:?}", value);
                    "".to_string()
                }
            };
            value_strs.push(value_str);
        }

        format!("animation: {};", value_strs.join(" "))
    }

    fn generate_media_rules(&mut self, pair: Pair<Rule>) -> String {
        let pairs = pair.into_inner();

        // let mut sub_selector = None;
        let mut query = None;
        let mut value_strs = Vec::new();

        for pair in pairs {
            match pair.as_rule() {
                Rule::media_query => query = Some(pair.as_str().to_string()),
                // Rule::sub_selector => sub_selector = Some(pair.as_str()),
                _ => {
                    value_strs.extend(self.generate_rule(pair));
                }
            };
        }

        if let Some(query) = query {
            format!("@media {query} {{{{ {} }}}};", value_strs.join(" "))
        } else {
            emit_warning!(self.call_site, "CSS: Generated empty media query");
            "".to_string()
        }
    }

    fn generate_sub_rule(&mut self, pair: Pair<Rule>) -> String {
        let pairs = pair.into_inner();

        let mut sub_selector = None;
        let mut value_strs = Vec::new();

        for pair in pairs {
            match pair.as_rule() {
                Rule::sub_selector => sub_selector = Some(pair.as_str()),
                _ => {
                    value_strs.extend(self.generate_rule(pair));
                }
            };
        }

        if let Some(sub_selector) = sub_selector {
            format!("{sub_selector} {{{{ {} }}}};", value_strs.join("\n"))
        } else {
            emit_warning!(self.call_site, "CSS: Generated empty sub-rule");
            "".to_string()
        }
    }
}

struct ParamsEnumerator {
    seq: Box<dyn Iterator<Item = String>>,
    params: HashMap<String, String>,
}

impl Default for ParamsEnumerator {
    fn default() -> Self {
        Self {
            seq: Box::new(
                (0..2)
                    .map(|_| (b'a'..=b'z'))
                    .multi_cartesian_product()
                    .map(|letters| String::from_utf8(letters).unwrap())
            ),
            params: HashMap::default(),
        }
    }
}

impl ParamsEnumerator {
    pub fn insert(&mut self, param: String) -> String {
        let key = if let Some(key) = self.params.get(&param) {
            key.clone()
        } else {
            let key = self.seq.next().unwrap();
            self.params.insert(param, key.clone());
            key
        };

        format!("{{{key}}}")
    }

    fn into_hashmap(self) -> HashMap<String, String> {
        self.params
    }
}

pub(crate) fn generate_css_string(input: TokenStream) -> (TokenStream2, bool) {
    let call_site = Span::call_site();
    CssParser::parse_stream(call_site, &get_string(input))
    // let result = CssParser::parse_stream(call_site, &get_string(input));
    // emit_warning!(call_site, "CSS: output: {}", result.0);
    // result
}

fn get_string(input: TokenStream) -> String {
    match syn::parse::<ExprLit>(input) {
        Ok(str_input) => match str_input.lit {
            Lit::Str(lit_str) => lit_str.value(),
            _ => panic!("Unsupported input type"),
        },
        Err(e) => panic!("Error parsing input: {e}"),
    }
}
