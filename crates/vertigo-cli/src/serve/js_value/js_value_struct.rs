use super::{
    js_json_struct::{decode_js_json_inner, JsJson},
    js_value_list_decoder::JsValueListDecoder,
    memory_block::MemoryBlock,
    memory_block_read::MemoryBlockRead,
    memory_block_write::MemoryBlockWrite,
};
use std::collections::HashMap;

const PARAM_TYPE: u32 = 1;
const STRING_SIZE: u32 = 4;
const VEC_SIZE: u32 = 4;
const LIST_COUNT: u32 = 4;
const OBJECT_COUNT: u32 = 2;

enum JsValueConst {
    U32,
    I32,
    U64,
    I64,

    True,
    False,
    Null,
    Undefined,

    Vec,
    String,
    List,
    Object,
    Json,
}

impl JsValueConst {
    fn from_byte(byte: u8) -> Option<JsValueConst> {
        match byte {
            1 => Some(JsValueConst::U32),
            2 => Some(JsValueConst::I32),
            3 => Some(JsValueConst::U64),
            4 => Some(JsValueConst::I64),
            5 => Some(JsValueConst::True),
            6 => Some(JsValueConst::False),
            7 => Some(JsValueConst::Null),
            8 => Some(JsValueConst::Undefined),
            9 => Some(JsValueConst::Vec),
            10 => Some(JsValueConst::String),
            11 => Some(JsValueConst::List),
            12 => Some(JsValueConst::Object),
            13 => Some(JsValueConst::Json),
            _ => None,
        }
    }
}

impl From<JsValueConst> for u8 {
    fn from(value: JsValueConst) -> Self {
        match value {
            JsValueConst::U32 => 1,
            JsValueConst::I32 => 2,
            JsValueConst::U64 => 3,
            JsValueConst::I64 => 4,

            JsValueConst::True => 5,
            JsValueConst::False => 6,
            JsValueConst::Null => 7,
            JsValueConst::Undefined => 8,

            JsValueConst::Vec => 9,
            JsValueConst::String => 10,
            JsValueConst::List => 11,
            JsValueConst::Object => 12,
            JsValueConst::Json => 13,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum JsValue {
    U32(u32),
    I32(i32),
    U64(u64),
    I64(i64),

    True,
    False,
    Null,
    Undefined,

    Vec(Vec<u8>),       //type, length of the sequence of bytes, sequence of bytes
    String(String),     //type, length of the sequence of bytes, sequence of bytes
    List(Vec<JsValue>), //type, length
    Object(HashMap<String, JsValue>),

    Json(JsJson),
}

impl JsValue {
    pub fn str(value: impl Into<String>) -> JsValue {
        JsValue::String(value.into())
    }

    pub fn from_block(block: MemoryBlock) -> Result<JsValue, std::string::String> {
        let mut buffer = MemoryBlockRead::new(block);
        decode_js_value_inner(&mut buffer)
    }

    pub fn bool(value: bool) -> JsValue {
        if value {
            JsValue::True
        } else {
            JsValue::False
        }
    }

    pub fn string_option(value: Option<String>) -> JsValue {
        match value {
            Some(body) => JsValue::String(body),
            None => JsValue::Null,
        }
    }

    fn get_string_size(value: &str) -> u32 {
        value.as_bytes().len() as u32
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

            Self::Vec(value) => PARAM_TYPE + VEC_SIZE + value.len() as u32,
            Self::String(value) => PARAM_TYPE + STRING_SIZE + JsValue::get_string_size(value),
            Self::List(items) => {
                let mut sum = PARAM_TYPE + LIST_COUNT;

                for param in items {
                    sum += param.get_size();
                }

                sum
            }
            Self::Object(map) => {
                let mut sum = PARAM_TYPE + OBJECT_COUNT;

                for (key, value) in map {
                    sum += STRING_SIZE + Self::get_string_size(key);
                    sum += value.get_size();
                }

                sum
            }
            Self::Json(json) => PARAM_TYPE + json.get_size(),
        }
    }

    fn write_to(&self, buff: &mut MemoryBlockWrite) {
        match self {
            Self::U32(value) => {
                buff.write_u8(JsValueConst::U32);
                buff.write_u32(*value);
            }
            Self::I32(value) => {
                buff.write_u8(JsValueConst::I32);
                buff.write_i32(*value);
            }
            Self::U64(value) => {
                buff.write_u8(JsValueConst::U64);
                buff.write_u64(*value);
            }
            Self::I64(value) => {
                buff.write_u8(JsValueConst::I64);
                buff.write_i64(*value);
            }

            Self::True => {
                buff.write_u8(JsValueConst::True);
            }
            Self::False => {
                buff.write_u8(JsValueConst::False);
            }
            Self::Null => {
                buff.write_u8(JsValueConst::Null);
            }
            Self::Undefined => {
                buff.write_u8(JsValueConst::Undefined);
            }

            Self::Vec(inner_buff) => {
                buff.write_u8(JsValueConst::Vec);
                let data = inner_buff.as_slice();
                buff.write_u32(data.len() as u32);
                buff.write(inner_buff.as_slice());
            }
            Self::String(value) => {
                buff.write_u8(JsValueConst::String);
                write_string_to(value.as_str(), buff);
            }
            Self::List(list) => {
                buff.write_u8(JsValueConst::List);
                buff.write_u32(list.len() as u32);

                for param in list {
                    param.write_to(buff);
                }
            }
            Self::Object(map) => {
                buff.write_u8(JsValueConst::Object);
                buff.write_u16(map.len() as u16);

                for (key, value) in map {
                    write_string_to(key.as_str(), buff);
                    value.write_to(buff);
                }
            }
            Self::Json(json) => {
                buff.write_u8(JsValueConst::Json);
                json.write_to(buff);
            }
        }
    }

    pub fn to_snapshot(&self) -> MemoryBlock {
        let buff_size = self.get_size();
        let block = MemoryBlock::new(buff_size);

        let mut buff = MemoryBlockWrite::new(block);
        self.write_to(&mut buff);
        buff.get_block()
    }

    pub fn typename(&self) -> &'static str {
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
            Self::Json(..) => "json",
        }
    }

    pub fn convert<T, F: FnOnce(JsValueListDecoder) -> Result<T, String>>(
        self,
        convert: F,
    ) -> Result<T, String> {
        match self {
            JsValue::List(list) => {
                let decoder = JsValueListDecoder::new(list);
                convert(decoder)
            }
            _ => Err(String::from("convert => ParamItem::Vec expected")),
        }
    }
}

impl Default for JsValue {
    fn default() -> Self {
        JsValue::List(Vec::new())
    }
}

fn write_string_to(value: &str, buff: &mut MemoryBlockWrite) {
    let data = value.as_bytes();
    buff.write_u32(data.len() as u32);
    buff.write(data);
}

fn decode_js_value_inner(buffer: &mut MemoryBlockRead) -> Result<JsValue, String> {
    let type_param = buffer.get_byte();

    let Some(type_param) = JsValueConst::from_byte(type_param) else {
        return Err(format!("JsValue: Unknown data type prefix {type_param}"));
    };

    let result = match type_param {
        JsValueConst::U32 => {
            let value = buffer.get_u32();
            JsValue::U32(value)
        }
        JsValueConst::I32 => {
            let value = buffer.get_i32();
            JsValue::I32(value)
        }
        JsValueConst::U64 => {
            let value = buffer.get_u64();
            JsValue::U64(value)
        }
        JsValueConst::I64 => {
            let value = buffer.get_i64();
            JsValue::I64(value)
        }
        JsValueConst::True => JsValue::True,
        JsValueConst::False => JsValue::False,
        JsValueConst::Null => JsValue::Null,
        JsValueConst::Undefined => JsValue::Undefined,
        JsValueConst::Vec => {
            let len = buffer.get_u32();
            let param = buffer.get_vec(len);
            JsValue::Vec(param)
        }
        JsValueConst::String => {
            let str_len = buffer.get_u32();
            let param = buffer.get_string(str_len)?;
            JsValue::String(param)
        }
        JsValueConst::List => {
            let mut param_list = Vec::new();

            let list_size = buffer.get_u32();

            for _ in 0..list_size {
                let param = decode_js_value_inner(buffer)?;
                param_list.push(param);
            }

            JsValue::List(param_list)
        }
        JsValueConst::Object => {
            let mut props = HashMap::new();
            let object_size = buffer.get_u16();

            for _ in 0..object_size {
                let prop_name = decode_js_value_inner(buffer)?;
                let JsValue::String(prop_name) = prop_name else {
                    return Err("string expected".into());
                };

                let prop_value = decode_js_value_inner(buffer)?;

                props.insert(prop_name, prop_value);
            }

            JsValue::Object(props)
        }
        JsValueConst::Json => {
            let json = decode_js_json_inner(buffer)?;
            JsValue::Json(json)
        }
    };

    Ok(result)
}
