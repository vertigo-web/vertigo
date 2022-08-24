use crate::driver_module::js_value::{js_value_struct::JsValue, memory_block::MemoryBlock};

#[derive(Debug)]
pub struct JsValueBuilder {
    list: Vec<JsValue>,
}

impl JsValueBuilder {
    pub fn new() -> JsValueBuilder {
        JsValueBuilder {
            list: Vec::new(),
        }
    }

    pub fn string(mut self, value: impl Into<String>) -> Self {
        let value = value.into();
        self.list.push(JsValue::String(value));
        self
    }

    pub fn string_option(self, value: Option<String>) -> Self {
        match value {
            Some(body) => self.string(body),
            None => self.null(),
        }
    }
    pub fn str(mut self, value: &str) -> Self {
        self.list.push(JsValue::String(value.into()));
        self
    }

    pub fn str_push(&mut self, value: &str) {
        self.list.push(JsValue::String(value.into()));
    }

    pub fn u32(mut self, value: u32) -> Self {
        self.list.push(JsValue::U32(value));
        self
    }

    #[allow(dead_code)]
    pub fn i32(mut self, value: i32) -> Self {
        self.list.push(JsValue::I32(value));
        self
    }

    pub fn u64(mut self, value: u64) -> Self {
        self.list.push(JsValue::U64(value));
        self
    }

    #[allow(dead_code)]
    pub fn i64(mut self, value: i64) -> Self {
        self.list.push(JsValue::I64(value));
        self
    }

    #[allow(dead_code)]
    pub fn bool(mut self, value: bool) -> Self {
        let value = if value {
            JsValue::True
        } else {
            JsValue::False
        };

        self.list.push(value);
        self
    }

    pub fn null(mut self) -> Self {
        self.list.push(JsValue::Null);
        self
    }

    pub fn value(mut self, value: JsValue) -> Self {
        self.list.push(value);
        self
    }

    pub fn value_push(&mut self, value: JsValue) {
        self.list.push(value);
    }

    pub fn list(mut self, create: impl FnOnce(JsValueBuilder) -> JsValueBuilder ) -> Self {
        let sub_list = JsValueBuilder::new();
        let sub_list = create(sub_list);
        self.list.push(JsValue::List(sub_list.list));
        self
    }

    #[allow(dead_code)]
    pub fn list_push(mut self, list: Vec<JsValue>) -> Self {
        self.list.push(JsValue::List(list));
        self
    }

    pub fn extend(mut self, list: Vec<JsValue>) -> Self {
        for item in list {
            self.list.push(item);
        }

        self
    }

    pub fn get(self) -> JsValue {
        JsValue::List(self.list)
    }

    pub fn build(self) -> MemoryBlock {
        self.get().to_snapshot()
    }
}

