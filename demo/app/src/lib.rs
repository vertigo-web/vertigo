#![deny(rust_2018_idioms)]

use vertigo_browserdriver::prelude::*;

mod app;

#[wasm_bindgen_derive(start)]
pub async fn start_application() {
    log::info!("Starting application ...");

    let driver = DriverBrowser::new();
    let state = app::State::new(&driver);
    start_browser_app(driver, state, app::render).await;
}
