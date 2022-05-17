use std::cell::RefCell;

use crate::{ApiImport, Client, Driver};

pub struct DriverConstruct {
    pub driver: Driver,
    pub subscription: RefCell<Option<Client>>,
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
            pub fn dom_get_bounding_client_rect_x(id: u64) -> i32;
            pub fn dom_get_bounding_client_rect_y(id: u64) -> i32;
            pub fn dom_get_bounding_client_rect_width(id: u64) -> u32;
            pub fn dom_get_bounding_client_rect_height(id: u64) -> u32;
            pub fn dom_scroll_top(node_id: u64) -> i32;
            pub fn dom_set_scroll_top(node_id: u64, value: i32);
            pub fn dom_scroll_left(node_id: u64) -> i32;
            pub fn dom_set_scroll_left(node_id: u64, value: i32);
            pub fn dom_scroll_width(node_id: u64) -> u32;
            pub fn dom_scroll_height(node_id: u64) -> u32;
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

        pub fn safe_dom_get_bounding_client_rect_x(id: u64) -> i32 {
            unsafe {
                dom_get_bounding_client_rect_x(id)
            }
        }

        pub fn safe_dom_get_bounding_client_rect_y(id: u64) -> i32 {
            unsafe {
                dom_get_bounding_client_rect_y(id)
            }
        }

        pub fn safe_dom_get_bounding_client_rect_width(id: u64) -> u32 {
            unsafe {
                dom_get_bounding_client_rect_width(id)
            }
        }

        pub fn safe_dom_get_bounding_client_rect_height(id: u64) -> u32 {
            unsafe {
                dom_get_bounding_client_rect_height(id)
            }
        }

        pub fn safe_dom_scroll_top(node_id: u64) -> i32 {
            unsafe {
                dom_scroll_top(node_id)
            }
        }

        pub fn safe_dom_set_scroll_top(node_id: u64, value: i32) {
            unsafe {
                dom_set_scroll_top(node_id, value)
            }
        }

        pub fn safe_dom_scroll_left(node_id: u64) -> i32 {
            unsafe {
                dom_scroll_left(node_id)
            }
        }

        pub fn safe_dom_set_scroll_left(node_id: u64, value: i32) {
            unsafe {
                dom_set_scroll_left(node_id, value)
            }
        }

        pub fn safe_dom_scroll_width(node_id: u64) -> u32 {
            unsafe {
                dom_scroll_width(node_id)
            }
        }

        pub fn safe_dom_scroll_height(node_id: u64) -> u32 {
            unsafe {
                dom_scroll_height(node_id)
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

        pub fn safe_dom_get_bounding_client_rect_x(_id: u64) -> i32 {
            unimplemented!("safe_dom_get_bounding_client_rect_x");
        }

        pub fn safe_dom_get_bounding_client_rect_y(_id: u64) -> i32 {
            unimplemented!("safe_dom_get_bounding_client_rect_y");
        }

        pub fn safe_dom_get_bounding_client_rect_width(_id: u64) -> u32 {
            unimplemented!("safe_dom_get_bounding_client_rect_width");
        }

        pub fn safe_dom_get_bounding_client_rect_height(_id: u64) -> u32 {
            unimplemented!("safe_dom_get_bounding_client_rect_height");
        }

        pub fn safe_dom_scroll_top(_node_id: u64) -> i32 {
            unimplemented!("safe_dom_scroll_top");
        }

        pub fn safe_dom_set_scroll_top(_node_id: u64, _value: i32) {
            unimplemented!("safe_dom_set_scroll_top");
        }

        pub fn safe_dom_scroll_left(_node_id: u64) -> i32 {
            unimplemented!("safe_dom_scroll_left");
        }

        pub fn safe_dom_set_scroll_left(_node_id: u64, _value: i32) {
            unimplemented!("safe_dom_set_scroll_left");
        }

        pub fn safe_dom_scroll_width(_node_id: u64) -> u32 {
            unimplemented!("safe_dom_scroll_width");
        }

        pub fn safe_dom_scroll_height(_node_id: u64) -> u32 {
            unimplemented!("safe_dom_scroll_width");
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
            safe_dom_get_bounding_client_rect_x,
            safe_dom_get_bounding_client_rect_y,
            safe_dom_get_bounding_client_rect_width,
            safe_dom_get_bounding_client_rect_height,
            safe_dom_scroll_top,
            safe_dom_set_scroll_top,
            safe_dom_scroll_left,
            safe_dom_set_scroll_left,
            safe_dom_scroll_width,
            safe_dom_scroll_height
        )
    });
}
