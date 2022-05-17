use super::arguments::{ArgumentsManager, ListId, ParamsListSnapshot};

enum Param {
    String(String),
    StringStatic(&'static str),

    U32(u32),
    I32(i32),
    U64(u64),
    I64(i64),

    Bool(bool),
    Null,
    Undefined,

    List(ParamListBuilder),
}

pub struct ParamListBuilder{
    arguments: ArgumentsManager,
    list: Vec<Param>,
}

impl ParamListBuilder {
    pub fn new(arguments: &ArgumentsManager) -> ParamListBuilder {
        ParamListBuilder {
            arguments: arguments.clone(),
            list: Vec::new(),
        }
    }

    pub fn string(mut self, value: impl Into<String>) -> Self {
        let value = value.into();
        self.list.push(Param::String(value));
        self
    }

    pub fn string_option(self, value: Option<String>) -> Self {
        match value {
            Some(body) => self.string(body),
            None => self.null(),
        }
    }
    pub fn str(mut self, value: &'static str) -> Self {
        self.list.push(Param::StringStatic(value));
        self
    }

    pub fn u32(mut self, value: u32) -> Self {
        self.list.push(Param::U32(value));
        self
    }

    #[allow(dead_code)]
    pub fn i32(mut self, value: i32) -> Self {
        self.list.push(Param::I32(value));
        self
    }

    pub fn u64(mut self, value: u64) -> Self {
        self.list.push(Param::U64(value));
        self
    }

    #[allow(dead_code)]
    pub fn i64(mut self, value: i64) -> Self {
        self.list.push(Param::I64(value));
        self
    }

    #[allow(dead_code)]
    pub fn bool(mut self, value: bool) -> Self {
        self.list.push(Param::Bool(value));
        self
    }

    pub fn null(mut self) -> Self {
        self.list.push(Param::Null);
        self
    }

    #[allow(dead_code)]
    pub fn undefined(mut self) -> Self {
        self.list.push(Param::Undefined);
        self
    }

    #[allow(dead_code)]
    pub fn list(mut self, create: impl FnOnce(ParamListBuilder) -> ParamListBuilder ) -> Self {
        let sub_list = ParamListBuilder::new(&self.arguments);
        let sub_list = create(sub_list);
        self.list.push(Param::List(sub_list));
        self
    }

    fn convert_to_list(self) -> ListId {
        let id = self.arguments.new_list();

        for item in self.list {
            match item {
                Param::String(value) => {
                    self.arguments.push_string(id, value);
                },
                Param::StringStatic(value) => {
                    self.arguments.push_string_static(id, value);
                }
                Param::Bool(value) => {
                    if value {
                        self.arguments.push_true(id);
                    } else {
                        self.arguments.push_false(id);
                    }
                },
                Param::U32(value) => {
                    self.arguments.push_u32(id, value);
                },
                Param::I32(value) => {
                    self.arguments.push_i32(id, value);
                },
                Param::U64(value) => {
                    self.arguments.push_u64(id, value);
                },
                Param::I64(value) => {
                    self.arguments.push_i64(id, value);
                },
                Param::Null => {
                    self.arguments.push_null(id);
                },
                Param::Undefined => {
                    self.arguments.push_undefined(id);
                },
                Param::List(sublist) => {
                    let sublist_id = sublist.convert_to_list();
                    self.arguments.push_list(id, sublist_id);
                }
            }
        }

        id
    }

    pub fn build(self) -> ParamsListSnapshot {
        let arguments = self.arguments.clone();

        let id = self.convert_to_list();
        arguments.get_snapshot(id)
    }
}
