#![allow(clippy::new_ret_no_self)]
#![allow(clippy::module_inception)]

mod utils;
mod driver_browser;
mod driver_browser_dom;
mod driver_browser_interval;
mod driver_browser_hashrouter;
mod driver_browser_fetch;

pub use driver_browser::DriverBrowser;
