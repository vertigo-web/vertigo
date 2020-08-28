mod application;
mod app_state;
mod view;
mod driver;

use wasm_bindgen::prelude::*;

use std::cell::RefCell;
use crate::application::Application;

use driver::DomDriverBrowser;

#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

thread_local! {
    static appState: RefCell<Application> = {
        println!("Tutaj trzeba będzie powołać do zycia obiekt drivera przegladarkowego który będzie odwoływał się do tych funkcji powyzej");
        let driver = DomDriverBrowser::new();
        RefCell::new(Application::new(driver))
    };
}

#[wasm_bindgen]
pub fn start_app() {
    appState.with(|state| {
        let state = state.borrow_mut();
        state.start_app();

        state.driver.consoleLog("wystartowano aplikację webassemblerową");
    });
}
