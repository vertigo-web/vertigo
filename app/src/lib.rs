use wasm_bindgen::prelude::wasm_bindgen;

use vertigo::{
    computed::Dependencies,
    App,
    VDomComponent,
};

use browserdriver::DomDriverBrowser;

mod app;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen(start)]
pub async fn start_app() {
    console_error_panic_hook::set_once();
    wasm_logger::init(wasm_logger::Config::default());

    log::info!("Start rustowego modułu ...");

    let driver = DomDriverBrowser::new();

    let root: Dependencies = Dependencies::default();
    let app_state = app::State::new(&root, &driver);

    //for testing
    // root.set_hook(
    //     Box::new(|| {
    //         log::info!("before start");
    //     }),
    //     Box::new(|| {
    //         log::info!("after end");
    //     })
    // );

    let app = App::new(driver.clone(), VDomComponent::new(app_state, app::render));

    app.start_app().await;
}
