use super::ordered_map::OrderedMap;

#[derive(Clone, Copy)]
struct Ident {
    value: Option<usize>,
}

impl Ident {
    fn new() -> Ident {
        Ident {
            value: Some(0),
        }
    }

    fn empty() -> Ident {
        Ident {
            value: None,
        }
    }

    fn get(&self) -> String {
        match self.value {
            Some(ident) => " ".repeat(ident),
            None => String::new()
        }
    }

    fn add(&self, up_value: usize) -> Self {
        Self {
            value: self.value.map(|value| value + up_value)
        }
    }
}

pub struct HtmlDocument {
    pub elements: Vec<HtmlNode>,
}

impl HtmlDocument {
    pub fn convert_to_string(self, format: bool) -> String {

        let mut result = vec!("<!DOCTYPE html>".to_owned());
        let root_ident = match format {
            true => Ident::new(),
            false => Ident::empty(),
        };

        for node in self.elements {
            html_node_to_string(&mut result, root_ident, node);
        }

        match format {
            true => result.join("\n"),
            false => result.concat()
        }
    }
}

fn escape(text: String) -> String {
    let mut result = Vec::<char>::with_capacity(text.len());

    for char in text.chars() {
        match char {
            '<' => {
                result.extend("&lt;".chars());
            },
            '>' => {
                result.extend("&gt;".chars());
            },
            '"' => {
                result.extend("&quot;".chars());
            },
            '\'' => {
                result.extend("&apos;".chars());
            },
            '&' => {
                result.extend("&amp;".chars());
            },
            other => {
                result.push(other);
            }
        }
    }

    result.into_iter().collect::<String>()
}

fn attribute_to_string(line: &mut Vec<String>, attr: OrderedMap) {
    for (name, value) in attr.get_iter() {
        line.push(format!(" {}=\"{}\"", escape(name), escape(value)));
    }
}

enum ChildMode {
    Child(Vec<HtmlNode>),
    Text(String),
}

fn last_text_add(last_text: &mut Option<Vec<String>>, text: String) {
    if let Some(last_text) = last_text {
        last_text.push(text);
        return;
    }

    *last_text = Some(vec!(text));
}

fn last_text_get(last_text: &mut Option<Vec<String>>) -> Option<String> {
    let prev = std::mem::take(last_text);
    prev.map(|inner| inner.concat())
}


fn get_render_child_mode(element: Vec<HtmlNode>) -> ChildMode {
    let mut result: Vec<HtmlNode> = Vec::new();
    let mut last_text: Option<Vec<String>> = None;

    for child in element {
        match child {
            HtmlNode::Text(child_text) => {
                last_text_add(&mut last_text, child_text);
            },
            HtmlNode::Comment(_) => {},
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
        return ChildMode::Child(vec!());
    };

    if result.is_empty() {
        if let HtmlNode::Text(last) = last {
            return ChildMode::Text(last);
        }
    }

    result.push(last);
    ChildMode::Child(result)
}

fn is_self_closing(element: &HtmlElement) -> bool {
    let tags = [
        "area",
        "base",
        "br",
        "col",
        "embed",
        "hr",
        "img",
        "input",
        "link",
        "meta", 
        "param",
        "source",
        "track",
        "wbr"
    ];

    tags.contains(&element.name.as_str())
}

fn html_node_to_string(result: &mut Vec<String>, ident: Ident, node: HtmlNode) {
    let ident_str = ident.get();

    match node {
        HtmlNode::Element(element) => {
            if is_self_closing(&element) {
                let mut line = Vec::new();
                line.push(ident_str);
                line.push("<".into());
                line.push(escape(element.name));
                attribute_to_string(&mut line, element.attr);
                line.push(" />".into());

                result.push(line.concat());
                return;
            }

            match get_render_child_mode(element.children) {
                ChildMode::Child(children) => {

                    //open tag
                    let mut line = Vec::new();
                    line.push(ident_str.clone());
                    line.push("<".into());
                    line.push(escape(element.name.clone()));
                    attribute_to_string(&mut line, element.attr);
                    line.push(">".into());

                    result.push(line.concat());

                    //render child
                    for child in children {
                        html_node_to_string(result, ident.add(2), child);
                    }

                    //close tag
                    let line = vec!(
                        ident_str,
                        "</".into(),
                        escape(element.name),
                        ">".into()
                    );

                    result.push(line.concat());
                },
                ChildMode::Text(text) => {
                    //open tag
                    let mut line = Vec::new();
                    line.push(ident_str);
                    line.push("<".into());
                    line.push(escape(element.name.clone()));
                    attribute_to_string(&mut line, element.attr);
                    line.push(">".into());

                    if element.name.to_lowercase() == "script" {
                        line.push(text);
                    } else {
                        line.push(escape(text));
                    }

                    //close tag
                    line.push("</".into());
                    line.push(escape(element.name));
                    line.push(">".into());

                    result.push(line.concat());
                }
            }
        },
        HtmlNode::Text(text) => {
            result.push(format!("{ident_str}{}", escape(text)));
        },
        HtmlNode::Comment(comment) => {
            result.push(format!("{ident_str}<!--{}-->", escape(comment)));
        }
    }
}

#[derive(Clone)]
pub enum HtmlNode {
    Element(HtmlElement),
    Text(String),
    Comment(String),
}

impl From<HtmlElement> for HtmlNode {
    fn from(value: HtmlElement) -> Self {
        HtmlNode::Element(value)
    }
}

#[derive(Clone)]
pub struct HtmlElement {
    pub name: String,
    pub attr: OrderedMap,
    pub children: Vec<HtmlNode>,
}

impl HtmlElement {
    pub fn new(name: impl Into<String>) -> Self {
        let name = name.into();

        HtmlElement {
            name,
            attr: OrderedMap::new(),
            children: Vec::new(),
        }
    }

    pub fn child(mut self, child: HtmlNode) -> Self {
        self.children.push(child);
        self
    }

    pub fn from(name: impl Into<String>, attr: OrderedMap, children: Vec<HtmlNode>) -> Self {
        HtmlElement {
            name: name.into(),
            attr,
            children
        }
    }
}
