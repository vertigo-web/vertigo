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

            pub fn cookie_get(cname_ptr: u64, cname_len: u64);
            pub fn cookie_set(
                cname_ptr: u64, cname_len: u64,
                cvalue_ptr: u64, cvalue_len: u64,
                expires_in: u64,
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

    pub mod safe_wrappers {
        use super::inner::*;

        pub fn safe_console_error_1(arg1_ptr: u64, arg1_len: u64) {
            unsafe {
                console_error_1(arg1_ptr, arg1_len);
            }
        }

        pub fn safe_console_debug_4(
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

        pub fn safe_console_log_4(
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

        pub fn safe_console_info_4(
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

        pub fn safe_console_warn_4(
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

        pub fn safe_console_error_4(
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

        pub fn safe_cookie_get(cname_ptr: u64, cname_len: u64) {
            unsafe {
                cookie_get(cname_ptr, cname_len)
            }
        }

        pub fn safe_cookie_set(
            cname_ptr: u64, cname_len: u64,
            cvalue_ptr: u64, cvalue_len: u64,
            expires_in: u64,
        ) {
            unsafe {
                cookie_set(
                    cname_ptr, cname_len,
                    cvalue_ptr, cvalue_len,
                    expires_in,
                );
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

        pub fn safe_hashrouter_get_hash_location() {
            unsafe {
                hashrouter_get_hash_location()
            }
        }

        pub fn safe_hashrouter_push_hash_location(new_hash_ptr: u64, new_hash_length: u64) {
            unsafe {
                hashrouter_push_hash_location(new_hash_ptr, new_hash_length)
            }
        }

        pub fn safe_fetch_send_request(
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

        pub fn safe_websocket_register_callback(host_ptr: u64, host_len: u64, callback_id: u32) {
            unsafe {
                websocket_register_callback(host_ptr, host_len, callback_id)
            }
        }

        pub fn safe_websocket_unregister_callback(callback_id: u32) {
            unsafe {
                websocket_unregister_callback(callback_id);
            }
        }

        pub fn safe_websocket_send_message(callback_id: u32, message_ptr: u64, message_len: u64) {
            unsafe {
                websocket_send_message(callback_id, message_ptr, message_len);
            }
        }

        pub fn safe_dom_bulk_update(value_ptr: u64, value_len: u64) {
            unsafe {
                dom_bulk_update(value_ptr, value_len);
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
        pub fn safe_console_error_1(_arg1_ptr: u64, _arg1_len: u64) {
            unimplemented!();
        }

        pub fn safe_console_debug_4(
            _arg1_ptr: u64, _arg1_len: u64,
            _arg2_ptr: u64, _arg2_len: u64,
            _arg3_ptr: u64, _arg3_len: u64,
            _arg4_ptr: u64, _arg4_len: u64,
        ) {
            unimplemented!("safe_console_debug_4");
        }

        pub fn safe_console_log_4(
            _arg1_ptr: u64, _arg1_len: u64,
            _arg2_ptr: u64, _arg2_len: u64,
            _arg3_ptr: u64, _arg3_len: u64,
            _arg4_ptr: u64, _arg4_len: u64,
        ) {
            unimplemented!("safe_console_log_4");
        }

        pub fn safe_console_info_4(
            _arg1_ptr: u64, _arg1_len: u64,
            _arg2_ptr: u64, _arg2_len: u64,
            _arg3_ptr: u64, _arg3_len: u64,
            _arg4_ptr: u64, _arg4_len: u64,
        ) {
            unimplemented!("safe_console_info_4");
        }

        pub fn safe_console_warn_4(
            _arg1_ptr: u64, _arg1_len: u64,
            _arg2_ptr: u64, _arg2_len: u64,
            _arg3_ptr: u64, _arg3_len: u64,
            _arg4_ptr: u64, _arg4_len: u64,
        ) {
            unimplemented!("safe_console_warn_4");
        }

        pub fn safe_console_error_4(
            _arg1_ptr: u64, _arg1_len: u64,
            _arg2_ptr: u64, _arg2_len: u64,
            _arg3_ptr: u64, _arg3_len: u64,
            _arg4_ptr: u64, _arg4_len: u64,
        ) {
            unimplemented!("safe_console_error_4");
        }

        pub fn safe_cookie_get(_cname_ptr: u64, _cname_len: u64) {
            unimplemented!("safe_cookie_get");
        }

        pub fn safe_cookie_set(
            _cname_ptr: u64, _cname_len: u64,
            _cvalue_ptr: u64, _cvalue_len: u64,
            _expires_in: u64,
        ) {
            unimplemented!("safe_cookie_set");
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

        pub fn safe_hashrouter_get_hash_location() {
            unimplemented!("safe_hashrouter_get_hash_location");
        }

        pub fn safe_hashrouter_push_hash_location(_new_hash_ptr: u64, _new_hash_length: u64) {
            unimplemented!("safe_hashrouter_push_hash_location");
        }

        pub fn safe_fetch_send_request(
            _request_id: u32,
            _method_ptr: u64,
            _method_len: u64,
            _url_ptr: u64,
            _url_len: u64,
            _headers_ptr: u64,
            _headers_len: u64,
            _body_ptr: u64,
            _body_len: u64,
        ) {
            unimplemented!("safe_fetch_send_request");
        }

        pub fn safe_websocket_register_callback(_host_ptr: u64, _host_len: u64, _callback_id: u32) {
            unimplemented!("safe_websocket_register_callback");
        }

        pub fn safe_websocket_unregister_callback(_callback_id: u32) {
            unimplemented!("safe_websocket_unregister_callback");
        }

        pub fn safe_websocket_send_message(_callback_id: u32, _message_ptr: u64, _message_len: u64) {
            unimplemented!("safe_websocket_send_message");
        }

        pub fn safe_dom_bulk_update(_value_ptr: u64, _value_len: u64) {
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
            safe_console_error_1,
            safe_console_debug_4,
            safe_console_log_4,
            safe_console_info_4,
            safe_console_warn_4,
            safe_console_error_4,
            safe_cookie_get,
            safe_cookie_set,
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
