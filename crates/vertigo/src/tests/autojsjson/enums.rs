use std::collections::BTreeMap;
use std::error::Error;

use crate::{self as vertigo, JsJson, JsJsonContext};
use crate::{AutoJsJson, JsJsonDeserialize, JsJsonSerialize};

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
fn test_adjacently_tagged_enum() -> Result<(), Box<dyn Error>> {
    #[derive(AutoJsJson, Clone, Debug, PartialEq)]
    #[js_json(tag = "t", content = "c")]
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
    let empty_tuple = TestType::EmptyTuple();
    let empty_struct = TestType::EmptyStruct {};
    let nothing = TestType::Nothing;

    // Verify on-wire shape.
    let nothing_json = nothing.clone().to_json();
    assert_eq!(
        nothing_json,
        JsJson::Object(BTreeMap::from([(
            "t".to_string(),
            JsJson::String("Nothing".to_string()),
        )]))
    );

    let somestring_json = somestring.clone().to_json();
    assert_eq!(
        somestring_json,
        JsJson::Object(BTreeMap::from([
            ("t".to_string(), JsJson::String("Somestring".to_string())),
            ("c".to_string(), JsJson::String("asdf".to_string())),
        ]))
    );

    let point_json = point.clone().to_json();
    assert_eq!(
        point_json,
        JsJson::Object(BTreeMap::from([
            ("t".to_string(), JsJson::String("Point".to_string())),
            (
                "c".to_string(),
                JsJson::Object(BTreeMap::from([
                    ("x".to_string(), 10u32.to_json()),
                    ("y".to_string(), JsJson::String("raz".to_string())),
                ]))
            ),
        ]))
    );

    let tuple_json = tuple.clone().to_json();
    assert_eq!(
        tuple_json,
        JsJson::Object(BTreeMap::from([
            ("t".to_string(), JsJson::String("Tuple".to_string())),
            (
                "c".to_string(),
                JsJson::List(vec![JsJson::String("raz".to_string()), 10u32.to_json()])
            ),
        ]))
    );

    // Round-trip every variant.
    for original in [
        somestring.clone(),
        point.clone(),
        tuple.clone(),
        number.clone(),
        empty_tuple.clone(),
        empty_struct.clone(),
        nothing.clone(),
    ] {
        let encoded = original.clone().to_json();
        let decoded = TestType::from_json(JsJsonContext::new(""), encoded)
            .unwrap_or_else(|err| panic!("round-trip failed: {}", err.convert_to_string()));
        assert_eq!(original, decoded);
    }

    // Unknown tag produces an error.
    let bogus = JsJson::Object(BTreeMap::from([(
        "t".to_string(),
        JsJson::String("Bogus".to_string()),
    )]));
    let err = TestType::from_json(JsJsonContext::new(""), bogus)
        .err()
        .ok_or("expected error for unknown tag")?;
    assert!(
        err.convert_to_string().contains("Bogus"),
        "error should mention the unknown tag, got: {}",
        err.convert_to_string()
    );

    Ok(())
}
