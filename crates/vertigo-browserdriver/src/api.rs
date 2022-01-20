use std::sync::Arc;

use vertigo::InstantType;
use crate::stack::StackStringAlloc;

fn str_to_pointer(value: &str) -> (u64, u64) {
    let value_ptr = value.as_ptr() as u64;
    let value_len = value.len() as u64;
    (value_ptr, value_len)
}

pub struct ApiLoggerImport {
    pub console_error_1: fn(arg1_ptr: u64, arg1_len: u64),
    pub console_debug_4: fn(
        arg1_ptr: u64, arg1_len: u64,
        arg2_ptr: u64, arg2_len: u64,
        arg3_ptr: u64, arg3_len: u64,
        arg4_ptr: u64, arg4_len: u64,
    ),
    pub console_log_4: fn (
        arg1_ptr: u64, arg1_len: u64,
        arg2_ptr: u64, arg2_len: u64,
        arg3_ptr: u64, arg3_len: u64,
        arg4_ptr: u64, arg4_len: u64,
    ),
    pub console_info_4: fn(
        arg1_ptr: u64, arg1_len: u64,
        arg2_ptr: u64, arg2_len: u64,
        arg3_ptr: u64, arg3_len: u64,
        arg4_ptr: u64, arg4_len: u64,
    ),
    pub console_warn_4: fn(
        arg1_ptr: u64, arg1_len: u64,
        arg2_ptr: u64, arg2_len: u64,
        arg3_ptr: u64, arg3_len: u64,
        arg4_ptr: u64, arg4_len: u64,
    ),
    pub console_error_4: fn(
        arg1_ptr: u64, arg1_len: u64,
        arg2_ptr: u64, arg2_len: u64,
        arg3_ptr: u64, arg3_len: u64,
        arg4_ptr: u64, arg4_len: u64,
    ),
}

impl ApiLoggerImport {
    pub fn console_error_1(&self, arg1: &str) {
        let (arg1_ptr, arg1_len) = str_to_pointer(arg1);
        let console_error_1 = self.console_error_1;
        console_error_1(arg1_ptr, arg1_len);
    }

    pub fn console_debug_4(&self, arg1: &str, arg2: &str, arg3: &str, arg4: &str) {
        let (arg1_ptr, arg1_len) = str_to_pointer(arg1);
        let (arg2_ptr, arg2_len) = str_to_pointer(arg2);
        let (arg3_ptr, arg3_len) = str_to_pointer(arg3);
        let (arg4_ptr, arg4_len) = str_to_pointer(arg4);
        let console_debug_4 = self.console_debug_4;
        console_debug_4(
            arg1_ptr, arg1_len,
            arg2_ptr, arg2_len,
            arg3_ptr, arg3_len,
            arg4_ptr, arg4_len,
        );
    }

    pub fn console_log_4(&self, arg1: &str, arg2: &str, arg3: &str, arg4: &str) {
        let (arg1_ptr, arg1_len) = str_to_pointer(arg1);
        let (arg2_ptr, arg2_len) = str_to_pointer(arg2);
        let (arg3_ptr, arg3_len) = str_to_pointer(arg3);
        let (arg4_ptr, arg4_len) = str_to_pointer(arg4);
        let console_log_4 = self.console_log_4;
        console_log_4(
            arg1_ptr, arg1_len,
            arg2_ptr, arg2_len,
            arg3_ptr, arg3_len,
            arg4_ptr, arg4_len,
        );
    }

    pub fn console_info_4(&self, arg1: &str, arg2: &str, arg3: &str, arg4: &str) {
        let (arg1_ptr, arg1_len) = str_to_pointer(arg1);
        let (arg2_ptr, arg2_len) = str_to_pointer(arg2);
        let (arg3_ptr, arg3_len) = str_to_pointer(arg3);
        let (arg4_ptr, arg4_len) = str_to_pointer(arg4);
        let console_info_4 = self.console_info_4;
        console_info_4(
            arg1_ptr, arg1_len,
            arg2_ptr, arg2_len,
            arg3_ptr, arg3_len,
            arg4_ptr, arg4_len,
        );
    }

    pub fn console_warn_4(&self, arg1: &str, arg2: &str, arg3: &str, arg4: &str) {
        let (arg1_ptr, arg1_len) = str_to_pointer(arg1);
        let (arg2_ptr, arg2_len) = str_to_pointer(arg2);
        let (arg3_ptr, arg3_len) = str_to_pointer(arg3);
        let (arg4_ptr, arg4_len) = str_to_pointer(arg4);
        let console_warn_4 = self.console_warn_4;
        console_warn_4(
            arg1_ptr, arg1_len,
            arg2_ptr, arg2_len,
            arg3_ptr, arg3_len,
            arg4_ptr, arg4_len,
        );
    }

    pub fn console_error_4(&self, arg1: &str, arg2: &str, arg3: &str, arg4: &str) {
        let (arg1_ptr, arg1_len) = str_to_pointer(arg1);
        let (arg2_ptr, arg2_len) = str_to_pointer(arg2);
        let (arg3_ptr, arg3_len) = str_to_pointer(arg3);
        let (arg4_ptr, arg4_len) = str_to_pointer(arg4);
        let console_error_4 = self.console_error_4;
        console_error_4(
            arg1_ptr, arg1_len,
            arg2_ptr, arg2_len,
            arg3_ptr, arg3_len,
            arg4_ptr, arg4_len,
        );
    }
}

pub struct ApiImport {
    pub logger: Arc<ApiLoggerImport>,

    pub cookie_get: fn(name_ptr: u64, name_len: u64),
    pub cookie_set: fn(
        name_ptr: u64, name_len: u64,
        value_ptr: u64, value_len: u64,
        expires_in: u64,
    ),

    pub interval_set: fn(duration: u32, callback_id: u32) -> u32,
    pub interval_clear: fn(timer_id: u32),
    pub timeout_set: fn(duration: u32, callback_id: u32) -> u32,
    pub timeout_clear: fn(timer_id: u32),

    pub instant_now: fn() -> u32,
    pub hashrouter_get_hash_location: fn (),
    pub hashrouter_push_hash_location: fn(new_hash_ptr: u64, new_hash_length: u64),
    pub fetch_send_request: fn (
        request_id: u32,
        method_ptr: u64,
        method_len: u64,
        url_ptr: u64,
        url_len: u64,
        headers_ptr: u64,
        headers_len: u64,
        body_ptr: u64,
        body_len: u64,
    ),
    pub websocket_register_callback: fn(host_ptr: u64, host_len: u64, callback_id: u32),
    pub websocket_unregister_callback: fn(callback_id: u32),
    pub websocket_send_message: fn(callback_id: u32, message_ptr: u64, message_len: u64),
    pub dom_bulk_update: fn(value_ptr: u64, value_len: u64),
    pub dom_get_bounding_client_rect_x: fn(id: u64) -> i32,
    pub dom_get_bounding_client_rect_y: fn(id: u64) -> i32,
    pub dom_get_bounding_client_rect_width: fn(id: u64) -> u32,
    pub dom_get_bounding_client_rect_height: fn(id: u64) -> u32,
    pub dom_scroll_top: fn(node_id: u64) -> i32,
    pub dom_set_scroll_top: fn(node_id: u64, value: i32),
    pub dom_scroll_left: fn(node_id: u64) -> i32,
    pub dom_set_scroll_left: fn(node_id: u64, value: i32),
    pub dom_scroll_width: fn(node_id: u64) -> u32,
    pub dom_scroll_height: fn(node_id: u64) -> u32,
    pub stack: StackStringAlloc,
}

impl ApiImport {

    pub fn new(
        console_error_1: fn(arg1_ptr: u64, arg1_len: u64),
        console_debug_4: fn(
            arg1_ptr: u64, arg1_len: u64,
            arg2_ptr: u64, arg2_len: u64,
            arg3_ptr: u64, arg3_len: u64,
            arg4_ptr: u64, arg4_len: u64,
        ),
        console_log_4: fn (
            arg1_ptr: u64, arg1_len: u64,
            arg2_ptr: u64, arg2_len: u64,
            arg3_ptr: u64, arg3_len: u64,
            arg4_ptr: u64, arg4_len: u64,
        ),
        console_info_4: fn(
            arg1_ptr: u64, arg1_len: u64,
            arg2_ptr: u64, arg2_len: u64,
            arg3_ptr: u64, arg3_len: u64,
            arg4_ptr: u64, arg4_len: u64,
        ),
        console_warn_4: fn(
            arg1_ptr: u64, arg1_len: u64,
            arg2_ptr: u64, arg2_len: u64,
            arg3_ptr: u64, arg3_len: u64,
            arg4_ptr: u64, arg4_len: u64,
        ),
        console_error_4: fn(
            arg1_ptr: u64, arg1_len: u64,
            arg2_ptr: u64, arg2_len: u64,
            arg3_ptr: u64, arg3_len: u64,
            arg4_ptr: u64, arg4_len: u64,
        ),

        cookie_get: fn(name_ptr: u64, name_len: u64),
        cookie_set: fn(
            name_ptr: u64, name_len: u64,
            value_ptr: u64, value_len: u64,
            expires_in: u64,
        ),
        interval_set: fn(duration: u32, callback_id: u32) -> u32,
        interval_clear: fn(timer_id: u32),
        timeout_set: fn(duration: u32, callback_id: u32) -> u32,
        timeout_clear: fn(timer_id: u32),

        instant_now: fn() -> u32,
        hashrouter_get_hash_location: fn (),
        hashrouter_push_hash_location: fn(new_hash_ptr: u64, new_hash_length: u64),
        fetch_send_request: fn (
            request_id: u32,
            method_ptr: u64,
            method_len: u64,
            url_ptr: u64,
            url_len: u64,
            headers_ptr: u64,
            headers_len: u64,
            body_ptr: u64,
            body_len: u64,
        ),
        websocket_register_callback: fn(host_ptr: u64, host_len: u64, callback_id: u32),
        websocket_unregister_callback: fn(callback_id: u32),
        websocket_send_message: fn(callback_id: u32, message_ptr: u64, message_len: u64),
        dom_bulk_update: fn(value_ptr: u64, value_len: u64),
        dom_get_bounding_client_rect_x: fn(id: u64) -> i32,
        dom_get_bounding_client_rect_y: fn(id: u64) -> i32,
        dom_get_bounding_client_rect_width: fn(id: u64) -> u32,
        dom_get_bounding_client_rect_height: fn(id: u64) -> u32,
        dom_scroll_top: fn(node_id: u64) -> i32,
        dom_set_scroll_top: fn(node_id: u64, value: i32),
        dom_scroll_left: fn(node_id: u64) -> i32,
        dom_set_scroll_left: fn(node_id: u64, value: i32),
        dom_scroll_width: fn(node_id: u64) -> u32,
        dom_scroll_height: fn(node_id: u64) -> u32,
    ) -> ApiImport {
        let logger = ApiLoggerImport {
            console_error_1,
            console_debug_4,
            console_log_4,
            console_info_4,
            console_warn_4,
            console_error_4
        };

        ApiImport {
            logger: Arc::new(logger),
            cookie_get,
            cookie_set,
            interval_set,
            interval_clear,
            timeout_set,
            timeout_clear,

            instant_now,
            hashrouter_get_hash_location,
            hashrouter_push_hash_location,
            fetch_send_request,
            websocket_register_callback,
            websocket_unregister_callback,
            websocket_send_message,
            dom_bulk_update,
            dom_get_bounding_client_rect_x,
            dom_get_bounding_client_rect_y,
            dom_get_bounding_client_rect_width,
            dom_get_bounding_client_rect_height,
            dom_scroll_top,
            dom_set_scroll_top,
            dom_scroll_left,
            dom_set_scroll_left,
            dom_scroll_width,
            dom_scroll_height,
            stack: StackStringAlloc::new(),
        }
    }

    pub fn console_error_1(&self, arg1: &str) {
        self.logger.console_error_1(arg1);
    }

    pub fn console_debug_4(&self, arg1: &str, arg2: &str, arg3: &str, arg4: &str) {
        self.logger.console_debug_4(arg1, arg2, arg3, arg4);
    }

    pub fn console_log_4(&self, arg1: &str, arg2: &str, arg3: &str, arg4: &str) {
        self.logger.console_log_4(arg1, arg2, arg3, arg4);
    }

    pub fn console_info_4(&self, arg1: &str, arg2: &str, arg3: &str, arg4: &str) {
        self.logger.console_info_4(arg1, arg2, arg3, arg4);
    }

    pub fn console_warn_4(&self, arg1: &str, arg2: &str, arg3: &str, arg4: &str) {
        self.logger.console_warn_4(arg1, arg2, arg3, arg4);
    }

    pub fn console_error_4(&self, arg1: &str, arg2: &str, arg3: &str, arg4: &str) {
        self.logger.console_error_4(arg1, arg2, arg3, arg4);
    }

    pub fn cookie_get(&self, cname: &str) -> String {
        let (cname_ptr, cname_len) = str_to_pointer(cname);
        let cookies_get = self.cookie_get;
        cookies_get(cname_ptr, cname_len);
        self.stack.pop()
    }

    pub fn cookie_set(&self, cname: &str, cvalue: &str, expires_in: u64) {
        let (cname_ptr, cname_len) = str_to_pointer(cname);
        let (cvalue_ptr, cvalue_len) = str_to_pointer(cvalue);
        let cookies_set = self.cookie_set;
        cookies_set(cname_ptr, cname_len, cvalue_ptr, cvalue_len, expires_in);
    }

    pub fn interval_set(&self, duration: u32, callback_id: u32) -> u32 {
        let interval_set = self.interval_set;
        interval_set(duration, callback_id)
    }

    pub fn interval_clear(&self, timer_id: u32) {
        let interval_clear = self.interval_clear;
        interval_clear(timer_id)
    }

    pub fn timeout_set(&self, duration: u32, callback_id: u32) -> u32 {
        let timeout_set = self.timeout_set;
        timeout_set(duration, callback_id)
    }

    #[allow(dead_code)]
    pub fn timeout_clear(&self, timer_id: u32) {
        let timeout_clear = self.timeout_clear;
        timeout_clear(timer_id)
    }

    pub fn instant_now(&self) -> InstantType {
        let instant_now = self.instant_now;
        instant_now() as InstantType
    }

    pub fn hashrouter_get_hash_location(&self) -> String {
        let hashrouter_get_hash_location = self.hashrouter_get_hash_location;
        hashrouter_get_hash_location();
        self.stack.pop()
    }

    pub fn hashrouter_push_hash_location(&self, new_hash: &str) {
        let (new_hash_ptr, new_hash_len) = str_to_pointer(new_hash);
        let hashrouter_push_hash_location = self.hashrouter_push_hash_location;
        hashrouter_push_hash_location(new_hash_ptr, new_hash_len);
    }

    pub fn fetch_send_request(
        &self,
        request_id: u32,
        method: String,
        url: String,
        headers: String,
        body: Option<String>,
    ) {
        let (method_ptr, method_len) = str_to_pointer(method.as_str());
        let (url_ptr, url_len) = str_to_pointer(url.as_str());
        let (headers_ptr, headers_len) = str_to_pointer(headers.as_str());

        let fetch_send_request = self.fetch_send_request;

        match body {
            Some(body) => {
                let (body_ptr, body_len) = str_to_pointer(body.as_str());

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
                );
            },
            None => {
                let body_ptr = 0;
                let body_len = 0;

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
                );
            }
        };

    }

    pub fn websocket_register_callback(&self, host: &str, callback_id: u32) {
        let (host_ptr, host_len) = str_to_pointer(host);
        let websocket_register_callback = self.websocket_register_callback;
        websocket_register_callback(host_ptr, host_len, callback_id);
    }

    pub fn websocket_unregister_callback(&self, callback_id: u32) {
        let websocket_unregister_callback = self.websocket_unregister_callback;
        websocket_unregister_callback(callback_id);
    }

    pub fn websocket_send_message(&self, callback_id: u32, message: &str) {
        let (message_ptr, message_len) = str_to_pointer(message);
        let websocket_send_message = self.websocket_send_message;
        websocket_send_message(callback_id, message_ptr, message_len);
    }

    pub fn dom_bulk_update(&self, value: &str) {
        let (value_ptr, value_len) = str_to_pointer(value);
        let dom_bulk_update = self.dom_bulk_update;
        dom_bulk_update(value_ptr, value_len);
    }

    pub fn dom_get_bounding_client_rect_x(&self, id: u64) -> i32 {
        let dom_get_bounding_client_rect_x = self.dom_get_bounding_client_rect_x;
        dom_get_bounding_client_rect_x(id)
    }

    pub fn dom_get_bounding_client_rect_y(&self, id: u64) -> i32 {
        let dom_get_bounding_client_rect_y = self.dom_get_bounding_client_rect_y;
        dom_get_bounding_client_rect_y(id)
    }

    pub fn dom_get_bounding_client_rect_width(&self, id: u64) -> u32 {
        let dom_get_bounding_client_rect_width = self.dom_get_bounding_client_rect_width;
        dom_get_bounding_client_rect_width(id)
    }

    pub fn dom_get_bounding_client_rect_height(&self, id: u64) -> u32 {
        let dom_get_bounding_client_rect_height = self.dom_get_bounding_client_rect_height;
        dom_get_bounding_client_rect_height(id)
    }

    pub fn dom_scroll_top(&self, node_id: u64) -> i32 {
        let dom_scroll_top = self.dom_scroll_top;
        dom_scroll_top(node_id)
    }

    pub fn dom_set_scroll_top(&self, node_id: u64, value: i32) {
        let dom_set_scroll_top = self.dom_set_scroll_top;
        dom_set_scroll_top(node_id, value);
    }

    pub fn dom_scroll_left(&self, node_id: u64) -> i32 {
        let dom_scroll_left = self.dom_scroll_left;
        dom_scroll_left(node_id)
    }

    pub fn dom_set_scroll_left(&self, node_id: u64, value: i32) {
        let dom_set_scroll_left = self.dom_set_scroll_left;
        dom_set_scroll_left(node_id, value);
    }

    pub fn dom_scroll_width(&self, node_id: u64) -> u32 {
        let dom_scroll_width = self.dom_scroll_width;
        dom_scroll_width(node_id)
    }

    pub fn dom_scroll_height(&self, node_id: u64) -> u32 {
        let dom_scroll_height = self.dom_scroll_height;
        dom_scroll_height(node_id)
    }
}


// #[macro_export]
// macro_rules! init_app {
//     ($driver:expr, $body:block) => {

//         #[link(wasm_import_module = "mod")]
//         extern "C" {
//             pub fn console_error_1(arg1_ptr: u64, arg1_len: u64);
//             pub fn console_debug_4(
//                 arg1_ptr: u64, arg1_len: u64,
//                 arg2_ptr: u64, arg2_len: u64,
//                 arg3_ptr: u64, arg3_len: u64,
//                 arg4_ptr: u64, arg4_len: u64,
//             );
//             pub fn console_log_4(
//                 arg1_ptr: u64, arg1_len: u64,
//                 arg2_ptr: u64, arg2_len: u64,
//                 arg3_ptr: u64, arg3_len: u64,
//                 arg4_ptr: u64, arg4_len: u64,
//             );
//             pub fn console_info_4(
//                 arg1_ptr: u64, arg1_len: u64,
//                 arg2_ptr: u64, arg2_len: u64,
//                 arg3_ptr: u64, arg3_len: u64,
//                 arg4_ptr: u64, arg4_len: u64,
//             );
//             pub fn console_warn_4(
//                 arg1_ptr: u64, arg1_len: u64,
//                 arg2_ptr: u64, arg2_len: u64,
//                 arg3_ptr: u64, arg3_len: u64,
//                 arg4_ptr: u64, arg4_len: u64,
//             );
//             pub fn console_error_4(
//                 arg1_ptr: u64, arg1_len: u64,
//                 arg2_ptr: u64, arg2_len: u64,
//                 arg3_ptr: u64, arg3_len: u64,
//                 arg4_ptr: u64, arg4_len: u64,
//             );

//             pub fn interval_set(duration: u32, callback_id: u32) -> u32;
//             pub fn interval_clear(timer_id: u32);
//             pub fn timeout_set(duration: u32, callback_id: u32) -> u32;
//             pub fn timeout_clear(timer_id: u32);

//             pub fn instant_now() -> u32;
//             pub fn hashrouter_get_hash_location();
//             pub fn hashrouter_push_hash_location(new_hash_ptr: u64, new_hash_length: u64);
//             pub fn fetch_send_request(
//                 request_id: u32,
//                 method_ptr: u64,
//                 method_len: u64,
//                 url_ptr: u64,
//                 url_len: u64,
//                 headers_ptr: u64,
//                 headers_len: u64,
//                 body_ptr: u64,
//                 body_len: u64,
//             );
//             pub fn websocket_register_callback(host_ptr: u64, host_len: u64, callback_id: u32);
//             pub fn websocket_unregister_callback(callback_id: u32);
//             pub fn websocket_send_message(callback_id: u32, message_ptr: u64, message_len: u64);
//             pub fn dom_bulk_update(value_ptr: u64, value_len: u64);
//             pub fn dom_get_bounding_client_rect_x(id: u64) -> i32;
//             pub fn dom_get_bounding_client_rect_y(id: u64) -> i32;
//             pub fn dom_get_bounding_client_rect_width(id: u64) -> u32;
//             pub fn dom_get_bounding_client_rect_height(id: u64) -> u32;
//             pub fn dom_scroll_top(node_id: u64) -> i32;
//             pub fn dom_set_scroll_top(node_id: u64, value: i32);
//             pub fn dom_scroll_left(node_id: u64) -> i32;
//             pub fn dom_set_scroll_left(node_id: u64, value: i32);
//             pub fn dom_scroll_width(node_id: u64) -> u32;
//             pub fn dom_scroll_height(node_id: u64) -> u32;
//         }

//         use crate::driver_browser::DriverBrowserInner;


//         thread_local! {
//             pub(crate) static DRIVER_BROWSER_INNER: DriverBrowserInner = DriverBrowserInner::new();
//         }

//         fn pop_string() -> String {
//             DRIVER_BROWSER_INNER.with(|state| state.stack.pop())
//         }

//         #[no_mangle]
//         pub fn alloc(len: u64) -> u64 {
//             DRIVER_BROWSER_INNER.with(|state| state.stack.alloc(len as usize) as u64)

//         }

//         #[no_mangle]
//         pub fn alloc_empty_string() {
//             DRIVER_BROWSER_INNER.with(|state| state.stack.alloc_empty_string())
//         }

//         #[no_mangle]
//         pub fn interval_run_callback(callback_id: u32) {
//             DRIVER_BROWSER_INNER.with(|state| state.driver_interval.export_interval_run_callback(callback_id));
//         }

//         #[no_mangle]
//         pub fn timeout_run_callback(callback_id: u32) {
//             DRIVER_BROWSER_INNER.with(|state| state.driver_interval.export_timeout_run_callback(callback_id));
//         }

//         #[no_mangle]
//         pub fn hashrouter_hashchange_callback() {
//             let new_hash = pop_string();
//             DRIVER_BROWSER_INNER.with(|state| state.driver_hashrouter.export_hashrouter_hashchange_callback(new_hash));
//         }

//         #[no_mangle]
//         pub fn fetch_callback(request_id: u32, success: u32, status: u32) {
//             let success = success > 0;
//             let response = pop_string();
//             DRIVER_BROWSER_INNER.with(|state| state.driver_fetch.export_fetch_callback(request_id, success, status, response));
//         }

//         #[no_mangle]
//         pub fn websocket_callback_socket(callback_id: u32) {
//             DRIVER_BROWSER_INNER.with(|state| state.driver_websocket.export_websocket_callback_socket(callback_id));
//         }

//         #[no_mangle]
//         pub fn websocket_callback_message(callback_id: u32) {
//             let message = pop_string();
//             DRIVER_BROWSER_INNER.with(|state| state.driver_websocket.export_websocket_callback_message(callback_id, message));
//         }

//         #[no_mangle]
//         pub fn websocket_callback_close(callback_id: u32) {
//             DRIVER_BROWSER_INNER.with(|state| state.driver_websocket.export_websocket_callback_close(callback_id));

//         }

//         #[no_mangle]
//         pub fn dom_keydown(
//             dom_id: u64,                                                                         // 0 - null
//             alt_key: u32,                                                                        // 0 - false, >0 - true
//             ctrl_key: u32,                                                                       // 0 - false, >0 - true
//             shift_key: u32,                                                                      // 0 - false, >0 - true
//             meta_key: u32                                                                        // 0 - false, >0 - true
//         ) -> u32 {
//             let code = pop_string();
//             let key = pop_string();

//             let dom_id = if dom_id == 0 { None } else { Some(dom_id) };
//             let alt_key = alt_key > 0;
//             let ctrl_key = ctrl_key > 0;
//             let shift_key = shift_key > 0;
//             let meta_key = meta_key > 0;

//             let prevent_default = DRIVER_BROWSER_INNER.with(|state|
//                 state.driver_dom.export_dom_keydown(
//                     dom_id,
//                     key,
//                     code,
//                     alt_key,
//                     ctrl_key,
//                     shift_key,
//                     meta_key
//                 )
//             );

//             match prevent_default {
//                 true => 1,
//                 false => 0
//             }
//         }

//         #[no_mangle]
//         pub fn dom_oninput(dom_id: u64) {
//             let text = pop_string();

//             DRIVER_BROWSER_INNER.with(|state| state.driver_dom.export_dom_oninput(dom_id, text));
//         }

//         #[no_mangle]
//         pub fn dom_mouseover(dom_id: u64) {
//             let dom_id = if dom_id == 0 { None } else { Some(dom_id) };
//             DRIVER_BROWSER_INNER.with(|state| state.driver_dom.export_dom_mouseover(dom_id));
//         }

//         #[no_mangle]
//         pub fn dom_mousedown(dom_id: u64) {
//             DRIVER_BROWSER_INNER.with(|state| state.driver_dom.export_dom_mousedown(dom_id));
//         }

//         #[no_mangle]
//         pub fn start_application($driver) {
//             $body
//         }
//     }
// }
