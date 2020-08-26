mod app;

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern {
    fn consoleLog(s: &str);
}

#[wasm_bindgen]
pub fn startApp() {
    app::startApp();
}

