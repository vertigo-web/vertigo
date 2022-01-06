#![deny(rust_2018_idioms)]

use vertigo_browserdriver::prelude::*;

mod app;

#[wasm_bindgen_derive(start)]
pub fn start_application() {
    let driver = DriverBrowser::new();
    let state = app::State::new(&driver);
    start_browser_app(driver, state, app::render);
}
