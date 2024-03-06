use crate::{self as vertigo, JsJsonContext};
use crate::{AutoJsJson, JsJsonSerialize};

#[test]
fn raw_field_name() {
    #[derive(AutoJsJson)]
    pub struct Test {
        pub r#type: String,
        pub name: String,
    }

    let test = Test {
        r#type: "one".to_string(),
        name: "two".to_string(),
    };

    let test_js = test.to_json();
    let ctx = JsJsonContext::new("");
    let hash_map = match test_js.get_hashmap(&ctx) {
        Ok(map) => map,
        Err(_err) => panic!("Error unwrapping hash_map"),
    };

    assert!(hash_map.contains_key("type"));
    assert!(hash_map.contains_key("name"));
}
