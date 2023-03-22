use crate::{Driver, struct_mut::{VecMut, ValueMut}, get_driver, Client, DropResource};
use super::dom_id::DomId;

/// A Real DOM representative - comment kind
pub struct DomComment {
    driver: Driver,
    pub id_dom: DomId,
    subscriptions: VecMut<Client>,
    _drop_list: VecMut<DropResource>,
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
            _drop_list: VecMut::new(),
        }
    }

    pub fn new_marker<F: Fn(DomId, DomId) -> Client + 'static>(comment_value: &'static str, mount: F) -> DomComment {
        let driver = get_driver();
        let id_comment = DomId::default();

        let when_mount = {
            let current_client: ValueMut<Option<Client>> = ValueMut::new(None);

            move |parent_id| {
                let client = mount(parent_id, id_comment);

                current_client.change(|current| {
                    *current = Some(client);
                });
            }
        };

        let drop_callback = driver.inner.dom.node_parent(id_comment, when_mount);


        let subscriptions = VecMut::new();

        let drop_list = VecMut::new();
        drop_list.push(drop_callback);

        driver.inner.dom.create_comment(id_comment, comment_value);

        DomComment {
            driver,
            id_dom: id_comment,
            subscriptions,
            _drop_list: drop_list
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
