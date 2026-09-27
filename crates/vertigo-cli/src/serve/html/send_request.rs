use actix_web::http::Method;
use serde_json::{Map, Number, Value};
use std::collections::BTreeMap;
use vertigo::{
    JsJson, JsJsonNumber,
    dev::{SsrFetchRequest, SsrFetchRequestBody, SsrFetchResponse, SsrFetchResponseContent},
};

use super::fetch_url::{FetchTarget, SSR_FETCH_HEADER};

fn convert_to_jsjson(value: Value) -> JsJson {
    match value {
        Value::Bool(true) => JsJson::True,
        Value::Bool(false) => JsJson::False,
        Value::Null => JsJson::Null,
        Value::Number(value) => {
            if let Some(value) = value.as_f64() {
                return JsJson::Number(JsJsonNumber(value));
            }

            if let Some(value) = value.as_i64() {
                return JsJson::Number(JsJsonNumber(value as f64));
            }

            if let Some(value) = value.as_u64() {
                return JsJson::Number(JsJsonNumber(value as f64));
            }

            log::error!("Unreachable in convert_to_jsjson, value: {value}");
            JsJson::Number(JsJsonNumber(0.0))
        }
        Value::String(value) => JsJson::String(value),
        Value::Array(list) => {
            JsJson::List(list.into_iter().map(convert_to_jsjson).collect::<Vec<_>>())
        }
        Value::Object(object) => {
            let mut result = vertigo::JsObject::new();

            for (prop_name, prop_value) in object {
                result.insert(prop_name, convert_to_jsjson(prop_value));
            }

            JsJson::Object(result)
        }
    }
}

fn convert_to_serde_value(value: JsJson) -> Value {
    match value {
        JsJson::True => Value::Bool(true),
        JsJson::False => Value::Bool(false),
        JsJson::Null => Value::Null,
        JsJson::Undefined => Value::Null, // JSON doesn't have undefined, use null
        JsJson::Number(JsJsonNumber(value)) => {
            let Some(n) = Number::from_f64(value) else {
                log::error!("Invalid float in convert_to_serde_value: {value}");
                return Value::Null;
            };
            Value::Number(n)
        }
        JsJson::String(value) => Value::String(value),
        JsJson::List(list) => {
            let list = list
                .into_iter()
                .map(convert_to_serde_value)
                .collect::<Vec<_>>();
            Value::Array(list)
        }
        JsJson::Object(object) => {
            let mut map = Map::new();

            for (prop_name, prop_value) in object {
                map.insert(prop_name, convert_to_serde_value(prop_value));
            }

            Value::Object(map)
        }
        JsJson::Vec(data) => {
            let list = data
                .into_iter()
                .map(|byte| Value::Number(Number::from(byte)))
                .collect();
            Value::Array(list)
        }
    }
}

/// `cookie` is the `Cookie` header of the browser request being rendered, if it had one.
pub async fn send_request(
    request_params: SsrFetchRequest,
    target: FetchTarget,
    cookie: Option<String>,
) -> SsrFetchResponse {
    send_request_inner(request_params, target, cookie).await
}

enum BodyToSend {
    None,
    String(String),
}

fn get_headers_and_body(
    mut headers: BTreeMap<String, String>,
    body: &SsrFetchRequestBody,
) -> (BTreeMap<String, String>, BodyToSend) {
    match body.clone() {
        SsrFetchRequestBody::None => (headers, BodyToSend::None),
        SsrFetchRequestBody::Data { data } => {
            if !headers.contains_key("content-type") {
                headers.insert(
                    "content-type".into(),
                    "application/json; charset=utf-8".into(),
                );
            }

            let value = convert_to_serde_value(data);
            let json_str = serde_json::to_string(&value)
                .inspect_err(|err| log::error!("Error serializing body: {err}"))
                .unwrap_or_default();

            (headers, BodyToSend::String(json_str))
        }
    }
}

fn clear_headers(headers: &BTreeMap<String, String>) -> BTreeMap<String, String> {
    headers
        .iter()
        .map(|(key, value)| {
            let key = key.to_lowercase().trim().to_string();
            (key, value.to_string())
        })
        .collect::<BTreeMap<_, _>>()
}

/// The app's headers, plus the browser's cookies when the fetch goes to the server's own origin.
///
/// That is what the browser does with a same-origin `fetch`: the API sees the same session
/// during SSR as after hydration, so the rendered page matches what the browser will show.
/// Other origins never get the cookies. The browser also ignores a `Cookie` header set by
/// the app (a forbidden header name) and sends its own, so ours takes precedence.
fn request_headers(
    app_headers: &BTreeMap<String, String>,
    target: &FetchTarget,
    cookie: Option<&str>,
) -> BTreeMap<String, String> {
    let mut headers = clear_headers(app_headers);

    if let (FetchTarget::Local(_), Some(cookie)) = (target, cookie) {
        headers.insert("cookie".into(), cookie.into());
    }

    headers
}

async fn send_request_inner(
    request_params: SsrFetchRequest,
    target: FetchTarget,
    cookie: Option<String>,
) -> SsrFetchResponse {
    let client = awc::Client::new();

    let mut request = {
        let method = request_params.method.to_str().trim().to_uppercase();
        let Ok(method) = Method::from_bytes(method.as_bytes()) else {
            return SsrFetchResponse::Err {
                message: "send_request_inner - InvalidMethod".into(),
            };
        };

        client.request(method, target.url())
    };

    if let FetchTarget::Local(_) = target {
        request = request.append_header((SSR_FETCH_HEADER, "1"));
    }

    let headers = request_headers(&request_params.headers, &target, cookie.as_deref());
    let (headers, body) = get_headers_and_body(headers, &request_params.body);

    for (key, value) in headers {
        request = request.append_header((key, value));
    }

    let response = match body {
        BodyToSend::None => request.send(),
        BodyToSend::String(body) => request.send_body(body),
    };

    let mut response = match response.await {
        Ok(response) => response,
        Err(error) => {
            return SsrFetchResponse::Err {
                message: format!("Error send: {error}"),
            };
        }
    };

    let status = response.status().as_u16() as u32;

    let content_type = response
        .headers()
        .get("Content-Type")
        .cloned()
        .and_then(|v| v.to_str().ok().map(ToString::to_string));

    let buffer = match response.body().await {
        Ok(response) => response.to_vec(),
        Err(error) => {
            return SsrFetchResponse::Err {
                message: format!("data fetching error: {error}"),
            };
        }
    };

    SsrFetchResponse::Ok {
        status,
        response: decode_body(content_type.as_deref(), &buffer),
    }
}

/// Decodes a response body the same way the browser does (`decodeBody` in `fetchExec.ts`).
///
/// A body that isn't JSON falls back to text rather than failing.
fn decode_body(content_type: Option<&str>, buffer: &[u8]) -> SsrFetchResponseContent {
    let is_text_plain = content_type
        .and_then(|v| v.split(';').next())
        .is_some_and(|mime| mime.trim().eq_ignore_ascii_case("text/plain"));

    if is_text_plain {
        return SsrFetchResponseContent::Text(String::from_utf8_lossy(buffer).into());
    }

    if buffer.is_empty() {
        return SsrFetchResponseContent::Json(JsJson::Null);
    }

    match serde_json::from_slice::<Value>(buffer) {
        Ok(json) => SsrFetchResponseContent::Json(convert_to_jsjson(json)),
        Err(_) => SsrFetchResponseContent::Text(String::from_utf8_lossy(buffer).into()),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use vertigo::{JsJson, dev::SsrFetchResponseContent};

    use super::decode_body;
    use super::{FetchTarget, request_headers};

    const COOKIE: &str = "session=abc; theme=dark";

    fn text(content: SsrFetchResponseContent) -> String {
        match content {
            SsrFetchResponseContent::Text(text) => text,
            SsrFetchResponseContent::Json(json) => panic!("expected text, got json: {json:?}"),
        }
    }

    fn json(content: SsrFetchResponseContent) -> JsJson {
        match content {
            SsrFetchResponseContent::Json(json) => json,
            SsrFetchResponseContent::Text(text) => panic!("expected json, got text: {text:?}"),
        }
    }

    #[test]
    fn json_body_is_parsed() {
        let mut expected = vertigo::JsObject::new();
        expected.insert("key".to_string(), JsJson::String("value".to_string()));
        assert_eq!(
            json(decode_body(Some("application/json"), br#"{"key":"value"}"#)),
            JsJson::Object(expected)
        );
        assert_eq!(json(decode_body(None, b"true")), JsJson::True);
    }

    #[test]
    fn non_json_body_without_content_type_falls_back_to_text() {
        assert_eq!(text(decode_body(None, b"OK")), "OK");
        assert_eq!(
            text(decode_body(Some("text/html"), b"<html>Bad Gateway</html>")),
            "<html>Bad Gateway</html>"
        );
    }

    #[test]
    fn text_plain_is_never_parsed_as_json() {
        assert_eq!(text(decode_body(Some("text/plain"), b"true")), "true");
        assert_eq!(
            text(decode_body(Some("text/plain; charset=utf-8"), b"123")),
            "123"
        );
        assert_eq!(text(decode_body(Some("Text/Plain;charset=utf-8"), b"")), "");
    }

    #[test]
    fn empty_body_is_null() {
        assert_eq!(json(decode_body(None, b"")), JsJson::Null);
        assert_eq!(
            json(decode_body(Some("application/json"), b"")),
            JsJson::Null
        );
    }

    fn app_headers() -> BTreeMap<String, String> {
        BTreeMap::from([("Accept".to_string(), "application/json".to_string())])
    }

    #[test]
    fn own_origin_gets_browser_cookies() {
        let target = FetchTarget::Local("http://127.0.0.1:8080/api/me".to_string());

        let headers = request_headers(&app_headers(), &target, Some(COOKIE));

        assert_eq!(headers.get("cookie").map(String::as_str), Some(COOKIE));
        assert_eq!(
            headers.get("accept").map(String::as_str),
            Some("application/json")
        );
    }

    #[test]
    fn other_origin_never_gets_them() {
        let target = FetchTarget::External("https://api.example.com/me".to_string());

        let headers = request_headers(&app_headers(), &target, Some(COOKIE));

        assert_eq!(headers.get("cookie"), None);
    }

    #[test]
    fn browser_cookies_replace_the_apps_own() {
        let target = FetchTarget::Local("http://127.0.0.1:8080/api/me".to_string());
        let mut app = app_headers();
        app.insert("Cookie".to_string(), "forged=1".to_string());

        let headers = request_headers(&app, &target, Some(COOKIE));

        assert_eq!(headers.get("cookie").map(String::as_str), Some(COOKIE));
    }

    #[test]
    fn without_cookies_app_headers_stay_as_they_were() {
        let target = FetchTarget::Local("http://127.0.0.1:8080/api/posts".to_string());

        assert_eq!(
            request_headers(&app_headers(), &target, None),
            BTreeMap::from([("accept".to_string(), "application/json".to_string())])
        );
    }
}
