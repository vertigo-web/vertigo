use crate::{dom::dom_node::{DomNode, DomNodeFragment}, DomText, DomElement, DomComment, DomCommentCreate};

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

// impl EmbedDom for 

impl<T: ToString> EmbedDom for T {
    fn embed(self) -> DomNodeFragment {
        DomNodeFragment::Text {
            node: DomText::new(self.to_string())
        }
    }
}
