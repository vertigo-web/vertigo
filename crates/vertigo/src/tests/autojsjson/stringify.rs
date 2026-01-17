use std::{fmt::Display, str::FromStr};

use vertigo_macro::AutoJsJson;

use crate::{JsJsonContext, JsJsonDeserialize, JsJsonSerialize};

#[derive(Clone, Debug, PartialEq)]
struct MyType(i64);

#[derive(Clone, Debug, PartialEq, AutoJsJson)]
struct MyType2(#[js_json(stringify)] MyType);

#[derive(AutoJsJson)]
struct MyStruct {
    #[js_json(stringify)]
    field: MyType,
    field2: MyType2,
}

impl Display for MyType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for MyType {
    type Err = std::num::ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.parse()?))
    }
}

#[test]
fn test_from_string() -> Result<(), JsJsonContext> {
    let x = MyType(5);
    let my_struct = MyStruct {
        field: x.clone(),
        field2: MyType2(x.clone()),
    };
    let json = my_struct.to_json();
    let recovered = MyStruct::from_json(JsJsonContext::new(""), json)?;
    assert_eq!(x, recovered.field);
    assert_eq!(x, recovered.field2.0);
    Ok(())
}
