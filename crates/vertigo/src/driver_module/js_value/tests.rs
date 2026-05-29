use std::{collections::BTreeMap, error::Error};

use super::{
    MemoryBlock, MemoryBlockWrite, from_json,
    js_json_struct::{JsJson, JsJsonNumber},
};

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
fn btreemap_from_empty_object() -> Result<(), Box<dyn Error>> {
    // An empty JSON object `{}` is a valid representation of an empty map.
    let json = JsJson::Object(BTreeMap::new());
    let result = from_json::<BTreeMap<String, u32>>(json)?;
    assert!(result.is_empty());
    Ok(())
}

#[test]
fn btreemap_from_empty_list() -> Result<(), Box<dyn Error>> {
    // An empty list `[]` still deserializes to an empty map.
    let json = JsJson::List(vec![]);
    let result = from_json::<BTreeMap<String, u32>>(json)?;
    assert!(result.is_empty());
    Ok(())
}

#[test]
fn btreemap_from_non_empty_object_string_keys() -> Result<(), Box<dyn Error>> {
    // serde serializes BTreeMap<String, V> as a plain object `{"key": value}`,
    // so the object form must deserialize into a map for String keys.
    let json = JsJson::Object(BTreeMap::from([
        ("foo".to_string(), JsJson::Number(JsJsonNumber(1.0))),
        ("bar".to_string(), JsJson::Number(JsJsonNumber(2.0))),
    ]));
    let result = from_json::<BTreeMap<String, u32>>(json)?;
    assert_eq!(result.len(), 2);
    assert_eq!(result.get("foo"), Some(&1));
    assert_eq!(result.get("bar"), Some(&2));
    Ok(())
}

#[test]
fn btreemap_from_object_non_string_keys_errors() {
    // Object form only makes sense for String keys; numeric keys arrive as
    // strings and must fail gracefully.
    let json = JsJson::Object(BTreeMap::from([(
        "1".to_string(),
        JsJson::Number(JsJsonNumber(10.0)),
    )]));
    assert!(from_json::<BTreeMap<u32, u32>>(json).is_err());
}

#[test]
fn btreemap_from_list_of_items() -> Result<(), Box<dyn Error>> {
    // A populated list of `{k, v}` items round-trips into a map.
    let json = JsJson::List(vec![
        JsJson::Object(BTreeMap::from([
            ("k".to_string(), JsJson::String("aaa".into())),
            ("v".to_string(), JsJson::Number(JsJsonNumber(1.0))),
        ])),
        JsJson::Object(BTreeMap::from([
            ("k".to_string(), JsJson::String("bbb".into())),
            ("v".to_string(), JsJson::Number(JsJsonNumber(2.0))),
        ])),
    ]);

    let result = from_json::<BTreeMap<String, u32>>(json)?;
    assert_eq!(result.len(), 2);
    assert_eq!(result.get("aaa"), Some(&1));
    assert_eq!(result.get("bbb"), Some(&2));
    Ok(())
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
