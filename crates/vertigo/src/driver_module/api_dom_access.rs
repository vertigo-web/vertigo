use super::{js_value::js_value_builder::JsValueBuilder, js_value::{Arguments, js_value_struct::JsValue}, api::PanicMessage};

pub struct DomAccess {
    panic_message: PanicMessage,
    arguments: Arguments,
    fn_dom_access: fn(ptr: u32, size: u32) -> u32,
    builder: JsValueBuilder,
}

impl DomAccess {
    #[must_use]
    pub fn new(panic_message: PanicMessage, arguments: Arguments, fn_dom_access: fn(ptr: u32, size: u32) -> u32) -> DomAccess {
        DomAccess {
            panic_message,
            fn_dom_access,
            arguments,
            builder: JsValueBuilder::new(),
        }
    }

    #[must_use]
    pub fn api(mut self) -> Self {
        let value = JsValueBuilder::new()
            .str("api")
            .get();
        
        self.builder.value_push(value);
        self
    }

    #[must_use]
    pub fn element(mut self, dom_id: u64) -> Self {
        let value = JsValueBuilder::new()
            .str("root")
            .u64(dom_id)
            .get();
        
        self.builder.value_push(value);
        self
    }

    #[must_use]
    pub fn root(mut self, name: impl Into<String>) -> Self {
        let value = JsValueBuilder::new()
            .str("root")
            .string(name)
            .get();
        
        self.builder.value_push(value);
        self
    }

    #[must_use]
    pub fn get(mut self, name: impl Into<String>) -> Self {
        let value = JsValueBuilder::new()
            .str("get")
            .string(name)
            .get();
        
        self.builder.value_push(value);
        self
    }

    #[must_use]
    pub fn set(mut self, name: impl Into<String>, value: JsValue) -> Self {
        let value = JsValueBuilder::new()
            .str("set")
            .string(name)
            .value(value)
            .get();
        
        self.builder.value_push(value);
        self
    }

    #[must_use]
    pub fn call(mut self, name: impl Into<String>, params: Vec<JsValue>) -> Self {
        let value = JsValueBuilder::new()
            .str("call")
            .string(name)
            .extend(params);

        self.builder.value_push(value.get());
        self
    }

    #[must_use]
    pub fn get_props(mut self, props: &[&str]) -> Self {
        let value = JsValueBuilder::new()
            .str("get_props")
            .list(|mut list| {
                for prop in props {
                    list.str_push(prop);
                }

                list
            });
            
        self.builder.value_push(value.get());
        self
    }

    pub fn exec(self) {
        let panic_message = self.panic_message;

        let result = self.fetch();

        if let JsValue::Undefined = result {
            //ok
        } else {
            let message = format!("Expected undefined dump={result:?}");
            panic_message.show(message);
        }
    }

    #[must_use]
    pub fn fetch(self) -> JsValue {
        let memory = self.builder.build();
        let (ptr, size) = memory.get_ptr_and_size();

        let result_ptr = (self.fn_dom_access)(ptr, size);
        self.arguments.get_by_ptr(result_ptr)
    }
}
