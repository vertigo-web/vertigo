use std::any::Any;

use crate::{ApiImport, Driver, DomElement, struct_mut::ValueMut};

pub struct DriverConstruct {
    pub driver: Driver,
    state: ValueMut<Option<Box<dyn Any>>>,
    subscription: ValueMut<Option<DomElement>>,
}

impl DriverConstruct {
    pub fn new(api: ApiImport) -> DriverConstruct {
        let driver = Driver::new(api);

        DriverConstruct {
            driver,
            state: ValueMut::new(None),
            subscription: ValueMut::new(None),
        }
    }

    pub fn set_root(&self, state: Box<dyn Any>, root: DomElement) {
        self.state.set(Some(state));
        self.subscription.set(Some(root));
    }
}

#[cfg(all(not(test), target_arch = "wasm32", target_os = "unknown"))]
mod api {
    mod inner {
        #[link(wasm_import_module = "mod")]
        extern "C" {
            pub fn panic_message(ptr: u32, size: u32);
            pub fn dom_access(ptr: u32, size: u32) -> u32;
        }
    }

    pub mod safe_wrappers {
        use super::inner::*;

        pub fn safe_panic_message(ptr: u32, size: u32) {
            unsafe {
                panic_message(ptr, size)
            }
        }

        pub fn safe_dom_access(ptr: u32, size: u32) -> u32 {
            unsafe {
                dom_access(ptr, size)
            }
        }
    }
}

#[cfg(any(test, not(target_arch = "wasm32"), not(target_os = "unknown")))]
mod api {
    pub mod safe_wrappers {
        pub fn safe_panic_message(_ptr: u32, _size: u32) {
        }

        pub fn safe_dom_access(_ptr: u32, _size: u32) -> u32 {
            0
        }
    }
}


thread_local! {
    pub static DRIVER_BROWSER: DriverConstruct = DriverConstruct::new({
        use api::safe_wrappers::*;

        ApiImport::new(
            safe_panic_message,
            safe_dom_access,
        )
    });
}
