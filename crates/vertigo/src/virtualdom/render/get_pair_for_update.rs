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
            vdom_text::VDomText,
        }
    },
};


pub enum NodePairs<'a> {
    Component {
        real: &'a RealDomComponent,
        new: &'a VDomComponent
    },
    Node {
        real: &'a RealDomElement,
        new: &'a VDomElement,
    },
    Text {
        real: &'a RealDomText,
        new: &'a VDomText,
    }
}

pub fn get_pair_for_update<'a>(real: &'a RealDomNode, new: &'a VDomNode) -> Option<NodePairs<'a>> {
    match real {
        RealDomNode::Component { node } => {
            if let VDomNode::Component { node: vnode } = new {
                if node.id == vnode.id {
                    return Some(NodePairs::Component {
                        real: node,
                        new: vnode
                    });
                }
            }
        },
        RealDomNode::Node { node } => {
            if let VDomNode::Element { node : vnode} = new {
                if node.name() == vnode.name {
                    return Some(NodePairs::Node {
                        real: node,
                        new: vnode,
                    });
                }
            }
        },
        RealDomNode::Text { node } => {
            if let VDomNode::Text { node: vnode } = new {
                return Some(NodePairs::Text {
                    real: node,
                    new: vnode
                });
            }
        }
    }

    None
}
