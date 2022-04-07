use vertigo::{dev::FetchMethod, FetchResult, FutureBox, FutureBoxSend};

use std::{
    collections::HashMap,
    future::Future,
    pin::Pin,
    rc::Rc,
};

use crate::{utils::{
    json::JsonMapBuilder,
}, api::ApiImport};

use vertigo::struct_mut::{
    CounterMut,
    HashMapMut,
};

#[derive(Clone)]
pub struct DriverBrowserFetch {
    api: Rc<ApiImport>,
    auto_id: Rc<CounterMut>,
    data: Rc<HashMapMut<u32, FutureBoxSend<FetchResult>>>,
}

impl DriverBrowserFetch {
    pub fn new(api: &Rc<ApiImport>) -> DriverBrowserFetch {
        let data: Rc<HashMapMut<u32, FutureBoxSend<FetchResult>>> = Rc::new(HashMapMut::new());

        DriverBrowserFetch {
            api: api.clone(),
            auto_id: Rc::new(CounterMut::new(1)),
            data,
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
        let (sender, receiver) = FutureBox::new();

        self.data.insert(id_request, sender);

        self.api.fetch_send_request(
            id_request,
            String::from(method.to_string()),
            url,
            self.serialize_headers(headers),
            body,
        );

        Box::pin(receiver)
    }

    pub(crate) fn export_fetch_callback(&self, request_id: u32, success: bool, status: u32, response: String) {
        let sender = self.data.remove(&request_id);

        if let Some(sender) = sender {
            let response = match success {
                true => Ok((status, response)),
                false => Err(response),
            };
            sender.publish(response);
        } else {
            log::error!("Request with ID={} not found", request_id);
        }
    }
}
