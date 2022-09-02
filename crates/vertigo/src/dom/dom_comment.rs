use crate::{Driver, struct_mut::VecMut, get_driver, Client};
use super::dom_id::DomId;

pub struct DomComment {
    dom_driver: Driver,
    pub id_dom: DomId,
    subscriptions: VecMut<Client>,
}

impl DomComment {
    pub fn new(text: impl Into<String>) -> DomComment {
        let text = text.into();
        let dom_driver = get_driver();
        let id_dom = DomId::default();

        dom_driver.create_comment(id_dom, text);

        DomComment {
            dom_driver,
            id_dom,
            subscriptions: VecMut::new(),
        }
    }

    pub fn id_dom(&self) -> DomId {
        self.id_dom
    }

    pub fn add_subscription(&self, client: Client) {
        self.subscriptions.push(client);
    }
}

impl Drop for DomComment {
    fn drop(&mut self) {
        self.dom_driver.remove_comment(self.id_dom);
    }
}
