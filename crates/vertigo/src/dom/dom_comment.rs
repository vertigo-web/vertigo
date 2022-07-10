use crate::{Driver, struct_mut::VecMut, get_driver, Client};
use super::dom_id::DomId;

pub struct DomComment {
    dom_driver: Driver,
    pub id_dom: DomId,
    on_mount: Option<Box<dyn FnOnce(DomId) -> Client>>,
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
            on_mount: None,
            subscriptions: VecMut::new(),
        }
    }

    pub fn id_dom(&self) -> DomId {
        self.id_dom
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
