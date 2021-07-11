use wasm_bindgen::prelude::{wasm_bindgen, Closure};

#[wasm_bindgen(module = "/src/driver_browser_interval/driver_browser_interval_js.js")]
extern "C" {
    pub type DriverBrowserIntervalJs;

    #[wasm_bindgen(constructor)]
    pub fn new(callback: &Closure<dyn Fn(u64)>) -> DriverBrowserIntervalJs;
    #[wasm_bindgen(method)]
    pub fn set_interval(this: &DriverBrowserIntervalJs, duration: u32, callback_id: u64) -> u32;
    #[wasm_bindgen(method)]
    pub fn clear_interval(this: &DriverBrowserIntervalJs, timer_id: u32);
}
