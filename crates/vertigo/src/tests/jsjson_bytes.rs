use vertigo::{AutoJsJson, JsJson, JsJsonContext, JsJsonDeserialize, JsJsonSerialize};

#[derive(AutoJsJson, PartialEq, Debug, Clone)]
struct TestStruct {
    pub data: Vec<u8>,
    pub name: String,
}

#[test]
fn test_auto_js_json_vec() -> Result<(), Box<dyn std::error::Error>> {
    let original = TestStruct {
        data: vec![10, 20, 30, 40],
        name: "test".to_string(),
    };

    let json = original.clone().to_json();

    if let JsJson::Object(map) = &json {
        if let Some(JsJson::Vec(data)) = map.get("data") {
            assert_eq!(data, &vec![10, 20, 30, 40]);
        } else {
            return Err("Expected JsJson::Vec for 'data' field".into());
        }
    } else {
        return Err("Expected JsJson::Object".into());
    }

    let context = JsJsonContext::new("test");
    let deserialized = TestStruct::from_json(context, json).map_err(|e| e.to_string())?;

    assert_eq!(original, deserialized);
    Ok(())
}
