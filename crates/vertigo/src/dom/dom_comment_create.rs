use super::dom_id::DomId;
use super::dom_comment::DomComment;

pub struct DomCommentCreate {
    create: Box<dyn FnOnce(DomId) -> DomComment>,
}

impl DomCommentCreate {
    pub fn new<F: FnOnce(DomId) -> DomComment + 'static>(create: F) -> Self {
        Self {
            create: Box::new(create)
        }
    }

    pub fn mount(self, parent_id: DomId) -> DomComment {
        let Self { create: mount } = self;
        mount(parent_id)
    }
}
