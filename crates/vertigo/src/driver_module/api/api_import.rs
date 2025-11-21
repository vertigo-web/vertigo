use crate::{driver_module::js_value::JsValue, JsJson};

use super::api_dom_access::DomAccess;

#[derive(Clone, Default)]
pub struct ApiImport {}

impl ApiImport {
    pub fn dom_bulk_update(&self, value: JsJson) {
        DomAccess::default()
            .api()
            .get("dom")
            .call("dom_bulk_update", vec![JsValue::Json(value)])
            .exec();
    }
}

use vertigo_macro::store;

#[store]
pub fn api_import() -> ApiImport {
    ApiImport::default()
}
