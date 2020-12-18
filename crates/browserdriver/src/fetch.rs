use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Request, RequestInit, /*RequestMode,*/ Response};

use std::pin::Pin;
use std::future::Future;

use std::collections::HashMap;

use virtualdom::FetchMethod;

pub fn fetch(
    method: FetchMethod,
    url: String,
    headers: Option<HashMap<String, String>>,
    _body: Option<String>
) -> Pin<Box<dyn Future<Output=String> + 'static>> {
    Box::pin(async move {
        let mut opts = RequestInit::new();
        opts.method(method.to_string());
        //opts.mode(RequestMode::Cors);

        let request = Request::new_with_str_and_init(&url, &opts).unwrap();

        if let Some(headers) = headers {
            let request_headers = request.headers();

            for (key, val) in headers.iter() {
                request_headers.set(key, val).unwrap();
            }
        }

        let window = web_sys::window().unwrap();

        let resp_value = JsFuture::from(window.fetch_with_request(&request)).await.unwrap();

        // `resp_value` is a `Response` object.
        assert!(resp_value.is_instance_of::<Response>());
        let resp: Response = resp_value.dyn_into().unwrap();

        // Convert this other `Promise` into a rust `Future`.
        let json: JsValue = JsFuture::from(resp.text().unwrap()).await.unwrap();

        json.as_string().unwrap()
    })
}