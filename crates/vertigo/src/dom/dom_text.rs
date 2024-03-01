use crate::{
    computed::ToComputed, dom::dom_id::DomId, driver_module::driver::Driver, get_driver,
    struct_mut::VecMut, DropResource,
};

/// A Real DOM representative - text kind
pub struct DomText {
    driver: Driver,
    id_dom: DomId,
    subscriptions: VecMut<DropResource>,
}

impl DomText {
    pub fn new(value: impl Into<String>) -> DomText {
        let value = value.into();
        let id = DomId::default();

        let driver = get_driver();
        driver.inner.dom.create_text(id, &value);

        DomText {
            driver,
            id_dom: id,
            subscriptions: VecMut::new(),
        }
    }

    pub fn new_computed<T: Into<String> + Clone + PartialEq + 'static>(
        computed: impl ToComputed<T>,
    ) -> Self {
        let text_node = DomText::new(String::new());
        let id_dom = text_node.id_dom;
        let driver = get_driver();

        let computed = computed.to_computed();
        let client = computed.subscribe(move |value| {
            let value: String = value.into();
            driver.inner.dom.update_text(id_dom, &value);
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
        self.driver.inner.dom.remove_text(self.id_dom);
    }
}
