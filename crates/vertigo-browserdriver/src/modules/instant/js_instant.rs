use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen(module = "/src/modules/instant/js_instant.js")]
extern "C" {
    pub fn now() -> f64;
}
