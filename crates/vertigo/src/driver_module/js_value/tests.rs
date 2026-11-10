use crate::driver_module::js_value::JsObject;
use std::{collections::BTreeMap, error::Error};

use super::{
    JsJsonSerialize, MemoryBlock, MemoryBlockWrite, from_json,
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
        JsJson::Object(JsObject::from([
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
    let json = JsJson::Object(JsObject::new());
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
    let json = JsJson::Object(JsObject::from([
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
    let json = JsJson::Object(JsObject::from([(
        "1".to_string(),
        JsJson::Number(JsJsonNumber(10.0)),
    )]));
    assert!(from_json::<BTreeMap<u32, u32>>(json).is_err());
}

#[test]
fn btreemap_from_list_of_items() -> Result<(), Box<dyn Error>> {
    // A populated list of `{k, v}` items round-trips into a map.
    let json = JsJson::List(vec![
        JsJson::Object(JsObject::from([
            ("k".to_string(), JsJson::String("aaa".into())),
            ("v".to_string(), JsJson::Number(JsJsonNumber(1.0))),
        ])),
        JsJson::Object(JsObject::from([
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

#[test]
fn integers_out_of_range_or_with_a_fraction_are_refused() {
    let number = |value: f64| JsJson::Number(JsJsonNumber(value));

    assert_eq!(from_json::<u8>(number(255.0)), Ok(255));
    assert_eq!(from_json::<u8>(number(0.0)), Ok(0));
    for value in [256.0, 300.0, -1.0, 1.5, f64::NAN, f64::INFINITY] {
        assert!(from_json::<u8>(number(value)).is_err(), "{value} as u8");
    }
    assert_eq!(from_json::<i8>(number(-128.0)), Ok(-128));
    assert!(from_json::<i8>(number(-129.0)).is_err());
    assert!(from_json::<i32>(number(-0.5)).is_err());
    assert_eq!(
        from_json::<u64>(number(9_007_199_254_740_992.0)),
        Ok(9_007_199_254_740_992)
    );
    // `to_json` rounds the largest u64 and i64 up to 2^64 and 2^63, and they come back
    assert_eq!(from_json::<u64>(u64::MAX.to_json()), Ok(u64::MAX));
    assert_eq!(from_json::<i64>(i64::MAX.to_json()), Ok(i64::MAX));
    // 2^65 and 2^64, past what even a rounded u64 and i64 can be
    assert!(from_json::<u64>(number(36_893_488_147_419_103_232.0)).is_err());
    assert!(from_json::<i64>(number(18_446_744_073_709_551_616.0)).is_err());
    assert_eq!(from_json::<u32>(number(4_294_967_295.0)), Ok(u32::MAX));
    assert!(from_json::<u32>(number(4_294_967_296.0)).is_err());
    assert_eq!(
        from_json::<i64>(number(-9_223_372_036_854_775_808.0)),
        Ok(i64::MIN)
    );
    assert_eq!(from_json::<f32>(number(1.5)), Ok(1.5));
}

#[test]
fn a_refused_number_says_what_was_expected() {
    let result = from_json::<u8>(JsJson::Number(JsJsonNumber(300.0)));
    assert!(
        matches!(&result, Err(error) if error.contains("integer(u8) expected, received 300")),
        "{result:?}"
    );

    for value in [1.5, f64::NAN, f64::INFINITY, 1e30] {
        let result = from_json::<u8>(JsJson::Number(JsJsonNumber(value)));
        assert!(
            matches!(&result, Err(error) if error.contains("received a fraction or a value beyond")),
            "{value}: {result:?}"
        );
    }

    let result = from_json::<u16>(JsJson::String("x".into()));
    assert!(
        matches!(&result, Err(error) if error.contains("number(u16) expected, received")),
        "{result:?}"
    );
}
