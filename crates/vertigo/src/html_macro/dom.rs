use crate::{
    dom::dom_node::{DomNode},
    Computed, DomText, DomElement, DomComment, Value,
};

pub trait EmbedDom {
    fn embed(self) -> DomNode;
}

impl EmbedDom for DomElement {
    fn embed(self) -> DomNode {
        self.into()
    }
}

impl EmbedDom for DomComment {
    fn embed(self) -> DomNode {
        self.into()
    }
}

impl EmbedDom for DomText {
    fn embed(self) -> DomNode {
        self.into()
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

impl<T: ToString + Clone + PartialEq + 'static> EmbedDom for &Computed<T> {
    fn embed(self) -> DomNode {
        self.render_value(|val|
            DomNode::Text {
                node: DomText::new(val.to_string())
            }
        )
    }
}

impl<T: ToString + Clone + PartialEq + 'static> EmbedDom for Computed<T> {
    fn embed(self) -> DomNode {
        (&self).embed()
    }
}

impl<T: ToString + Clone + PartialEq + 'static> EmbedDom for Value<T> {
    fn embed(self) -> DomNode {
        self.to_computed().embed()
    }
}

impl<T: ToString + Clone + PartialEq + 'static> EmbedDom for &Value<T> {
    fn embed(self) -> DomNode {
        self.to_computed().embed()
    }
}
