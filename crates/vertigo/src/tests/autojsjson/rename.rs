use crate::{
    self as vertigo, AutoJsJson, JsJson, JsJsonContext, JsJsonDeserialize, JsJsonSerialize,
};

#[test]
fn test_serialize_and_deserialize_struct() {
    #[derive(AutoJsJson, Clone, Debug, PartialEq)]
    #[js_json(rename_all = "camelCase")]
    pub struct TestObj {
        pub r#type: String,
        pub first_name: String,
        #[js_json(rename = "fancy data name")]
        pub data: JsJson,
    }

    let test_obj = TestObj {
        r#type: "one".to_string(),
        first_name: "two".to_string(),
        data: JsJson::String("test test".into()),
    };

    let test_obj_json = test_obj.clone().to_json();
    let ctx = JsJsonContext::new("");
    let hash_map = match test_obj_json.clone().get_hashmap(&ctx) {
        Ok(map) => map,
        Err(_err) => panic!("Error unwrapping hash_map"),
    };

    assert!(hash_map.contains_key("type"));
    assert!(hash_map.contains_key("firstName"));
    assert!(!hash_map.contains_key("first_name"));
    assert!(hash_map.contains_key("fancy data name"));
    assert!(!hash_map.contains_key("data"));

    let restored_obj = TestObj::from_json(JsJsonContext::new(""), test_obj_json).unwrap();

    assert_eq!(test_obj, restored_obj);
}

#[test]
fn test_simple_enum() {
    #[derive(AutoJsJson, Clone, Debug, PartialEq)]
    #[js_json(rename_all = "SHOUTY_SNAKE_CASE")]
    pub enum Side {
        TurnLeft,
        TurnRight,
    }

    let left = Side::TurnLeft;
    let right = Side::TurnRight;

    let left_js = left.clone().to_json();
    let right_js = right.clone().to_json();

    match left_js {
        JsJson::String(val) => assert_eq!(val, "TURN_LEFT"),
        _ => panic!("Invalid type of left_js"),
    }
    match right_js {
        JsJson::String(val) => assert_eq!(val, "TURN_RIGHT"),
        _ => panic!("Invalid type of right_js"),
    }
}

#[test]
fn test_compound_enum() {
    #[derive(AutoJsJson, Clone, Debug, PartialEq)]
    #[js_json(rename_all = "kebab-case")]
    pub enum TestType {
        SomeString(String),
        ThePoint {
            x: u32,
            y: String,
        },
        MyTuple(String, u32),
        YourNumber(u32),
        #[js_json(rename = "the empty tuple")]
        EmptyTuple(),
        EmptyStruct {},
        NothingWhatSoEver,
    }

    let somestring = TestType::SomeString("asdf".to_string());
    let point = TestType::ThePoint {
        x: 10,
        y: "raz".to_string(),
    };
    let tuple = TestType::MyTuple("raz".to_string(), 10);
    let number = TestType::YourNumber(50);
    let empty_tuple = TestType::EmptyTuple();
    let empty_struct = TestType::EmptyStruct {};
    let nothing = TestType::NothingWhatSoEver;

    let somestring_json = somestring.clone().to_json();
    let point_json = point.clone().to_json();
    let tuple_json = tuple.clone().to_json();
    let number_json = number.clone().to_json();
    let empty_tuple_json = empty_tuple.clone().to_json();
    let empty_struct_json = empty_struct.clone().to_json();
    let nothing_json = nothing.clone().to_json();

    fn test_obj(name: &str, obj: &JsJson, key: &str) {
        match obj {
            JsJson::Object(map) => assert!(map.contains_key(key)),
            _ => panic!("Invalid type of {name}"),
        }
    }

    test_obj("somestring_json", &somestring_json, "some-string");
    test_obj("point_json", &point_json, "the-point");
    test_obj("tuple_json", &tuple_json, "my-tuple");
    test_obj("number_json", &number_json, "your-number");
    test_obj("empty_tuple_json", &empty_tuple_json, "the empty tuple");
    test_obj("empty_struct_json", &empty_struct_json, "empty-struct");

    println!("nothing = {nothing_json:?}");
    match &nothing_json {
        JsJson::String(val) => assert_eq!(val, "nothing-what-so-ever"),
        _ => panic!("Invalid type of nothing"),
    }

    let again_somestring = TestType::from_json(JsJsonContext::new(""), somestring_json)
        .unwrap_or_else(|err| panic!("1. {}", err.convert_to_string()));
    let again_point = TestType::from_json(JsJsonContext::new(""), point_json)
        .unwrap_or_else(|err| panic!("2. {}", err.convert_to_string()));
    let again_tuple = TestType::from_json(JsJsonContext::new(""), tuple_json)
        .unwrap_or_else(|err| panic!("3. {}", err.convert_to_string()));
    let again_number = TestType::from_json(JsJsonContext::new(""), number_json)
        .unwrap_or_else(|err| panic!("4. {}", err.convert_to_string()));
    let again_empty_tuple = TestType::from_json(JsJsonContext::new(""), empty_tuple_json)
        .unwrap_or_else(|err| panic!("5. {}", err.convert_to_string()));
    let again_empty_struct = TestType::from_json(JsJsonContext::new(""), empty_struct_json)
        .unwrap_or_else(|err| panic!("4. {}", err.convert_to_string()));
    let again_nothing = TestType::from_json(JsJsonContext::new(""), nothing_json)
        .unwrap_or_else(|err| panic!("7. {}", err.convert_to_string()));

    assert_eq!(somestring, again_somestring);
    assert_eq!(point, again_point);
    assert_eq!(tuple, again_tuple);
    assert_eq!(number, again_number);
    assert_eq!(empty_tuple, again_empty_tuple);
    assert_eq!(empty_struct, again_empty_struct);
    assert_eq!(nothing, again_nothing);
}
