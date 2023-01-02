use std::collections::HashMap;

use super::{ordered_map::OrderedMap, element_children::ElementChildren, DomCommand, HtmlNode, html_element::HtmlElement};

pub enum Node {
    Element(Element),
    Text(String),
    Comment(String),
}

pub struct Element {
    name: String,
    attr: OrderedMap,
    children: ElementChildren,
}

impl Element {
    pub fn new(name: impl Into<String>) -> Self {
        let name = name.into();

        Element {
            name,
            attr: OrderedMap::new(),
            children: ElementChildren::new(),
        }
    }
}

pub struct AllElements {
    parent: HashMap<u64, u64>,
    all: HashMap<u64, Node>,
    css: OrderedMap,
}

impl AllElements {
    pub fn new() -> AllElements {
        let mut inst = Self {
            parent: HashMap::new(),
            all: HashMap::new(),
            css: OrderedMap::new(),
        };

        inst.create_node(1, "div");
        inst
    }

    fn create_node(&mut self, id: u64, name: impl Into<String>) {
        let element = Node::Element(Element::new(name));
        self.all.insert(id, element);
    }

    fn create_text(&mut self, id: u64, value: impl Into<String>) {
        let value = value.into();
        let element = Node::Text(value);
        self.all.insert(id, element);
    }

    fn update_text(&mut self, id: u64, value: impl Into<String>) {
        self.create_text(id, value);
    }

    fn set_attr(&mut self, id: u64, name: impl Into<String>, value: impl Into<String>) {
        let Some(Node::Element(element)) = self.all.get_mut(&id) else {
            unreachable!();
        };

        element.attr.set(name, value);
    }

    fn remove_from_parent(&mut self, child_id: u64) {
        let Some(parent_id) = self.parent.get(&child_id) else {
            return;
        };

        let Some(Node::Element(parent_node)) = self.all.get_mut(parent_id) else {
            return;
        };

        parent_node.children.remove(child_id);
    }

    fn remove(&mut self, child_id: u64) {
        self.remove_from_parent(child_id);
        self.parent.remove(&child_id);

        self.all.remove(&child_id);
    }

    fn insert_before(&mut self, parent_id: u64, ref_id: Option<u64>, child_id: u64) {
        self.remove_from_parent(child_id);
        self.parent.insert(child_id, parent_id);

        let Some(Node::Element(parent_node)) = self.all.get_mut(&parent_id) else {
            unreachable!();
        };

        parent_node.children.insert_before(ref_id, child_id);
    }

    fn insert_css(&mut self, selector: String, value: String) {
        self.css.set(selector, value);
    }

    fn create_comment(&mut self, id: u64, value: String) {
        let element = Node::Comment(value);
        self.all.insert(id, element);
    }

    pub fn feed(&mut self, commands: Vec<DomCommand>) {


        for node in commands {
            match node {
                DomCommand::CallbackAdd { .. } => {
                    //ignore
                }
                DomCommand::CallbackRemove { .. } => {
                    //ignore
                }
                DomCommand::CreateNode { id, name } => {
                    self.create_node(id, name);
                }
                DomCommand::CreateText { id, value } => {
                    self.create_text(id, value);
                }
                DomCommand::UpdateText { id, value } => {
                    self.update_text(id, value);
                }
                DomCommand::RemoveText { id } => {
                    self.remove(id);
                }
                DomCommand::SetAttr { id, name, value } => {
                    self.set_attr(id, name, value);
                }
                DomCommand::RemoveNode { id, } => {
                    self.remove(id);
                }
                DomCommand::InsertBefore { parent, ref_id, child } => {
                    self.insert_before(parent, ref_id, child)
                }
                DomCommand::InsertCss { selector, value } => {
                    self.insert_css(selector, value);
                }
                DomCommand::CreateComment { id, value } => {
                    self.create_comment(id, value);
                }
                DomCommand::RemoveComment { id } => {
                    self.remove(id);
                }
            }
        }
    }

    fn get_response_one_elements(&self, node_id: u64, with_id: bool) -> HtmlNode {
        let Some(node) = self.all.get(&node_id) else {
            unreachable!();
        };

        match node {
            Node::Element(node) => {
                let mut attr = node.attr.clone();

                if with_id {
                    attr.set("data-id", node_id.to_string());
                }

                let children = self.get_response_elements(node, with_id);
                HtmlElement::from(node.name.clone(), attr, children).into()
            },
            Node::Text(text) => {
                HtmlNode::Text(text.clone())
            },
            Node::Comment(comment) => {
                HtmlNode::Comment(comment.clone())
            },
        }
    }

    fn get_response_elements(&self, element: &Element, with_id: bool) -> Vec<HtmlNode> {
        let mut result = Vec::new();

        for child_id in element.children.childs() {
            result.push(self.get_response_one_elements(child_id, with_id));
        }

        result
    }

    fn get_css(&self) -> Vec<HtmlNode> {
        let mut result = Vec::new();

        for (prop, value) in self.css.get_iter() {
            let content = [prop.as_str(), " { ", value.as_str(), " }"].concat();

            result.push(
                HtmlElement::new("style")
                    .child(HtmlNode::Text(content))
                    .into()
            );
        }

        result
    }

    pub fn get_response_nodes(&self, with_id: bool) -> Vec<HtmlNode> {
        let Some(Node::Element(root)) = self.all.get(&1) else {
            unreachable!();
        };

        let mut css = self.get_css();
        css.extend(self.get_response_elements(root, with_id).into_iter());
        css
    }

    #[cfg(test)]
    pub fn get_response(&self, with_id: bool) -> super::HtmlDocument {
        let elements = self.get_response_nodes(with_id);
        super::HtmlDocument {
            elements
        }
    }
}


#[cfg(test)]
mod tests {
    use super::{AllElements, DomCommand};

    #[test]
    fn test_html() {
        fn create_and_add(all: &mut AllElements, parent_id: u64, id: u64, name: impl Into<String>) {
            let name = name.into();

            let commands = vec!(
                DomCommand::CreateNode {
                    id,
                    name,
                },
                DomCommand::InsertBefore {
                    parent: parent_id,
                    child: id,
                    ref_id: None
                },
            );
            all.feed(commands);
        }

        fn add_to_parent(all: &mut AllElements, parent: u64, child: u64) {
            let commands = vec!(
                DomCommand::InsertBefore {
                    parent,
                    child,
                    ref_id: None
                },
            );
            all.feed(commands);
        }

        let mut all = AllElements::new();

        create_and_add(&mut all, 1, 2, "div");
        assert_eq!(all.get_response(true).convert_to_string(false), r#"<!DOCTYPE html><div data-id="2"></div>"#);

        create_and_add(&mut all, 1, 3, "div");
        assert_eq!(all.get_response(true).convert_to_string(false), r#"<!DOCTYPE html><div data-id="2"></div><div data-id="3"></div>"#);

        create_and_add(&mut all, 3, 4, "span");
        assert_eq!(all.get_response(true).convert_to_string(false), r#"<!DOCTYPE html><div data-id="2"></div><div data-id="3"><span data-id="4"></span></div>"#);

        add_to_parent(&mut all, 2, 4);
        assert_eq!(all.get_response(true).convert_to_string(false), r#"<!DOCTYPE html><div data-id="2"><span data-id="4"></span></div><div data-id="3"></div>"#);
    }


    #[test]
    fn test_children() {
        let commands_str = r#"[
            {"id":2,"name":"div","type":"create_node"},
            {"id":3,"name":"ul","type":"create_node"},
            {"id":4,"name":"li","type":"create_node"},
            {"child":4,"parent":3,"ref_id":null,"type":"insert_before"},
            {"id":6,"name":"li","type":"create_node"},
            {"child":6,"parent":3,"ref_id":null,"type":"insert_before"},
            {"id":8,"name":"li","type":"create_node"},
            {"child":8,"parent":3,"ref_id":null,"type":"insert_before"},
            {"id":10,"name":"li","type":"create_node"},
            {"child":10,"parent":3,"ref_id":null,"type":"insert_before"},
            {"id":12,"name":"li","type":"create_node"},
            {"child":12,"parent":3,"ref_id":null,"type":"insert_before"},
            {"id":14,"name":"li","type":"create_node"},
            {"child":14,"parent":3,"ref_id":null,"type":"insert_before"},
            {"id":16,"name":"li","type":"create_node"},
            {"child":16,"parent":3,"ref_id":null,"type":"insert_before"},
            {"id":18,"name":"li","type":"create_node"},
            {"child":18,"parent":3,"ref_id":null,"type":"insert_before"},
            {"id":20,"name":"li","type":"create_node"},
            {"child":20,"parent":3,"ref_id":null,"type":"insert_before"},
            {"child":3,"parent":2,"ref_id":null,"type":"insert_before"},
            {"id":22,"type":"create_comment","value":"value element"},
            {"id":23,"name":"div","type":"create_node"},
            {"child":2,"parent":23,"ref_id":null,"type":"insert_before"},
            {"child":22,"parent":23,"ref_id":null,"type":"insert_before"},
            {"child":22,"parent":23,"ref_id":null,"type":"insert_before"},
            {"id":25,"name":"div","type":"create_node"},
            {"id":26,"name":"div","type":"create_node"},
            {"id":27,"name":"div","type":"create_node"},
            {"child":27,"parent":26,"ref_id":null,"type":"insert_before"},
            {"child":26,"parent":25,"ref_id":null,"type":"insert_before"},
            {"id":28,"name":"button","type":"create_node"},
            {"id":29,"name":"span","type":"create_node"},
            {"child":29,"parent":28,"ref_id":null,"type":"insert_before"},
            {"id":31,"name":"span","type":"create_node"},
            {"child":31,"parent":28,"ref_id":null,"type":"insert_before"},
            {"child":28,"parent":25,"ref_id":null,"type":"insert_before"},
            {"child":25,"parent":23,"ref_id":22,"type":"insert_before"},
            {"child":23,"parent":1,"ref_id":null,"type":"insert_before"}
        ]"#;

        let commands = serde_json::from_str::<Vec<DomCommand>>(commands_str).unwrap();

        let mut all = AllElements::new();
        all.feed(commands);

        let result = all.get_response(true).convert_to_string(false);

        assert_eq!(result, r#"<!DOCTYPE html><div data-id="23"><div data-id="2"><ul data-id="3"><li data-id="4"></li><li data-id="6"></li><li data-id="8"></li><li data-id="10"></li><li data-id="12"></li><li data-id="14"></li><li data-id="16"></li><li data-id="18"></li><li data-id="20"></li></ul></div><div data-id="25"><div data-id="26"><div data-id="27"></div></div><button data-id="28"><span data-id="29"></span><span data-id="31"></span></button></div></div>"#);
    }


    #[test]
    fn test_deserialize() {
        let commands_str = r#"[
            {"id":2,"name":"div","type":"create_node"},
            {"callback_id":2,"event_name":"hook_keydown","id":2,"type":"callback_add"},
            {"id":3,"name":"ul","type":"create_node"},
            {"selector":".autocss_1","type":"insert_css","value":"list-style-type: none; margin: 10px; padding: 0"},
            {"id":3,"name":"class","type":"set_attr","value":"autocss_1"},
            {"id":4,"name":"li","type":"create_node"},
            {"selector":".autocss_2:hover","type":"insert_css","value":"text-decoration: underline;"},
            {"selector":".autocss_2","type":"insert_css","value":"display: inline; width: 60px; padding: 5px 10px; margin: 5px; cursor: pointer; background-color: lightgreen"},
            {"id":4,"name":"class","type":"set_attr","value":"autocss_2"},
            {"callback_id":3,"event_name":"click","id":4,"type":"callback_add"},
            {"id":5,"type":"create_text","value":"Counters"},
            {"child":5,"parent":4,"ref_id":null,"type":"insert_before"},
            {"child":4,"parent":3,"ref_id":null,"type":"insert_before"},
            {"id":6,"name":"li","type":"create_node"},
            {"selector":".autocss_3:hover","type":"insert_css","value":"text-decoration: underline;"},
            {"selector":".autocss_3","type":"insert_css","value":"display: inline; width: 60px; padding: 5px 10px; margin: 5px; cursor: pointer; background-color: lightblue"},
            {"id":6,"name":"class","type":"set_attr","value":"autocss_3"},
            {"callback_id":4,"event_name":"click","id":6,"type":"callback_add"},
            {"id":7,"type":"create_text","value":"Animations"},
            {"child":7,"parent":6,"ref_id":null,"type":"insert_before"},
            {"child":6,"parent":3,"ref_id":null,"type":"insert_before"},
            {"id":8,"name":"li","type":"create_node"},
            {"id":8,"name":"class","type":"set_attr","value":"autocss_2"},
            {"callback_id":5,"event_name":"click","id":8,"type":"callback_add"},
            {"id":9,"type":"create_text","value":"Sudoku"},
            {"child":9,"parent":8,"ref_id":null,"type":"insert_before"},
            {"child":8,"parent":3,"ref_id":null,"type":"insert_before"},
            {"id":10,"name":"li","type":"create_node"},
            {"id":10,"name":"class","type":"set_attr","value":"autocss_2"},
            {"callback_id":6,"event_name":"click","id":10,"type":"callback_add"},
            {"id":11,"type":"create_text","value":"Input"},
            {"child":11,"parent":10,"ref_id":null,"type":"insert_before"},
            {"child":10,"parent":3,"ref_id":null,"type":"insert_before"},
            {"id":12,"name":"li","type":"create_node"},
            {"id":12,"name":"class","type":"set_attr","value":"autocss_2"},
            {"callback_id":7,"event_name":"click","id":12,"type":"callback_add"},
            {"id":13,"type":"create_text","value":"Github Explorer"},
            {"child":13,"parent":12,"ref_id":null,"type":"insert_before"},
            {"child":12,"parent":3,"ref_id":null,"type":"insert_before"},
            {"id":14,"name":"li","type":"create_node"},
            {"id":14,"name":"class","type":"set_attr","value":"autocss_2"},
            {"callback_id":8,"event_name":"click","id":14,"type":"callback_add"},
            {"id":15,"type":"create_text","value":"Game Of Life"},
            {"child":15,"parent":14,"ref_id":null,"type":"insert_before"},
            {"child":14,"parent":3,"ref_id":null,"type":"insert_before"},
            {"id":16,"name":"li","type":"create_node"},
            {"id":16,"name":"class","type":"set_attr","value":"autocss_2"},
            {"callback_id":9,"event_name":"click","id":16,"type":"callback_add"},
            {"id":17,"type":"create_text","value":"Chat"},
            {"child":17,"parent":16,"ref_id":null,"type":"insert_before"},
            {"child":16,"parent":3,"ref_id":null,"type":"insert_before"},
            {"id":18,"name":"li","type":"create_node"},
            {"id":18,"name":"class","type":"set_attr","value":"autocss_2"},
            {"callback_id":10,"event_name":"click","id":18,"type":"callback_add"},
            {"id":19,"type":"create_text","value":"Todo"},
            {"child":19,"parent":18,"ref_id":null,"type":"insert_before"},
            {"child":18,"parent":3,"ref_id":null,"type":"insert_before"},
            {"id":20,"name":"li","type":"create_node"},
            {"id":20,"name":"class","type":"set_attr","value":"autocss_2"},
            {"callback_id":11,"event_name":"click","id":20,"type":"callback_add"},
            {"id":21,"type":"create_text","value":"Drop File"},
            {"child":21,"parent":20,"ref_id":null,"type":"insert_before"},
            {"child":20,"parent":3,"ref_id":null,"type":"insert_before"},
            {"child":3,"parent":2,"ref_id":null,"type":"insert_before"},
            {"id":22,"type":"create_comment","value":"value element"},
            {"id":23,"name":"div","type":"create_node"},
            {"callback_id":12,"event_name":"keydown","id":23,"type":"callback_add"},
            {"selector":".autocss_4","type":"insert_css","value":"padding: 5px"},
            {"id":23,"name":"class","type":"set_attr","value":"autocss_4"},
            {"child":2,"parent":23,"ref_id":null,"type":"insert_before"},
            {"child":22,"parent":23,"ref_id":null,"type":"insert_before"},
            {"child":22,"parent":23,"ref_id":null,"type":"insert_before"},
            {"id":24,"type":"create_comment","value":"list element"},
            {"id":25,"name":"div","type":"create_node"},
            {"id":26,"name":"div","type":"create_node"},
            {"selector":".autocss_5","type":"insert_css","value":"border: 1px solid black; padding: 10px; background-color: #e0e0e0; margin-bottom: 10px"},
            {"id":26,"name":"class","type":"set_attr","value":"autocss_5"},
            {"callback_id":13,"event_name":"mouseenter","id":26,"type":"callback_add"},
            {"callback_id":14,"event_name":"mouseleave","id":26,"type":"callback_add"},
            {"id":27,"name":"div","type":"create_node"},
            {"selector":"@keyframes autocss_7","type":"insert_css","value":"0% { -webkit-transform: scale(0);\\ntransform: scale(0); }\\n100% { -webkit-transform: scale(1.0);\\ntransform: scale(1.0);\\nopacity: 0; }"},
            {"selector":".autocss_6","type":"insert_css","value":"width: 40px; height: 40px; background-color: #d26913; border-radius: 100%; animation: 1.0s infinite ease-in-out autocss_7 "},
            {"id":27,"name":"class","type":"set_attr","value":"autocss_6"},
            {"child":27,"parent":26,"ref_id":null,"type":"insert_before"},
            {"child":26,"parent":25,"ref_id":null,"type":"insert_before"},
            {"id":28,"name":"button","type":"create_node"},
            {"callback_id":15,"event_name":"click","id":28,"type":"callback_add"},
            {"id":29,"name":"span","type":"create_node"},
            {"id":30,"type":"create_text","value":"start the progress bar"},
            {"child":30,"parent":29,"ref_id":null,"type":"insert_before"},
            {"child":29,"parent":28,"ref_id":null,"type":"insert_before"},
            {"id":31,"name":"span","type":"create_node"},
            {"child":24,"parent":31,"ref_id":null,"type":"insert_before"},
            {"child":31,"parent":28,"ref_id":null,"type":"insert_before"},
            {"child":28,"parent":25,"ref_id":null,"type":"insert_before"},
            {"child":25,"parent":23,"ref_id":22,"type":"insert_before"},
            {"child":23,"parent":1,"ref_id":null,"type":"insert_before"}
        ]"#;

        let commands = serde_json::from_str::<Vec<DomCommand>>(commands_str).unwrap();

        let mut all = AllElements::new();
        all.feed(commands);

        let result = all.get_response(true).convert_to_string(false);

        assert_eq!(result, r#"<!DOCTYPE html><style>.autocss_1 { list-style-type: none; margin: 10px; padding: 0 }</style><style>.autocss_2:hover { text-decoration: underline; }</style><style>.autocss_2 { display: inline; width: 60px; padding: 5px 10px; margin: 5px; cursor: pointer; background-color: lightgreen }</style><style>.autocss_3:hover { text-decoration: underline; }</style><style>.autocss_3 { display: inline; width: 60px; padding: 5px 10px; margin: 5px; cursor: pointer; background-color: lightblue }</style><style>.autocss_4 { padding: 5px }</style><style>.autocss_5 { border: 1px solid black; padding: 10px; background-color: #e0e0e0; margin-bottom: 10px }</style><style>@keyframes autocss_7 { 0% { -webkit-transform: scale(0);\ntransform: scale(0); }\n100% { -webkit-transform: scale(1.0);\ntransform: scale(1.0);\nopacity: 0; } }</style><style>.autocss_6 { width: 40px; height: 40px; background-color: #d26913; border-radius: 100%; animation: 1.0s infinite ease-in-out autocss_7  }</style><div class="autocss_4" data-id="23"><div data-id="2"><ul class="autocss_1" data-id="3"><li class="autocss_2" data-id="4">Counters</li><li class="autocss_3" data-id="6">Animations</li><li class="autocss_2" data-id="8">Sudoku</li><li class="autocss_2" data-id="10">Input</li><li class="autocss_2" data-id="12">Github Explorer</li><li class="autocss_2" data-id="14">Game Of Life</li><li class="autocss_2" data-id="16">Chat</li><li class="autocss_2" data-id="18">Todo</li><li class="autocss_2" data-id="20">Drop File</li></ul></div><div data-id="25"><div class="autocss_5" data-id="26"><div class="autocss_6" data-id="27"></div></div><button data-id="28"><span data-id="29">start the progress bar</span><span data-id="31"></span></button></div></div>"#);
    }
}
