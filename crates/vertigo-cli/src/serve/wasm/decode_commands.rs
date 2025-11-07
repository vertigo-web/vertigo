use std::collections::HashMap;
use vertigo::{from_json, JsJson, JsValue};

use crate::serve::html::RequestBody;

use super::{get_now::get_now, js_value_match::Match, message::CallWebsocketResult, FetchRequest};

pub fn convert_value_to_body(body: JsValue) -> Result<Option<RequestBody>, String> {
    match body {
        JsValue::Json(json) => Ok(Some(RequestBody::Json(json))),
        JsValue::String(text) => Ok(Some(RequestBody::Text(text))),
        JsValue::Vec(buffer) => Ok(Some(RequestBody::Binary(buffer))),
        JsValue::Undefined => Ok(None),
        other => {
            let typename = other.typename();
            let message = format!("expected JsValue::Json or JsValue::Text or JsValue::Binary, received JsValue::{typename}");
            Err(message)
        }
    }
}

pub fn convert_body_to_value(body: RequestBody) -> JsValue {
    match body {
        RequestBody::Json(json) => JsValue::Json(json),
        RequestBody::Text(text) => JsValue::String(text),
        RequestBody::Binary(buffer) => JsValue::Vec(buffer),
    }
}

pub fn match_is_browser(arg: &JsValue) -> Result<(), ()> {
    let matcher = Match::new(arg)?;
    let matcher = matcher.test_list(&["api"])?;
    let _ = matcher.test_list(&["call", "isBrowser"])?;

    Ok(())
}

pub fn match_cookie_command(arg: &JsValue) -> Result<(), ()> {
    let matcher = Match::new(arg)?;
    let matcher = matcher.test_list(&["api"])?;
    let _ = matcher.test_list(&["get", "cookie"])?;

    Ok(())
}

pub fn match_history_router(arg: &JsValue) -> Result<(), ()> {
    let matcher = Match::new(arg)?;
    let matcher = matcher.test_list(&["api"])?;
    let matcher = matcher.test_list(&["get", "historyLocation"])?;
    let matcher = matcher.test_list(&["call", "get"])?;
    matcher.end()?;

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

pub fn match_history_router_callback(arg: &JsValue) -> Result<(), ()> {
    let matcher = Match::new(arg)?;
    let matcher = matcher.test_list(&["api"])?;
    let matcher = matcher.test_list(&["get", "historyLocation"])?;
    let (matcher, _) = matcher.test_list_with_fn(|matcher: Match| -> Result<u64, ()> {
        let matcher = matcher.str("call")?;
        let matcher = matcher.str("add")?;
        let (matcher, callback_id) = matcher.u64()?;
        matcher.end()?;

        Ok(callback_id)
    })?;
    matcher.end()?;

    Ok(())
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

pub fn match_date_now(arg: &JsValue) -> Result<JsValue, ()> {
    let matcher = Match::new(arg)?;

    let matcher = matcher.test_list(&["root", "window"])?;
    let matcher = matcher.test_list(&["get", "Date"])?;
    let matcher = matcher.test_list(&["call", "now"])?;
    matcher.end()?;

    let time = get_now().as_millis();
    Ok(JsValue::I64(time as i64))
}

pub fn match_websocket(arg: &JsValue) -> Result<(), ()> {
    let matcher = Match::new(arg)?;

    let matcher = matcher.test_list(&["api"])?;
    let _ = matcher.test_list(&["get", "websocket"])?;

    Ok(())
}

pub fn match_interval(arg: &JsValue) -> Result<CallWebsocketResult, ()> {
    let matcher = Match::new(arg)?;

    let matcher = matcher.test_list(&["api"])?;
    let matcher = matcher.test_list(&["get", "interval"])?;

    let (matcher, result) = matcher.test_list_with_fn(|matcher| {
        let matcher = matcher.str("call")?;
        if let Ok(matcher) = matcher.str("timeout_set") {
            let (matcher, time) = matcher.u32()?;
            let (_, callback_id) = matcher.u64()?;

            return Ok(CallWebsocketResult::TimeoutSet { time, callback_id });
        }

        Ok(CallWebsocketResult::NoResult)
    })?;

    matcher.end()?;

    Ok(result)
}

pub fn match_fetch(arg: &JsValue) -> Result<(u64, FetchRequest), ()> {
    let matcher = Match::new(arg)?;

    let matcher = matcher.test_list(&["api"])?;
    let matcher = matcher.test_list(&["get", "fetch"])?;

    let (matcher, result) = matcher.test_list_with_fn(|matcher| {
        let matcher = matcher.str("call")?;
        let matcher = matcher.str("fetch_send_request")?;
        let (matcher, callback_id) = matcher.u64()?;
        let (matcher, method) = matcher.string()?;
        let (matcher, url) = matcher.string()?;
        let (matcher, headers) = matcher.json()?;
        let (matcher, body) = matcher.get_any()?;
        matcher.end()?;

        let headers = from_json::<HashMap<String, String>>(headers).map_err(|error| {
            log::error!("error decode headers: {error}");
        })?;

        let body = convert_value_to_body(body).map_err(|error| {
            log::error!("error decode body: {error}");
        })?;

        Ok((
            callback_id,
            FetchRequest {
                method,
                url,
                headers,
                body,
            },
        ))
    })?;

    matcher.end()?;

    Ok(result)
}

pub fn match_is_set_status(arg: &JsValue) -> Result<u16, ()> {
    let matcher = Match::new(arg)?;
    let matcher = matcher.test_list(&["set_status"])?;
    let (matcher, status) = matcher.u32()?;
    matcher.end()?;

    Ok(status as u16)
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use vertigo::{JsJson, JsValue};

    use super::*;

    #[test]
    fn test_convert_body_roundtrip() {
        // Json
        let json_val = json_obj(vec![("a", json_num(1.0))]);
        let js_value_json = JsValue::Json(json_val.clone());
        let body_json = convert_value_to_body(js_value_json.clone())
            .unwrap()
            .unwrap();
        assert_eq!(body_json, RequestBody::Json(json_val));
        assert_eq!(convert_body_to_value(body_json), js_value_json);

        // Text
        let js_value_text = JsValue::String("hello".to_string());
        let body_text = convert_value_to_body(js_value_text.clone())
            .unwrap()
            .unwrap();
        assert_eq!(body_text, RequestBody::Text("hello".to_string()));
        assert_eq!(convert_body_to_value(body_text), js_value_text);

        // Binary
        let js_value_bin = JsValue::Vec(vec![1, 2, 3]);
        let body_bin = convert_value_to_body(js_value_bin.clone())
            .unwrap()
            .unwrap();
        assert_eq!(body_bin, RequestBody::Binary(vec![1, 2, 3]));
        assert_eq!(convert_body_to_value(body_bin), js_value_bin);

        // Undefined
        let js_value_undef = JsValue::Undefined;
        let body_none = convert_value_to_body(js_value_undef).unwrap();
        assert!(body_none.is_none());
    }

    #[test]
    fn test_convert_value_to_body_invalid() {
        let value_invalid = JsValue::True;
        let result = convert_value_to_body(value_invalid);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "expected JsValue::Json or JsValue::Text or JsValue::Binary, received JsValue::true"
        );
    }

    #[test]
    fn test_match_is_browser() {
        let value = JsValue::List(vec![
            JsValue::List(vec![JsValue::from("api")]),
            JsValue::List(vec![JsValue::from("call"), JsValue::from("isBrowser")]),
        ]);
        assert_eq!(match_is_browser(&value), Ok(()));

        let invalid_value = JsValue::List(vec![
            JsValue::List(vec![JsValue::from("api")]),
            JsValue::List(vec![JsValue::from("call"), JsValue::from("wrong")]),
        ]);
        assert_eq!(match_is_browser(&invalid_value), Err(()));
    }

    #[test]
    fn test_match_cookie_command() {
        let value = JsValue::List(vec![
            JsValue::List(vec![JsValue::from("api")]),
            JsValue::List(vec![JsValue::from("get"), JsValue::from("cookie")]),
        ]);
        assert_eq!(match_cookie_command(&value), Ok(()));
    }

    #[test]
    fn test_match_history_router() {
        let value = JsValue::List(vec![
            JsValue::List(vec![JsValue::from("api")]),
            JsValue::List(vec![JsValue::from("get"), JsValue::from("historyLocation")]),
            JsValue::List(vec![JsValue::from("call"), JsValue::from("get")]),
        ]);
        assert_eq!(match_history_router(&value), Ok(()));
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
    fn test_match_history_router_callback() {
        let value = JsValue::List(vec![
            JsValue::List(vec![JsValue::from("api")]),
            JsValue::List(vec![JsValue::from("get"), JsValue::from("historyLocation")]),
            JsValue::List(vec![
                JsValue::from("call"),
                JsValue::from("add"),
                JsValue::U64(12345),
            ]),
        ]);
        assert_eq!(match_history_router_callback(&value), Ok(()));
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

    #[test]
    fn test_match_date_now() {
        let value = JsValue::List(vec![
            JsValue::List(vec![JsValue::from("root"), JsValue::from("window")]),
            JsValue::List(vec![JsValue::from("get"), JsValue::from("Date")]),
            JsValue::List(vec![JsValue::from("call"), JsValue::from("now")]),
        ]);
        let result = match_date_now(&value);
        assert!(result.is_ok());
    }

    #[test]
    fn test_match_websocket() {
        let value = JsValue::List(vec![
            JsValue::List(vec![JsValue::from("api")]),
            JsValue::List(vec![JsValue::from("get"), JsValue::from("websocket")]),
        ]);
        assert_eq!(match_websocket(&value), Ok(()));
    }

    #[test]
    fn test_match_interval() {
        let value_set = JsValue::List(vec![
            JsValue::List(vec![JsValue::from("api")]),
            JsValue::List(vec![JsValue::from("get"), JsValue::from("interval")]),
            JsValue::List(vec![
                JsValue::from("call"),
                JsValue::from("timeout_set"),
                JsValue::U32(1000),
                JsValue::U64(99),
            ]),
        ]);
        assert_eq!(
            match_interval(&value_set),
            Ok(CallWebsocketResult::TimeoutSet {
                time: 1000,
                callback_id: 99
            })
        );

        let value_other = JsValue::List(vec![
            JsValue::List(vec![JsValue::from("api")]),
            JsValue::List(vec![JsValue::from("get"), JsValue::from("interval")]),
            JsValue::List(vec![JsValue::from("call"), JsValue::from("other_call")]),
        ]);
        assert_eq!(
            match_interval(&value_other),
            Ok(CallWebsocketResult::NoResult)
        );
    }

    #[test]
    fn test_match_fetch() {
        let headers_json = json_obj(vec![("Content-Type", json_str("application/json"))]);
        let body_json = json_obj(vec![("data", json_str("test"))]);

        let value = JsValue::List(vec![
            JsValue::List(vec![JsValue::from("api")]),
            JsValue::List(vec![JsValue::from("get"), JsValue::from("fetch")]),
            JsValue::List(vec![
                JsValue::from("call"),
                JsValue::from("fetch_send_request"),
                JsValue::U64(123),                    // callback_id
                JsValue::from("POST"),                // method
                JsValue::from("https://example.com"), // url
                JsValue::Json(headers_json.clone()),  // headers
                JsValue::Json(body_json.clone()),     // body
            ]),
        ]);

        let expected_headers: HashMap<String, String> = HashMap::from_iter(vec![(
            "Content-Type".to_string(),
            "application/json".to_string(),
        )]);

        let expected_request = FetchRequest {
            method: "POST".to_string(),
            url: "https://example.com".to_string(),
            headers: expected_headers,
            body: Some(RequestBody::Json(body_json)),
        };

        let result = match_fetch(&value);
        assert!(result.is_ok());
        let (callback_id, request) = result.unwrap();

        assert_eq!(callback_id, 123);
        assert_eq!(request, expected_request);
    }

    #[test]
    fn test_match_fetch_no_body() {
        let headers_json = json_obj(vec![]);
        let value = JsValue::List(vec![
            JsValue::List(vec![JsValue::from("api")]),
            JsValue::List(vec![JsValue::from("get"), JsValue::from("fetch")]),
            JsValue::List(vec![
                JsValue::from("call"),
                JsValue::from("fetch_send_request"),
                JsValue::U64(124),                        // callback_id
                JsValue::from("GET"),                     // method
                JsValue::from("https://example.com/get"), // url
                JsValue::Json(headers_json.clone()),      // headers
                JsValue::Undefined,                       // body
            ]),
        ]);

        let expected_request = FetchRequest {
            method: "GET".to_string(),
            url: "https://example.com/get".to_string(),
            headers: HashMap::new(),
            body: None,
        };

        let result = match_fetch(&value);
        assert!(result.is_ok());
        let (callback_id, request) = result.unwrap();

        assert_eq!(callback_id, 124);
        assert_eq!(request, expected_request);
    }

    #[test]
    fn test_match_is_set_status() {
        let value = JsValue::List(vec![
            JsValue::List(vec![JsValue::from("set_status")]),
            JsValue::U32(404),
        ]);
        assert_eq!(match_is_set_status(&value), Ok(404u16));

        let invalid_value = JsValue::List(vec![JsValue::from("set_status"), JsValue::from("404")]);
        assert_eq!(match_is_set_status(&invalid_value), Err(()));
    }

    fn json_obj(items: Vec<(&str, JsJson)>) -> JsJson {
        JsJson::Object(items.into_iter().map(|(k, v)| (k.to_string(), v)).collect())
    }

    fn json_str(val: &str) -> JsJson {
        JsJson::String(val.to_string())
    }

    fn json_num(val: f64) -> JsJson {
        JsJson::Number(val)
    }
}
