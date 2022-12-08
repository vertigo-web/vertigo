use std::collections::HashMap;
use html_query_parser::Node;

fn process_node(node: &Node, test_node: &impl Fn(&str, &HashMap<String, String>) -> bool, get_content: &impl Fn() -> Vec<Node>) -> Node {
    match node {
        Node::Element { name, attrs, children } => {
            if test_node(name.trim(), attrs) {
                let list_attrs = attrs
                    .iter()
                    .map(|(key, value)| (key.as_str(), value.as_str()))
                    .collect::<Vec<_>>();

                return Node::new_element(name, list_attrs, get_content());
            }

            let new_children = process_node_list(children, test_node, get_content);

            Node::Element { name: name.to_string(), attrs: attrs.clone(), children: new_children }
        },
        rest => rest.clone(),
    }
}

fn process_node_list(nodes: &[Node], test_node: &impl Fn(&str, &HashMap<String, String>) -> bool, get_content: &impl Fn() -> Vec<Node>) -> Vec<Node> {
    let mut new_children = Vec::new();

    for child in nodes {
        new_children.push(process_node(child, test_node, get_content));
    }

    new_children
}

pub fn replace_html(input: &[Node], test_node: &impl Fn(&str, &HashMap<String, String>) -> bool, get_content: &impl Fn() -> Vec<Node>) -> Vec<Node> {
    process_node_list(input, test_node, get_content)
}


#[cfg(test)] 
mod tests {
    use std::collections::HashMap;
    use html_query_parser::{Node, Htmlifiable};
    use super::replace_html;

    #[test]
    fn test_parser() {
        

        let document = html_query_parser::parse(r#"<!doctype html>
    <html>
        <head></head>
        <body>
            <div>
                header
            </div>
            <div data-type="content">
                init text
            </div>
            <div>
                footer
            </div>
        </body>
    </html>
    "#);

        fn test_node(name: &str, attrs: &HashMap<String, String>) -> bool {
            name.trim() == "div" && attrs.get("data-type") == Some(&"content".to_string())
        }

        fn get_content() -> Vec<Node> {
            let inner = Node::new_element("span", Vec::new(), vec!(
                Node::Text("Lorem ipsum ...".to_string()),
            ));
        
            vec!(inner)
        }

        let document = replace_html(document.as_slice(), &test_node, &get_content);

        let new_html = document.html();

        let expect_html = r#"<!DOCTYPE html>
    <html>
        <head></head>
        <body>
            <div>
                header
            </div>
            <div data-type="content"><span>Lorem ipsum ...</span></div>
            <div>
                footer
            </div>
        </body>
    </html>"#;

        assert_eq!(new_html, expect_html);
    }
}
