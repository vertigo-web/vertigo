use crate::{Driver, struct_mut::{VecMut, ValueMut}, get_driver, DropResource, DomNode};
use super::dom_id::DomId;

/// A Real DOM representative - comment kind
pub struct DomComment {
    driver: Driver,
    pub id_dom: DomId,
    subscriptions: VecMut<DropResource>,
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

    pub fn new_marker<F: Fn(DomId, DomId) -> Option<DropResource> + 'static>(comment_value: &'static str, mount: F) -> DomComment {
        let driver = get_driver();
        let id_comment = DomId::default();

        let when_mount = {
            let current_client: ValueMut<Option<DropResource>> = ValueMut::new(None);

            move |parent_id| {
                let client = mount(parent_id, id_comment);

                current_client.change(|current| {
                    *current = client;
                });
            }
        };

        let drop_callback = driver.inner.dom.node_parent(id_comment, when_mount);


        let subscriptions = VecMut::new();

        subscriptions.push(drop_callback);

        driver.inner.dom.create_comment(id_comment, comment_value);

        DomComment {
            driver,
            id_dom: id_comment,
            subscriptions,
        }
    }

    pub fn id_dom(&self) -> DomId {
        self.id_dom
    }

    pub fn add_subscription(&self, client: DropResource) {
        self.subscriptions.push(client);
    }

    pub fn dom_fragment(mut list: Vec<DomNode>) -> DomComment {
        list.reverse();

        Self::new_marker("list dom node", move |parent_id, comment_id| {
            let mut prev_node = comment_id;
            let driver = get_driver();

            for node in list.iter() {
                let node_id = node.id_dom();
                driver.inner.dom.insert_before(parent_id, node_id, Some(prev_node));
                prev_node = node_id;
            }

            None
        })
    }
}

impl Drop for DomComment {
    fn drop(&mut self) {
        self.driver.inner.dom.remove_comment(self.id_dom);
    }
}
