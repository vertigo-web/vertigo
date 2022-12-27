use html_query_parser::Node;

use super::{html_element::{HtmlNode, HtmlElement, HtmlDocument}, ordered_map::OrderedMap};

fn map_node(node: Node) -> Option<HtmlNode> {
    match node {
        Node::Comment(comment) => {
            if comment.trim() == "" {
                None
            } else {
                Some(HtmlNode::Comment(comment))
            }
        }
        Node::Text(text) => {
            if text.trim() == "" {
                None
            } else {
                Some(HtmlNode::Text(text))
            }
        }
        Node::Doctype => {
            None
        }
        Node::Element { name, attrs, children } => {
            let children = children
                .into_iter()
                .filter_map(map_node)
                .collect::<Vec<_>>();

            let mut attrs_ordered = OrderedMap::new();

            for (key, value) in attrs.into_iter() {
                let key = key.trim();

                if !key.is_empty() {
                    attrs_ordered.set(key, value.trim());
                }
            }

            Some(HtmlNode::Element(HtmlElement::from(name.trim(), attrs_ordered, children)))
        }
    }
}

pub fn parse_html(html: &str) -> HtmlDocument {
    let nodes = html_query_parser::parse(html);

    let elements = nodes
        .into_iter()
        .filter_map(map_node)
        .collect::<Vec<_>>();

    HtmlDocument {
        elements
    }
}
