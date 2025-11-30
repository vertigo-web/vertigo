use crate::{
    computed::{struct_mut::VecMut, DropResource, ToComputed},
    driver_module::get_driver_dom,
};

use super::dom_id::DomId;

/// A Real DOM representative - text kind
pub struct DomText {
    id_dom: DomId,
    subscriptions: VecMut<DropResource>,
}

impl DomText {
    pub fn new(value: impl Into<String>) -> DomText {
        let value = value.into();
        let id = DomId::default();

        get_driver_dom().create_text(id, &value);

        DomText {
            id_dom: id,
            subscriptions: VecMut::new(),
        }
    }

    pub fn new_computed<T: Into<String> + Clone + PartialEq + 'static>(
        computed: impl ToComputed<T>,
    ) -> Self {
        let text_node = DomText::new(String::new());
        let id_dom = text_node.id_dom;

        let computed = computed.to_computed();
        let client = computed.subscribe(move |value| {
            let value: String = value.into();
            get_driver_dom().update_text(id_dom, &value);
        });

        text_node.subscriptions.push(client);
        text_node
    }

    pub fn id_dom(&self) -> DomId {
        self.id_dom
    }
}

impl Drop for DomText {
    fn drop(&mut self) {
        get_driver_dom().remove_text(self.id_dom);
    }
}
