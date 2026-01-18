use std::rc::Rc;
use vertigo_macro::store;

use crate::dev::{CallbackId, FutureBox, SsrFetchRequest, SsrFetchResponse};

use super::{CallbackStore, api_browser_command};

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

        api_browser_command().fetch_exec(request, callback);

        receiver.await
    }

    pub fn callback(&self, callback: CallbackId, response: SsrFetchResponse) {
        self.store.call(callback, response);
    }
}
