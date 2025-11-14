use crate::{
    driver_module::api::{api_command_browser, CallbackStore},
    CallbackId, FutureBox, SsrFetchRequest, SsrFetchResponse,
};
use std::rc::Rc;
use vertigo_macro::store;

#[store]
pub fn api_fetch() -> Rc<ApiFetch> {
    ApiFetch::new()
}

pub struct ApiFetch {
    store: CallbackStore<SsrFetchResponse, ()>,
}

impl ApiFetch {
    fn new() -> Rc<ApiFetch> {
        Rc::new(ApiFetch {
            store: CallbackStore::new(),
        })
    }

    pub async fn fetch(&self, request: SsrFetchRequest) -> SsrFetchResponse {
        let (sender, receiver) = FutureBox::<SsrFetchResponse>::new();

        let callback = self.store.register_once(move |response| {
            sender.publish(response);
        });

        api_command_browser().fetch_exec(request, callback);
        let response = receiver.await;
        response
    }

    pub fn callbak(&self, callback: CallbackId, response: SsrFetchResponse) {
        self.store.call(callback, response);
    }
}
