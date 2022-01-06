#![deny(rust_2018_idioms)]
#![allow(clippy::new_ret_no_self)]
#![allow(clippy::module_inception)]
#![allow(clippy::type_complexity)]

mod browser_app;
mod driver_browser;
mod modules;
mod utils;

pub use browser_app::start_browser_app;
pub use driver_browser::DriverBrowser;

pub mod prelude {
    pub use crate::browser_app::start_browser_app;
    pub use crate::driver_browser::DriverBrowser;
    pub use wasm_bindgen;
    pub use wasm_bindgen::prelude::wasm_bindgen as wasm_bindgen_derive;
}
