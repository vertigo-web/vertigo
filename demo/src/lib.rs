#![allow(clippy::nonstandard_macro_braces)]

use wasm_bindgen::prelude::wasm_bindgen;

use vertigo::{
    computed::Dependencies,
    start_app,
    VDomComponent,
};

use vertigo_browserdriver::DriverBrowser;

mod app;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen(start)]
pub async fn start_application() {
    console_error_panic_hook::set_once();
    wasm_logger::init(wasm_logger::Config::default());

    log::info!("Start rustowego modułu ...");

    let root: Dependencies = Dependencies::default();
    let driver = DriverBrowser::new(&root);
    let app_state = app::State::new(&root, &driver);

    start_app(driver, VDomComponent::new(app_state, app::render)).await;
}
