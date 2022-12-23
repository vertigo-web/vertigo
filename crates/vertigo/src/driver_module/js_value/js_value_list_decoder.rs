use std::collections::VecDeque;

use super::js_json_struct::JsJson;

use super::js_value_struct::JsValue;


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
        let Some(value) = self.data.pop_front() else {
            return Err(format!("{label} -> has no more params"));
        };

        match value {
            JsValue::Vec(buffer) => Ok(buffer),
            item => {
                let name = item.typename();
                Err(format!("{label} -> buffer expected, received {name}"))
            }
        }
    }

    pub fn get_u64(&mut self, label: &'static str) -> Result<u64, String> {
        let Some(value) = self.data.pop_front() else {
            return Err(format!("{label} -> has no more params"));
        };

        match value {
            JsValue::U64(value) => Ok(value),
            item => {
                let name = item.typename();
                Err(format!("{label} -> u64 expected, received {name}"))
            }
        }
    }

    pub fn get_u64_or_null(&mut self, label: &'static str) -> Result<Option<u64>, String> {
        let Some(value) = self.data.pop_front() else {
            return Err(format!("{label} -> has no more params"));
        };

        match value {
            JsValue::U64(value) => Ok(Some(value)),
            JsValue::Null => Ok(None),
            item => {
                let name = item.typename();
                Err(format!("{label} -> Option<u64> expected, received {name}"))
            }
        }
    }

    pub fn get_string(&mut self, label: &'static str) -> Result<String, String> {
        let Some(value) = self.data.pop_front() else {
            return Err(format!("{label} -> has no more params"));
        };

        match value {
            JsValue::String(value) => Ok(value),
            item => {
                let name = item.typename();
                Err(format!("{label} -> String expected, received {name}"))
            }
        }
    }

    pub fn get_u32(&mut self, label: &'static str) -> Result<u32, String> {
        let Some(value) = self.data.pop_front() else {
            return Err(format!("{label} -> has no more params"));
        };

        match value {
            JsValue::U32(value) => Ok(value),
            item => {
                let name = item.typename();
                Err(format!("{label} -> u32 expected, received {name}"))
            }
        }
    }

    pub fn get_json(&mut self, label: &'static str) -> Result<JsJson, String> {
        let Some(value) = self.data.pop_front() else {
            return Err(format!("{label} -> has no more params"));
        };

        match value {
            JsValue::Json(value) => Ok(value),
            item => {
                let name = item.typename();
                Err(format!("{label} -> json expected, received {name}"))
            }
        }
    }

    pub fn get_bool(&mut self, label: &'static str) -> Result<bool, String> {
        let Some(value) = self.data.pop_front() else {
            return Err(format!("{label} -> has no more params"));
        };

        match value {
            JsValue::True => Ok(true),
            JsValue::False => Ok(false),
            item => {
                let name = item.typename();
                Err(format!("{label} -> bool expected, received {name}"))
            }
        }
    }

    pub fn get_vec<
        R,
        F: Fn(JsValue) -> Result<R, String>,
    >(
        &mut self,
        label: &'static str,
        conver: F
    ) -> Result<Vec<R>, String> {
            let Some(value) = self.data.pop_front() else {
                return Err(format!("{label} -> has no more params"));
            };

            let inner_list = match value {
                JsValue::List(list) => list,
                item => {
                    let name = item.typename();
                    return Err(format!("{label} -> list expected, received {name}"));
                }
            };

            let mut result = Vec::new();

            for (index, item) in inner_list.into_iter().enumerate() {
                match conver(item) {
                    Ok(value) => {
                        result.push(value);
                    },
                    Err(error) => {
                        return Err(format!("{label} -> index:{index} -> {error}"));
                    }
                }
            }

            Ok(result)
    }

    pub fn expect_no_more(self) -> Result<(), String> {
        if self.data.is_empty() {
            Ok(())
        } else {
            Err(String::from("Too many params"))
        }
    }
}

