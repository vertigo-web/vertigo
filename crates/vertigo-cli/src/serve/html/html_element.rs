use std::collections::{BTreeMap, VecDeque};
use super::HtmlNode;

#[derive(Clone)]
pub struct HtmlElement {
    pub name: String,
    pub attr: BTreeMap<String, String>,
    pub children: VecDeque<HtmlNode>,
}

impl HtmlElement {
    pub fn new(name: impl Into<String>) -> Self {
        let name = name.into();

        HtmlElement {
            name,
            attr: BTreeMap::new(),
            children: VecDeque::new(),
        }
    }

    pub fn attr(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.attr.insert(name.into(), value.into());
        self
    }

    pub fn add_attr(&mut self, name: impl Into<String>, value: impl Into<String>) {
        self.attr.insert(name.into(), value.into());
    }

    pub fn add_child(&mut self, child: impl Into<HtmlNode>) {
        let child = child.into();
        self.children.push_back(child);
    }

    #[cfg(test)]
    pub fn child(mut self, child: HtmlNode) -> Self {
        self.children.push_back(child);
        self
    }

    pub fn from(name: impl Into<String>, attr: BTreeMap<String, String>, children: VecDeque<HtmlNode>) -> Self {
        HtmlElement {
            name: name.into(),
            attr,
            children,
        }
    }

    pub(super) fn modify(&mut self, path: &[(&str, usize)], callback: impl FnOnce(&mut HtmlElement)) -> bool {
        if path.is_empty() {
            callback(self);
            return true;
        }

        if let Some(((tag_name, index), rest_path)) = path.split_first() {
            let mut current: usize = 0;

            for node in self.children.iter_mut() {
                if let Some(element) = node.get_element() {
                    if element.name.as_str() == *tag_name {
                        if &current == index {
                            return element.modify(rest_path, callback);
                        }
                        current += 1;
                    }
                }
            }
        }

        false
    }
}
