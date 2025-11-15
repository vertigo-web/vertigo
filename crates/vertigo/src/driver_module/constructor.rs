use std::rc::Rc;

use crate::{store, struct_mut::ValueMut, DomNode};

use super::driver::Driver;

#[store]
pub(crate) fn get_browser_driver() -> Rc<DriverConstruct> {
    Rc::new(DriverConstruct::new())
}

pub(crate) struct DriverConstruct {
    pub(crate) driver: Driver,
    subscription: ValueMut<Option<DomNode>>,
}

impl DriverConstruct {
    pub(crate) fn new() -> DriverConstruct {
        let driver = Driver::default();

        DriverConstruct {
            driver,
            subscription: ValueMut::new(None),
        }
    }

    pub(crate) fn set_root(&self, root_view: DomNode) {
        self.subscription.set(Some(root_view));
    }
}
