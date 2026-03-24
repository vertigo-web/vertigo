use crate::{
    self as vertigo, AutoJsJson, JsJson, JsJsonContext, JsJsonDeserialize, JsJsonNumber,
    JsJsonSerialize,
};

/// Skipped fields must be absent from the serialized JSON object,
/// and restored to `Default::default()` on deserialization.
#[test]
fn test_skip_field_not_serialized() {
    #[derive(AutoJsJson, Clone, Debug, PartialEq, Default)]
    pub struct TestObj {
        pub name: String,
        #[js_json(skip)]
        pub internal: u32,
    }

    let obj = TestObj {
        name: "hello".to_string(),
        internal: 42,
    };

    let json = obj.clone().to_json();

    // The skipped field must not appear in the JSON output.
    let ctx = JsJsonContext::new("");
    let map = match json.clone().get_hashmap(&ctx) {
        Ok(m) => m,
        Err(_) => panic!("Expected JSON object"),
    };

    assert!(map.contains_key("name"), "\"name\" should be in JSON");
    assert!(
        !map.contains_key("internal"),
        "\"internal\" should not be in JSON"
    );

    // Deserialization: skipped field is set to Default::default().
    let restored = TestObj::from_json(JsJsonContext::new(""), json)
        .unwrap_or_else(|e| panic!("Deserialization failed: {}", e.convert_to_string()));

    assert_eq!(restored.name, obj.name);
    assert_eq!(restored.internal, 0, "skipped field should be Default (0)");
}

/// Skipped fields can coexist with renamed fields.
#[test]
fn test_skip_with_rename() {
    #[derive(AutoJsJson, Clone, Debug, PartialEq, Default)]
    pub struct Config {
        #[js_json(rename = "userName")]
        pub name: String,
        #[js_json(skip)]
        pub cache: Vec<String>,
    }

    let obj = Config {
        name: "Alice".to_string(),
        cache: vec!["a".to_string(), "b".to_string()],
    };

    let json = obj.clone().to_json();

    let ctx = JsJsonContext::new("");
    let map = match json.clone().get_hashmap(&ctx) {
        Ok(m) => m,
        Err(_) => panic!("Expected JSON object"),
    };

    assert!(map.contains_key("userName"));
    assert!(!map.contains_key("cache"));
    assert!(!map.contains_key("name"));

    let restored = Config::from_json(JsJsonContext::new(""), json)
        .unwrap_or_else(|e| panic!("Deserialization failed: {}", e.convert_to_string()));

    assert_eq!(restored.name, "Alice");
    assert_eq!(
        restored.cache,
        Vec::<String>::new(),
        "skipped field should be Default (empty Vec)"
    );
}

/// Deserialization succeeds even when the JSON contains no entry for a skipped field.
#[test]
fn test_skip_deserialize_missing_key() {
    #[derive(AutoJsJson, Clone, Debug, PartialEq, Default)]
    pub struct Data {
        pub value: i32,
        #[js_json(skip)]
        pub tmp: String,
    }

    // Build JSON that only has "value" — no "tmp" key at all.
    let json = JsJson::Object(std::collections::BTreeMap::from([(
        "value".to_string(),
        JsJson::Number(JsJsonNumber(7.0)),
    )]));

    let restored = Data::from_json(JsJsonContext::new(""), json)
        .unwrap_or_else(|e| panic!("Deserialization failed: {}", e.convert_to_string()));

    assert_eq!(restored.value, 7);
    assert_eq!(restored.tmp, String::new());
}
