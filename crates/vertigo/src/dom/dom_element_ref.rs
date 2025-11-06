use crate::{dom::dom_id::DomId, driver_module::api::DomAccess};

/// A reference to [DomElement](crate::DomElement).
///
/// Use [DomElement::get_ref](crate::DomElement::get_ref) to obtain. See [js!](crate::js!) macro for an example.
#[derive(Clone)]
pub struct DomElementRef {
    id: DomId,
}

impl DomElementRef {
    pub fn new(id: DomId) -> DomElementRef {
        DomElementRef { id }
    }

    pub fn dom_access(&self) -> DomAccess {
        DomAccess::default().element(self.id)
    }
}

impl PartialEq for DomElementRef {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
