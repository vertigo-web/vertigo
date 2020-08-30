#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

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
    static APP_STATE: RefCell<Application> = {
        RefCell::new(Application::new())
    };
}

#[wasm_bindgen]
pub fn start_app() {
    APP_STATE.with(|state| {
        let state = state.borrow();
        state.start_app();
    });
}
