use std::rc::Rc;
use vertigo_macro::store;

use super::{CallbackStore, api_browser_command};
#[cfg(test)]
use crate::dev::ValueMut;
use crate::dev::{CallbackId, FutureBox, SsrFetchRequest, SsrFetchResponse};

#[store]
pub fn api_fetch() -> Rc<ApiFetch> {
    ApiFetch::new()
}

#[cfg(test)]
type MockFetchHandler =
    crate::dev::ValueMut<Option<Rc<dyn Fn(SsrFetchRequest) -> SsrFetchResponse>>>;

pub struct ApiFetch {
    #[cfg(test)]
    mock_handler: MockFetchHandler,
    store: CallbackStore<SsrFetchResponse, ()>,
}

impl ApiFetch {
    fn new() -> Rc<ApiFetch> {
        Rc::new(ApiFetch {
            #[cfg(test)]
            mock_handler: ValueMut::new(None),
            store: CallbackStore::new(),
        })
    }

    #[cfg(test)]
    pub fn set_mock_handler(
        &self,
        handler: impl Fn(SsrFetchRequest) -> SsrFetchResponse + 'static,
    ) {
        self.mock_handler.set(Some(Rc::new(handler)));
    }

    pub async fn fetch(&self, request: SsrFetchRequest) -> SsrFetchResponse {
        #[cfg(test)]
        if let Some(handler) = self.mock_handler.get() {
            return handler(request);
        }

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::FetchMethod;
    use crate::dev::{SsrFetchRequestBody, SsrFetchResponseContent};
    use std::collections::BTreeMap;

    #[tokio::test]
    async fn test_mock_fetch() {
        let api = api_fetch();

        api.set_mock_handler(|request| {
            assert_eq!(request.url, "https://test.com");
            SsrFetchResponse::Ok {
                status: 200,
                response: SsrFetchResponseContent::Text("mocked response".to_string()),
            }
        });

        let request = SsrFetchRequest {
            method: FetchMethod::GET,
            url: "https://test.com".to_string(),
            headers: BTreeMap::new(),
            body: SsrFetchRequestBody::None,
        };

        let response = api.fetch(request).await;

        match response {
            SsrFetchResponse::Ok { status, response } => {
                assert_eq!(status, 200);
                match response {
                    SsrFetchResponseContent::Text(text) => {
                        assert_eq!(text, "mocked response");
                    }
                    _ => panic!("Expected text response"),
                }
            }
            _ => panic!("Expected Ok response"),
        }
    }
}
