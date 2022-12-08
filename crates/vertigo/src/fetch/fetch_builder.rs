use std::collections::HashMap;

use crate::{
    driver_module::driver::{FetchMethod, FetchResult}, get_driver,
};

/// Builder for simple requests.
pub struct FetchBuilder {
    url: String,
    headers: Option<HashMap<String, String>>,
    body: Option<String>,
}

impl FetchBuilder {
    pub fn new(url: String) -> FetchBuilder {
        FetchBuilder {
            url,
            headers: None,
            body: None,
        }
    }

    #[must_use]
    pub fn set_headres(self, headers: HashMap<String, String>) -> Self {
        let FetchBuilder { url, body, .. } = self;
        FetchBuilder {
            url,
            headers: Some(headers),
            body,
        }
    }

    #[must_use]
    pub fn set_body(self, body: String) -> Self {
        let FetchBuilder { url, headers, .. } = self;
        FetchBuilder {
            url,
            headers,
            body: Some(body),
        }
    }

    async fn run(self, method: FetchMethod) -> FetchResult {
        let fut = get_driver().inner.api.fetch(method, self.url, self.headers, self.body);
        fut.await
    }

    pub async fn get(self) -> FetchResult {
        self.run(FetchMethod::GET).await
    }

    pub async fn post(self) -> FetchResult {
        self.run(FetchMethod::POST).await
    }
}
