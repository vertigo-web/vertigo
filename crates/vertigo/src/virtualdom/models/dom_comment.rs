use crate::{Driver, struct_mut::{ValueMut, VecMut}, get_driver, Client};
use super::dom_id::DomId;

pub struct DomComment {
    dom_driver: Driver,
    pub id_dom: DomId,
    pub value: ValueMut<String>,
    on_mount: Option<Box<dyn FnOnce(DomId) -> Client>>,
    subscriptions: VecMut<Client>,
}

impl DomComment {
    pub fn new(text: impl Into<String>) -> DomComment {
        let text = text.into();
        let dom_driver = get_driver();
        let id_dom = DomId::default();

        dom_driver.create_comment(id_dom, text.clone());

        DomComment {
            dom_driver,
            id_dom,
            value: ValueMut::new(text),
            on_mount: None,
            subscriptions: VecMut::new(),
        }
    }

    pub fn id_dom(&self) -> DomId {
        self.id_dom
    }

    pub fn update(&self, new_value: String) {
        let should_update = self.value.set_and_check(new_value.to_string());
        if should_update {
            self.dom_driver.update_comment(self.id_dom, new_value);
        }
    }

    pub(crate) fn set_on_mount(mut self, on_mount: impl FnOnce(DomId) -> Client + 'static) -> Self {
        self.on_mount = Some(Box::new(on_mount));
        self
    }

    pub(crate) fn run_on_mount(mut self, parent_id: DomId) -> Self {
        if self.on_mount.is_some() {
            let on_mount = std::mem::take(&mut self.on_mount);
            if let Some(on_mount) = on_mount {
                let client = on_mount(parent_id);
                self.subscriptions.push(client);
            }
        }

        self
    }

}

impl Drop for DomComment {
    fn drop(&mut self) {
        self.dom_driver.remove_comment(self.id_dom);
    }
}
