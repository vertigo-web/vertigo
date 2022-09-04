use super::dom_id::DomId;
use super::dom_comment::DomComment;

pub struct DomCommentCreate {
    id: DomId,
    create: Box<dyn FnOnce(DomId) -> DomComment>,
}

impl DomCommentCreate {
    pub fn new<F: FnOnce(DomId) -> DomComment + 'static>(id: DomId, create: F) -> Self {
        Self {
            id,
            create: Box::new(create)
        }
    }

    pub fn mount(self, parent_id: DomId) -> DomComment {
        let Self { create: mount, .. } = self;
        mount(parent_id)
    }

    pub fn id(&self) -> DomId {
        self.id
    }
}
