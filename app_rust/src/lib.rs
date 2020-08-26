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

#[wasm_bindgen]
extern {
    fn consoleLog(s: &str);
}

#[wasm_bindgen]
pub fn start_app() {
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
