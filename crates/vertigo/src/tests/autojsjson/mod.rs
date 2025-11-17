use std::collections::BTreeMap;

use crate::{self as vertigo, JsJson, JsJsonContext};
use crate::{AutoJsJson, JsJsonDeserialize, JsJsonSerialize};

mod rename;

#[test]
fn test_serialize_and_deserialize_struct() {
    #[derive(AutoJsJson, Clone, PartialEq, Eq, Debug)]
    pub struct TestObj {
        pub r#type: String,
        pub name: String,
        pub data: JsJson,
        pub data_opt1: Option<JsJson>,
        pub data_opt2: Option<JsJson>,
    }

    let test_obj = TestObj {
        r#type: "one".to_string(),
        name: "two".to_string(),
        data: JsJson::String("test test".into()),
        data_opt1: Some(JsJson::Object(BTreeMap::from([
            ("three".to_string(), JsJson::String("value3".to_string())),
            ("four".to_string(), JsJson::String("value4".to_string())),
        ]))),
        data_opt2: None,
    };

    let test_obj_json = test_obj.clone().to_json();
    let ctx = JsJsonContext::new("");
    let hash_map = match test_obj_json.clone().get_hashmap(&ctx) {
        Ok(map) => map,
        Err(_err) => panic!("Error unwrapping hash_map"),
    };

    assert!(matches!(hash_map.get("type"), Some(JsJson::String(x)) if x == "one"));
    assert!(matches!(hash_map.get("name"), Some(JsJson::String(x)) if x == "two"));
    assert!(matches!(hash_map.get("data"), Some(JsJson::String(x)) if x == "test test"));

    assert!(matches!(
        hash_map.get("data_opt1"),
        Some(JsJson::Object(map)) if {
            assert!(matches!(
                map.get("three"), Some(JsJson::String(x)) if x == "value3"
            ));
            assert!(matches!(
                map.get("four"), Some(JsJson::String(x)) if x == "value4"
            ));
            true
        }
    ));

    assert!(matches!(hash_map.get("data_opt2"), Some(JsJson::Null)));

    let restored_obj = TestObj::from_json(JsJsonContext::new(""), test_obj_json).unwrap();

    assert_eq!(test_obj, restored_obj);
}

#[test]
fn test_simple_enum() {
    #[derive(AutoJsJson, Clone, Debug, PartialEq)]
    pub enum Side {
        Left,
        Right,
    }

    let left = Side::Left;
    let right = Side::Right;

    let left_js = left.clone().to_json();
    let right_js = right.clone().to_json();

    let again_left = Side::from_json(JsJsonContext::new(""), left_js).unwrap_or_else(|_| panic!());
    let again_right =
        Side::from_json(JsJsonContext::new(""), right_js).unwrap_or_else(|_| panic!());

    assert_eq!(again_left, left);
    assert_eq!(again_right, right);
}

#[test]
fn test_compound_enum() {
    #[derive(AutoJsJson, Clone, Debug, PartialEq)]
    pub enum TestType {
        Somestring(String),
        Point { x: u32, y: String },
        Tuple(String, u32),
        Number(u32),
        EmptyTuple(),
        EmptyStruct {},
        Nothing,
    }

    let somestring = TestType::Somestring("asdf".to_string());
    let point = TestType::Point {
        x: 10,
        y: "raz".to_string(),
    };
    let tuple = TestType::Tuple("raz".to_string(), 10);
    let number = TestType::Number(50);
    let nothing = TestType::Nothing;

    let somestring_json = somestring.clone().to_json();
    let point_json = point.clone().to_json();
    let tuple_json = tuple.clone().to_json();
    let number_json = number.clone().to_json();
    let nothing_json = nothing.clone().to_json();

    use vertigo::JsJsonDeserialize;

    let again_somestring = TestType::from_json(JsJsonContext::new(""), somestring_json)
        .unwrap_or_else(|err| panic!("1. {}", err.convert_to_string()));
    let again_point = TestType::from_json(JsJsonContext::new(""), point_json)
        .unwrap_or_else(|err| panic!("2. {}", err.convert_to_string()));
    let again_tuple = TestType::from_json(JsJsonContext::new(""), tuple_json)
        .unwrap_or_else(|err| panic!("3. {}", err.convert_to_string()));
    let again_number = TestType::from_json(JsJsonContext::new(""), number_json)
        .unwrap_or_else(|err| panic!("4. {}", err.convert_to_string()));
    let again_nothing = TestType::from_json(JsJsonContext::new(""), nothing_json)
        .unwrap_or_else(|err| panic!("4. {}", err.convert_to_string()));

    assert_eq!(somestring, again_somestring);
    assert_eq!(point, again_point);
    assert_eq!(tuple, again_tuple);
    assert_eq!(number, again_number);
    assert_eq!(nothing, again_nothing);
}

#[test]
fn test_newtype() {
    #[derive(AutoJsJson, Clone, Debug, PartialEq)]
    pub struct MyNumber(pub i32);
    #[derive(AutoJsJson, Clone, Debug, PartialEq)]
    pub struct MyString(pub String);

    let my_number = MyNumber(3);
    let my_string = MyString("test".to_string());

    let my_number_js = my_number.clone().to_json();
    let my_string_js = my_string.clone().to_json();

    use vertigo::JsJsonDeserialize;

    let again_my_number =
        MyNumber::from_json(JsJsonContext::new(""), my_number_js).unwrap_or_else(|_| panic!());
    let again_my_string =
        MyString::from_json(JsJsonContext::new(""), my_string_js).unwrap_or_else(|_| panic!());

    assert_eq!(again_my_number, my_number);
    assert_eq!(my_string, again_my_string);
}

#[test]
fn test_newtype_tuple() {
    #[derive(AutoJsJson, Clone, Debug, PartialEq)]
    pub struct MyType(pub i32, String);

    let my_type = MyType(3, "test".to_string());

    let my_type_js = my_type.clone().to_json();

    use vertigo::JsJsonDeserialize;

    let again_my_type =
        MyType::from_json(JsJsonContext::new(""), my_type_js).unwrap_or_else(|_| panic!());

    assert_eq!(again_my_type, my_type);
}

#[test]
fn test_optional_field() {
    #[derive(Default, AutoJsJson)]
    pub struct Test {
        pub first_name: String,
        #[js_json(default = None)]
        pub second_name: Option<String>,
    }

    let test = Test {
        first_name: "Greg".to_string(),
        second_name: None,
    };

    let mut test_js = test.to_json();
    let ctx = JsJsonContext::new("");
    let _: Result<Option<String>, _> = test_js.get_property::<Option<String>>(&ctx, "second_name");

    let text_out = Test::from_json(JsJsonContext::new(""), test_js).unwrap_or_else(|_| panic!());

    assert_eq!(text_out.first_name.as_str(), "Greg");
    assert_eq!(text_out.second_name, None);
}
