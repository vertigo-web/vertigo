use std::collections::{BTreeMap, HashMap, VecDeque};

use vertigo::{DomId, dev::command::DriverDomCommand};

use super::{HtmlNode, element_children::ElementChildren, html_element::HtmlElement};

pub enum Node {
    Element(Element),
    Text(String),
    Comment(String),
}

pub struct Element {
    name: String,
    attr: BTreeMap<String, String>,
    children: ElementChildren,
}

impl Element {
    pub fn new(name: impl Into<String>) -> Self {
        let name = name.into();

        Element {
            name,
            attr: BTreeMap::new(),
            children: ElementChildren::new(),
        }
    }
}

pub struct AllElements {
    parent: HashMap<DomId, DomId>,
    all: HashMap<DomId, Node>,
    css: Vec<(Option<String>, String)>,
}

impl AllElements {
    pub fn new() -> AllElements {
        Self {
            parent: HashMap::new(),
            all: HashMap::new(),
            css: Vec::new(),
        }
    }

    fn create_node(&mut self, id: DomId, name: impl Into<String>) {
        let element = Node::Element(Element::new(name));
        self.all.insert(id, element);
    }

    fn create_text(&mut self, id: DomId, value: impl Into<String>) {
        let value = value.into();
        let element = Node::Text(value);
        self.all.insert(id, element);
    }

    fn update_text(&mut self, id: DomId, value: impl Into<String>) {
        self.create_text(id, value);
    }

    fn set_attr(&mut self, id: DomId, name: impl Into<String>, value: impl Into<String>) {
        if let Some(Node::Element(element)) = self.all.get_mut(&id) {
            element.attr.insert(name.into(), value.into());
        } else {
            log::error!(
                "Tried to set attr `{}` for non-existent element `{id}`",
                name.into()
            );
        }
    }

    fn remove_attr(&mut self, id: DomId, name: impl AsRef<str>) {
        if let Some(Node::Element(element)) = self.all.get_mut(&id) {
            element.attr.remove(name.as_ref());
        } else {
            log::error!(
                "Tried to remove non-existent attr `{}` from element `{id}`",
                name.as_ref()
            );
        }
    }

    fn remove_from_parent(&mut self, child_id: DomId) {
        let Some(parent_id) = self.parent.get(&child_id) else {
            return;
        };

        let Some(Node::Element(parent_node)) = self.all.get_mut(parent_id) else {
            return;
        };

        parent_node.children.remove(child_id);
    }

    fn remove(&mut self, child_id: DomId) {
        self.remove_from_parent(child_id);
        self.parent.remove(&child_id);

        self.all.remove(&child_id);
    }

    fn insert_before(&mut self, parent_id: DomId, ref_id: Option<DomId>, child_id: DomId) {
        self.remove_from_parent(child_id);
        self.parent.insert(child_id, parent_id);

        if let Some(Node::Element(parent_node)) = self.all.get_mut(&parent_id) {
            parent_node.children.insert_before(ref_id, child_id)
        } else {
            log::error!("Can't find parent `{parent_id}` for inserting child `{child_id}`");
        }
    }

    fn insert_css(&mut self, selector: Option<String>, value: String) {
        self.css.push((selector, value));
    }

    fn create_comment(&mut self, id: DomId, value: String) {
        let element = Node::Comment(value);
        self.all.insert(id, element);
    }

    pub fn feed(&mut self, commands: Vec<DriverDomCommand>) {
        for node in commands {
            match node {
                DriverDomCommand::CallbackAdd { .. } => {
                    // ignored on server-side
                }
                DriverDomCommand::CallbackRemove { .. } => {
                    // ignored on server-side
                }
                DriverDomCommand::CreateNode { id, name } => {
                    self.create_node(id, name.to_string());
                }
                DriverDomCommand::CreateText { id, value } => {
                    self.create_text(id, value);
                }
                DriverDomCommand::UpdateText { id, value } => {
                    self.update_text(id, value);
                }
                DriverDomCommand::RemoveText { id } => {
                    self.remove(id);
                }
                DriverDomCommand::SetAttr { id, name, value } => {
                    self.set_attr(id, name.to_string(), value);
                }
                DriverDomCommand::RemoveAttr { id, name } => {
                    self.remove_attr(id, name.to_string());
                }
                DriverDomCommand::RemoveNode { id } => {
                    self.remove(id);
                }
                DriverDomCommand::InsertBefore {
                    parent,
                    ref_id,
                    child,
                } => self.insert_before(parent, ref_id, child),
                DriverDomCommand::InsertCss { selector, value } => {
                    self.insert_css(selector, value);
                }
                DriverDomCommand::CreateComment { id, value } => {
                    self.create_comment(id, value);
                }
                DriverDomCommand::RemoveComment { id } => {
                    self.remove(id);
                }
            }
        }
    }

    fn get_response_one_elements(&self, node_id: DomId, with_id: bool) -> HtmlNode {
        let Some(node) = self.all.get(&node_id) else {
            return HtmlNode::Comment("No <html> element".to_string());
        };

        match node {
            Node::Element(node) => {
                let mut attr = node.attr.clone();

                if with_id {
                    attr.insert("data-id".into(), node_id.to_u64().to_string());
                }

                let children = self.get_response_elements(node, with_id);
                HtmlElement::from(node.name.clone(), attr, children).into()
            }
            Node::Text(text) => HtmlNode::Text(text.clone()),
            Node::Comment(comment) => HtmlNode::Comment(comment.clone()),
        }
    }

    fn get_response_elements(&self, element: &Element, with_id: bool) -> VecDeque<HtmlNode> {
        let mut result = VecDeque::new();

        for child_id in element.children.get_all() {
            result.push_back(self.get_response_one_elements(child_id, with_id));
        }

        result
    }

    fn get_css(&self) -> HtmlNode {
        let mut style = HtmlElement::new("style");

        for (prop, value) in self.css.iter() {
            let content = if let Some(prop) = prop {
                // Autocss rule
                [prop.as_str(), " { ", value.as_str(), " }\n"].concat()
            } else {
                // Bundle (i.e. a tailwind bundle)
                value.clone()
            };

            style.add_child(HtmlNode::Text(content));
        }

        style.into()
    }

    // Root and css
    pub fn get_response(&self, with_id: bool) -> (HtmlNode, HtmlNode) {
        let root_html = self.get_response_one_elements(DomId::from_u64(1), with_id);
        let css = self.get_css();
        (root_html, css)
    }

    #[cfg(test)]
    pub fn get_response_document(&self, with_id: bool) -> (HtmlNode, HtmlNode) {
        let (root, css) = self.get_response(with_id);
        (root, css)
    }
}

#[cfg(test)]
mod tests {
    use vertigo::{DomId, dev::command::DriverDomCommand};

    use super::AllElements;

    #[test]
    fn test_html() {
        fn create_element(all: &mut AllElements, id: DomId, name: impl Into<String>) {
            let name = name.into().into();

            let commands = vec![DriverDomCommand::CreateNode { id, name }];
            all.feed(commands);
        }

        fn insert_before(
            all: &mut AllElements,
            parent_id: DomId,
            id: DomId,
            ref_id: Option<DomId>,
        ) {
            all.feed(vec![DriverDomCommand::InsertBefore {
                parent: parent_id,
                child: id,
                ref_id,
            }])
        }

        fn create_and_add(
            all: &mut AllElements,
            parent_id: DomId,
            id: DomId,
            name: impl Into<String>,
        ) {
            create_element(all, id, name);
            insert_before(all, parent_id, id, None);
        }

        fn add_to_parent(all: &mut AllElements, parent: DomId, child: DomId) {
            let commands = vec![DriverDomCommand::InsertBefore {
                parent,
                child,
                ref_id: None,
            }];
            all.feed(commands);
        }

        let mut all = AllElements::new();

        create_element(&mut all, DomId::from_u64(1), "div");
        create_and_add(&mut all, DomId::from_u64(1), DomId::from_u64(2), "div");
        let (document, _) = all.get_response_document(true);
        assert_eq!(
            document.convert_to_string(false),
            r#"<!DOCTYPE html><div data-id="1"><div data-id="2"></div></div>"#
        );

        create_and_add(&mut all, DomId::from_u64(1), DomId::from_u64(3), "div");
        let (document, _) = all.get_response_document(true);
        assert_eq!(
            document.convert_to_string(false),
            r#"<!DOCTYPE html><div data-id="1"><div data-id="2"></div><div data-id="3"></div></div>"#
        );

        create_and_add(&mut all, DomId::from_u64(3), DomId::from_u64(4), "span");
        let (document, _) = all.get_response_document(true);
        assert_eq!(
            document.convert_to_string(false),
            r#"<!DOCTYPE html><div data-id="1"><div data-id="2"></div><div data-id="3"><span data-id="4"></span></div></div>"#
        );

        add_to_parent(&mut all, DomId::from_u64(2), DomId::from_u64(4));
        let (document, _) = all.get_response_document(true);
        assert_eq!(
            document.convert_to_string(false),
            r#"<!DOCTYPE html><div data-id="1"><div data-id="2"><span data-id="4"></span></div><div data-id="3"></div></div>"#
        );

        create_element(&mut all, DomId::from_u64(5), "br");
        insert_before(
            &mut all,
            DomId::from_u64(1),
            DomId::from_u64(5),
            Some(DomId::from_u64(2)),
        );
        let (document, _) = all.get_response_document(true);
        assert_eq!(
            document.convert_to_string(false),
            r#"<!DOCTYPE html><div data-id="1"><br data-id="5" /><div data-id="2"><span data-id="4"></span></div><div data-id="3"></div></div>"#
        );
    }
}
