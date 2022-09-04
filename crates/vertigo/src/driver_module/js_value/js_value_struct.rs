use std::collections::{VecDeque, HashMap};

use super::{memory_block_write::MemoryBlockWrite, memory_block_read::MemoryBlockRead, memory_block::MemoryBlock};

const PARAM_TYPE: u32 = 1;
const SIZE: u32 = 4;
const LIST_COUNT: u32 = 2;
const OBJECT_COUNT: u32 = 2;

pub enum JsValueNumberConst {
    U32 = 1,
    I32 = 2,
    U64 = 3,
    I64 = 4,

    True = 5,
    False = 6,
    Null = 7,
    Undefined = 8,

    Vec = 9,
    String = 10,
    List = 11,
    Object = 12,
}

#[derive(Debug, PartialEq)]
pub enum JsValue {
    U32(u32),
    I32(i32),
    U64(u64),
    I64(i64),

    True,
    False,
    Null,
    Undefined,

    Vec(Vec<u8>),               //type, length of the sequence of bytes, sequence of bytes
    String(String),             //type, length of the sequence of bytes, sequence of bytes
    List(Vec<JsValue>),         //type, length
    Object(HashMap<String, JsValue>)
}

impl JsValue {
    pub fn str(value: &str) -> JsValue {
        JsValue::String(value.into())
    }

    fn get_string_size(value: &str) -> u32 {
        PARAM_TYPE + SIZE + value.as_bytes().len() as u32
    }

    fn get_size(&self) -> u32 {
        match self {
            Self::U32(..) => PARAM_TYPE + 4,
            Self::I32(..) => PARAM_TYPE + 4,
            Self::U64(..) => PARAM_TYPE + 8,
            Self::I64(..) => PARAM_TYPE + 8,

            Self::True => PARAM_TYPE,
            Self::False => PARAM_TYPE,
            Self::Null => PARAM_TYPE,
            Self::Undefined => PARAM_TYPE,

            Self::Vec(value) => PARAM_TYPE + SIZE + value.len() as u32,
            Self::String(value) => JsValue::get_string_size(value),
            Self::List(items) => {
                let mut sum = PARAM_TYPE + LIST_COUNT;

                for param in items {
                    sum += param.get_size();
                }
        
                sum
            },
            Self::Object(map) => {
                let mut sum = PARAM_TYPE + OBJECT_COUNT;

                for (key, value) in map {
                    sum += Self::get_string_size(key);
                    sum += value.get_size();
                }

                sum
            }
        }
    }

    fn write_string_to(value: &str, buff: &mut MemoryBlockWrite) {
        buff.write_param_type(JsValueNumberConst::String);
        let data = value.as_bytes();
        buff.write_u32(data.len() as u32);
        buff.write(data);

    }

    fn write_to(&self, buff: &mut MemoryBlockWrite) {
        match self {
            Self::U32(value) => {
                buff.write_param_type(JsValueNumberConst::U32);
                buff.write_u32(*value);
            },
            Self::I32(value) => {
                buff.write_param_type(JsValueNumberConst::I32);
                buff.write_i32(*value);
            },
            Self::U64(value) => {
                buff.write_param_type(JsValueNumberConst::U64);
                buff.write_u64(*value);
            },
            Self::I64(value) => {
                buff.write_param_type(JsValueNumberConst::I64);
                buff.write_i64(*value);
            },

            Self::True => {
                buff.write_param_type(JsValueNumberConst::True);
            },
            Self::False => {
                buff.write_param_type(JsValueNumberConst::False);
            },
            Self::Null => {
                buff.write_param_type(JsValueNumberConst::Null);
            },
            Self::Undefined => {
                buff.write_param_type(JsValueNumberConst::Undefined);
            },

            Self::Vec(inner_buff) => {
                buff.write_param_type(JsValueNumberConst::Vec);
                let data = inner_buff.as_slice();
                buff.write_u32(data.len() as u32);
                buff.write(inner_buff.as_slice());
            },
            Self::String(value) => {
                Self::write_string_to(value.as_str(), buff);
            },        
            Self::List(list) => {
                buff.write_param_type(JsValueNumberConst::List);
                buff.write_u16(list.len() as u16);
        
                for param in list {
                    param.write_to(buff);
                }
            },
            Self::Object(map) => {
                buff.write_param_type(JsValueNumberConst::Object);
                buff.write_u16(map.len() as u16);

                for (key, value) in map {
                    Self::write_string_to(key.as_str(), buff);
                    value.write_to(buff);
                }
            }
        }
    }

    pub fn to_snapshot(&self) -> MemoryBlock {
        let buff_size = self.get_size();

        let mut buff = MemoryBlockWrite::new(buff_size);
        self.write_to(&mut buff);
        buff.get_block()
    }


    fn name(&self) -> &'static str {
        match self {
            Self::U32(..) => "u32",
            Self::I32(..) => "i32",
            Self::U64(..) => "u64",
            Self::I64(..) => "i64",
            Self::True => "true",
            Self::False => "false",
            Self::Null => "null",
            Self::Undefined => "undefined",
            Self::Vec(..) => "vec",
            Self::String(..) => "string",
            Self::List(..) => "list",
            Self::Object(..) => "object",
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

    pub fn try_get_list(self) -> Result<Vec<JsValue>, String> {
        match self {
            Self::List(list) => Ok(list),
            item => {
                let name = item.name();
                Err(format!("list expected, received {name}"))
            }
        }
    }

    pub fn try_get_buffer(self) -> Result<Vec<u8>, String> {
        match self {
            Self::Vec(buffer) => Ok(buffer),
            item => {
                let name = item.name();
                Err(format!("buffer expected, received {name}"))
            }
        }
    }

    pub fn convert<T, F: FnOnce(JsValueListDecoder) -> Result<T, String>>(self, convert: F) -> Result<T, String> {
        match self {
            JsValue::List(list) => {
                let decoder = JsValueListDecoder::new(list);
                convert(decoder)        
            },
            _ => {
                Err(String::from("convert => ParamItem::Vec expected"))
            }
        }
    }

}

impl Default for JsValue {
    fn default() -> Self {
        JsValue::List(Vec::new())
    }
}

pub struct JsValueListDecoder {
    data: VecDeque<JsValue>,
}

impl JsValueListDecoder {
    pub fn new(data: Vec<JsValue>) -> JsValueListDecoder {
        JsValueListDecoder {
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
        F: Fn(JsValueListDecoder) -> Result<R, String>,
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
                                let decoder = JsValueListDecoder::new(sublist);
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


fn decode_js_value_inner(buffor: &mut MemoryBlockRead) -> Result<JsValue, String> {
    let type_param = buffor.get_byte();

    if type_param == 1 {
        let value = buffor.get_u32();
        return Ok(JsValue::U32(value));
    }

    if type_param == 2 {
        let value = buffor.get_i32();
        return Ok(JsValue::I32(value));
    }

    if type_param == 3 {
        let value = buffor.get_u64();
        return Ok(JsValue::U64(value));
    }

    if type_param == 4 {
        let value = buffor.get_i64();
        return Ok(JsValue::I64(value));
    }

    if type_param == 5 {
        return Ok(JsValue::True);
    }

    if type_param == 6 {
        return Ok(JsValue::False);
    }

    if type_param == 7 {
        return Ok(JsValue::Null);
    }

    if type_param == 8 {
        return Ok(JsValue::Undefined);
    }

    if type_param == 9 {
        let len = buffor.get_u32();
        let param = buffor.get_vec(len);
        return Ok(JsValue::Vec(param));
    }

    if type_param == 10 {
        let str_len = buffor.get_u32();
        let param = buffor.get_string(str_len)?;
        return Ok(JsValue::String(param));
    }

    if type_param == 11 {
        let mut param_list = Vec::new();

        let list_size = buffor.get_u16();

        for _ in 0..list_size {
            let param = decode_js_value_inner(buffor)?;
            param_list.push(param);
        }

        return Ok(JsValue::List(param_list));
    }

    Err(format!("Unknown data type prefix {type_param}"))
}


pub fn decode_js_value(buffor: MemoryBlock) -> Result<JsValue, String> {
    let mut buffor = MemoryBlockRead::new(buffor);
    decode_js_value_inner(&mut buffor)
}