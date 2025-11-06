use crate::{
    driver_module::api::{arguments::api_arguments, panic_message::api_panic_message},
    DomId, JsValue,
};

#[derive(Default)]
pub struct DomAccess {
    builder: Vec<JsValue>,
}

impl DomAccess {
    #[must_use]
    pub fn api(mut self) -> Self {
        self.builder.push(JsValue::List(vec![JsValue::str("api")]));

        self
    }

    #[must_use]
    pub fn element(mut self, dom_id: DomId) -> Self {
        self.builder.push(JsValue::List(vec![
            JsValue::str("root"),
            JsValue::U64(dom_id.to_u64()),
        ]));

        self
    }

    #[must_use]
    pub fn root(mut self, name: impl Into<String>) -> Self {
        self.builder.push(JsValue::List(vec![
            JsValue::str("root"),
            JsValue::str(name),
        ]));

        self
    }

    #[must_use]
    pub fn get(mut self, name: impl Into<String>) -> Self {
        self.builder
            .push(JsValue::List(vec![JsValue::str("get"), JsValue::str(name)]));

        self
    }

    #[must_use]
    pub fn set(mut self, name: impl Into<String>, value: JsValue) -> Self {
        self.builder.push(JsValue::List(vec![
            JsValue::str("set"),
            JsValue::str(name),
            value,
        ]));

        self
    }

    #[must_use]
    pub fn call(mut self, name: impl Into<String>, params: Vec<JsValue>) -> Self {
        let mut value_params = vec![JsValue::str("call"), JsValue::str(name)];

        value_params.extend(params);

        self.builder.push(JsValue::List(value_params));
        self
    }

    /// Synthetic command that is not meant to be passed to the browser.
    /// It can be used to communicate between WASM and driver implementation, for example SSR
    #[must_use]
    pub fn synthetic(mut self, name: impl Into<String>, params: JsValue) -> Self {
        self.builder.push(JsValue::List(vec![JsValue::str(name)]));
        self.builder.push(params);
        self
    }

    #[must_use]
    pub fn get_props(mut self, props: &[&str]) -> Self {
        let mut new_params = vec![JsValue::str("get_props")];

        new_params.extend(props.iter().map(|item| JsValue::String(item.to_string())));

        self.builder.push(JsValue::List(new_params));
        self
    }

    pub fn exec(self) {
        let result = self.fetch();

        if let JsValue::Undefined = result {
            //ok
        } else {
            let message = format!("Expected undefined dump={result:?}");
            api_panic_message().show(message);
        }
    }

    pub fn fetch(self) -> JsValue {
        use super::external_api::api::safe_wrappers::safe_dom_access;

        let arguments_ptr_long = JsValue::List(self.builder).to_ptr_long();

        let result_long_ptr = safe_dom_access(arguments_ptr_long);
        api_arguments().get_by_long_ptr(result_long_ptr)
    }
}
