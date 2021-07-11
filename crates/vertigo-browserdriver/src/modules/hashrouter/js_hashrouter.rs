use wasm_bindgen::prelude::{wasm_bindgen, Closure};

#[wasm_bindgen(module = "/src/modules/hashrouter/js_hashrouter.js")]
extern "C" {
    pub type DriverBrowserHashRouteJs;

    #[wasm_bindgen(constructor)]
    pub fn new(callback: &Closure<dyn Fn(String)>) -> DriverBrowserHashRouteJs;
    #[wasm_bindgen(method)]
    pub fn get_hash_location(this: &DriverBrowserHashRouteJs) -> String;
    #[wasm_bindgen(method)]
    pub fn push_hash_location(this: &DriverBrowserHashRouteJs, new_hash: String);
}
