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

#[derive(AutoJsJson, PartialEq, Debug, Clone)]
enum TestEnum {
    Layer { heights: Vec<u8> },
}

/// A JSON list of numbers, as a server's response or a fetch made while rendering on the
/// server brings `Vec<u8>`.
fn numbers(numbers: &[f64]) -> JsJson {
    JsJson::List(
        numbers
            .iter()
            .map(|&number| JsJson::Number(vertigo::JsJsonNumber(number)))
            .collect(),
    )
}

fn object(fields: Vec<(&str, JsJson)>) -> JsJson {
    JsJson::from(fields)
}

#[test]
fn bytes_are_read_from_a_list_of_numbers_too() {
    let json = object(vec![
        ("data", numbers(&[0.0, 1.0, 255.0])),
        ("name", JsJson::String("test".to_string())),
    ]);
    assert_eq!(
        TestStruct::from_json(JsJsonContext::new("test"), json).map_err(|e| e.to_string()),
        Ok(TestStruct {
            data: vec![0, 1, 255],
            name: "test".to_string(),
        })
    );

    let json = object(vec![(
        "Layer",
        object(vec![("heights", numbers(&[20.0, 21.0]))]),
    )]);
    assert_eq!(
        TestEnum::from_json(JsJsonContext::new("test"), json).map_err(|e| e.to_string()),
        Ok(TestEnum::Layer {
            heights: vec![20, 21]
        })
    );
}

#[test]
fn a_list_with_a_number_that_is_no_byte_is_refused() {
    for number in [256.0, -1.0, 1.5] {
        let json = object(vec![
            ("data", numbers(&[1.0, number])),
            ("name", JsJson::String("test".to_string())),
        ]);
        let result = TestStruct::from_json(JsJsonContext::new("test"), json);
        assert!(
            matches!(&result, Err(error) if error.to_string().contains("integer(u8) expected")),
            "{number}: {result:?}"
        );
    }
}

#[test]
fn bytes_are_neither_a_string_nor_a_list_of_strings() {
    for data in [
        JsJson::String("AAEC".to_string()),
        JsJson::List(vec![JsJson::String("1".to_string())]),
    ] {
        let json = object(vec![
            ("data", data),
            ("name", JsJson::String("test".to_string())),
        ]);
        assert!(TestStruct::from_json(JsJsonContext::new("test"), json).is_err());
    }
}
