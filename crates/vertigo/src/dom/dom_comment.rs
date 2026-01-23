use crate::{
    DomNode,
    computed::{
        DropResource,
        struct_mut::{ValueMut, VecMut},
    },
    driver_module::get_driver_dom,
};

use super::dom_id::DomId;

/// A Real DOM representative - comment kind
pub struct DomComment {
    pub id_dom: DomId,
    subscriptions: VecMut<DropResource>,
}

impl DomComment {
    pub fn new(text: impl Into<String>) -> DomComment {
        let text = text.into();
        let id_dom = DomId::default();

        get_driver_dom().create_comment(id_dom, text);

        DomComment {
            id_dom,
            subscriptions: VecMut::new(),
        }
    }

    pub fn new_marker<F: Fn(DomId, DomId) -> Option<DropResource> + 'static>(
        comment_value: &'static str,
        mount: F,
    ) -> DomComment {
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

        let drop_callback = get_driver_dom().node_parent(id_comment, when_mount);

        let subscriptions = VecMut::new();

        subscriptions.push(drop_callback);

        get_driver_dom().create_comment(id_comment, comment_value);

        DomComment {
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

            for node in list.iter() {
                let node_id = node.id_dom();
                get_driver_dom().insert_before(parent_id, node_id, Some(prev_node));
                prev_node = node_id;
            }

            None
        })
    }

    pub fn append_drop_resource(&self, resource: DropResource) {
        self.subscriptions.push(resource);
    }
}

impl Drop for DomComment {
    fn drop(&mut self) {
        get_driver_dom().remove_comment(self.id_dom);
    }
}
