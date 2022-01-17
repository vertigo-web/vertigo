#![deny(rust_2018_idioms)]
#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]

mod driver_browser;
mod modules;
mod utils;
mod stack;
mod api;
mod init_env;

use vertigo::{start_app, Computed, VDomElement};
use driver_browser::DriverConstruct;
pub use api::ApiImport;
use vertigo::{Driver};


mod inner_unsafe {
    #[link(wasm_import_module = "mod")]
    extern "C" {
        pub fn console_error_1(arg1_ptr: u64, arg1_len: u64);
        pub fn console_debug_4(
            arg1_ptr: u64, arg1_len: u64,
            arg2_ptr: u64, arg2_len: u64,
            arg3_ptr: u64, arg3_len: u64,
            arg4_ptr: u64, arg4_len: u64,
        );
        pub fn console_log_4(
            arg1_ptr: u64, arg1_len: u64,
            arg2_ptr: u64, arg2_len: u64,
            arg3_ptr: u64, arg3_len: u64,
            arg4_ptr: u64, arg4_len: u64,
        );
        pub fn console_info_4(
            arg1_ptr: u64, arg1_len: u64,
            arg2_ptr: u64, arg2_len: u64,
            arg3_ptr: u64, arg3_len: u64,
            arg4_ptr: u64, arg4_len: u64,
        );
        pub fn console_warn_4(
            arg1_ptr: u64, arg1_len: u64,
            arg2_ptr: u64, arg2_len: u64,
            arg3_ptr: u64, arg3_len: u64,
            arg4_ptr: u64, arg4_len: u64,
        );
        pub fn console_error_4(
            arg1_ptr: u64, arg1_len: u64,
            arg2_ptr: u64, arg2_len: u64,
            arg3_ptr: u64, arg3_len: u64,
            arg4_ptr: u64, arg4_len: u64,
        );

        pub fn interval_set(duration: u32, callback_id: u32) -> u32;
        pub fn interval_clear(timer_id: u32);
        pub fn timeout_set(duration: u32, callback_id: u32) -> u32;
        pub fn timeout_clear(timer_id: u32);

        pub fn instant_now() -> u32;
        pub fn hashrouter_get_hash_location();
        pub fn hashrouter_push_hash_location(new_hash_ptr: u64, new_hash_length: u64);
        pub fn fetch_send_request(
            request_id: u32,
            method_ptr: u64,
            method_len: u64,
            url_ptr: u64,
            url_len: u64,
            headers_ptr: u64,
            headers_len: u64,
            body_ptr: u64,
            body_len: u64,
        );
        pub fn websocket_register_callback(host_ptr: u64, host_len: u64, callback_id: u32);
        pub fn websocket_unregister_callback(callback_id: u32);
        pub fn websocket_send_message(callback_id: u32, message_ptr: u64, message_len: u64);
        pub fn dom_bulk_update(value_ptr: u64, value_len: u64);
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

thread_local! {
    static DRIVER_BROWSER: DriverConstruct = DriverConstruct::new({
        use inner_unsafe::*;

        fn safe_console_error_1(arg1_ptr: u64, arg1_len: u64) {
            unsafe {
                console_error_1(arg1_ptr, arg1_len);
            }
        }

        fn safe_console_debug_4(
            arg1_ptr: u64, arg1_len: u64,
            arg2_ptr: u64, arg2_len: u64,
            arg3_ptr: u64, arg3_len: u64,
            arg4_ptr: u64, arg4_len: u64,
        ) {
            unsafe {
                console_debug_4(
                    arg1_ptr, arg1_len,
                    arg2_ptr, arg2_len,
                    arg3_ptr, arg3_len,
                    arg4_ptr, arg4_len,
                );
            }
        }

        fn safe_console_log_4(
            arg1_ptr: u64, arg1_len: u64,
            arg2_ptr: u64, arg2_len: u64,
            arg3_ptr: u64, arg3_len: u64,
            arg4_ptr: u64, arg4_len: u64,
        ) {
            unsafe {
                console_log_4(
                    arg1_ptr, arg1_len,
                    arg2_ptr, arg2_len,
                    arg3_ptr, arg3_len,
                    arg4_ptr, arg4_len,
                );
            }
        }
        
        fn safe_console_info_4(
            arg1_ptr: u64, arg1_len: u64,
            arg2_ptr: u64, arg2_len: u64,
            arg3_ptr: u64, arg3_len: u64,
            arg4_ptr: u64, arg4_len: u64,
        ) {
            unsafe {
                console_info_4(
                    arg1_ptr, arg1_len,
                    arg2_ptr, arg2_len,
                    arg3_ptr, arg3_len,
                    arg4_ptr, arg4_len,
                );
            }
        }

        fn safe_console_warn_4(
            arg1_ptr: u64, arg1_len: u64,
            arg2_ptr: u64, arg2_len: u64,
            arg3_ptr: u64, arg3_len: u64,
            arg4_ptr: u64, arg4_len: u64,
        ) {
            unsafe {
                console_warn_4(
                    arg1_ptr, arg1_len,
                    arg2_ptr, arg2_len,
                    arg3_ptr, arg3_len,
                    arg4_ptr, arg4_len,
                );
            }
        }

        fn safe_console_error_4(
            arg1_ptr: u64, arg1_len: u64,
            arg2_ptr: u64, arg2_len: u64,
            arg3_ptr: u64, arg3_len: u64,
            arg4_ptr: u64, arg4_len: u64,
        ) {
            unsafe {
                console_error_4(
                    arg1_ptr, arg1_len,
                    arg2_ptr, arg2_len,
                    arg3_ptr, arg3_len,
                    arg4_ptr, arg4_len,
                );
            }
        }

        fn safe_interval_set(duration: u32, callback_id: u32) -> u32 {
            unsafe {
                interval_set(duration, callback_id)
            }
        }

        fn safe_interval_clear(timer_id: u32) {
            unsafe {
                interval_clear(timer_id);
            }
        }

        fn safe_timeout_set(duration: u32, callback_id: u32) -> u32 {
            unsafe {
                timeout_set(duration, callback_id)
            }
        }

        fn safe_timeout_clear(timer_id: u32) {
            unsafe {
                timeout_clear(timer_id)
            }
        }

        fn safe_instant_now() -> u32 {
            unsafe {
                instant_now()
            }
        }

        fn safe_hashrouter_get_hash_location() {
            unsafe {
                hashrouter_get_hash_location()
            }
        }

        fn safe_hashrouter_push_hash_location(new_hash_ptr: u64, new_hash_length: u64) {
            unsafe {
                hashrouter_push_hash_location(new_hash_ptr, new_hash_length)
            }
        }

        fn safe_fetch_send_request(
            request_id: u32,
            method_ptr: u64,
            method_len: u64,
            url_ptr: u64,
            url_len: u64,
            headers_ptr: u64,
            headers_len: u64,
            body_ptr: u64,
            body_len: u64,
        ) {
            unsafe {
                fetch_send_request(
                    request_id,
                    method_ptr,
                    method_len,
                    url_ptr,
                    url_len,
                    headers_ptr,
                    headers_len,
                    body_ptr,
                    body_len,
                )
            }
        }

        fn safe_websocket_register_callback(host_ptr: u64, host_len: u64, callback_id: u32) {
            unsafe {
                websocket_register_callback(host_ptr, host_len, callback_id)
            }
        }

        fn safe_websocket_unregister_callback(callback_id: u32) {
            unsafe {
                websocket_unregister_callback(callback_id);
            }
        }

        fn safe_websocket_send_message(callback_id: u32, message_ptr: u64, message_len: u64) {
            unsafe {
                websocket_send_message(callback_id, message_ptr, message_len);
            }
        }

        fn safe_dom_bulk_update(value_ptr: u64, value_len: u64) {
            unsafe {
                dom_bulk_update(value_ptr, value_len);
            }
        }

        fn safe_dom_get_bounding_client_rect_x(id: u64) -> i32 {
            unsafe {
                dom_get_bounding_client_rect_x(id)
            }
        }
        fn safe_dom_get_bounding_client_rect_y(id: u64) -> i32 {
            unsafe {
                dom_get_bounding_client_rect_y(id)
            }
        }

        fn safe_dom_get_bounding_client_rect_width(id: u64) -> u32 {
            unsafe {
                dom_get_bounding_client_rect_width(id)
            }
        }

        fn safe_dom_get_bounding_client_rect_height(id: u64) -> u32 {
            unsafe {
                dom_get_bounding_client_rect_height(id)
            }
        }

        fn safe_dom_scroll_top(node_id: u64) -> i32 {
            unsafe {
                dom_scroll_top(node_id)
            }
        }

        fn safe_dom_set_scroll_top(node_id: u64, value: i32) {
            unsafe {
                dom_set_scroll_top(node_id, value)
            }
        }

        fn safe_dom_scroll_left(node_id: u64) -> i32 {
            unsafe {
                dom_scroll_left(node_id)
            }
        }

        fn safe_dom_set_scroll_left(node_id: u64, value: i32) {
            unsafe {
                dom_set_scroll_left(node_id, value)
            }
        }

        fn safe_dom_scroll_width(node_id: u64) -> u32 {
            unsafe {
                dom_scroll_width(node_id)
            }
        }

        fn safe_dom_scroll_height(node_id: u64) -> u32 {
            unsafe {
                dom_scroll_height(node_id)
            }
        }

        ApiImport::new(
            safe_console_error_1,
            safe_console_debug_4,
            safe_console_log_4,
            safe_console_info_4,
            safe_console_warn_4,
            safe_console_error_4,
            safe_interval_set,
            safe_interval_clear,
            safe_timeout_set,
            safe_timeout_clear,
            safe_instant_now,
            safe_hashrouter_get_hash_location,
            safe_hashrouter_push_hash_location,
            safe_fetch_send_request,
            safe_websocket_register_callback,
            safe_websocket_unregister_callback,
            safe_websocket_send_message,
            safe_dom_bulk_update,
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

#[no_mangle]
pub fn alloc(len: u64) -> u64 {
    DRIVER_BROWSER.with(|state| state.driver_inner.alloc(len))
}

#[no_mangle]
pub fn alloc_empty_string() {
    DRIVER_BROWSER.with(|state| state.driver_inner.alloc_empty_string())
}

#[no_mangle]
pub fn interval_run_callback(callback_id: u32) {
    DRIVER_BROWSER.with(|state| state.driver_inner.export_interval_run_callback(callback_id));
}

#[no_mangle]
pub fn timeout_run_callback(callback_id: u32) {
    DRIVER_BROWSER.with(|state| state.driver_inner.export_timeout_run_callback(callback_id));
}

#[no_mangle]
pub fn hashrouter_hashchange_callback() {
    DRIVER_BROWSER.with(|state| state.driver_inner.export_hashrouter_hashchange_callback());
}

#[no_mangle]
pub fn fetch_callback(request_id: u32, success: u32, status: u32) {
    DRIVER_BROWSER.with(|state| state.driver_inner.export_fetch_callback(request_id, success, status));
}

#[no_mangle]
pub fn websocket_callback_socket(callback_id: u32) {
    DRIVER_BROWSER.with(|state| state.driver_inner.export_websocket_callback_socket(callback_id));
}

#[no_mangle]
pub fn websocket_callback_message(callback_id: u32) {
    DRIVER_BROWSER.with(|state| state.driver_inner.export_websocket_callback_message(callback_id));
}

#[no_mangle]
pub fn websocket_callback_close(callback_id: u32) {
    DRIVER_BROWSER.with(|state| state.driver_inner.export_websocket_callback_close(callback_id));
}

#[no_mangle]
pub fn dom_keydown(dom_id: u64, alt_key: u32, ctrl_key: u32, shift_key: u32, meta_key: u32) -> u32 {
    DRIVER_BROWSER.with(|state|
        state.driver_inner.export_dom_keydown(
            dom_id,
            alt_key,
            ctrl_key,
            shift_key,
            meta_key
        )
    )
}

#[no_mangle]
pub fn dom_oninput(dom_id: u64) {
    DRIVER_BROWSER.with(|state| state.driver_inner.export_dom_oninput(dom_id));
}

#[no_mangle]
pub fn dom_mouseover(dom_id: u64) {
    DRIVER_BROWSER.with(|state| state.driver_inner.export_dom_mouseover(dom_id));
}

#[no_mangle]
pub fn dom_mousedown(dom_id: u64) {
    DRIVER_BROWSER.with(|state| state.driver_inner.export_dom_mousedown(dom_id));
}

pub fn start_browser_app<
    T: PartialEq + 'static,
>(create_state: fn(&Driver) -> Computed<T>, render: fn(&Computed<T>) -> VDomElement) {
    DRIVER_BROWSER.with(|state| {
        state.driver_inner.init_env();
        let driver = state.driver.clone();
        let app_state = create_state(&driver);

        let client = start_app(driver, app_state, render);

        let mut inner = state.subscription.borrow_mut();
        *inner = Some(client);
    });
}