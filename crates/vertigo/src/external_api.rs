use std::{cell::RefCell, any::Any};

use crate::{ApiImport, Driver};

pub struct DriverConstruct {
    pub driver: Driver,
    pub subscription: RefCell<Option<Box<dyn Any>>>,
}

impl DriverConstruct {
    pub fn new(api: ApiImport) -> DriverConstruct {
        let driver = Driver::new(api);

        DriverConstruct {
            driver,
            subscription: RefCell::new(None),
        }
    }
}

#[cfg(all(not(test), target_arch = "wasm32", target_os = "unknown"))]
mod api {
    mod inner {
        #[link(wasm_import_module = "mod")]
        extern "C" {
            pub fn panic_message(ptr: u32, size: u32);
            pub fn js_call(ptr: u32) -> u32;

            pub fn interval_set(duration: u32, callback_id: u32) -> u32;
            pub fn interval_clear(timer_id: u32);
            pub fn timeout_set(duration: u32, callback_id: u32) -> u32;
            pub fn timeout_clear(timer_id: u32);

            pub fn instant_now() -> u32;
        }
    }

    pub mod safe_wrappers {
        use super::inner::*;

        pub fn safe_panic_message(ptr: u32, size: u32) {
            unsafe {
                panic_message(ptr, size)
            }
        }

        pub fn safe_js_call(params: u32) -> u32 {
            unsafe {
                js_call(params)
            }
        }

        pub fn safe_interval_set(duration: u32, callback_id: u32) -> u32 {
            unsafe {
                interval_set(duration, callback_id)
            }
        }

        pub fn safe_interval_clear(timer_id: u32) {
            unsafe {
                interval_clear(timer_id);
            }
        }

        pub fn safe_timeout_set(duration: u32, callback_id: u32) -> u32 {
            unsafe {
                timeout_set(duration, callback_id)
            }
        }

        pub fn safe_timeout_clear(timer_id: u32) {
            unsafe {
                timeout_clear(timer_id)
            }
        }

        pub fn safe_instant_now() -> u32 {
            unsafe {
                instant_now()
            }
        }
    }
}

#[cfg(any(test, not(target_arch = "wasm32"), not(target_os = "unknown")))]
mod api {
    pub mod safe_wrappers {
        pub fn safe_panic_message(_ptr: u32, _size: u32) {
            unimplemented!();
        }

        pub fn safe_js_call(_params: u32) -> u32 {
            println!("safe js call");
            0
        }

        pub fn safe_interval_set(_duration: u32, _callback_id: u32) -> u32 {
            unimplemented!("safe_interval_set");
        }

        pub fn safe_interval_clear(_timer_id: u32) {
            unimplemented!("safe_interval_clear");
        }

        pub fn safe_timeout_set(_duration: u32, _callback_id: u32) -> u32 {
            unimplemented!("safe_timeout_set");
        }

        pub fn safe_timeout_clear(_timer_id: u32) {
            unimplemented!("safe_timeout_clear");
        }

        pub fn safe_instant_now() -> u32 {
            unimplemented!("safe_instant_now");
        }
    }
}


thread_local! {
    pub static DRIVER_BROWSER: DriverConstruct = DriverConstruct::new({
        use api::safe_wrappers::*;

        ApiImport::new(
            safe_panic_message,
            safe_js_call,
            safe_interval_set,
            safe_interval_clear,
            safe_timeout_set,
            safe_timeout_clear,
            safe_instant_now,
        )
    });
}
