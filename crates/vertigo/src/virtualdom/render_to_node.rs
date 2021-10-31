use std::{collections::VecDeque, rc::Rc};

use crate::{
    NodeRefs,
    computed::Client,
    driver::EventCallback
};

use crate::{
    virtualdom::{
        models::{
            realdom::RealDomNode,
            realdom_node::RealDomElement,
            realdom_text::RealDomText,
            realdom_component::RealDomComponent,
            vdom_node::VDomNode,
            vdom_element::VDomElement,
            vdom_component::VDomComponent,
            vdom_component_id::VDomComponentId,
            vdom_text::VDomText,
            realdom_id::RealDomId,
        }
    },
    css::css_manager::CssManager,
};

use super::render::{
    CacheNode,
};

#[derive(Clone, Hash, PartialEq, Eq, Debug)]
enum DomNodeKey {
    Tag {
        name: &'static str,
    },
    Iframe {
        src: String,
    }
}

impl DomNodeKey {
    fn from_virtual(dom: &VDomElement) -> DomNodeKey {
        if dom.name == "iframe" {
            if let Some(src) = dom.attr.get("src") {
                return DomNodeKey::Iframe { src: src.clone() }
            }
        }

        DomNodeKey::Tag {
            name: dom.name
        }
    }

    fn from_real(dom: &RealDomElement) -> DomNodeKey {
        if dom.name() == "iframe" {
            if let Some(src) = dom.get_attr("src") {
                return DomNodeKey::Iframe { src }
            }
        }

        DomNodeKey::Tag {
            name: dom.name()
        }
    }

    fn test_eq(real: &RealDomElement, vdom: &VDomElement) -> bool {
        let key1 = DomNodeKey::from_real(real);
        let key2 = DomNodeKey::from_virtual(vdom);
        key1 == key2
    }
}

struct RefsContext {
    apply: Vec<Rc<dyn Fn(&NodeRefs)>>,
    node_refs: NodeRefs,
}

enum CurrentNodePairs<'a> {
    Component {
        node: RealDomComponent,
        // new: &'a VDomComponent
    },
    Node {
        node: RealDomElement,
        new: &'a VDomElement,
    },
    Text {
        node: RealDomText,
        new: &'a VDomText,
    }
}

impl<'a> CurrentNodePairs<'a> {
    fn id_dom(&self) -> RealDomId {
        match self {
            Self::Component { node, .. } => node.dom_id(),
            Self::Node { node, .. } => node.id_dom(),
            Self::Text { node, .. } => node.id_dom(),
        }
    }
}

fn get_pair_for_update(real: RealDomNode, new: &VDomNode) -> Result<CurrentNodePairs, (RealDomNode, &VDomNode)> {
    match real {
        RealDomNode::Component { node } => {
            if let VDomNode::Component { node: vnode } = new {
                if node.id == vnode.id {
                    return Ok(CurrentNodePairs::Component {
                        node,
                        // new: vnode
                    });
                }
            }

            Err((
                RealDomNode::new_component(node),
                new
            ))
        },
        RealDomNode::Node { node } => {
            if let VDomNode::Element { node : vnode} = new {
                if DomNodeKey::test_eq(&node, vnode) {
                    return Ok(CurrentNodePairs::Node {
                        node,
                        new: vnode,
                    });
                }
            }

            Err((
                RealDomNode::new_node(node),
                new
            ))
        },
        RealDomNode::Text { node } => {
            if let VDomNode::Text { node: vnode } = new {
                return Ok(CurrentNodePairs::Text {
                    node,
                    new: vnode
                });
            }

            Err((
                RealDomNode::new_text(node),
                new
            ))
        }
    }
}

fn find_first_dom(list: &VecDeque<CurrentNodePairs>) -> Option<RealDomId> {
    if let Some(first) = list.get(0) {
        return Some(first.id_dom());
    }

    None
}


//próbuj dopasować od góry
fn get_pairs_top<'a>(real_child: &mut VecDeque<RealDomNode>, new_child: &mut VecDeque<&'a VDomNode>) -> VecDeque<CurrentNodePairs<'a>> {
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
                    },
                    Err((node, child)) => {
                        real_child.push_front(node);
                        new_child.push_front(child);
                    }
                }
            },
            (Some(node), None) => {
                real_child.push_front(node);
            },
            (None, Some(child)) => {
                new_child.push_front(child);
            },
            (None, None) => {}
        }

        return pairs_top;
    }
}

//próbuj dopasować od dołu
fn get_pairs_bottom<'a>(real_child: &mut VecDeque<RealDomNode>, new_child: &mut VecDeque<&'a VDomNode>) -> VecDeque<CurrentNodePairs<'a>> {
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
                    },
                    Err((node, child)) => {
                        real_child.push_back(node);
                        new_child.push_back(child);
                    }
                }
            },
            (Some(node), None) => {
                real_child.push_back(node);
            },
            (None, Some(child)) => {
                new_child.push_back(child);
            },
            (None, None) => {}
        }

        return pairs_bottom;
    }
}

fn get_pairs_middle<'a>(
    target: &RealDomElement,
    css_manager: &CssManager,
    last_before: Option<RealDomId>,
    real_child: VecDeque<RealDomNode>,
    new_child: VecDeque<&'a VDomNode>
) -> VecDeque<CurrentNodePairs<'a>> {

    let mut pairs_middle = VecDeque::new();

    let mut real_node: CacheNode<DomNodeKey, RealDomElement, VDomElement> = CacheNode::new(
        |_css_manager: &CssManager, target: &RealDomElement, node: &VDomElement| -> RealDomElement {
            target.create_node(node.name)
        },
    );
    let mut real_text: CacheNode<String, RealDomText, VDomText> = CacheNode::new(
        |_css_manager: &CssManager, target: &RealDomElement, node: &VDomText| -> RealDomText {
            target.create_text(node.value.clone())
        },
    );
    let mut real_component: CacheNode<VDomComponentId, RealDomComponent, VDomComponent> = CacheNode::new(
        |css_manager: &CssManager, target: &RealDomElement, component: &VDomComponent| -> RealDomComponent {

            // TODO - to rethink the component concept
            // let node_root = target.create_node(component.view.get_value().name);
            let node = target.create_node("div");

            let subscription = render_to_node(css_manager.clone(), node.clone(), component.clone());

            RealDomComponent {
                id: component.id.clone(),
                subscription,
                node,
            }
        },
    );

    for item in real_child {
        match item {
            RealDomNode::Node { node } => {
                let key = DomNodeKey::from_real(&node);
                real_node.insert(key, node);
            },
            RealDomNode::Text { node } => {
                let id = node.get_value();
                real_text.insert(id, node);
            },
            RealDomNode::Component { node } => {
                let id = node.id.clone();
                real_component.insert(id, node);
            }
        }
    }

    let mut last_before = last_before;

    for item in new_child.into_iter().rev() {

        let child_id = match item {
            VDomNode::Element { node } => {
                let key = DomNodeKey::from_virtual(node);
                let child = real_node.get_or_create(css_manager, target, key, node);
                let child_id = child.id_dom();

                pairs_middle.push_front(CurrentNodePairs::Node {
                    node: child,
                    new: node
                });

                child_id
            },
            VDomNode::Text { node } => {
                let id = node.value.clone();
                let child = real_text.get_or_create(css_manager, target, id, node);
                let child_id = child.id_dom();

                pairs_middle.push_front(CurrentNodePairs::Text {
                    node: child,
                    new: node
                });

                child_id
            },
            VDomNode::Component { node } => {
                let id = node.id.clone();
                let child = real_component.get_or_create(css_manager, target, id, node);
                let child_id = child.dom_id();

                pairs_middle.push_front(CurrentNodePairs::Component {
                    node: child,
                    // new: node,
                });

                child_id
            }
        };

        target.insert_before(child_id.clone(), last_before);
        last_before = Some(child_id);
    }

    pairs_middle
}

fn update_node_child(
    css_manager: &CssManager,
    refs_context: &mut RefsContext,
    target: &RealDomElement,
    new_version: &VDomElement
) {

    if let Some(ref_name) = new_version.dom_ref {
        refs_context.node_refs.set(ref_name, target.get_ref().unwrap());            //TODO - we always expect ref
    }

    if let Some(dom_apply) = &new_version.dom_apply {
        refs_context.apply.push(dom_apply.clone());
    }

    let pairs: VecDeque<CurrentNodePairs> = {
        let mut real_child: VecDeque<RealDomNode> = target.extract_child();
        let mut new_child: VecDeque<&VDomNode> = new_version.children.iter().collect();

        let pairs_top = get_pairs_top(&mut real_child, &mut new_child);
        let mut pairs_bottom = get_pairs_bottom(&mut real_child, &mut new_child);

        let last_before: Option<RealDomId> = find_first_dom(&pairs_bottom);
        let mut pairs_middle = get_pairs_middle(
            target,
            css_manager,
            last_before,
            real_child,
            new_child
        );

        let mut pairs = pairs_top;
        pairs.append(&mut pairs_middle);
        pairs.append(&mut pairs_bottom);
        pairs
    };


    let new_child: VecDeque<RealDomNode> = {
        let mut new_child = VecDeque::new();

        for item in pairs.into_iter() {
            match item {
                CurrentNodePairs::Node { node, new } => {
                    update_node_attr(css_manager, &node, new);
                    update_node_child(css_manager, refs_context, &node, new);
                    new_child.push_back(RealDomNode::new_node(node));
                },
                CurrentNodePairs::Text { node, new } => {
                    node.update(&new.value);
                    new_child.push_back(RealDomNode::new_text(node));
                },
                CurrentNodePairs::Component { node, .. } => {
                    new_child.push_back(RealDomNode::new_component(node));
                },
            }
        }

        new_child
    };

    target.put_child(new_child);
}


fn update_node_attr(css_manager: &CssManager, real_node: &RealDomElement, node: &VDomElement) {
    let css = &node.css;
    let class_name = css.as_ref().map(|css| css_manager.get_class_name(css));

    real_node.update_attr(&node.attr, class_name);
    real_node.set_event(EventCallback::OnClick { callback: node.on_click.clone() });
    real_node.set_event(EventCallback::OnInput { callback: node.on_input.clone() });
    real_node.set_event(EventCallback::OnMouseEnter { callback: node.on_mouse_enter.clone() });
    real_node.set_event(EventCallback::OnMouseLeave { callback: node.on_mouse_leave.clone() });
    real_node.set_event(EventCallback::OnKeyDown { callback: node.on_key_down.clone() });
}

fn update_node(
    css_manager: &CssManager,
    refs_context: &mut RefsContext,
    target: &RealDomElement,
    new_version: &VDomElement
) {

    //update tag name
    target.update_name(new_version.name);

    //update attributes
    update_node_attr(css_manager, target, new_version);

    //update child
    update_node_child(css_manager, refs_context, target, new_version);
}

pub fn render_to_node(css_manager: CssManager, target: RealDomElement, component: VDomComponent) -> Client {
    let subscription: Client = component.view.subscribe(move |new_version| {

        let mut refs_context = RefsContext {
            apply: Vec::new(),
            node_refs: NodeRefs::new(),
        };

        update_node(
            &css_manager,
            &mut refs_context,
            &target,
            new_version
        );

        let RefsContext { apply, node_refs} = refs_context;

        for apply_fun in apply {
            apply_fun(&node_refs);
        }
    });

    subscription
}
