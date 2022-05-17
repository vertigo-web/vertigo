use std::collections::VecDeque;

use super::alloc::{WasmMemorySize, WasmMemoryPtr};
use super::alloc_string::AllocString;
use super::alloc_buffer::AllocBuffer;

const PARAM_TYPE: u32 = 1;
const STRING_PTR: u32 = 4;
const STRING_SIZE: u32 = 4;
const BUFF_PTR: u32 = 4;
const BUFF_SIZE: u32 = 4;
const LIST_COUNT: u32 = 2;

#[repr(u8)]
pub enum ParamItemNumber {
    StringEmpty = 1,
    StringAlloc = 2,
    StringStatic = 3,
    String = 4,
    U32 = 5,
    I32 = 6,
    U64 = 7,
    I64 = 8,
    True = 9,
    False = 10,
    Null = 11,
    Undefined = 12,
    List = 13,
    Vec = 14,
}

#[derive(Debug)]
pub enum ParamItem {
    StringEmpty,                //from js for rust
    StringAlloc(AllocString),   //from js for rust

    StringStatic(&'static str),
    String(String),             //from rust for js

    U32(u32),
    I32(i32),
    U64(u64),
    I64(i64),

    True,
    False,
    Null,
    Undefined,
    List(ParamList),
    Buffer(AllocBuffer),
}

impl ParamItem {
    fn get_size(&self) -> u32 {
        match self {
            Self::StringEmpty => PARAM_TYPE,
            Self::StringAlloc(..) => PARAM_TYPE + STRING_PTR + STRING_SIZE,
            Self::StringStatic(..) => PARAM_TYPE + STRING_PTR + STRING_SIZE,
            Self::String(..) => PARAM_TYPE + STRING_PTR + STRING_SIZE,
            Self::U32(..) => PARAM_TYPE + 4,
            Self::I32(..) => PARAM_TYPE + 4,
            Self::U64(..) => PARAM_TYPE + 8,
            Self::I64(..) => PARAM_TYPE + 8,
            Self::True => PARAM_TYPE,
            Self::False => PARAM_TYPE,
            Self::Null => PARAM_TYPE,
            Self::Undefined => PARAM_TYPE,
            Self::List(items) => items.get_size(),
            Self::Buffer(..) => BUFF_PTR + BUFF_SIZE,
        }
    }

    fn write_to(&self, buff: &mut AllocBuffer) {
        match self {
            Self::StringEmpty => {
                buff.write_param_type(ParamItemNumber::StringEmpty);
            },
            Self::StringAlloc(value) => {
                buff.write_param_type(ParamItemNumber::StringAlloc);
                buff.write_mem_ptr(value.get_ptr());
                buff.write_mem_size(value.get_size());
            },
            Self::StringStatic(value) => {
                buff.write_param_type(ParamItemNumber::StringStatic);
                buff.write_mem_ptr(WasmMemoryPtr::from(value));
                buff.write_mem_size(WasmMemorySize::from(value));
            },
            Self::String(value) => {
                let value_str = value.as_str();
                buff.write_param_type(ParamItemNumber::String);
                buff.write_mem_ptr(WasmMemoryPtr::from(value_str));
                buff.write_mem_size(WasmMemorySize::from(value_str));
            },
        
            Self::U32(value) => {
                buff.write_param_type(ParamItemNumber::U32);
                buff.write_u32(*value);
            },
            Self::I32(value) => {
                buff.write_param_type(ParamItemNumber::I32);
                buff.write_i32(*value);
            },
            Self::U64(value) => {
                buff.write_param_type(ParamItemNumber::U64);
                buff.write_u64(*value);
            },
            Self::I64(value) => {
                buff.write_param_type(ParamItemNumber::I64);
                buff.write_i64(*value);
            },

            Self::True => {
                buff.write_param_type(ParamItemNumber::True);
            },
            Self::False => {
                buff.write_param_type(ParamItemNumber::False);
            },
            Self::Null => {
                buff.write_param_type(ParamItemNumber::Null);
            },
            Self::Undefined => {
                buff.write_param_type(ParamItemNumber::Undefined);
            },
            Self::List(list) => {
                list.write_to(buff);
            },
            Self::Buffer(inner_buff) => {
                buff.write_param_type(ParamItemNumber::Vec);
                buff.write_mem_ptr(inner_buff.get_ptr());
                buff.write_mem_size(inner_buff.get_size());
            },
        }
    }

    fn name(&self) -> &'static str {
        match self {
            Self::StringEmpty => "string",
            Self::StringAlloc(..) => "string",
            Self::StringStatic(..) => "string",
            Self::String(..) => "string",
            Self::U32(..) => "u32",
            Self::I32(..) => "i32",
            Self::U64(..) => "u64",
            Self::I64(..) => "i64",
            Self::True => "true",
            Self::False => "false",
            Self::Null => "null",
            Self::Undefined => "undefined",
            Self::List(..) => "list",
            Self::Buffer(..) => "vec",
        }
    }

    pub fn try_get_u64(self) -> Result<u64, String> {
        match self {
            Self::U64(value) => Ok(value),
            item => {
                let name = item.name();
                Err(format!("u64 expected, received {name}"))
            }
        }
    }

    pub fn try_get_u64_or_null(self) -> Result<Option<u64>, String> {
        match self {
            Self::U64(value) => Ok(Some(value)),
            Self::Null => Ok(None),
            item => {
                let name = item.name();
                Err(format!("Option<u64> expected, received {name}"))
            }
        }
    }

    pub fn try_get_string(self) -> Result<String, String> {
        match self {
            Self::StringEmpty => Ok(String::from("")),
            Self::StringAlloc(value) => Ok(value.convert_to_string()),
            Self::StringStatic(value) => Ok(value.into()),
            Self::String(value) => Ok(value),
            item => {
                let name = item.name();
                Err(format!("String expected, received {name}"))
            }
        }
    }

    pub fn try_get_u32(self) -> Result<u32, String> {
        match self {
            Self::U32(value) => Ok(value),
            item => {
                let name = item.name();
                Err(format!("u32 expected, received {name}"))
            }
        }
    }

    pub fn try_get_bool(self) -> Result<bool, String> {
        match self {
            Self::True => Ok(true),
            Self::False => Ok(false),
            item => {
                let name = item.name();
                Err(format!("bool expected, received {name}"))
            }
        }
    }

    pub fn try_get_list(self) -> Result<Vec<ParamItem>, String> {
        match self {
            Self::List(ParamList(list)) => Ok(list),
            item => {
                let name = item.name();
                Err(format!("list expected, received {name}"))
            }
        }
    }

    pub fn try_get_buffer(self) -> Result<Vec<u8>, String> {
        match self {
            Self::Buffer(buffer) => Ok(buffer.convert_to_vec()),
            item => {
                let name = item.name();
                Err(format!("buffer expected, received {name}"))
            }
        }
    }
}


#[derive(Default, Debug)]
pub struct ParamList(pub Vec<ParamItem>);

impl ParamList {
    pub fn new() -> ParamList {
        ParamList(Vec::new())
    }

    fn get_size(&self) -> u32 {
        let mut sum = PARAM_TYPE + LIST_COUNT;

        let ParamList(list) = self;
        for param in list {
            sum += param.get_size();
        }

        sum
    }

    fn write_to(&self, buff: &mut AllocBuffer) {
        buff.write_param_type(ParamItemNumber::List);

        let ParamList(list) = self;
        buff.write_u16(list.len() as u16);

        for param in list {
            param.write_to(buff);
        }
    }

    pub fn to_snapshot(&self) -> AllocBuffer {
        let buff_size = self.get_size();

        let mut buff = AllocBuffer::new(buff_size);
        self.write_to(&mut buff);
        buff
    }


    pub fn convert<T, F: FnOnce(ParamListDecoder) -> Result<T, String>>(self, convert: F) -> Result<T, String> {
        let decoder = ParamListDecoder::new(self.0);
        convert(decoder)
    }

}



pub struct ParamListDecoder {
    data: VecDeque<ParamItem>,
}

impl ParamListDecoder {
    pub fn new(data: Vec<ParamItem>) -> ParamListDecoder {
        ParamListDecoder {
            data: VecDeque::from(data),
        }
    }

    pub fn get_buffer(&mut self, label: &'static str) -> Result<Vec<u8>, String> {
        if let Some(value) = self.data.pop_front() {
            match value.try_get_buffer() {
                Ok(buffer) => Ok(buffer),
                Err(error) => Err(format!("{label} -> {error}")),
            }
        } else {
            Err(format!("{label} -> has no more params"))
        }
    }

    pub fn get_u64(&mut self, label: &'static str) -> Result<u64, String> {
        if let Some(value) = self.data.pop_front() {
            match value.try_get_u64() {
                Ok(value_str) => Ok(value_str),
                Err(error) => Err(format!("{label} -> {error}")),
            }
        } else {
            Err(format!("{label} -> has no more params"))
        }
    }

    pub fn get_u64_or_null(&mut self, label: &'static str) -> Result<Option<u64>, String> {
        if let Some(value) = self.data.pop_front() {
            match value.try_get_u64_or_null() {
                Ok(value_str) => Ok(value_str),
                Err(error) => Err(format!("{label} -> {error}")),
            }
        } else {
            Err(format!("{label} -> has no more params"))
        }
    }

    pub fn get_string(&mut self, label: &'static str) -> Result<String, String> {
        if let Some(value) = self.data.pop_front() {
            match value.try_get_string() {
                Ok(value_str) => Ok(value_str),
                Err(error) => Err(format!("{label} -> {error}")),
            }
        } else {
            Err(format!("{label} -> has no more params"))
        }
    }

    pub fn get_u32(&mut self, label: &'static str) -> Result<u32, String> {
        if let Some(value) = self.data.pop_front() {
            match value.try_get_u32() {
                Ok(value_str) => Ok(value_str),
                Err(error) => Err(format!("{label} -> {error}")),
            }
        } else {
            Err(format!("{label} -> has no more params"))
        }
    }

    pub fn get_bool(&mut self, label: &'static str) -> Result<bool, String> {
        if let Some(value) = self.data.pop_front() {
            match value.try_get_bool() {
                Ok(value_str) => Ok(value_str),
                Err(error) => Err(format!("{label} -> {error}")),
            }
        } else {
            Err(format!("{label} -> has no more params"))
        }
    }

    pub fn get_list<
        R,
        F: Fn(ParamListDecoder) -> Result<R, String>,
    >(
        &mut self,
        label: &'static str,
        conver: F
    ) -> Result<Vec<R>, String> {
        if let Some(value) = self.data.pop_front() {
            match value.try_get_list() {
                Ok(list) => {
                    let mut result = Vec::new();

                    for (index, item) in list.into_iter().enumerate() {
                        match item.try_get_list() {
                            Ok(sublist) => {
                                let decoder = ParamListDecoder::new(sublist);
                                match conver(decoder) {
                                    Ok(value) => {
                                        result.push(value);
                                    },
                                    Err(error) => {
                                        return Err(format!("{label} -> index:{index} -> {error}"));
                                    }
                                }
                            },
                            Err(error) => {
                                return Err(format!("{label} -> index:{index} -> {error}"));
                            }
                        }
                    }

                    Ok(result)
                },
                Err(error) => Err(format!("{label} -> {error}")),
            }
        } else {
            Err(format!("{label} -> has no more params"))
        }
    }

    pub fn expect_no_more(self) -> Result<(), String> {
        if self.data.is_empty() {
            Ok(())
        } else {
            Err(String::from("Too many params"))
        }
    }
}


