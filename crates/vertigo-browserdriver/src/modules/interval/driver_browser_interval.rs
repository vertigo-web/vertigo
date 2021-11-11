use std::rc::Rc;

use vertigo::utils::DropResource;
use wasm_bindgen::prelude::Closure;

use crate::{utils::callback_manager::CallbackManager};

use super::js_interval::DriverBrowserIntervalJs;

pub struct DriverBrowserInterval {
    driver_js: Rc<DriverBrowserIntervalJs>,
    _closure: Closure<dyn Fn(u64)>,
    callback_manager: CallbackManager<()>,
}

impl DriverBrowserInterval {
    pub fn new() -> DriverBrowserInterval {
        let callback_manager = CallbackManager::new();

        let closure = {
            let callback_manager = callback_manager.clone();

            Closure::new(move |callback_id: u64| {
                log::info!("Interval ... {}", callback_id);

                let callback = callback_manager.get(callback_id);

                if let Some(callback) = callback {
                    callback(&());
                } else {
                    log::error!("Missing callback for id={}", callback_id);
                }
            })
        };

        let driver_js = Rc::new(DriverBrowserIntervalJs::new(&closure));

        DriverBrowserInterval {
            driver_js,
            _closure: closure,
            callback_manager
        }
    }

    pub fn set_interval<F: Fn(&()) + 'static>(&self, time: u32, callback: F) -> DropResource {
        let callback_id = self.callback_manager.set(callback);

        let timer_id = self.driver_js.set_interval(time, callback_id);
        let driver_js = self.driver_js.clone();

        DropResource::new(move || {
            driver_js.clear_interval(timer_id);
        })
    }
}
