mod app;

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn startApp() {
    app::startApp();
}

