use std::rc::Rc;

use crate::{
    DomNode,
    computed::{
        DropResource,
        struct_mut::{ValueMut, VecMut},
    },
    driver_module::get_driver_dom,
};

use super::dom_id::DomId;

/// The nodes a marker keeps directly in front of itself, in document order.
///
/// A marker created with [`DomComment::new_marker`] renders its content as siblings
/// placed just before the marker comment. Reporting those ids here lets the marker
/// carry them along when it is moved inside its parent, instead of tearing the
/// content down and building it again — so DOM state (focus, selection, scroll
/// position, running animations) survives a move.
///
/// Whoever creates the content is responsible for keeping this up to date; a marker
/// that reports nothing is rebuilt on every move.
#[derive(Clone)]
pub struct MarkerContent {
    ids: Rc<ValueMut<Vec<DomId>>>,
}

impl MarkerContent {
    fn new() -> MarkerContent {
        MarkerContent {
            ids: Rc::new(ValueMut::new(Vec::new())),
        }
    }

    /// Replace the list of nodes standing in front of the marker.
    pub fn set(&self, ids: Vec<DomId>) {
        self.ids.set(ids);
    }
}

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

    /// Create a comment that marks a place in the DOM, and mount content in front of it.
    ///
    /// `mount` runs when the marker enters a parent, and again if it is ever moved to a
    /// *different* parent. Moving the marker inside the same parent does **not** re-run it:
    /// the marker instead re-inserts the nodes reported through [`MarkerContent`] ahead of
    /// itself, so the existing subtree travels with the marker rather than being rebuilt.
    pub fn new_marker<F: Fn(DomId, DomId, &MarkerContent) -> Option<DropResource> + 'static>(
        comment_value: &'static str,
        mount: F,
    ) -> DomComment {
        let id_comment = DomId::default();
        let content = MarkerContent::new();

        let when_mount = {
            let current_client: ValueMut<Option<DropResource>> = ValueMut::new(None);
            let mounted_in: ValueMut<Option<DomId>> = ValueMut::new(None);
            let content = content.clone();

            move |parent_id| {
                let owned = content.ids.get();

                if !owned.is_empty() && mounted_in.get() == Some(parent_id) {
                    // Already mounted here, so this is a move within the parent. Bring the
                    // content along - each nested marker does the same for its own content.
                    for child_id in owned {
                        get_driver_dom().insert_before(parent_id, child_id, Some(id_comment));
                    }

                    return;
                }

                let client = mount(parent_id, id_comment, &content);
                mounted_in.set(Some(parent_id));

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

        Self::new_marker("list dom node", move |parent_id, comment_id, content| {
            let mut prev_node = comment_id;

            for node in list.iter() {
                let node_id = node.id_dom();
                get_driver_dom().insert_before(parent_id, node_id, Some(prev_node));
                prev_node = node_id;
            }

            // `list` was reversed up front, so document order is back-to-front here.
            content.set(list.iter().rev().map(|node| node.id_dom()).collect());

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
