use std::rc::Rc;
use vertigo_macro::store;

use crate::{
    computed::DropResource,
    dev::{
        command::{LocationCallbackMode, LocationSetMode, LocationTarget},
        CallbackId,
    },
};

use super::{api_browser_command, CallbackStore};

#[store]
pub fn api_location() -> Rc<ApiLocation> {
    ApiLocation::new()
}

pub struct ApiLocation {
    callback: CallbackStore<String, ()>,
}

impl ApiLocation {
    fn new() -> Rc<ApiLocation> {
        Rc::new(ApiLocation {
            callback: CallbackStore::new(),
        })
    }

    pub fn callback(&self, callback: CallbackId, value: String) {
        self.callback.call(callback, value);
    }

    pub fn get_location(&self, target: LocationTarget) -> String {
        api_browser_command().location_get(target) //LocationTarget::Hash)
    }

    pub fn on_change(
        &self,
        target: LocationTarget,
        callback: impl Fn(String) + 'static,
    ) -> DropResource {
        let (callback, drop) = self.callback.register(callback);

        api_browser_command().location_callback(target, LocationCallbackMode::Add, callback);

        DropResource::new(move || {
            api_browser_command().location_callback(target, LocationCallbackMode::Remove, callback);
            drop.off();
        })
    }

    pub fn push_location(&self, target: LocationTarget, mode: LocationSetMode, new_location: &str) {
        api_browser_command().location_set(target, mode, new_location.to_string());
    }
}
