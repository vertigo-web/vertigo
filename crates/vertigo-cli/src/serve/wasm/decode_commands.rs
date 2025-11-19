use super::js_value_match::Match;
use vertigo::{JsJson, JsValue};

pub fn match_cookie_command(arg: &JsValue) -> Result<(), ()> {
    let matcher = Match::new(arg)?;
    let matcher = matcher.test_list(&["api"])?;
    let _ = matcher.test_list(&["get", "cookie"])?;

    Ok(())
}

pub fn match_history_router_push(arg: &JsValue) -> Result<String, ()> {
    let matcher = Match::new(arg)?;
    let matcher = matcher.test_list(&["api"])?;
    let matcher = matcher.test_list(&["get", "historyLocation"])?;
    let (matcher, url) = matcher.test_list_with_fn(|matcher: Match| -> Result<String, ()> {
        let matcher = matcher.str("call")?;
        let matcher = matcher.str("push")?;
        let (matcher, url) = matcher.string()?;
        matcher.end()?;

        Ok(url)
    })?;
    matcher.end()?;

    Ok(url)
}

pub fn match_get_env(arg: &JsValue) -> Result<String, ()> {
    let matcher = Match::new(arg)?;
    let matcher = matcher.test_list(&["api"])?;
    let (matcher, name) = matcher.test_list_with_fn(|matcher: Match| -> Result<String, ()> {
        let matcher = matcher.str("call")?;
        let matcher = matcher.str("get_env")?;
        let (matcher, name) = matcher.string()?;
        matcher.end()?;

        Ok(name)
    })?;
    matcher.end()?;

    Ok(name)
}

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

pub fn match_log(arg: &JsValue) -> Result<(String, String), ()> {
    let matcher = Match::new(arg)?;

    let matcher = matcher.test_list(&["root", "window"])?;
    let matcher = matcher.test_list(&["get", "console"])?;
    let (matcher, (log_type, log_message)) =
        matcher.test_list_with_fn(|matcher: Match| -> Result<(String, String), ()> {
            let matcher = matcher.str("call")?;
            let (matcher, log_type) = matcher.string()?;
            let (matcher, log_message) = matcher.string()?;
            let (matcher, _) = matcher.string()?;
            let (matcher, _) = matcher.string()?;
            let (matcher, _) = matcher.string()?;
            matcher.end()?;

            Ok((log_type, log_message))
        })?;

    matcher.end()?;

    Ok((log_type, log_message))
}

#[cfg(test)]
mod tests {
    use vertigo::{JsJson, JsValue};

    use super::*;

    #[test]
    fn test_match_cookie_command() {
        let value = JsValue::List(vec![
            JsValue::List(vec![JsValue::from("api")]),
            JsValue::List(vec![JsValue::from("get"), JsValue::from("cookie")]),
        ]);
        assert_eq!(match_cookie_command(&value), Ok(()));
    }

    #[test]
    fn test_match_history_router_push() {
        let value = JsValue::List(vec![
            JsValue::List(vec![JsValue::from("api")]),
            JsValue::List(vec![JsValue::from("get"), JsValue::from("historyLocation")]),
            JsValue::List(vec![
                JsValue::from("call"),
                JsValue::from("push"),
                JsValue::from("/new/url"),
            ]),
        ]);
        assert_eq!(
            match_history_router_push(&value),
            Ok("/new/url".to_string())
        );
    }

    #[test]
    fn test_match_get_env() {
        let value = JsValue::List(vec![
            JsValue::List(vec![JsValue::from("api")]),
            JsValue::List(vec![
                JsValue::from("call"),
                JsValue::from("get_env"),
                JsValue::from("MY_VAR"),
            ]),
        ]);
        assert_eq!(match_get_env(&value), Ok("MY_VAR".to_string()));
    }

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

    #[test]
    fn test_match_log() {
        let value = JsValue::List(vec![
            JsValue::List(vec![JsValue::from("root"), JsValue::from("window")]),
            JsValue::List(vec![JsValue::from("get"), JsValue::from("console")]),
            JsValue::List(vec![
                JsValue::from("call"),
                JsValue::from("log"),         // log_type
                JsValue::from("Hello world"), // log_message
                JsValue::from(""),            // ignored
                JsValue::from(""),            // ignored
                JsValue::from(""),            // ignored
            ]),
        ]);
        assert_eq!(
            match_log(&value),
            Ok(("log".to_string(), "Hello world".to_string()))
        );
    }

    fn json_obj(items: Vec<(&str, JsJson)>) -> JsJson {
        JsJson::Object(items.into_iter().map(|(k, v)| (k.to_string(), v)).collect())
    }

    fn json_str(val: &str) -> JsJson {
        JsJson::String(val.to_string())
    }
}
