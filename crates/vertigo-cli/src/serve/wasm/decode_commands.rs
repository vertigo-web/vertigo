use std::collections::HashMap;

use crate::serve::{
    html::RequestBody,
    js_value::{from_json, JsJson, JsValue},
};

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

pub fn match_plain_response(arg: &JsValue) -> Result<String, ()> {
    let matcher = Match::new(arg)?;
    let matcher = matcher.test_list(&["plain_response"])?;
    let (matcher, body) = matcher.string()?;
    matcher.end()?;

    Ok(body)
}

pub fn match_history_router(arg: &JsValue) -> Result<(), ()> {
    let matcher = Match::new(arg)?;
    let matcher = matcher.test_list(&["api"])?;
    let matcher = matcher.test_list(&["get", "historyLocation"])?;
    let matcher = matcher.test_list(&["call", "get"])?;
    matcher.end()?;

    Ok(())
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
