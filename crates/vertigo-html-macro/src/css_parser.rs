use pest::{Parser, iterators::Pair};
use std::{
    collections::HashMap,
    iter::FromIterator
};

use proc_macro2::{Span, TokenStream};
use syn::{Ident, Expr, parse_str};
use itertools::Itertools;

#[derive(Parser)]
#[grammar = "css.pest"]
pub struct CssParser;

impl CssParser {
    pub fn parse_stream(call_site: Span, input: &str) -> (TokenStream, bool) {
        let mut params_enumerator = ParamsEnumerator::default();

        match CssParser::parse(Rule::css_block, input) {
            Ok(pairs) => {
                let mut children = Vec::new();
                for pair in pairs {
                    // emit_warning!(call_site, "Pair: {:?}", pair);
                    match pair.as_rule() {
                        Rule::unknown_rule => children.push(Self::generate_unknown_rule(call_site, &mut params_enumerator, pair)),
                        Rule::animation_rule => {
                            children.push(Self::generate_animation_rule(call_site, &mut params_enumerator, pair));
                        }
                        _ => (),
                    }
                }

                let css_output = children.join("\n");

                let params = params_enumerator.into_hashmap();

                if params.is_empty() {
                    (quote! { #css_output }, false)
                } else {
                    let params_stream = TokenStream::from_iter(
                        params.into_iter().map(|(param_expr, param_key)| {
                            // let ident = Ident::new(&p, call_site);
                            let data_expr: Option<Expr> = parse_str(&param_expr).map_err(|e| {
                                emit_error!(call_site, "Error while parsing `{}`: {}", param_expr, e);
                                e
                            }).ok();
                            if let Some(data_expr) = data_expr {
                                let param_ident = Ident::new(&param_key, call_site);
                                quote! { #param_ident=#data_expr, }
                            } else {
                                quote! { }
                            }
                        })
                    );
                    (quote! { format!(#css_output, #params_stream) }, true)
                }
            },
            Err(e) => {
                emit_error!(call_site, "HTML Parsing fatal error: {}", e);
                (quote! { }, false)
            },
        }
    }

    fn generate_unknown_rule(call_site: Span, params: &mut ParamsEnumerator, pair: Pair<Rule>) -> String {
        let mut pairs = pair.into_inner();
        let rule_ident = pairs.next().unwrap();

        let ident_str = rule_ident.as_str().replace(" ", "");
        let mut value_strs = Vec::new();

        for value in pairs {
            let value_str = match value.as_rule() {
                Rule::color_value => value.as_str().replace(" ", ""),
                Rule::unquoted_value => value.as_str().replace(" %", "%"),
                Rule::quoted_value => value.as_str().to_string(),
                Rule::expression => {
                    params.insert(
                        value.into_inner().next().unwrap().as_str().to_string()
                    )
                }
                _ => {
                    emit_warning!(call_site, "CSS: unhandler value in generate_unknown_rule: {:?}", value);
                    "".to_string()
                }
            };
            value_strs.push(value_str);
        }

        format!("{}: {};", ident_str, value_strs.join(" ").replace("} px", "}px"))
    }

    fn generate_animation_rule(call_site: Span, params: &mut ParamsEnumerator, pair: Pair<Rule>) -> String {
        let pairs = pair.into_inner();

        let mut value_strs = Vec::new();

        for value in pairs {
            let value_str = match value.as_rule() {
                Rule::animation_params_fallback => value.as_str().replace(" - ", "-").replace(" %", "%"),
                Rule::frames => {
                    let frames_strs = value.into_inner()
                        .into_iter()
                        .map(|frame| {
                            let mut frame_children = frame.into_inner();
                            let step = frame_children.next().unwrap().as_str().replace(" %", "%");
                            let frame_rules = frame_children
                                .map(|rule| {
                                    // emit_warning!(call_site, "{:?}", rule);
                                    CssParser::generate_unknown_rule(call_site, params, rule)
                                })
                                .collect::<Vec<_>>()
                                .join("\n");
                            format!("{} {{ {} }}", step, frame_rules)
                        })
                        .collect::<Vec<_>>()
                        .join("\n");
                    format!("{{ {} }}", frames_strs)
                },
                _ => {
                    emit_warning!(call_site, "CSS: unhandler value in generate_unknown_rule: {:?}", value);
                    "".to_string()
                }
            };
            value_strs.push(value_str);
        }

        format!("animation: {};", value_strs.join(" "))
    }
}

struct ParamsEnumerator {
    seq: Box<dyn Iterator<Item=String>>,
    params: HashMap<String, String>
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

        format!("{{{}}}", key)
    }

    fn into_hashmap(self) -> HashMap<String, String> {
        self.params
    }
}
