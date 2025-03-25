use super::{html_element::HtmlElement, HtmlNode};
use std::{borrow::Cow, collections::BTreeMap};
use html_escape::{encode_safe, encode_quoted_attribute};
use std::collections::VecDeque;

enum ChildMode {
    Child(Vec<HtmlNode>),
    Text(String),
}


#[derive(Clone, Copy)]
struct Ident {
    value: Option<usize>,
}

impl Ident {
    fn new() -> Ident {
        Ident { value: Some(0) }
    }

    fn empty() -> Ident {
        Ident { value: None }
    }

    fn get(&self) -> String {
        match self.value {
            Some(ident) => " ".repeat(ident),
            None => String::new(),
        }
    }

    fn add(&self, up_value: usize) -> Self {
        Self {
            value: self.value.map(|value| value + up_value),
        }
    }
}

pub fn convert_to_string(root: HtmlNode, format: bool) -> String {
    let mut result = vec!["<!DOCTYPE html>".to_owned()];
    let root_ident = match format {
        true => Ident::new(),
        false => Ident::empty(),
    };

    html_node_to_string(&mut result, root_ident, root);

    match format {
        true => result.join("\n"),
        false => result.concat(),
    }
}

fn html_node_to_string(result: &mut Vec<String>, ident: Ident, node: HtmlNode) {
    let ident_str = ident.get();

    match node {
        HtmlNode::Element(element) => {
            let is_self_closing = is_self_closing(&element);
            let el_name = encode_safe(&element.name);
            let attrs = attributes_to_string(element.attr);

            if is_self_closing {
                let line = [&ident_str, "<", &el_name, &attrs, " />"];

                result.push(line.concat());
                return;
            }

            match get_render_child_mode(element.children) {
                ChildMode::Child(children) => {
                    //open tag
                    let line = [&ident_str, "<", &el_name, &attrs, ">"];

                    result.push(line.concat());

                    //render child
                    for child in children {
                        html_node_to_string(result, ident.add(2), child);
                    }

                    //close tag
                    let line = [&ident_str, "</", &el_name, ">"];

                    result.push(line.concat());
                }
                ChildMode::Text(text) => {
                    let escaped_text =
                        if ["script", "style"].contains(&element.name.to_lowercase().as_str()) {
                            Cow::from(text)
                        } else {
                            encode_safe(&text)
                        };

                    let line = [
                        //open tag
                        &ident_str,
                        "<",
                        &el_name,
                        &attrs,
                        ">",
                        // content
                        &escaped_text,
                        //close tag
                        "</",
                        &el_name,
                        ">",
                    ];

                    result.push(line.concat());
                }
            }
        }
        HtmlNode::Text(text) => {
            result.push(format!("{ident_str}{}", encode_safe(&text)));
        }
        HtmlNode::Comment(comment) => {
            result.push(format!("{ident_str}<!--{}-->", encode_safe(&comment)));
        }
    }
}


fn is_self_closing(element: &HtmlElement) -> bool {
    let tags = [
        "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "param",
        "source", "track", "wbr",
    ];

    tags.contains(&element.name.as_str())
}


fn attributes_to_string(attr: BTreeMap<String, String>) -> String {
    let mut line = Vec::new();
    for (name, value) in attr.iter() {
        line.push(format!(
            " {}=\"{}\"",
            encode_safe(&name),
            encode_quoted_attribute(&value)
        ));
    }
    line.concat()
}


fn get_render_child_mode(element: VecDeque<HtmlNode>) -> ChildMode {
    let mut result: Vec<HtmlNode> = Vec::new();
    let mut last_text: Option<Vec<String>> = None;

    for child in element {
        match child {
            HtmlNode::Text(child_text) => {
                last_text_add(&mut last_text, child_text);
            }
            HtmlNode::Comment(_) => {}
            element => {
                if let Some(text) = last_text_get(&mut last_text) {
                    result.push(HtmlNode::Text(text));
                }

                result.push(element);
            }
        };
    }

    if let Some(text) = last_text_get(&mut last_text) {
        result.push(HtmlNode::Text(text));
    }

    let last = result.pop();

    let Some(last) = last else {
        return ChildMode::Child(vec![]);
    };

    if result.is_empty() {
        if let HtmlNode::Text(last) = last {
            return ChildMode::Text(last);
        }
    }

    result.push(last);
    ChildMode::Child(result)
}


fn last_text_add(last_text: &mut Option<Vec<String>>, text: String) {
    if let Some(last_text) = last_text {
        last_text.push(text);
        return;
    }

    *last_text = Some(vec![text]);
}

fn last_text_get(last_text: &mut Option<Vec<String>>) -> Option<String> {
    let prev = std::mem::take(last_text);
    prev.map(|inner| inner.concat())
}
