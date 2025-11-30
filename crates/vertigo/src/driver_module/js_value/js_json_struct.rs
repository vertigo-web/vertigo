use std::{
    cmp::Ordering,
    collections::BTreeMap,
    hash::{Hash, Hasher},
};

use crate::{
    dev::{JsJsonListDecoder, LongPtr},
    driver_module::js_value::{
        vec_to_string::{string_to_vec, vec_to_string},
        MemoryBlock,
    },
};

use super::{
    memory_block_read::MemoryBlockRead,
    memory_block_write::MemoryBlockWrite,
    serialize::{JsJsonContext, JsJsonDeserialize, JsJsonSerialize},
};

const PARAM_TYPE: u32 = 1;
const STRING_SIZE: u32 = 4;
const LIST_COUNT: u32 = 4;
const OBJECT_COUNT: u32 = 2;

enum JsJsonConst {
    True = 1,
    False = 2,
    Null = 3,
    Undefined = 4,
    String = 5,
    Number = 6,
    List = 7,
    Object = 8,
    Vec = 9,
}

impl JsJsonConst {
    fn from_byte(byte: u8) -> Option<JsJsonConst> {
        match byte {
            1 => Some(JsJsonConst::True),
            2 => Some(JsJsonConst::False),
            3 => Some(JsJsonConst::Null),
            4 => Some(JsJsonConst::Undefined),
            5 => Some(JsJsonConst::String),
            6 => Some(JsJsonConst::Number),
            7 => Some(JsJsonConst::List),
            8 => Some(JsJsonConst::Object),
            9 => Some(JsJsonConst::Vec),
            _ => None,
        }
    }
}

impl From<JsJsonConst> for u8 {
    fn from(value: JsJsonConst) -> Self {
        value as u8
    }
}

//https://www.json.org/json-en.html

/*
    https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Number#number_encoding

    The JavaScript Number type is a double-precision 64-bit binary format IEEE 754 value, like double in Java or C#.
    This means it can represent fractional values, but there are some limits to the stored number's magnitude
    and precision. Very briefly, an IEEE 754 double-precision number uses 64 bits to represent 3 parts:

    1 bit for the sign (positive or negative)
    11 bits for the exponent (-1022 to 1023)
    52 bits for the mantissa (representing a number between 0 and 1)
*/

#[derive(Debug, Clone)]
pub struct JsJsonNumber(pub f64);

impl PartialEq for JsJsonNumber {
    fn eq(&self, other: &Self) -> bool {
        self.0.to_bits() == other.0.to_bits()
    }
}

impl Eq for JsJsonNumber {}

impl PartialOrd for JsJsonNumber {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for JsJsonNumber {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0
            .partial_cmp(&other.0)
            .unwrap_or_else(|| self.0.to_bits().cmp(&other.0.to_bits()))
    }
}

impl Hash for JsJsonNumber {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.to_bits().hash(state);
    }
}

impl JsJsonNumber {
    pub fn as_f64(&self) -> f64 {
        self.0
    }
}

/// JSON object serialized to travel between JS-WASM boundary.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum JsJson {
    True,
    False,
    Null,
    Undefined,
    String(String),
    Number(JsJsonNumber),
    List(Vec<JsJson>),
    Object(BTreeMap<String, JsJson>),
    Vec(Vec<u8>),
}

impl JsJsonDeserialize for JsJson {
    fn from_json(_context: JsJsonContext, json: JsJson) -> Result<Self, JsJsonContext> {
        Ok(json)
    }
}

impl JsJsonSerialize for JsJson {
    fn to_json(self) -> JsJson {
        self
    }
}

impl JsJson {
    pub fn str(value: impl Into<String>) -> JsJson {
        JsJson::String(value.into())
    }

    pub fn bool(value: bool) -> JsJson {
        if value {
            JsJson::True
        } else {
            JsJson::False
        }
    }

    pub fn to_ptr_long(&self) -> LongPtr {
        use crate::driver_module::{api::api_arguments, js_value::MemoryBlock};

        let buff_size = self.get_size();
        let block = MemoryBlock::new(buff_size);

        let mut buff = MemoryBlockWrite::new(block);
        self.write_to(&mut buff);
        let memory_block = buff.get_block();
        let ptr_long = memory_block.get_ptr_long();
        api_arguments().set(memory_block);
        ptr_long
    }

    pub fn from_block(block: MemoryBlock) -> Result<JsJson, std::string::String> {
        let mut buffer = MemoryBlockRead::new(block);
        decode_js_json_inner(&mut buffer)
    }

    pub fn get_size(&self) -> u32 {
        match self {
            Self::True => PARAM_TYPE,
            Self::False => PARAM_TYPE,
            Self::Null => PARAM_TYPE,
            Self::Undefined => PARAM_TYPE,

            Self::String(value) => PARAM_TYPE + STRING_SIZE + Self::get_string_size(value),
            Self::Number(..) => PARAM_TYPE + 8,
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
            Self::Vec(data) => PARAM_TYPE + 4 + data.len() as u32,
        }
    }

    fn get_string_size(value: &str) -> u32 {
        value.len() as u32
    }

    pub fn write_to(&self, buff: &mut MemoryBlockWrite) {
        match self {
            Self::True => {
                buff.write_u8(JsJsonConst::True);
            }
            Self::False => {
                buff.write_u8(JsJsonConst::False);
            }
            Self::Null => {
                buff.write_u8(JsJsonConst::Null);
            }
            Self::Undefined => {
                buff.write_u8(JsJsonConst::Undefined);
            }
            Self::String(value) => {
                buff.write_u8(JsJsonConst::String);
                write_string_to(value.as_str(), buff);
            }
            Self::Number(JsJsonNumber(value)) => {
                buff.write_u8(JsJsonConst::Number);
                buff.write_f64(*value);
            }
            Self::List(list) => {
                buff.write_u8(JsJsonConst::List);
                buff.write_u32(list.len() as u32);

                for param in list {
                    param.write_to(buff);
                }
            }
            Self::Object(map) => {
                buff.write_u8(JsJsonConst::Object);
                buff.write_u16(map.len() as u16);

                for (key, value) in map {
                    write_string_to(key.as_str(), buff);
                    value.write_to(buff);
                }
            }
            Self::Vec(data) => {
                buff.write_u8(JsJsonConst::Vec);
                buff.write_u32(data.len() as u32);
                buff.write(data);
            }
        }
    }

    pub fn typename(&self) -> &'static str {
        match self {
            Self::True => "boolean",
            Self::False => "boolean",
            Self::Null => "null",
            Self::Undefined => "undefined",
            Self::String(..) => "string",
            Self::Number(..) => "number",
            Self::List(..) => "list",
            Self::Object(..) => "object",
            Self::Vec(..) => "vec",
        }
    }

    pub fn get_hashmap(
        self,
        context: &JsJsonContext,
    ) -> Result<BTreeMap<String, JsJson>, JsJsonContext> {
        let object = match self {
            JsJson::Object(object) => object,
            other => {
                let message = ["object expected, received ", other.typename()].concat();
                return Err(context.add(message));
            }
        };

        Ok(object)
    }

    pub fn get_property<T: JsJsonDeserialize>(
        &mut self,
        context: &JsJsonContext,
        param: &'static str,
    ) -> Result<T, JsJsonContext> {
        let object = match self {
            JsJson::Object(object) => object,
            other => {
                let message = ["object expected, received ", other.typename()].concat();
                return Err(context.add(message));
            }
        };

        let context = context.add(["field: '", param, "'"].concat());

        let item = object
            .remove(param)
            .ok_or_else(|| context.add("missing field"))?;

        T::from_json(context, item)
    }

    pub fn get_property_jsjson(
        &mut self,
        context: &JsJsonContext,
        param: &'static str,
    ) -> Result<JsJson, JsJsonContext> {
        let object = match self {
            JsJson::Object(object) => object,
            other => {
                let message = ["object expected, received ", other.typename()].concat();
                return Err(context.add(message));
            }
        };

        let context = context.add(["field: '", param, "'"].concat());

        object
            .remove(param)
            .ok_or_else(|| context.add("missing field"))
    }
}

fn write_string_to(value: &str, buff: &mut MemoryBlockWrite) {
    let data = value.as_bytes();
    buff.write_u32(data.len() as u32);
    buff.write(data);
}

pub fn decode_js_json_inner(buffer: &mut MemoryBlockRead) -> Result<JsJson, String> {
    let type_param = buffer.get_byte();

    let Some(type_param) = JsJsonConst::from_byte(type_param) else {
        return Err(format!("JsJson: Unknown data type prefix {type_param}"));
    };

    let result = match type_param {
        JsJsonConst::True => JsJson::True,
        JsJsonConst::False => JsJson::False,
        JsJsonConst::Null => JsJson::Null,
        JsJsonConst::Undefined => JsJson::Undefined,
        JsJsonConst::String => {
            let str_len = buffer.get_u32();
            let param = buffer.get_string(str_len)?;
            JsJson::String(param)
        }
        JsJsonConst::Number => {
            let value = buffer.get_f64();
            JsJson::Number(JsJsonNumber(value))
        }
        JsJsonConst::List => {
            let mut param_list = Vec::new();

            let list_size = buffer.get_u32();

            for _ in 0..list_size {
                let param = decode_js_json_inner(buffer)?;
                param_list.push(param);
            }

            JsJson::List(param_list)
        }
        JsJsonConst::Object => {
            let mut props = BTreeMap::new();
            let object_size = buffer.get_u16();

            for _ in 0..object_size {
                let str_len = buffer.get_u32();
                let prop_name = buffer.get_string(str_len)?;

                let prop_value = decode_js_json_inner(buffer)?;
                props.insert(prop_name, prop_value);
            }

            JsJson::Object(props)
        }
        JsJsonConst::Vec => {
            let len = buffer.get_u32();
            let data = buffer.get_vec(len);
            JsJson::Vec(data)
        }
    };

    Ok(result)
}

impl JsJson {
    pub fn convert_to_string(&self) -> String {
        let block_bytes = self.to_vec();
        vec_to_string(&block_bytes)
    }

    pub fn to_vec(&self) -> Vec<u8> {
        let size = self.get_size();
        let block = MemoryBlock::new(size);
        let mut block = MemoryBlockWrite::new(block);
        self.write_to(&mut block);

        block.get_block().convert_to_vec()
    }

    pub fn from_string(data: &str) -> Result<JsJson, String> {
        let fff = string_to_vec(data)?;

        let block = MemoryBlock::from_slice(&fff);
        let mut block = MemoryBlockRead::new(block);

        let json = decode_js_json_inner(&mut block)?;
        Ok(json)
    }

    pub fn map_list<T, F>(self, convert: F) -> Result<T, String>
    where
        F: FnOnce(JsJsonListDecoder) -> Result<T, String>,
    {
        match self {
            JsJson::List(list) => convert(JsJsonListDecoder::new(list)),
            _ => Err("map_list => ParamItem::Vec expected".to_string()),
        }
    }
}

macro_rules! impl_from {
    ($($type:ty),+) => {
        $(
            impl From<$type> for JsJson {
                fn from(value: $type) -> Self {
                    JsJson::Number(JsJsonNumber(value as f64))
                }
            }
        )+
    };
}

impl_from!(i32, u32, u64, i64, usize, isize, f64);

impl From<&str> for JsJson {
    fn from(value: &str) -> Self {
        JsJson::String(value.to_string())
    }
}

impl From<String> for JsJson {
    fn from(value: String) -> Self {
        JsJson::String(value)
    }
}

impl From<bool> for JsJson {
    fn from(value: bool) -> Self {
        if value {
            JsJson::True
        } else {
            JsJson::False
        }
    }
}

impl From<Vec<u8>> for JsJson {
    fn from(value: Vec<u8>) -> Self {
        JsJson::Vec(value)
    }
}
impl From<Vec<(&str, JsJson)>> for JsJson {
    fn from(value: Vec<(&str, JsJson)>) -> Self {
        let mut map = BTreeMap::new();
        for (key, val) in value {
            map.insert(key.to_string(), val);
        }
        JsJson::Object(map)
    }
}
