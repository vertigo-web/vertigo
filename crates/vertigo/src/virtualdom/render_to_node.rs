use std::{collections::{VecDeque}, rc::Rc};

use crate::{NodeRefs, computed::{
        Client,
    }, driver::EventCallback};

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
    css_manager::css_manager::CssManager,
};

use super::render::{CacheNode, get_pair_for_update, NodePairs};

struct RefsContext {
    apply: Vec<Rc<dyn Fn(&NodeRefs) -> ()>>,
    node_refs: NodeRefs,
}

fn update_node_child_updated_with_order(
    css_manager: &CssManager,
    refs_context: &mut RefsContext,
    target: &VecDeque<RealDomNode>,
    new_version: &[VDomNode]
) -> bool {
    if target.len() != new_version.len() {
        return false;
    }

    let max_index = target.len();

    let mut for_update: Vec<NodePairs> = Vec::new();

    for index in 0..max_index {
        let real = &target[index];
        let new = &new_version[index];

        if let Some(pair) = get_pair_for_update(real, new) {
            for_update.push(pair);
        } else {
            return false;
        }
    }

    for item in for_update {
        match item {
            NodePairs::Component { real: _real, new: _new } => {
            },
            NodePairs::Node { real, new } => {
                update_node_attr(&css_manager, real, new);
                update_node_child(css_manager, refs_context, real, new);
            },
            NodePairs::Text { real, new } => {
                real.update(&new.value);
            },
        }
    }

    true
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

    let real_child = target.extract_child();

    let update_order_ok = update_node_child_updated_with_order(css_manager, refs_context, &real_child, &new_version.children);
    if update_order_ok {
        target.put_child(real_child);
        return;
    }

    let mut real_node: CacheNode<&'static str, RealDomElement, VDomElement> = CacheNode::new(
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
        |css_manager: &CssManager, target: &RealDomElement, node: &VDomComponent| -> RealDomComponent {

            let node_root = target.create_node(node.view.get_value().name);

            let node_root_for_id = node_root.clone();

            let subscription = render_to_node(css_manager.clone(), node_root, node.clone());

            RealDomComponent {
                id: node.id.clone(),
                subscription,
                node: node_root_for_id,
            }
        },
    );

    for item in real_child {
        match item {
            RealDomNode::Node { node }=> {
                real_node.insert(node.name(), node);
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


    let mut ref_id: Option<RealDomId> = None;

    for item in new_version.children.iter().rev() {

        match item {
            VDomNode::Element { node } => {
                let id = node.name;
                let dom_child = real_node.get_or_create(css_manager, target, id, node);
                let new_ref_id = dom_child.id_dom();

                update_node_attr(&css_manager, &dom_child, &node);
                update_node_child(css_manager, refs_context, &dom_child, &node);

                target.insert_before(RealDomNode::Node { node: dom_child }, ref_id);
                ref_id = Some(new_ref_id);
            },
            VDomNode::Text { node } => {
                let id = node.value.clone();
                let dom_child = real_text.get_or_create(css_manager, target,id, node);
                let new_ref_id = dom_child.id_dom.clone();

                dom_child.update(&node.value);

                target.insert_before(RealDomNode::Text { node: dom_child }, ref_id);
                ref_id = Some(new_ref_id);
            },
            VDomNode::Component { node } => {
                let id = node.id.clone();
                let dom_child = real_component.get_or_create(css_manager, target,id, node);
                let new_ref_id = dom_child.dom_id();

                target.insert_before(RealDomNode::Component { node: dom_child }, ref_id);
                ref_id = Some(new_ref_id);
            }
        }
    }
}


fn update_node_attr(css_manager: &CssManager, real_node: &RealDomElement, node: &VDomElement) {
    let css = &node.css;
    let class_name = match css {
        Some (css) => Some(css_manager.get_class_name(css)),
        None => None,
    };

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

    //updejt tag name
    target.update_name(new_version.name);

    //updejt atrybutów
    update_node_attr(&css_manager, target, &new_version);

    //odpal updejt dzieci
    update_node_child(css_manager, refs_context, target, &new_version);
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
