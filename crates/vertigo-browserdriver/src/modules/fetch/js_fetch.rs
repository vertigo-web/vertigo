use wasm_bindgen::prelude::{wasm_bindgen, Closure};

#[wasm_bindgen(module = "/src/modules/fetch/js_fetch.js")]
extern "C" {
    pub type DriverBrowserFetchJs;

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
