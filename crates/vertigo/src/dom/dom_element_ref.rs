use crate::{dom::dom_id::DomId, driver_module::api::DomAccess, ApiImport};

#[derive(Clone)]
pub struct DomElementRef {
    api: ApiImport,
    id: DomId,
}

impl DomElementRef {
    pub fn new(api: ApiImport, id: DomId) -> DomElementRef {
        DomElementRef { api, id }
    }

    pub fn dom_access(&self) -> DomAccess {
        self.api.dom_access().element(self.id)
    }
}

impl PartialEq for DomElementRef {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
