#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

mod application;
mod app_state;
mod view;

use wasm_bindgen::prelude::*;

use std::cell::RefCell;
use crate::application::applicationStart;

use virtualdom::vdom::App::App;

#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

thread_local! {
    static APP_STATE: RefCell<App> = RefCell::new(applicationStart());
}

#[wasm_bindgen(start)]
pub fn start_app() {
    APP_STATE.with(|state| state.borrow().start_app());
}
