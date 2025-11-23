use std::collections::BTreeMap;

use crate::driver_module::js_value::js_json_struct::JsJsonNumber;

use super::js_json_struct::JsJson;
use crate::{MemoryBlock, MemoryBlockWrite};

fn to_block(json: JsJson) -> MemoryBlock {
    let size = json.get_size();
    let block = MemoryBlock::new(size);
    let mut block_write = MemoryBlockWrite::new(block);
    json.write_to(&mut block_write);
    block_write.get_block()
}

#[test]
fn json_json_string() {
    let data1 = JsJson::String("test string".into());

    let block = to_block(data1.clone());

    let Ok(data2) = JsJson::from_block(block) else {
        unreachable!();
    };

    assert_eq!(data1, data2);
}

#[test]
fn json_json_float() {
    let data1 = JsJson::Number(JsJsonNumber(3.15));

    let block = to_block(data1.clone());

    let Ok(data2) = JsJson::from_block(block) else {
        unreachable!();
    };

    assert_eq!(data2, data1);
}

#[test]
fn json_json_list() {
    let data1 = JsJson::List(vec![
        JsJson::String("aaaa".into()),
        JsJson::String("bbbb".into()),
        JsJson::True,
        JsJson::Null,
        JsJson::Number(JsJsonNumber(12.3)),
        JsJson::List(vec![
            JsJson::String("cccc".into()),
            JsJson::String("dddd".into()),
            JsJson::Null,
        ]),
        JsJson::Object(BTreeMap::from([
            ("aaa".to_string(), JsJson::Number(JsJsonNumber(2.0))),
            ("bbb".to_string(), JsJson::String(String::from("ccc"))),
        ])),
    ]);

    let block = to_block(data1.clone());

    let Ok(data2) = JsJson::from_block(block) else {
        unreachable!();
    };

    assert_eq!(data1, data2);
}

#[test]
fn json_json_vec() {
    let data1 = JsJson::Vec(vec![1, 2, 3, 4, 5]);

    let block = to_block(data1.clone());

    let Ok(data2) = JsJson::from_block(block) else {
        unreachable!();
    };

    assert_eq!(data1, data2);
}
