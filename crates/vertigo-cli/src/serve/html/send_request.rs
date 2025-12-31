use actix_web::http::Method;
use serde_json::{Map, Number, Value};
use std::collections::BTreeMap;
use vertigo::{
    JsJson, JsJsonNumber,
    dev::{SsrFetchRequest, SsrFetchRequestBody, SsrFetchResponse, SsrFetchResponseContent},
};

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
            let mut result = BTreeMap::new();

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

pub async fn send_request(request_params: SsrFetchRequest) -> SsrFetchResponse {
    send_request_inner(request_params).await
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

async fn send_request_inner(request_params: SsrFetchRequest) -> SsrFetchResponse {
    let client = awc::Client::new();

    let mut request = {
        let method = request_params.method.to_str().trim().to_uppercase();
        let Ok(method) = Method::from_bytes(method.as_bytes()) else {
            return SsrFetchResponse::Err {
                message: "send_request_inner - InvalidMethod".into(),
            };
        };

        client.request(method, &request_params.url)
    };

    let headers = clear_headers(&request_params.headers);
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

    match content_type {
        Some(v) if v.starts_with("text/plain;") => {
            let text_response = String::from_utf8_lossy(&buffer);
            SsrFetchResponse::Ok {
                status,
                response: SsrFetchResponseContent::Text(text_response.into()),
            }
        }
        _ => match serde_json::from_slice::<Value>(buffer.as_slice()) {
            Ok(json) => {
                let json = convert_to_jsjson(json);

                SsrFetchResponse::Ok {
                    status,
                    response: SsrFetchResponseContent::Json(json),
                }
            }
            Err(error) => SsrFetchResponse::Err {
                message: format!("response decoding json problem error={error}"),
            },
        },
    }
}
