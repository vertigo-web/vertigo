use itertools::Itertools;
use pest::{Parser, iterators::Pair};
use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;
use std::{collections::HashMap, error::Error};
use syn::{Expr, ExprLit, Ident, Lit, parse_str};

#[derive(Parser)]
#[grammar = "css.pest"]
pub struct CssParser {
    call_site: Span,
    children: Vec<String>,
    params: ParamsEnumerator,
    /// Css objects references inside rules (to be replaced with auto-class-names)
    css_refs: Vec<Ident>,
}

impl CssParser {
    pub fn new(call_site: Span) -> Self {
        Self {
            call_site,
            children: Vec::new(),
            params: ParamsEnumerator::default(),
            css_refs: Vec::new(),
        }
    }

    pub fn parse_stream(call_site: Span, input: &str) -> (TokenStream2, bool, Vec<Ident>) {
        match CssParser::parse(Rule::css_block, input) {
            Ok(pairs) => {
                let mut parser = Self::new(call_site);
                for pair in pairs {
                    let Ok(new_children) = parser.generate_rule(pair) else {
                        emit_error!(call_site, "Error while parsing CSS");
                        return (quote! {}, false, vec![]);
                    };
                    parser.children.extend(new_children);
                }

                let css_output = parser.children.join("\n");

                let params = parser.params.into_hashmap();

                if params.is_empty() {
                    let css_output = css_output.replace("{{", "{").replace("}}", "}");
                    (quote! { #css_output }, false, parser.css_refs)
                } else {
                    let params_stream: TokenStream2 = params
                        .into_iter()
                        .map(|(param_expr, param_key)| {
                            // let ident = Ident::new(&p, call_site);
                            let data_expr: Option<Expr> = parse_str(&param_expr)
                                .map_err(|e| {
                                    emit_error!(
                                        call_site,
                                        "Error while parsing `{}`: {}",
                                        param_expr,
                                        e
                                    );
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
                    (
                        quote! { format!(#css_output, #params_stream) },
                        true,
                        parser.css_refs,
                    )
                }
            }
            Err(e) => {
                emit_error!(call_site, "CSS Parsing fatal error: {}", e);
                (quote! {}, false, vec![])
            }
        }
    }

    fn generate_rule(&mut self, pair: Pair<Rule>) -> Result<Vec<String>, Box<dyn Error>> {
        let mut children = Vec::new();
        match pair.as_rule() {
            Rule::unknown_rule => {
                let child = self.generate_unknown_rule(pair)?;
                children.push(child)
            }
            Rule::animation_rule => {
                let child = self.generate_animation_rule(pair)?;
                children.push(child);
            }
            Rule::media_rules => {
                let child = self.generate_media_rules(pair)?;
                children.push(child);
            }
            Rule::sub_rule => {
                let child = self.generate_sub_rule(pair)?;
                children.push(child);
            }
            _ => (),
        }
        Ok(children)
    }

    fn generate_unknown_rule(&mut self, pair: Pair<Rule>) -> Result<String, Box<dyn Error>> {
        let rule_start = pair.as_span().start();
        let rule_str = pair.as_str();

        let mut pairs = pair.into_inner();
        let rule_ident = pairs.next().ok_or("Missing rule identifier")?;

        let ident_str = rule_ident.as_str();
        let mut value_str = String::new();
        let mut prev_end = None;

        for value in pairs {
            let span = value.as_span();

            // Keep the whitespace written between parts of the value, as for example in
            // `rotate({deg}deg)` no space can be added, while in `1px solid {color}` it can't be lost.
            if let Some(prev_end) = prev_end {
                value_str.push_str(&rule_str[prev_end - rule_start..span.start() - rule_start]);
            }
            prev_end = Some(span.end());

            let part_str = match value.as_rule() {
                Rule::color_value => value.as_str().to_string(),
                Rule::unquoted_value => value.as_str().to_string(),
                Rule::quoted_value => escape_braces(value.as_str()),
                Rule::expression => self.params.insert(
                    value
                        .into_inner()
                        .next()
                        .ok_or("Missing expression")?
                        .as_str()
                        .to_string(),
                ),
                Rule::url_value => {
                    // url_value -> expression/quoted_value/url_raw
                    let inner = value.clone().into_inner().next().ok_or("Missing URL")?;
                    match inner.as_rule() {
                        Rule::expression => {
                            // expression -> expression_value
                            let expr_value = inner
                                .into_inner()
                                .next()
                                .ok_or("Missing URL value")?
                                .as_str()
                                .to_string();
                            format!("url('{}')", self.params.insert(expr_value))
                        }
                        _ => {
                            // quoted_value or url_raw
                            escape_braces(value.as_str())
                        }
                    }
                }
                _ => {
                    emit_warning!(
                        self.call_site,
                        "CSS: unhandler value in generate_unknown_rule: {:?}",
                        value
                    );
                    "".to_string()
                }
            };
            value_str.push_str(&part_str);
        }

        Ok(format!(
            "{ident_str}: {};",
            value_str.replace("} px", "}px")
        ))
    }

    fn generate_animation_rule(&mut self, pair: Pair<Rule>) -> Result<String, Box<dyn Error>> {
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
                            let step = frame_children
                                .next()
                                .ok_or("Missing animation step")?
                                .as_str();
                            frame_children
                                .map(|rule| match rule.as_rule() {
                                    Rule::unknown_rule => self.generate_unknown_rule(rule),
                                    Rule::comment => Ok(String::new()),
                                    _ => Ok(String::new()),
                                })
                                .collect::<Result<Vec<_>, _>>()
                                .map(|vec| {
                                    vec.into_iter()
                                        .filter(|s| !s.is_empty())
                                        .collect::<Vec<_>>()
                                        .join("\n")
                                })
                                .map(|frame_rules| format!("{step} {{{{ {frame_rules} }}}}"))
                        })
                        .collect::<Result<Vec<_>, _>>()?
                        .join("\n");
                    format!("{{{{ {frames_strs} }}}}")
                }
                _ => {
                    emit_warning!(
                        self.call_site,
                        "CSS: unhandled value in generate_animation_rule: {:?}",
                        value
                    );
                    "".to_string()
                }
            };
            value_strs.push(value_str);
        }

        Ok(format!("animation: {};", value_strs.join(" ")))
    }

    fn generate_media_rules(&mut self, pair: Pair<Rule>) -> Result<String, Box<dyn Error>> {
        let pairs = pair.into_inner();

        // let mut sub_selector = None;
        let mut query = None;
        let mut value_strs = Vec::new();

        for pair in pairs {
            match pair.as_rule() {
                Rule::media_query => query = Some(pair.as_str().to_string()),
                // Rule::sub_selector => sub_selector = Some(pair.as_str()),
                _ => {
                    value_strs.extend(self.generate_rule(pair)?);
                }
            };
        }

        if let Some(query) = query {
            Ok(format!(
                "@media {query} {{{{ {} }}}};",
                value_strs.join(" ")
            ))
        } else {
            emit_warning!(self.call_site, "CSS: Generated empty media query");
            Ok("".to_string())
        }
    }

    fn generate_sub_rule(&mut self, pair: Pair<Rule>) -> Result<String, Box<dyn Error>> {
        let rule_start = pair.as_span().start();
        let rule_str = pair.as_str();
        let pairs = pair.into_inner();

        // Range of the selector in the rule
        let mut selector_range: Option<(usize, usize)> = None;
        let mut value_strs = Vec::new();

        for pair in pairs {
            match pair.as_rule() {
                Rule::sub_selector | Rule::sub_selector_ref => {
                    if pair.as_rule() == Rule::sub_selector_ref {
                        // Save reference identifier without brackets
                        self.css_refs.push(Ident::new(
                            pair.as_str().trim_matches(|c| c == '[' || c == ']'),
                            self.call_site,
                        ));
                    }
                    let span = pair.as_span();
                    let start = selector_range.map_or(span.start(), |(start, _)| start);
                    selector_range = Some((start, span.end()));
                }
                _ => {
                    value_strs.extend(self.generate_rule(pair)?);
                }
            };
        }

        if let Some((start, end)) = selector_range {
            // Selector as written, as spaces between its parts matter (`[ref]:hover` vs `[ref] :hover`)
            let selector = &rule_str[start - rule_start..end - rule_start];
            Ok(format!("{selector} {{{{ {} }}}};", value_strs.join("\n")))
        } else {
            emit_warning!(self.call_site, "CSS: Generated empty sub-rule");
            Ok("".to_string())
        }
    }
}

/// Braces in a literal text need to be doubled, as the output can end up as a `format!` string
fn escape_braces(text: &str) -> String {
    text.replace('{', "{{").replace('}', "}}")
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
                    .map(|_| b'a'..=b'z')
                    .multi_cartesian_product()
                    .map(|letters| String::from_utf8(letters).unwrap_or_default()),
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
            let Some(key) = self.seq.next() else {
                emit_call_site_error!("Missing seq in params enumerator");
                return "".to_string();
            };
            self.params.insert(param, key.clone());
            key
        };

        format!("{{{key}}}")
    }

    fn into_hashmap(self) -> HashMap<String, String> {
        self.params
    }
}

// Returns:
// .0 css string
// .1 if is dynamic
// .2 replacement of referenced Css with auto-class-name
pub(crate) fn generate_css_string(input: TokenStream) -> (TokenStream2, bool, TokenStream2) {
    let call_site = Span::call_site();
    let (css_output, is_dynamic, refs) = CssParser::parse_stream(call_site, &get_string(input));
    if refs.is_empty() {
        (css_output, is_dynamic, quote! {})
    } else {
        let mut replacements = quote! {
            let css_output = #css_output.to_string();
        };
        for reference in refs {
            let ref_str = reference.to_string();
            replacements = quote! {
                #replacements
                let css_output = css_output.replace(
                    &["[", #ref_str, "]"].concat(),
                    &[".", &vertigo::get_driver().class_name_for(&#reference)].concat(),
                );
            };
        }
        (quote! { css_output }, true, replacements)
    }
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
