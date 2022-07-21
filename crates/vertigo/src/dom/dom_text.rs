use crate::{
    driver_module::driver::Driver,
    dom::dom_id::DomId, get_driver, Client, struct_mut::VecMut, Computed, Value,
};

pub trait ToComputed<T: Clone> {
    fn to_computed_param(&self) -> Computed<T>;
}

impl<T: Clone + 'static> ToComputed<T> for Computed<T> {
    fn to_computed_param(&self) -> Computed<T> {
        self.clone()
    }
}

impl<T: Clone + 'static> ToComputed<T> for &Computed<T> {
    fn to_computed_param(&self) -> Computed<T> {
        (*self).clone()
    }
}

impl<T: Clone + 'static> ToComputed<T> for Value<T> {
    fn to_computed_param(&self) -> Computed<T> {
        self.to_computed()
    }
}

impl<T: Clone + 'static> ToComputed<T> for &Value<T> {
    fn to_computed_param(&self) -> Computed<T> {
        self.to_computed()
    }
}


pub struct DomText {
    dom_driver: Driver,
    id_dom: DomId,
    subscriptions: VecMut<Client>,
}

impl DomText {
    pub fn new(value: impl Into<String>) -> DomText {
        let value = value.into();
        let id = DomId::default();

        let dom_driver = get_driver();
        dom_driver.create_text(id, &value);

        DomText {
            dom_driver,
            id_dom: id,
            subscriptions: VecMut::new(),
        }
    }

    pub fn new_computed<T: Into<String> + Clone + PartialEq + 'static>(computed: impl ToComputed<T>) -> Self {
        let text_node = DomText::new(String::new());
        let id_dom = text_node.id_dom;
        let driver = get_driver();

        let computed = computed.to_computed_param();
        let client = computed.subscribe(move |value| {
            let value: String = value.into();
            driver.update_text(id_dom, &value);
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
        self.dom_driver.remove_text(self.id_dom);
    }
}
