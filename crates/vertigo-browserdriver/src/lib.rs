#![allow(clippy::new_ret_no_self)]

use wasm_bindgen::prelude::*;

mod utils;
mod driver_browser;
mod driver_browser_dom;
mod driver_browser_interval;
mod driver_browser_hashrouter;
mod driver_browser_fetch;

#[wasm_bindgen(module = "/jsdriver/out/driver.js")]
extern "C" {

    type DriverBrowserDomJs;

    #[wasm_bindgen(constructor)]
    pub fn new(
        mouse_down: &Closure<dyn Fn(u64)>,
        mouse_over: &Closure<dyn Fn(Option<u64>)>,
        keydown: &Closure<dyn Fn(
            Option<u64>,
            String,
            String,
            bool,
            bool,
            bool,
            bool,
        ) -> bool>,
        oninput: &Closure<dyn Fn(u64, String)>,
    ) -> DriverBrowserDomJs;
    #[wasm_bindgen(method)]
    pub fn mount_root(this: &DriverBrowserDomJs, id: u64);
    #[wasm_bindgen(method)]
    pub fn create_node(this: &DriverBrowserDomJs, id: u64, name: &str);
    #[wasm_bindgen(method)]
    pub fn set_attribute(this: &DriverBrowserDomJs, id: u64, attr: &str, value: &str);
    #[wasm_bindgen(method)]
    pub fn remove_attribute(this: &DriverBrowserDomJs, id: u64, attr: &str);
    #[wasm_bindgen(method)]
    pub fn remove_node(this: &DriverBrowserDomJs, id: u64);
    #[wasm_bindgen(method)]
    pub fn create_text(this: &DriverBrowserDomJs, id: u64, value: &str);
    #[wasm_bindgen(method)]
    pub fn remove_text(this: &DriverBrowserDomJs, id: u64);
    #[wasm_bindgen(method)]
    pub fn update_text(this: &DriverBrowserDomJs, id: u64, value: &str);
    #[wasm_bindgen(method)]
    pub fn insert_before(this: &DriverBrowserDomJs, parent: u64, child: u64, ref_id: Option<u64>);
    #[wasm_bindgen(method)]
    pub fn insert_css(this: &DriverBrowserDomJs, selector: &str, value: &str);

    #[wasm_bindgen(method)]
    pub fn get_bounding_client_rect_x(this: &DriverBrowserDomJs, id: u64) -> f64;
    #[wasm_bindgen(method)]
    pub fn get_bounding_client_rect_y(this: &DriverBrowserDomJs, id: u64) -> f64;
    #[wasm_bindgen(method)]
    pub fn get_bounding_client_rect_width(this: &DriverBrowserDomJs, id: u64) -> f64;
    #[wasm_bindgen(method)]
    pub fn get_bounding_client_rect_height(this: &DriverBrowserDomJs, id: u64) -> f64;

    #[wasm_bindgen(method)]
    pub fn scroll_top(this: &DriverBrowserDomJs, node_id: u64) -> i32;
    #[wasm_bindgen(method)]
    pub fn set_scroll_top(this: &DriverBrowserDomJs, node_id: u64, value: i32);
    #[wasm_bindgen(method)]
    pub fn scroll_left(this: &DriverBrowserDomJs, node_id: u64) -> i32;
    #[wasm_bindgen(method)]
    pub fn set_scroll_left(this: &DriverBrowserDomJs, node_id: u64, value: i32);
    #[wasm_bindgen(method)]
    pub fn scroll_width(this: &DriverBrowserDomJs, node_id: u64) -> i32;
    #[wasm_bindgen(method)]
    pub fn scroll_height(this: &DriverBrowserDomJs, node_id: u64) -> i32;


    type DriverBrowserIntervalJs;

    #[wasm_bindgen(constructor)]
    pub fn new(callback: &Closure<dyn Fn(u64)>) -> DriverBrowserIntervalJs;
    #[wasm_bindgen(method)]
    pub fn set_interval(this: &DriverBrowserIntervalJs, duration: u32, callback_id: u64) -> u32;
    #[wasm_bindgen(method)]
    pub fn clear_interval(this: &DriverBrowserIntervalJs, timer_id: u32);


    type DriverBrowserHashRouteJs;

    #[wasm_bindgen(constructor)]
    pub fn new(callback: &Closure<dyn Fn(String)>) -> DriverBrowserHashRouteJs;
    #[wasm_bindgen(method)]
    pub fn get_hash_location(this: &DriverBrowserHashRouteJs) -> String;
    #[wasm_bindgen(method)]
    pub fn push_hash_location(this: &DriverBrowserHashRouteJs, new_hash: String);

    type DriverBrowserFetchJs;

    #[wasm_bindgen(constructor)]
    pub fn new(callback: &Closure<dyn Fn(u64, bool, String)>) -> DriverBrowserFetchJs;
    #[wasm_bindgen(method)]
    pub fn send_request(
        this: &DriverBrowserFetchJs,
        request_id: u64,
        method: String,
        url: String,
        headers: String,
        body: Option<String>
    );
}

pub use driver_browser::DriverBrowser;
