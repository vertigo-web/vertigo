use vertigo::utils::DropResource;
use wasm_bindgen::prelude::Closure;

use crate::utils::callback_manager::CallbackManager;

use super::js_hashrouter::DriverBrowserHashRouteJs;

pub struct DriverBrowserHashrouter {
    driver: DriverBrowserHashRouteJs,
    _closure: Closure<dyn Fn(String)>,
    callback_manager: CallbackManager<String>,
}

impl DriverBrowserHashrouter {
    pub fn new() -> DriverBrowserHashrouter {
        let callback_manager = CallbackManager::new();

        let closure: Closure<dyn Fn(String)> = {
            let callback_manager = callback_manager.clone();

            Closure::new(move |new_hash: String| {
                callback_manager.trigger(new_hash);
            })
        };

        let driver = DriverBrowserHashRouteJs::new(&closure);

        DriverBrowserHashrouter {
            driver,
            _closure: closure,
            callback_manager,
        }
    }

    pub fn on_hash_route_change<F: Fn(&String) + 'static>(&self, callback: F) -> DropResource {
        let callback_id = self.callback_manager.set(callback);
        let callback_manager = self.callback_manager.clone();

        DropResource::new(move || {
            callback_manager.remove(callback_id);
        })
    }

    pub fn get_hash_location(&self) -> String {
        self.driver.get_hash_location()
    }

    pub fn push_hash_location(&self, hash: &str) {
        self.driver.push_hash_location(hash.into());
    }
}
