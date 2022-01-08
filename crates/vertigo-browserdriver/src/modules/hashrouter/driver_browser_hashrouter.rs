use std::rc::Rc;

use vertigo::utils::DropResource;

use crate::{utils::callback_manager::CallbackManager, api::ApiImport};

#[derive(Clone)]
pub struct DriverBrowserHashrouter {
    api: Rc<ApiImport>,
    callback_manager: CallbackManager<String>,
}

impl DriverBrowserHashrouter {
    pub fn new(api: &Rc<ApiImport>) -> DriverBrowserHashrouter {
        let callback_manager = CallbackManager::new();

        DriverBrowserHashrouter {
            api: api.clone(),
            callback_manager,
        }
    }

    pub fn export_hashrouter_hashchange_callback(&self, new_hash: String) {
        self.callback_manager.trigger(new_hash);
    }

    pub fn on_hash_route_change<F: Fn(&String) + 'static>(&self, callback: F) -> DropResource {
        let callback_id = self.callback_manager.set(callback);
        let callback_manager = self.callback_manager.clone();

        DropResource::new(move || {
            callback_manager.remove(callback_id);
        })
    }

    pub fn get_hash_location(&self) -> String {
        self.api.hashrouter_get_hash_location()
    }

    pub fn push_hash_location(&self, hash: &str) {
        self.api.hashrouter_push_hash_location(hash);
    }
}
