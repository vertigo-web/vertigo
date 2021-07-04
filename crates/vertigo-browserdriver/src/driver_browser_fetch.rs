use wasm_bindgen::prelude::*;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;

use vertigo::FetchMethod;

use crate::DriverBrowserFetchJs;
use crate::utils::counter_rc::CounterRc;
use crate::utils::future::{CbFutureSend, new_future};
use crate::utils::hash_map_rc::HashMapRc;
use crate::utils::json::JsonMapBuilder;

pub struct DriverBrowserFetch {
    driver_js: DriverBrowserFetchJs,
    auto_id: CounterRc,
    data: HashMapRc<u64, CbFutureSend<Result<String, String>>>,
    _clouser: Closure<dyn Fn(u64, bool, String)>,
}

impl DriverBrowserFetch {
    pub fn new() -> DriverBrowserFetch {
        let data: HashMapRc<u64, CbFutureSend<Result<String, String>>> = HashMapRc::new("driver fetch HashMapRc");

        let closure = {
            let data = data.clone();

            Closure::new(Box::new(move |request_id: u64, success: bool, response: String| {
                let sender = data.remove(&request_id);
                
                if let Some(sender) = sender {
                    let response = match success {
                        true => Ok(response),
                        false => Err(response),
                    };
                    sender.publish(response);
                } else {
                    log::error!("Request with ID={} not found", request_id);
                }
            }))
        };

        let driver_js = DriverBrowserFetchJs::new(&closure);

        DriverBrowserFetch {
            driver_js,
            auto_id: CounterRc::new(1),
            data,
            _clouser: closure,
        }
    }

    fn serialize_headers(&self, headers: Option<HashMap<String, String>>) -> String {
        let mut headers_builder = JsonMapBuilder::new();

        if let Some(headers) = headers {
            for (key, value) in headers.into_iter() {
                headers_builder.set_string(&key, &value);
            }
        }

        headers_builder.build()
    }

    pub fn fetch(
        &self,
        method: FetchMethod,
        url: String,
        headers: Option<HashMap<String, String>>,
        body: Option<String>
    ) -> Pin<Box<dyn Future<Output=Result<String, String>> + 'static>> {

        let id_request = self.auto_id.get_next();
        let (sender, receiver) = new_future();

        self.data.insert(id_request, sender);

        self.driver_js.send_request(
            id_request,
            String::from(method.to_string()),
            url,
            self.serialize_headers(headers),
            body
        );

        Box::pin(receiver)
    }
}


