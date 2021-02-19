use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Request, RequestInit, /*RequestMode,*/ Response};

use std::pin::Pin;
use std::future::Future;

use std::collections::HashMap;

use vertigo::{FetchMethod, FetchError};

pub fn fetch(
    method: FetchMethod,
    url: String,
    headers: Option<HashMap<String, String>>,
    body: Option<String>
) -> Pin<Box<dyn Future<Output=Result<String, FetchError>> + 'static>> {
    Box::pin(async move {
        let mut opts = RequestInit::new();
        opts.method(method.to_string());
        //opts.mode(RequestMode::Cors);

        if let Some(body) = body {
            let value = JsValue::from_str(body.as_str());
            opts.body(Some(&value));
        }
    
        let request = Request::new_with_str_and_init(&url, &opts).unwrap();

        if let Some(headers) = headers {
            let request_headers = request.headers();

            for (key, val) in headers.iter() {
                request_headers.set(key, val).unwrap();
            }
        }

        request.headers().set("Content-Type", "application/json").unwrap();

        let window = web_sys::window().unwrap();

        let resp_value = JsFuture::from(window.fetch_with_request(&request)).await;
        
        let resp_value = match resp_value {
            Ok(resp_value) => resp_value,
            Err(_) => {
                return Err(FetchError::Error);
            }
        };

        // `resp_value` is a `Response` object.
        assert!(resp_value.is_instance_of::<Response>());
        let resp: Response = resp_value.dyn_into().unwrap();

        // Convert this other `Promise` into a rust `Future`.
        let json: JsValue = JsFuture::from(resp.text().unwrap()).await.unwrap();

        Ok(json.as_string().unwrap())
    })
}