use crate::{virtualdom::models::dom::DomNode, DomText, DomElement, DomComment};

pub trait EmbedDom {
    fn embed(self) -> DomNode;
}

impl EmbedDom for DomElement {
    fn embed(self) -> DomNode {
        DomNode::Node { node: self }
    }
}

impl EmbedDom for DomComment {
    fn embed(self) -> DomNode {
        DomNode::Comment { node: self }
    }
}

impl EmbedDom for DomText {
    fn embed(self) -> DomNode {
        DomNode::Text { node: self }
    }
}

impl EmbedDom for DomNode {
    fn embed(self) -> DomNode {
        self
    }
}

impl<T: ToString> EmbedDom for T {
    fn embed(self) -> DomNode {
        DomNode::Text {
            node: DomText::new(self.to_string())
        }
    }
}
