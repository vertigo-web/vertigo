use super::js_value_match::Match;
use vertigo::{JsJson, JsValue};

pub fn match_dom_bulk_update(arg: &JsValue) -> Result<JsJson, ()> {
    let matcher = Match::new(arg)?;
    let matcher = matcher.test_list(&["api"])?;
    let matcher = matcher.test_list(&["get", "dom"])?;
    let (matcher, data) = matcher.test_list_with_fn(|matcher: Match| -> Result<JsJson, ()> {
        let matcher = matcher.str("call")?;
        let matcher = matcher.str("dom_bulk_update")?;
        let (matcher, data) = matcher.json()?;
        matcher.end()?;

        Ok(data)
    })?;
    matcher.end()?;

    Ok(data)
}

#[cfg(test)]
mod tests {
    use vertigo::{JsJson, JsValue};

    use super::*;

    #[test]
    fn test_match_dom_bulk_update() {
        let json_data = json_obj(vec![("action", json_str("update"))]);
        let value = JsValue::List(vec![
            JsValue::List(vec![JsValue::from("api")]),
            JsValue::List(vec![JsValue::from("get"), JsValue::from("dom")]),
            JsValue::List(vec![
                JsValue::from("call"),
                JsValue::from("dom_bulk_update"),
                JsValue::Json(json_data.clone()),
            ]),
        ]);
        assert_eq!(match_dom_bulk_update(&value), Ok(json_data));
    }

    fn json_obj(items: Vec<(&str, JsJson)>) -> JsJson {
        JsJson::Object(items.into_iter().map(|(k, v)| (k.to_string(), v)).collect())
    }

    fn json_str(val: &str) -> JsJson {
        JsJson::String(val.to_string())
    }
}
