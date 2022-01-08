#![deny(rust_2018_idioms)]

use vertigo_browserdriver::start_browser_app;

mod app;

#[no_mangle]
pub fn start_application() {
    start_browser_app(app::State::new, app::render);
}
