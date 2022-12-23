use std::collections::HashMap;

use super::{
    memory_block_write::MemoryBlockWrite,
    memory_block_read::MemoryBlockRead
};
use super::serialize::{JsJsonContext, JsJsonDeserialize, JsJsonSerialize};

const PARAM_TYPE: u32 = 1;
const STRING_SIZE: u32 = 4;
const LIST_COUNT: u32 = 2;
const OBJECT_COUNT: u32 = 2;

enum JsJsonConst {
    True,
    False,
    Null,

    String,
    Number,
    List,
    Object,
}

impl JsJsonConst {
    fn from_byte(byte: u8) -> Option<JsJsonConst> {
        match byte {
            1  => Some(JsJsonConst::True),
            2  => Some(JsJsonConst::False),
            3  => Some(JsJsonConst::Null),
            4  => Some(JsJsonConst::String),
            5  => Some(JsJsonConst::Number),
            6  => Some(JsJsonConst::List),
            7  => Some(JsJsonConst::Object),
            _  => None,
        }
    }

    fn as_byte(&self) -> u8 {
        match self {
            JsJsonConst::True => 1,
            JsJsonConst::False => 2,
            JsJsonConst::Null => 3,
            JsJsonConst::String => 4,
            JsJsonConst::Number => 5,
            JsJsonConst::List => 6,
            JsJsonConst::Object => 7,
        }
    }
}

//https://www.json.org/json-en.html

/*
    https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Number#number_encoding

    The JavaScript Number type is a double-precision 64-bit binary format IEEE 754 value, like double in Java or C#. This means it can represent fractional values, but there are some limits to the stored number's magnitude and precision. Very briefly, an IEEE 754 double-precision number uses 64 bits to represent 3 parts:

    1 bit for the sign (positive or negative)
    11 bits for the exponent (-1022 to 1023)
    52 bits for the mantissa (representing a number between 0 and 1)
*/

#[derive(Debug, PartialEq, Clone)]
pub enum JsJson {
    True,
    False,
    Null,
    String(String),
    Number(f64),
    List(Vec<JsJson>),
    Object(HashMap<String, JsJson>),
}

impl JsJson {
    pub fn get_size(&self) -> u32 {
        match self {
            Self::True => PARAM_TYPE,
            Self::False => PARAM_TYPE,
            Self::Null => PARAM_TYPE,

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
        }
    }

    fn get_string_size(value: &str) -> u32 {
        value.as_bytes().len() as u32
    }

    pub fn write_to(&self, buff: &mut MemoryBlockWrite) {
        match self {
            Self::True => {
                buff.write_u8(JsJsonConst::True.as_byte());
            }
            Self::False => {
                buff.write_u8(JsJsonConst::False.as_byte());
            }
            Self::Null => {
                buff.write_u8(JsJsonConst::Null.as_byte());
            }
            Self::String(value) => {
                buff.write_u8(JsJsonConst::String.as_byte());
                write_string_to(value.as_str(), buff);
            }
            Self::Number(value) => {
                buff.write_u8(JsJsonConst::Number.as_byte());
                buff.write_f64(*value);
            }
            Self::List(list) => {
                buff.write_u8(JsJsonConst::List.as_byte());
                buff.write_u16(list.len() as u16);
        
                for param in list {
                    param.write_to(buff);
                }
            }
            Self::Object(map) => {
                buff.write_u8(JsJsonConst::Object.as_byte());
                buff.write_u16(map.len() as u16);

                for (key, value) in map {
                    write_string_to(key.as_str(), buff);
                    value.write_to(buff);
                }
            }
        }
    }

    pub fn typename(&self) -> &'static str {
        match self {
            Self::True => "bool",
            Self::False => "bool",
            Self::Null => "null",
            Self::String(..) => "string",
            Self::Number(..) => "number",
            Self::List(..) => "list",
            Self::Object(..) => "object",
        }
    }

    pub fn get_hashmap(self, context: &JsJsonContext) -> Result<HashMap<String, JsJson>, JsJsonContext> {
        let object = match self {
            JsJson::Object(object) => object,
            other => {
                let message = ["object expected, received ", other.typename()].concat();
                return Err(context.add(message));
            }
        };

        Ok(object)
    }

    pub fn get_property<T: JsJsonDeserialize>(&mut self, context: &JsJsonContext, param: &'static str) -> Result<T, JsJsonContext> {
        let object = match self {
            JsJson::Object(object) => object,
            other => {
                let message = ["object expected, received ", other.typename()].concat();
                return Err(context.add(message));
            }
        };

        let context = context.add(["field: '", param, "'"].concat());

        let item = object.remove(param).ok_or_else(|| {
            context.add("missing field")
        })?;

        T::from_json(context, item)
    }

}

fn write_string_to(value: &str, buff: &mut MemoryBlockWrite) {
    // TODO: impl From<JsJsonConst> for u8, and accept 'impl Into<u8>` in write_u8 to get rid of reapeting .as_byte()
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
        JsJsonConst::String => {
            let str_len = buffer.get_u32();
            let param = buffer.get_string(str_len)?;
            JsJson::String(param)
        }
        JsJsonConst::Number => {
            let value = buffer.get_f64();
            JsJson::Number(value)
        }
        JsJsonConst::List => {
            let mut param_list = Vec::new();

            let list_size = buffer.get_u16();
    
            for _ in 0..list_size {
                let param = decode_js_json_inner(buffer)?;
                param_list.push(param);
            }
    
            JsJson::List(param_list)
        },
        JsJsonConst::Object => {
            let mut props = HashMap::new();
            let object_size = buffer.get_u16();

            for _ in 0..object_size {
                let str_len = buffer.get_u32();
                let prop_name = buffer.get_string(str_len)?;

                let prop_value = decode_js_json_inner(buffer)?;
                props.insert(prop_name, prop_value);
            }

            JsJson::Object(props)
        }
    };

    Ok(result)
}

#[derive(Default)]
pub struct JsJsonObjectBuilder {
    data: HashMap<String, JsJson>
}

impl JsJsonObjectBuilder {
    pub fn insert(mut self, name: impl ToString, value: impl JsJsonSerialize) -> Self {
        let name = name.to_string();
        let value = value.to_json();
        self.data.insert(name, value);
        self
    }

    pub fn get(self) -> JsJson {
        JsJson::Object(self.data)
    }
}