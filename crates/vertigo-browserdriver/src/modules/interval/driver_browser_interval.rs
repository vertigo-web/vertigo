use std::rc::Rc;
use vertigo::DropResource;

use crate::{utils::callback_manager::{CallbackManager, CallbackManagerOnce}, api::ApiImport};

#[derive(Clone)]
pub struct DriverBrowserInterval {
    api: Rc<ApiImport>,
    interval_callback_manager: CallbackManager<()>,
    timeout_callback_manager: CallbackManagerOnce<()>,
}

impl DriverBrowserInterval {
    pub fn new(api: &Rc<ApiImport>) -> DriverBrowserInterval {
        DriverBrowserInterval {
            api: api.clone(),
            interval_callback_manager: CallbackManager::new(),
            timeout_callback_manager: CallbackManagerOnce::new(),
        }
    }

    pub fn set_interval<F: Fn(&()) + 'static>(&self, duration: u32, callback: F) -> DropResource {
        let callback_id = self.interval_callback_manager.set(callback);

        let timer_id = self.api.interval_set(duration, callback_id);

        DropResource::new({
            let interval_callback_manager = self.interval_callback_manager.clone();
            let api = self.api.clone();
            move || {
                api.interval_clear(timer_id);
                interval_callback_manager.remove(callback_id);
            }
        })
    }

    pub(crate) fn export_interval_run_callback(&self, callback_id: u32) {
        let callback = self.interval_callback_manager.get(callback_id);

        if let Some(callback) = callback {
            callback(&());
        } else {
            log::error!("Missing callback for id={}", callback_id);
        }
    }

    pub fn set_timeout_and_detach<F: FnOnce(()) + 'static>(&self, duration: u32, callback: F) {
        let callback_id = self.timeout_callback_manager.set(callback);

        let _ = self.api.timeout_set(duration, callback_id);
    }

    #[allow(dead_code)]
    pub fn set_timeout<F: FnOnce(()) + 'static>(&self, duration: u32, callback: F) -> DropResource {
        let callback_id = self.timeout_callback_manager.set(callback);

        let timer_id = self.api.timeout_set(duration, callback_id);

        DropResource::new({
            let timeout_callback_manager = self.timeout_callback_manager.clone();
            let api = self.api.clone();
            move || {
                api.timeout_clear(timer_id);
                timeout_callback_manager.remove(callback_id);
            }
        })
    }

    pub(crate) fn export_timeout_run_callback(&self, callback_id: u32) {
        let callback = self.timeout_callback_manager.remove(callback_id);

        if let Some(callback) = callback {
            callback(());
        } else {
            log::error!("Missing callback for id={}", callback_id);
        }
    }
}
