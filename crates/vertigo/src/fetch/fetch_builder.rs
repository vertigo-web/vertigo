use std::collections::HashMap;

use crate::{
    driver_module::driver::{FetchMethod, FetchResult}, ApiImport,
};

/// Builder for simple requests.
pub struct FetchBuilder {
    api: ApiImport,
    url: String,
    headers: Option<HashMap<String, String>>,
    body: Option<String>,
}

impl FetchBuilder {
    pub fn new(api: ApiImport, url: String) -> FetchBuilder {
        FetchBuilder {
            api,
            url,
            headers: None,
            body: None,
        }
    }

    #[must_use]
    pub fn set_headres(self, headers: HashMap<String, String>) -> Self {
        let FetchBuilder { api, url, body, .. } = self;
        FetchBuilder {
            api,
            url,
            headers: Some(headers),
            body,
        }
    }

    #[must_use]
    pub fn set_body(self, body: String) -> Self {
        let FetchBuilder { api, url, headers, .. } = self;
        FetchBuilder {
            api,
            url,
            headers,
            body: Some(body),
        }
    }

    async fn run(self, method: FetchMethod) -> FetchResult {
        let fut = self.api.fetch(method, self.url, self.headers, self.body);
        fut.await
    }

    pub async fn get(self) -> FetchResult {
        self.run(FetchMethod::GET).await
    }

    pub async fn post(self) -> FetchResult {
        self.run(FetchMethod::POST).await
    }
}
