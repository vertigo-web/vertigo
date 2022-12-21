use std::collections::HashMap;
use super::js_value_struct::JsValue;
use super::js_json_struct::JsJson;


#[test]
fn json_json_string() {
    let data1 = JsValue::Json(JsJson::String("test string".into()));

    let block = data1.to_snapshot();

    let data2 = JsValue::from_block(block).unwrap();

    assert_eq!(data1, data2);
}


#[test]
fn json_json_list() {
    let data1 = JsValue::Json(JsJson::List(vec!(
        JsJson::String("aaaa".into()),
        JsJson::String("bbbb".into()),
        JsJson::True,
        JsJson::Null,
        JsJson::Number(12.3),
        JsJson::List(vec!(
            JsJson::String("cccc".into()),
            JsJson::String("dddd".into()),
            JsJson::Null,
        )),
        JsJson::Object(HashMap::from([
            ("aaa".to_string(), JsJson::Number(2.0)),
            ("bbb".to_string(), JsJson::String(String::from("ccc")))
        ]))
    )));

    let block = data1.to_snapshot();

    let data2 = JsValue::from_block(block).unwrap();

    assert_eq!(data1, data2);
}


