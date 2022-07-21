use crate::driver_module::arguments::{params::ParamItem, memory_block::MemoryBlock};

#[derive(Debug)]
pub struct ParamListBuilder {
    list: Vec<ParamItem>,
}

impl ParamListBuilder {
    pub fn new() -> ParamListBuilder {
        ParamListBuilder {
            list: Vec::new(),
        }
    }

    pub fn string(mut self, value: impl Into<String>) -> Self {
        let value = value.into();
        self.list.push(ParamItem::String(value));
        self
    }

    pub fn string_option(self, value: Option<String>) -> Self {
        match value {
            Some(body) => self.string(body),
            None => self.null(),
        }
    }
    pub fn str(mut self, value: &'static str) -> Self {
        self.list.push(ParamItem::String(value.into()));
        self
    }

    pub fn u32(mut self, value: u32) -> Self {
        self.list.push(ParamItem::U32(value));
        self
    }

    #[allow(dead_code)]
    pub fn i32(mut self, value: i32) -> Self {
        self.list.push(ParamItem::I32(value));
        self
    }

    pub fn u64(mut self, value: u64) -> Self {
        self.list.push(ParamItem::U64(value));
        self
    }

    #[allow(dead_code)]
    pub fn i64(mut self, value: i64) -> Self {
        self.list.push(ParamItem::I64(value));
        self
    }

    #[allow(dead_code)]
    pub fn bool(mut self, value: bool) -> Self {
        let value = if value {
            ParamItem::True
        } else {
            ParamItem::False
        };

        self.list.push(value);
        self
    }

    pub fn null(mut self) -> Self {
        self.list.push(ParamItem::Null);
        self
    }

    #[allow(dead_code)]
    pub fn list(mut self, create: impl FnOnce(ParamListBuilder) -> ParamListBuilder ) -> Self {
        let sub_list = ParamListBuilder::new();
        let sub_list = create(sub_list);
        self.list.push(ParamItem::List(sub_list.list));
        self
    }

    pub fn build(self) -> MemoryBlock {
        let list = ParamItem::List(self.list);
        list.to_snapshot()
    }
}
