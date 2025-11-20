use crate::{
    driver_module::{
        api::{api_browser_command, panic_message::api_panic_message},
        js_value::JsValue,
    },
    JsJson,
};

use super::api_dom_access::DomAccess;

#[derive(Clone, Default)]
pub struct ApiImport {}

impl ApiImport {
    pub fn timezone_offset(&self) -> i32 {
        let result = DomAccess::default()
            .api()
            .call("getTimezoneOffset", vec![])
            .fetch();

        if let JsValue::I32(result) = result {
            // Return in seconds to be compatible with chrono
            // Opposite as JS returns the offset backwards
            result * -60
        } else {
            api_panic_message().show(format!(
                "api.timezone_offset -> incorrect result {result:?}"
            ));
            0
        }
    }

    pub fn history_back(&self) {
        DomAccess::default()
            .root("window")
            .get("history")
            .call("back", Vec::new())
            .exec();
    }

    ///////////////////////////////////////////////////////////////////////////////////
    ///////////////////////////////////////////////////////////////////////////////////

    pub fn dom_bulk_update(&self, value: JsJson) {
        DomAccess::default()
            .api()
            .get("dom")
            .call("dom_bulk_update", vec![JsValue::Json(value)])
            .exec();
    }

    pub fn get_random(&self, min: u32, max: u32) -> u32 {
        let result = DomAccess::default()
            .api()
            .call("getRandom", vec![JsValue::U32(min), JsValue::U32(max)])
            .fetch();

        if let JsValue::I32(result) = result {
            result as u32
        } else {
            api_panic_message().show(format!("api.get_random -> incorrect result {result:?}"));
            min
        }
    }

    pub fn route_from_public(&self, path: impl Into<String>) -> String {
        let path: String = path.into();
        if api_browser_command().is_browser() {
            // In the browser use env variable attached during SSR
            let mount_point = api_browser_command()
                .get_env("vertigo-mount-point")
                .unwrap_or_else(|| "/".to_string());
            if mount_point != "/" {
                path.trim_start_matches(&mount_point).to_string()
            } else {
                path
            }
        } else {
            // On the server no need to do anything
            path
        }
    }
}

use vertigo_macro::store;

#[store]
pub fn api_import() -> ApiImport {
    ApiImport::default()
}
