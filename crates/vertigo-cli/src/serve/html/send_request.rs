use std::sync::Arc;

use crate::serve::wasm::{FetchRequest, FetchResponse};

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
                body: String::from(""),
            }
        }
    }
}

async fn send_request_inner(request_params: Arc<FetchRequest>) -> Option<(u32, String)> {
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

    Some((status, body))
}

