#![deny(rust_2018_idioms)]

use wasm_bindgen::prelude::wasm_bindgen;

use vertigo::start_app;

use vertigo_browserdriver::DriverBrowser;

mod app;

#[wasm_bindgen(start)]
pub async fn start_application() {
    let driver = DriverBrowser::new();
    let app_state = app::State::new(&driver);

    start_app(driver, app_state, app::render).await;
}
