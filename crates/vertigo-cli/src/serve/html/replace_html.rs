use super::html_element::{HtmlNode, HtmlElement, HtmlDocument};

fn process_node(node: &HtmlNode, test_node: &impl Fn(&HtmlElement) -> bool, get_content: &impl Fn() -> Vec<HtmlNode>) -> HtmlNode {
    match node {
        HtmlNode::Element(element) => {
            let new_children = if test_node(element) {
                get_content()
            } else {
                process_node_list(&element.children, test_node, get_content)
            };

            HtmlElement::from(element.name.clone(), element.attr.clone(), new_children).into()
        },
        rest => rest.clone(),
    }
}

fn process_node_list(nodes: &[HtmlNode], test_node: &impl Fn(&HtmlElement) -> bool, get_content: &impl Fn() -> Vec<HtmlNode>) -> Vec<HtmlNode> {
    let mut new_children = Vec::new();

    for child in nodes {
        new_children.push(process_node(child, test_node, get_content));
    }

    new_children
}

pub fn replace_html(input: &HtmlDocument, test_node: &impl Fn(&HtmlElement) -> bool, get_content: &impl Fn() -> Vec<HtmlNode>) -> HtmlDocument {
    let elements = process_node_list(
        input.elements.as_slice(),
        test_node,
        get_content
    );

    HtmlDocument {
        elements
    }
}

#[cfg(test)] 
mod tests {
    use crate::serve::html::html_element::HtmlElement;
    use crate::serve::html::html_element::HtmlNode;
    use crate::serve::html::parse_html::parse_html;

    use super::replace_html;

    #[test]
    fn test_parser() {

        let document = parse_html(r#"<!doctype html><html><head></head><body><div>header</div><div data-type="content">init text</div><div>footer</div></body></html>"#);

        fn test_node(node: &HtmlElement) -> bool {
            node.name == "div" && node.attr.get("data-type") == Some(&"content".to_string())
        }

        fn get_content() -> Vec<HtmlNode> {
            let span = HtmlElement::new("span").child(HtmlNode::Text("Lorem ipsum ...".to_string()));

            let span = HtmlNode::Element(span);
            vec!(span)
        }

        let new_html = replace_html(&document, &test_node, &get_content).convert_to_string(false);

        let expect_html = r#"<!DOCTYPE html><html><head></head><body><div>header</div><div data-type="content"><span>Lorem ipsum ...</span></div><div>footer</div></body></html>"#;

        assert_eq!(new_html, expect_html);
    }
}
