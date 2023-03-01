use std::{sync::Arc, collections::{HashMap, VecDeque}};
use reqwest::Response;
use serde_json::{Value, Number, Map};

use crate::serve::{
    wasm::{FetchRequest, FetchResponse},
    js_value::JsJson
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RequestBody {
    Text(String),
    Json(JsJson),
    Binary(Vec<u8>),
}

fn convert_to_jsjson(value: Value) -> JsJson {
    match value {
        Value::Bool(true) => JsJson::True,
        Value::Bool(false) => JsJson::False,
        Value::Null => JsJson::Null,
        Value::Number(value) => {
            if let Some(value) = value.as_f64() {
                return JsJson::Number(value);
            }

            if let Some(value) = value.as_i64() {
                return JsJson::Number(value as f64);
            }

            if let Some(value) = value.as_u64() {
                return JsJson::Number(value as f64);
            }

            unreachable!();
        }
        Value::String(value) => JsJson::String(value),
        Value::Array(list) => {
            JsJson::List(
                list
                    .into_iter()
                    .map(convert_to_jsjson)
                    .collect::<Vec<_>>()
            )
        },
        Value::Object(object) => {
            let mut result = HashMap::new();

            for (prop_name, prop_value) in object {
                result.insert(prop_name, convert_to_jsjson(prop_value));
            }

            JsJson::Object(result)
        }
    }
}

fn convert_to_jsvalue(value: JsJson) -> Value {
    match value {
        JsJson::True => Value::Bool(true),
        JsJson::False => Value::Bool(false),
        JsJson::Null => Value::Null,
        JsJson::Number(value) => {
            Value::Number(Number::from_f64(value).unwrap())
        },
        JsJson::String(value) => Value::String(value),
        JsJson::List(list) => {
            let list = list
                .into_iter()
                .map(convert_to_jsvalue)
                .collect::<Vec<_>>();
            Value::Array(list)
        }
        JsJson::Object(object) => {
            let mut map = Map::new();

            for (prop_name, prop_value) in object {
                map.insert(prop_name, convert_to_jsvalue(prop_value));
            }

            Value::Object(map)
        }
    }
}

pub async fn send_request(request_params: Arc<FetchRequest>) -> FetchResponse {
    match send_request_inner(request_params).await {
        Some((status, body)) => {
            FetchResponse {
                success: true,
                status,
                body,
            }
        },
        None => {
            FetchResponse {
                success: false,
                status: 0,
                body: RequestBody::Text(String::from("")),
            }
        }
    }
}

enum BodyToSend {
    None,
    String(String),
    Vec(Vec<u8>),
}

fn get_headers_and_body(mut headers: HashMap<String, String>, body: &Option<RequestBody>) -> (HashMap<String, String>, BodyToSend) {
    match body.clone() {
        None => {
            (headers, BodyToSend::None)
        }
        Some(RequestBody::Text(text)) => {
            if !headers.contains_key("content-type") {
                headers.insert("content-type".into(), "text/plain; charset=utf-8".into());
            }

            (headers, BodyToSend::String(text))
        }
        Some(RequestBody::Binary(buffer)) => {
            (headers, BodyToSend::Vec(buffer))
        }
        Some(RequestBody::Json(json)) => {
            if !headers.contains_key("content-type") {
                headers.insert("content-type".into(), "application/json; charset=utf-8".into());
            }

            let value = convert_to_jsvalue(json);
            let json_str = serde_json::to_string(&value).unwrap();

            (headers, BodyToSend::String(json_str))
        }
    }
}

fn clear_headers(headers: &HashMap<String, String>) -> HashMap<String, String> {
    headers.iter()
        .map(|(key, value)| {
            let key = key.to_lowercase().trim().to_string();
            (key, value.to_string())
        })
        .collect::<HashMap<_, _>>()
}

enum ResponseType {
    Json,
    Text,
    Bin
}

fn get_response_type(response: &Response) -> ResponseType {
    let content_type = response.headers().get("content-type");

    let Some(content_type) = content_type else {
        return ResponseType::Bin;
    };

    let content_type = match content_type.to_str() {
        Ok(content_type) => content_type.to_string(),
        Err(_) => {
            return ResponseType::Bin;
        }
    };

    let mut chunks = content_type.split(';').collect::<VecDeque<_>>();

    match chunks.pop_front() {
        Some(chunk_content_type) => {
            let chunk_content_type = chunk_content_type.trim().to_lowercase();

            if chunk_content_type == "application/json" {
                return ResponseType::Json;
            }

            if chunk_content_type == "text/plain" {
                return ResponseType::Text;
            }

            ResponseType::Bin
        },
        None => {
            ResponseType::Bin
        }
    }
}

async fn send_request_inner(request_params: Arc<FetchRequest>) -> Option<(u32, RequestBody)> {
    let client = reqwest::Client::new();

    let mut request = match request_params.method.trim().to_lowercase().as_str() {
        "get" => client.get(&request_params.url),
        "post" => client.post(&request_params.url),
        _ => {
            unreachable!();
        }
    };

    let headers = clear_headers(&request_params.headers);
    let (headers, body) = get_headers_and_body(headers, &request_params.body);

    for (key, value) in headers {
        request = request.header(key, value);
    }

    match body {
        BodyToSend::None => {},
        BodyToSend::String(body) => {
            request = request.body(body);
        },
        BodyToSend::Vec(buffer) => {
            request = request.body(buffer);
        },
    }

    let response = match request.send().await {
        Ok(response) => response,
        Err(error) => {
            log::error!("Error send: {error}");
            return None;
        }
    };

    let status = response.status().as_u16() as u32;
    let response_type = get_response_type(&response);

    let buffer = match response.bytes().await {
        Ok(response) => response.to_vec(),
        Err(error) => {
            log::error!("data fetching error: {error}");
            return None;
        }
    };

    match response_type {
        ResponseType::Text => {
            let Ok(body) = String::from_utf8(buffer) else {
                log::error!("response decoding problem");
                return None;
            };

            Some((status, RequestBody::Text(body)))
        }
        ResponseType::Json => {
            match serde_json::from_slice::<Value>(buffer.as_slice()) {
                Ok(json) => {
                    let json = convert_to_jsjson(json);
                    Some((status, RequestBody::Json(json)))
                },
                Err(error) => {
                    log::error!("response decoding json problem error={error}");
                    None
                }
            }
        }
        ResponseType::Bin => {
            Some((status, RequestBody::Binary(buffer)))
        }
    }
}

