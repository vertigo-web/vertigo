use vertigo::{AutoJsJson, JsJson, JsJsonContext, JsJsonDeserialize, JsJsonSerialize};

#[derive(AutoJsJson, PartialEq, Debug, Clone)]
struct TestStruct {
    pub data: Vec<u8>,
    pub name: String,
}

#[test]
fn test_auto_js_json_vec() {
    let original = TestStruct {
        data: vec![10, 20, 30, 40],
        name: "test".to_string(),
    };

    let json = original.clone().to_json();

    if let JsJson::Object(map) = &json {
        if let Some(JsJson::Vec(data)) = map.get("data") {
            assert_eq!(data, &vec![10, 20, 30, 40]);
        } else {
            panic!("Expected JsJson::Vec for 'data' field");
        }
    } else {
        panic!("Expected JsJson::Object");
    }

    let context = JsJsonContext::new("test");
    let deserialized = TestStruct::from_json(context, json).expect("Deserialization failed");

    assert_eq!(original, deserialized);
}
