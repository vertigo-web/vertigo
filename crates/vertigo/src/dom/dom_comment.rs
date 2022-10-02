use crate::{Driver, struct_mut::VecMut, get_driver, Client};
use super::dom_id::DomId;

/// A Real DOM representative - comment kind
pub struct DomComment {
    driver: Driver,
    pub id_dom: DomId,
    subscriptions: VecMut<Client>,
}

impl DomComment {
    pub fn new(text: impl Into<String>) -> DomComment {
        let text = text.into();
        let driver = get_driver();
        let id_dom = DomId::default();

        driver.inner.dom.create_comment(id_dom, text);

        DomComment {
            driver,
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
        self.driver.inner.dom.remove_comment(self.id_dom);
    }
}
