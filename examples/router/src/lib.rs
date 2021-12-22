#![deny(rust_2018_idioms)]

use vertigo::start_app;

use vertigo_browserdriver::prelude::*;

mod app;

#[wasm_bindgen_derive(start)]
pub async fn start_application() {
    let driver = DriverBrowser::new();
    let app_state = app::State::new(&driver);

    start_app(driver, app_state, app::render).await;
}
