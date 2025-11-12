use std::collections::BTreeMap;

use crate::driver_module::js_value::js_json_struct::JsJsonNumber;

use super::js_json_struct::JsJson;
use super::js_value_struct::JsValue;

#[test]
fn json_json_string() {
    let data1 = JsValue::Json(JsJson::String("test string".into()));

    let block = data1.to_block();

    let Ok(data2) = JsValue::from_block(block) else {
        unreachable!();
    };

    assert_eq!(data1, data2);
}

#[test]
fn json_json_float() {
    let data1 = JsValue::F64(3.15);

    let block = data1.to_block();

    let Ok(data2) = JsValue::from_block(block) else {
        unreachable!();
    };

    assert_eq!(data2, data1);

    assert_eq!(data2, JsValue::F64(3.15));
}

#[test]
fn json_json_list() {
    let data1 = JsValue::Json(JsJson::List(vec![
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
    ]));

    let block = data1.to_block();

    let Ok(data2) = JsValue::from_block(block) else {
        unreachable!();
    };

    assert_eq!(data1, data2);
}
