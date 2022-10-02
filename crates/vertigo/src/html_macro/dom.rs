use crate::{
    dom::dom_node::{DomNode, DomNodeFragment},
    Computed, DomText, DomElement, DomComment, DomCommentCreate, Value,
};

pub trait EmbedDom {
    fn embed(self) -> DomNodeFragment;
}

impl EmbedDom for DomElement {
    fn embed(self) -> DomNodeFragment {
        DomNodeFragment::Node { node: self }
    }
}

impl EmbedDom for DomComment {
    fn embed(self) -> DomNodeFragment {
        DomNodeFragment::Comment { node: self }
    }
}

impl EmbedDom for DomCommentCreate {
    fn embed(self) -> DomNodeFragment {
        DomNodeFragment::CommentCreate { node: self }
    }
}

impl EmbedDom for DomText {
    fn embed(self) -> DomNodeFragment {
        DomNodeFragment::Text { node: self }
    }
}

impl EmbedDom for DomNode {
    fn embed(self) -> DomNodeFragment {
        self.into()
    }
}

impl<T: ToString> EmbedDom for T {
    fn embed(self) -> DomNodeFragment {
        DomNodeFragment::Text {
            node: DomText::new(self.to_string())
        }
    }
}

impl<T: ToString + Clone + PartialEq + 'static> EmbedDom for &Computed<T> {
    fn embed(self) -> DomNodeFragment {
        let comment_create = self.render_value(|val|
            DomNodeFragment::Text {
                node: DomText::new(val.to_string())
            }
        );

        comment_create.embed()
    }
}

impl<T: ToString + Clone + PartialEq + 'static> EmbedDom for Computed<T> {
    fn embed(self) -> DomNodeFragment {
        (&self).embed()
    }
}

impl<T: ToString + Clone + PartialEq + 'static> EmbedDom for Value<T> {
    fn embed(self) -> DomNodeFragment {
        self.to_computed().embed()
    }
}
