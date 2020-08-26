mod app;
mod app_state;
mod view;

use wasm_bindgen::prelude::*;

#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
extern {
    fn consoleLog(s: &str);
}

#[wasm_bindgen]
pub fn start_app() {
    app::mainApp();
}

