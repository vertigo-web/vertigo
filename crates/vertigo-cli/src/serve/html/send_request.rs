use std::{sync::Arc, collections::HashMap};
use crate::serve::{wasm::{FetchRequest, FetchResponse}, js_value::JsJson};
use serde_json::Value;

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
                body: JsJson::Null,
            }
        }
    }
}

async fn send_request_inner(request_params: Arc<FetchRequest>) -> Option<(u32, JsJson)> {
    let client = reqwest::Client::new();

    let mut request = match request_params.method.trim().to_lowercase().as_str() {
        "get" => client.get(&request_params.url),
        "post" => client.post(&request_params.url),
        _ => {
            unreachable!();
        }
    };

    for (key, value) in &request_params.headers {
        request = request.header(key, value);
    }

    if let Some(body) = &request_params.body {
        request = request.body(body.clone());
    }

    let response = match request.send().await {
        Ok(response) => response,
        Err(error) => {
            log::error!("Error send: {error}");
            return None;
        }
    };

    let status = response.status().as_u16() as u32;
    let response = response.bytes().await;

    let response = match response {
        Ok(response) => response.to_vec(),
        Err(error) => {
            log::error!("data fetching error: {error}");
            return None;
        }
    };

    let Ok(body) = String::from_utf8(response) else {
        log::error!("response decoding problem");
        return None;
    };

    match serde_json::from_str::<Value>(&body) {
        Ok(json) => {
            let json = convert_to_jsjson(json);
            Some((status, json))
        },
        Err(error) => {
            log::error!("response decoding json problem error={error}");
            None
        }
    }
}

