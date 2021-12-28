use vertigo::{dev::FetchMethod, FetchResult};

use std::{
    collections::HashMap,
    future::Future,
    pin::Pin,
    rc::Rc,
};
use wasm_bindgen::prelude::*;

use super::js_fetch::DriverBrowserFetchJs;
use crate::utils::{
    future::{new_future, CbFutureSend},
    json::JsonMapBuilder,
};

use vertigo::struct_mut::{
    CounterMut,
    HashMapMut,
};

pub struct DriverBrowserFetch {
    driver_js: DriverBrowserFetchJs,
    auto_id: CounterMut,
    data: Rc<HashMapMut<u64, CbFutureSend<FetchResult>>>,
    _clouser: Closure<dyn Fn(u64, bool, u32, String)>,
}

impl DriverBrowserFetch {
    pub fn new() -> DriverBrowserFetch {
        let data: Rc<HashMapMut<u64, CbFutureSend<FetchResult>>> = Rc::new(HashMapMut::new());

        let closure = {
            let data = data.clone();

            Closure::new(move |request_id: u64, success: bool, status: u32, response: String| {
                let sender = data.remove(&request_id);

                if let Some(sender) = sender {
                    let response = match success {
                        true => Ok((status, response)),
                        false => Err(response),
                    };
                    sender.publish(response);
                } else {
                    log::error!("Request with ID={} not found", request_id);
                }
            })
        };

        let driver_js = DriverBrowserFetchJs::new(&closure);

        DriverBrowserFetch {
            driver_js,
            auto_id: CounterMut::new(1),
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
        body: Option<String>,
    ) -> Pin<Box<dyn Future<Output = FetchResult> + 'static>> {
        let id_request = self.auto_id.get_next();
        let (sender, receiver) = new_future();

        self.data.insert(id_request, sender);

        self.driver_js.send_request(
            id_request,
            String::from(method.to_string()),
            url,
            self.serialize_headers(headers),
            body,
        );

        Box::pin(receiver)
    }
}
