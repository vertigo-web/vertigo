use super::{html_element::HtmlElement, HtmlNode};
use html_escape::{encode_quoted_attribute, encode_safe};
use std::collections::VecDeque;
use std::{borrow::Cow, collections::BTreeMap};

enum ChildMode {
    Child(Vec<HtmlNode>),
    Text(String),
}

pub fn convert_to_string(root: HtmlNode, pretty: bool) -> String {
    let mut result = vec!["<!DOCTYPE html>".to_owned()];

    if pretty {
        result.push("\n".to_string())
    }

    let root_ident = match pretty {
        true => Format::some(),
        false => Format::none(),
    };

    html_node_to_string(&mut result, root_ident, root);

    result.concat()
}

fn html_node_to_string(result: &mut Vec<String>, ident: Format, node: HtmlNode) {
    let mut ident_str = ident.get();

    match node {
        HtmlNode::Element(element) => {
            let is_self_closing = is_self_closing(&element);
            let el_name = encode_safe(&element.name);
            let attrs = attributes_to_string(element.attr);

            let l_chevron_str = ident.l_chevron();
            let l_chevron = l_chevron_str.as_str();

            if is_self_closing {
                let line = [l_chevron, &el_name, &attrs, ident.self_closing()];

                result.push(line.concat());
                return;
            }

            let r_chevron = ident.r_chevron();

            match get_render_child_mode(element.children) {
                ChildMode::Child(children) => {
                    // Treat <pre> separately if formatting is enabled
                    let inner_ident = if ident.is_some() && el_name != "pre" {
                        ident.add(2)
                    } else {
                        Format::none()
                    };

                    // open tag
                    let line = [l_chevron, &el_name, &attrs, inner_ident.r_chevron()];
                    result.push(line.concat());

                    // render child
                    for child in children {
                        html_node_to_string(result, inner_ident, child);
                    }

                    // If </pre> then do not ident
                    if el_name == "pre" {
                        ident_str = String::new();
                    }

                    // close tag
                    let line = [&ident_str, "</", &el_name, r_chevron];
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
                        l_chevron,
                        &el_name,
                        &attrs,
                        r_chevron,
                        // content
                        &escaped_text,
                        //close tag
                        "</",
                        &el_name,
                        r_chevron,
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

const SELF_CLOSING_TAGS: [&str; 14] = [
    "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "param", "source",
    "track", "wbr",
];

fn is_self_closing(element: &HtmlElement) -> bool {
    SELF_CLOSING_TAGS.contains(&element.name.as_str())
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

#[derive(Clone, Copy)]
struct Format {
    ident: Option<usize>,
}

impl Format {
    fn some() -> Format {
        Format { ident: Some(0) }
    }

    fn none() -> Format {
        Format { ident: None }
    }

    fn get(&self) -> String {
        match self.ident {
            Some(ident) => " ".repeat(ident),
            None => String::new(),
        }
    }

    fn add(&self, up_value: usize) -> Self {
        Self {
            ident: self.ident.map(|value| value + up_value),
        }
    }

    fn is_some(&self) -> bool {
        self.ident.is_some()
    }

    fn l_chevron(&self) -> String {
        if self.is_some() {
            [&self.get(), "<"].concat()
        } else {
            "<".to_string()
        }
    }

    fn r_chevron(&self) -> &'static str {
        if self.is_some() {
            ">\n"
        } else {
            ">"
        }
    }

    fn self_closing(&self) -> &'static str {
        if self.is_some() {
            " />\n"
        } else {
            " />"
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::serve::html::{html_element::HtmlElement, HtmlNode};

    use super::convert_to_string;

    #[test]
    fn html_pre_formatting() {
        let div: HtmlNode = HtmlElement::new("div")
            .child(
                HtmlElement::new("pre")
                    .child(
                        HtmlElement::new("span")
                            .child(HtmlNode::Text("    ".into()))
                            .into(),
                    )
                    .child(
                        HtmlElement::new("span")
                            .child(HtmlNode::Text("let".into()))
                            .into(),
                    )
                    .child(
                        HtmlElement::new("span")
                            .child(HtmlNode::Text(" ".into()))
                            .into(),
                    )
                    .child(
                        HtmlElement::new("span")
                            .child(HtmlNode::Text("x".into()))
                            .into(),
                    )
                    .child(
                        HtmlElement::new("span")
                            .child(HtmlNode::Text(" ".into()))
                            .into(),
                    )
                    .child(
                        HtmlElement::new("span")
                            .child(HtmlNode::Text(";".into()))
                            .into(),
                    )
                    .child(
                        HtmlElement::new("span")
                            .child(HtmlNode::Text("\n".into()))
                            .into(),
                    )
                    .into(),
            )
            .child(HtmlElement::new("img").into())
            .into();

        let output = convert_to_string(div.clone(), true);

        assert_eq!(
            output,
            "<!DOCTYPE html>
<div>
  <pre><span>    </span><span>let</span><span> </span><span>x</span><span> </span><span>;</span><span>\n</span></pre>
  <img />
</div>
"
        );

        let output = convert_to_string(div, false);

        assert_eq!(
            output,
            "<!DOCTYPE html><div><pre><span>    </span><span>let</span><span> </span><span>x</span><span> </span><span>;</span><span>\n</span></pre><img /></div>"
        );
    }
}
