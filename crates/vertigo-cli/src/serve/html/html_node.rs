use super::html_element::HtmlElement;
use super::html_node_convert_to_string::convert_to_string;

#[derive(Clone)]
pub enum HtmlNode {
    Element(HtmlElement),
    Text(String),
    Comment(String),
}

impl HtmlNode {
    pub(super) fn modify(
        &mut self,
        path: &[(&str, usize)],
        callback: impl FnOnce(&mut HtmlElement),
    ) -> bool {
        match self {
            Self::Element(element) => element.modify(path, callback),
            _ => false,
        }
    }

    pub fn get_element(&mut self) -> Option<&mut HtmlElement> {
        match self {
            Self::Element(element) => Some(element),
            _ => None,
        }
    }

    pub fn convert_to_string(self, format: bool) -> String {
        convert_to_string(self, format)
    }
}

impl From<HtmlElement> for HtmlNode {
    fn from(value: HtmlElement) -> Self {
        HtmlNode::Element(value)
    }
}
