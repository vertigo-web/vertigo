mod application;
mod app_state;
mod view;

use wasm_bindgen::prelude::*;

use std::cell::RefCell;
use crate::application::Application;

#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;


thread_local! {
    static appState: RefCell<Application> = RefCell::new(Application::new());
}


#[wasm_bindgen(module = "/src/driver.js")]
extern "C" {
    fn add(a: u32, b: u32) -> u32;
    fn consoleLog(message: &str);
    fn startDriverLoop();
}

#[wasm_bindgen]
pub fn start_app() {
    startDriverLoop();

    let aa = add(3, 4);
    log::info!("z funkcji add ... {}", aa);
    consoleLog("aaaarrr333");

    appState.with(|state| {
        let state = state.borrow_mut();
        state.start_app();
    });
}

#[wasm_bindgen]
pub fn click_button() {

    appState.with(|state| {
        let state = state.borrow_mut();
        state.increment();
        consoleLog("Zwiększyłem licznik");
    });
}
