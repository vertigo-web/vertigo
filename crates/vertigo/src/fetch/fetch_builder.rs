use std::collections::HashMap;

use crate::{
    driver_module::driver_browser::{FetchMethod, FetchResult},
    driver_module::{modules::fetch::DriverBrowserFetch},
};

/// Builder for simple requests.
pub struct FetchBuilder {
    driver_fetch: DriverBrowserFetch,
    url: String,
    headers: Option<HashMap<String, String>>,
    body: Option<String>,
}

impl FetchBuilder {
    pub fn new(driver_fetch: DriverBrowserFetch, url: String) -> FetchBuilder {
        FetchBuilder {
            driver_fetch,
            url,
            headers: None,
            body: None,
        }
    }

    #[must_use]
    pub fn set_headres(self, headers: HashMap<String, String>) -> Self {
        let FetchBuilder { driver_fetch, url, body, .. } = self;
        FetchBuilder {
            driver_fetch,
            url,
            headers: Some(headers),
            body,
        }
    }

    #[must_use]
    pub fn set_body(self, body: String) -> Self {
        let FetchBuilder { driver_fetch, url, headers, .. } = self;
        FetchBuilder {
            driver_fetch,
            url,
            headers,
            body: Some(body),
        }
    }

    async fn run(self, method: FetchMethod) -> FetchResult {
        let fut = self.driver_fetch.fetch(method, self.url, self.headers, self.body);
        fut.await
    }

    pub async fn get(self) -> FetchResult {
        self.run(FetchMethod::GET).await
    }

    pub async fn post(self) -> FetchResult {
        self.run(FetchMethod::POST).await
    }
}
