use crate::{
    DropResource,
    dom::{dom_element::DomElement, dom_text::DomText},
};

use super::{dom_comment::DomComment, dom_id::DomId};

/// A Real DOM representative.
///
/// It is used to generate DOM changing commands. To create `DomNode` tree conveniently use [dom!](crate::dom) macro.
pub enum DomNode {
    Node { node: DomElement },
    Text { node: DomText },
    Comment { node: DomComment },
}

impl DomNode {
    pub fn new_node(node: DomElement) -> DomNode {
        DomNode::Node { node }
    }

    pub fn new_text(node: DomText) -> DomNode {
        DomNode::Text { node }
    }

    pub fn id_dom(&self) -> DomId {
        match self {
            Self::Node { node } => node.id_dom(),
            Self::Text { node } => node.id_dom(),
            Self::Comment { node } => node.id_dom(),
        }
    }

    pub fn append_drop_resource(&self, resource: DropResource) {
        match &self {
            Self::Node { node } => {
                node.append_drop_resource(resource);
            }
            Self::Text { node } => {
                node.append_drop_resource(resource);
            }
            Self::Comment { node } => {
                node.append_drop_resource(resource);
            }
        }
    }
}

impl From<DomElement> for DomNode {
    fn from(node: DomElement) -> Self {
        DomNode::Node { node }
    }
}

impl From<DomText> for DomNode {
    fn from(node: DomText) -> Self {
        DomNode::Text { node }
    }
}

impl From<DomComment> for DomNode {
    fn from(node: DomComment) -> Self {
        DomNode::Comment { node }
    }
}

impl<T: Into<String>> From<T> for DomNode {
    fn from(text: T) -> Self {
        DomNode::Text {
            node: DomText::new(text),
        }
    }
}
