#![deny(rust_2018_idioms)]
#![allow(clippy::new_ret_no_self)]
#![allow(clippy::module_inception)]

mod driver_browser;
mod modules;
mod utils;

pub use driver_browser::DriverBrowser;
