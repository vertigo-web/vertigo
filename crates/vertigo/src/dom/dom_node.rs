use crate::{dom::{
    dom_element::DomElement,
    dom_text::DomText,
}, DomCommentCreate};

use super::{dom_id::DomId, dom_comment::DomComment};

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
        DomNode::Text { node: DomText::new(text) }
    }
}


pub enum DomNodeFragment {
    Node { node: DomElement },
    Text { node: DomText },
    Comment { node: DomComment },
    CommentCreate { node: DomCommentCreate },
}

impl DomNodeFragment {
    pub fn convert_to_node(self, parent_id: DomId) -> DomNode {
        match self {
            Self::Node { node } => DomNode::Node { node },
            Self::Text { node } => DomNode::Text { node },
            Self::Comment { node } => DomNode::Comment { node },
            Self::CommentCreate { node } => DomNode::Comment {
                node: node.mount(parent_id)
            }
        }
    }
}

impl From<DomElement> for DomNodeFragment {
    fn from(node: DomElement) -> Self {
        DomNodeFragment::Node { node }
    }
}

impl From<DomText> for DomNodeFragment {
    fn from(node: DomText) -> Self {
        DomNodeFragment::Text { node }
    }
}

impl From<DomComment> for DomNodeFragment {
    fn from(node: DomComment) -> Self {
        DomNodeFragment::Comment { node }
    }
}

impl From<DomCommentCreate> for DomNodeFragment {
    fn from(node: DomCommentCreate) -> Self {
        DomNodeFragment::CommentCreate { node }
    }
}

impl From<DomNode> for DomNodeFragment {
    fn from(dom_node: DomNode) -> Self {
        match dom_node {
            DomNode::Node { node } => DomNodeFragment::Node { node },
            DomNode::Text { node } => DomNodeFragment::Text { node },
            DomNode::Comment { node } => DomNodeFragment::Comment { node },
        }
    }
}
