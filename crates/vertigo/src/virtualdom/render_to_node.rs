use std::{collections::VecDeque};

use crate::{
    driver_module::driver_browser::EventCallback,
    virtualdom::models::{
        dom::DomNode,
        dom_component::DomComponent,
        dom_id::DomId,
        dom_node::DomElement,
        dom_text::DomText,
        vdom_component::VDomComponent,
        vdom_element::VDomElement,
        vdom_node::VDomNode, vdom_text::VDomText,
    },
};
use crate::virtualdom::models::vdom_component_id::VDomComponentId;
use super::render::CacheNode;

#[derive(Clone, Hash, PartialEq, Eq, Debug)]
enum DomNodeKey {
    Tag { name: &'static str },
    Iframe { src: String },
}

impl DomNodeKey {
    fn from_virtual(dom: &VDomElement) -> DomNodeKey {
        if dom.name == "iframe" {
            if let Some(src) = dom.attr.get("src") {
                return DomNodeKey::Iframe { src: src.clone() };
            }
        }

        DomNodeKey::Tag { name: dom.name }
    }

    fn from_real(dom: &DomElement) -> DomNodeKey {
        if dom.name() == "iframe" {
            if let Some(src) = dom.get_attr("src") {
                return DomNodeKey::Iframe { src };
            }
        }

        DomNodeKey::Tag { name: dom.name() }
    }

    fn test_eq(real: &DomElement, vdom: &VDomElement) -> bool {
        let key1 = DomNodeKey::from_real(real);
        let key2 = DomNodeKey::from_virtual(vdom);
        key1 == key2
    }
}

enum CurrentNodePairs<'a> {
    Component {
        node: DomComponent,
        // new: &'a VDomComponent
    },
    Node {
        node: DomElement,
        new: &'a VDomElement,
    },
    Text {
        node: DomText,
        new: &'a VDomText,
    },
}

impl<'a> CurrentNodePairs<'a> {
    fn id_dom(&self) -> DomId {
        match self {
            Self::Component { node, .. } => node.id_dom(),
            Self::Node { node, .. } => node.id_dom(),
            Self::Text { node, .. } => node.id_dom(),
        }
    }
}

fn get_pair_for_update(real: DomNode, new: &VDomNode) -> Result<CurrentNodePairs<'_>, (DomNode, &VDomNode)> {
    match real {
        DomNode::Component { node } => {
            if let VDomNode::Component { node: vnode } = new {
                if node.id == vnode.id() {
                    return Ok(CurrentNodePairs::Component {
                        node,
                        // new: vnode
                    });
                }
            }

            Err((DomNode::new_component(node), new))
        }
        DomNode::Node { node } => {
            if let VDomNode::Element { node: vnode } = new {
                if DomNodeKey::test_eq(&node, vnode) {
                    return Ok(CurrentNodePairs::Node { node, new: vnode });
                }
            }

            Err((DomNode::new_node(node), new))
        }
        DomNode::Text { node } => {
            if let VDomNode::Text { node: vnode } = new {
                return Ok(CurrentNodePairs::Text { node, new: vnode });
            }

            Err((DomNode::new_text(node), new))
        },
        DomNode::Comment { .. } => {
            unreachable!()
        }
    }
}

fn find_first_dom(list: &VecDeque<CurrentNodePairs<'_>>) -> Option<DomId> {
    if let Some(first) = list.get(0) {
        return Some(first.id_dom());
    }

    None
}

// Try to match starting from top
fn get_pairs_top<'a>(
    real_child: &mut VecDeque<DomNode>,
    new_child: &mut VecDeque<&'a VDomNode>,
) -> VecDeque<CurrentNodePairs<'a>> {
    let mut pairs_top = VecDeque::new();

    loop {
        let node = real_child.pop_front();
        let child = new_child.pop_front();

        match (node, child) {
            (Some(node), Some(child)) => {
                let pair = get_pair_for_update(node, child);

                match pair {
                    Ok(update_item) => {
                        pairs_top.push_back(update_item);
                        continue;
                    }
                    Err((node, child)) => {
                        real_child.push_front(node);
                        new_child.push_front(child);
                    }
                }
            }
            (Some(node), None) => {
                real_child.push_front(node);
            }
            (None, Some(child)) => {
                new_child.push_front(child);
            }
            (None, None) => {}
        }

        return pairs_top;
    }
}

// Try to match starting from bottom
fn get_pairs_bottom<'a>(
    real_child: &mut VecDeque<DomNode>,
    new_child: &mut VecDeque<&'a VDomNode>,
) -> VecDeque<CurrentNodePairs<'a>> {
    let mut pairs_bottom = VecDeque::new();

    loop {
        let node = real_child.pop_back();
        let child = new_child.pop_back();

        match (node, child) {
            (Some(node), Some(child)) => {
                let pair = get_pair_for_update(node, child);

                match pair {
                    Ok(update_item) => {
                        pairs_bottom.push_front(update_item);
                        continue;
                    }
                    Err((node, child)) => {
                        real_child.push_back(node);
                        new_child.push_back(child);
                    }
                }
            }
            (Some(node), None) => {
                real_child.push_back(node);
            }
            (None, Some(child)) => {
                new_child.push_back(child);
            }
            (None, None) => {}
        }

        return pairs_bottom;
    }
}

fn get_pairs_middle<'a>(
    target: &DomElement,
    last_before: Option<DomId>,
    real_child: VecDeque<DomNode>,
    new_child: VecDeque<&'a VDomNode>,
) -> VecDeque<CurrentNodePairs<'a>> {
    let mut pairs_middle = VecDeque::new();

    let mut real_node: CacheNode<DomNodeKey, DomElement, VDomElement> = CacheNode::new(
        |node: &VDomElement| -> DomElement {
            DomElement::new(node.name)
        },
    );
    let mut real_text: CacheNode<String, DomText, VDomText> = CacheNode::new(
        |node: &VDomText| -> DomText {
            DomText::new(node.value.clone())
        },
    );

    let mut real_component: CacheNode<VDomComponentId, DomComponent, VDomComponent> = CacheNode::new(
        move |component: &VDomComponent| -> DomComponent {
            // TODO - to rethink the component concept
            // let node_root = target.create_node(component.view.get_value().name);
            let node = DomElement::new("div");

            let subscription = component.clone().render_to(node.clone());

            DomComponent {
                id: component.id(),
                subscription,
                node,
            }
        },
    );

    for item in real_child {
        match item {
            DomNode::Node { node } => {
                let key = DomNodeKey::from_real(&node);
                real_node.insert(key, node);
            }
            DomNode::Text { node } => {
                let id = node.get_value();
                real_text.insert(id, node);
            }
            DomNode::Component { node } => {
                let id = node.id;
                real_component.insert(id, node);
            },
            DomNode::Comment { node: _ } => {}
        }
    }

    let mut last_before = last_before;

    for item in new_child.into_iter().rev() {
        let child_id = match item {
            VDomNode::Element { node } => {
                let key = DomNodeKey::from_virtual(node);
                let child = real_node.get_or_create(key, node);
                let child_id = child.id_dom();

                pairs_middle.push_front(CurrentNodePairs::Node { node: child, new: node });

                child_id
            }
            VDomNode::Text { node } => {
                let id = node.value.clone();
                let child = real_text.get_or_create(id, node);
                let child_id = child.id_dom();

                pairs_middle.push_front(CurrentNodePairs::Text { node: child, new: node });

                child_id
            }
            VDomNode::Component { node } => {
                let id = node.id();
                let child = real_component.get_or_create(id, node);
                let child_id = child.id_dom();

                pairs_middle.push_front(CurrentNodePairs::Component {
                    node: child,
                    // new: node,
                });

                child_id
            }
        };

        target.insert_before(child_id, last_before);
        last_before = Some(child_id);
    }

    pairs_middle
}

fn update_node_child(
    target: &DomElement,
    new_version: &VDomElement,
) {
    let pairs: VecDeque<CurrentNodePairs<'_>> = {
        let mut real_child: VecDeque<DomNode> = target.extract_child();
        let mut new_child: VecDeque<&VDomNode> = new_version.children.iter().collect();

        let pairs_top = get_pairs_top(&mut real_child, &mut new_child);
        let mut pairs_bottom = get_pairs_bottom(&mut real_child, &mut new_child);

        let last_before: Option<DomId> = find_first_dom(&pairs_bottom);
        let mut pairs_middle = get_pairs_middle(
            target, last_before, real_child, new_child
        );

        let mut pairs = pairs_top;
        pairs.append(&mut pairs_middle);
        pairs.append(&mut pairs_bottom);
        pairs
    };

    let new_child: VecDeque<DomNode> = {
        let mut new_child = VecDeque::new();

        for item in pairs.into_iter() {
            match item {
                CurrentNodePairs::Node { node, new } => {
                    update_node_attr(&node, new);
                    update_node_child(&node, new);
                    new_child.push_back(DomNode::new_node(node));
                }
                CurrentNodePairs::Text { node, new } => {
                    node.update(&new.value);
                    new_child.push_back(DomNode::new_text(node));
                }
                CurrentNodePairs::Component { node, .. } => {
                    new_child.push_back(DomNode::new_component(node));
                }
            }
        }

        new_child
    };

    target.put_child(new_child);
}

fn update_node_attr(real_node: &DomElement, node: &VDomElement) {
    let css = &node.css;
    let class_name = css.as_ref().map(|css| css.convert_to_string());

    real_node.update_attr(&node.attr, class_name);
    real_node.set_event(EventCallback::OnClick { callback: node.on_click.clone() });
    real_node.set_event(EventCallback::OnInput { callback: node.on_input.clone() });
    real_node.set_event(EventCallback::OnMouseEnter { callback: node.on_mouse_enter.clone() });
    real_node.set_event(EventCallback::OnMouseLeave { callback: node.on_mouse_leave.clone() });
    real_node.set_event(EventCallback::OnKeyDown { callback: node.on_key_down.clone() });
    real_node.set_event(EventCallback::HookKeyDown { callback: node.hook_key_down.clone() });
    real_node.set_event(EventCallback::OnDropFile { callback: node.on_dropfile.clone() })
}

pub fn update_node(
    target: &DomElement,
    new_version: &VDomElement,
) {
    //update tag name
    target.update_name(new_version.name);

    //update attributes
    update_node_attr(target, new_version);

    //update child
    update_node_child(target, new_version);
}

