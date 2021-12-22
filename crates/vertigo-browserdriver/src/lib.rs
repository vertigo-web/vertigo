#![deny(rust_2018_idioms)]
#![allow(clippy::new_ret_no_self)]
#![allow(clippy::module_inception)]

mod driver_browser;
mod modules;
mod utils;

pub use driver_browser::DriverBrowser;

pub mod prelude {
    pub use crate::driver_browser::DriverBrowser;
    pub use wasm_bindgen;
    pub use wasm_bindgen_futures;
    pub use wasm_bindgen::prelude::wasm_bindgen as wasm_bindgen_derive;
}
