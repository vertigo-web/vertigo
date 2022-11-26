use std::collections::HashMap;

use html_query_parser::{Node, Htmlifiable};
use super::{DomCommand, element::AllElements, replace_html};

pub struct HtmlResponse {
    //TODO - information on the number of pending requests
    all_elements: AllElements,
}

impl HtmlResponse {
    pub fn new() -> Self {
        Self {
            all_elements: AllElements::new()
        }
    }

    pub fn feed(&mut self, commands: Vec<DomCommand>) {
        self.all_elements.feed(commands);
    }

    pub fn result(self, index: &[Node]) -> String {
        let content = self.all_elements.get_response_nodes(false);

        let get_content = move || -> Vec<Node> {
            content.clone()
        };

        replace_html(index, &test_node, &get_content).html()
    }

    pub fn waiting_request(&self) -> u32 {
        //Todo - information on the number of pending requests
        0
    }
}

fn test_node(_: &str, attrs: &HashMap<String, String>) -> bool {
    attrs.contains_key("data-vertigo-run-wasm")
}
