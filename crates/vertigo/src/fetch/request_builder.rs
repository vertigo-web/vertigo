use std::collections::BTreeMap;
use std::time::Duration;

use crate::{
    dev::SsrFetchRequestBody, driver_module::api::api_fetch, from_json, FetchMethod, JsJson,
    JsJsonDeserialize, JsJsonSerialize, LazyCache, SsrFetchRequest, SsrFetchResponse,
    SsrFetchResponseContent,
};

#[derive(Debug, Clone)]
pub enum RequestBody {
    Json(JsJson),
}

impl RequestBody {
    pub fn into<T: JsJsonDeserialize>(self) -> Result<T, String> {
        match self {
            RequestBody::Json(json) => match from_json::<T>(json) {
                Ok(data) => Ok(data),
                Err(err) => Err(err),
            },
        }
    }
}

/// Builder for typed requests.
#[derive(Clone)]
pub struct RequestBuilder {
    method: FetchMethod,
    url: String,
    headers: BTreeMap<String, String>,
    body: Option<RequestBody>,
    ttl: Option<Duration>,
}

impl RequestBuilder {
    pub fn new(method: FetchMethod, url: impl Into<String>) -> Self {
        Self {
            method,
            url: url.into(),
            headers: BTreeMap::new(),
            body: None,
            ttl: None,
        }
    }

    #[must_use]
    pub fn get(url: impl Into<String>) -> Self {
        Self::new(FetchMethod::GET, url)
    }

    #[must_use]
    pub fn post(url: impl Into<String>) -> Self {
        Self::new(FetchMethod::POST, url)
    }

    #[must_use]
    pub fn body(mut self, body: RequestBody) -> Self {
        self.body = Some(body);
        self
    }

    #[must_use]
    pub fn bearer_auth(self, token: impl Into<String>) -> Self {
        let token: String = token.into();
        self.set_header("Authorization", format!("Bearer {token}"))
    }

    #[must_use]
    pub fn set_header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        let name: String = name.into();
        let value: String = value.into();
        self.headers.insert(name, value);
        self
    }

    #[must_use]
    pub fn body_json(self, body: impl JsJsonSerialize) -> Self {
        let body = body.to_json();
        self.body(RequestBody::Json(body))
    }

    #[must_use]
    pub fn headers(mut self, headers: BTreeMap<String, String>) -> Self {
        self.headers = headers;
        self
    }

    #[must_use]
    pub fn ttl_seconds(mut self, seconds: u64) -> Self {
        self.ttl = Some(Duration::from_secs(seconds));
        self
    }

    #[must_use]
    pub fn ttl_minutes(mut self, minutes: u64) -> Self {
        self.ttl = Some(Duration::from_secs(minutes * 60));
        self
    }

    #[must_use]
    pub fn ttl_hours(mut self, hours: u64) -> Self {
        self.ttl = Some(Duration::from_secs(hours * 60 * 60));
        self
    }

    #[must_use]
    pub fn ttl_days(mut self, days: u64) -> Self {
        self.ttl = Some(Duration::from_secs(days * 24 * 60 * 60));
        self
    }

    #[must_use]
    pub fn get_ttl(&self) -> Option<Duration> {
        self.ttl
    }

    pub fn to_request(self) -> SsrFetchRequest {
        let mut headers = self.headers;

        let body = match self.body {
            None => SsrFetchRequestBody::None,
            Some(RequestBody::Json(data)) => {
                if !headers.contains_key("Content-Type") {
                    headers.insert(
                        "Content-Type".into(),
                        "application/json;charset=UTF-8".into(),
                    );
                }
                SsrFetchRequestBody::Data { data }
            }
        };

        SsrFetchRequest {
            method: self.method,
            url: self.url,
            headers,
            body,
        }
    }

    pub async fn call(self) -> RequestResponse {
        let request = self.to_request();

        let result = api_fetch().fetch(request.clone()).await;

        RequestResponse::new(request, result)
    }

    #[must_use]
    pub fn lazy_cache<T>(
        self,
        map_response: impl Fn(u32, RequestBody) -> Option<Result<T, String>> + 'static,
    ) -> LazyCache<T> {
        LazyCache::new(self, map_response)
    }
}

/// Result from request made using [RequestBuilder].
#[derive(Debug)]
pub struct RequestResponse {
    request: SsrFetchRequest,
    response: SsrFetchResponse,
}

impl RequestResponse {
    pub fn new(request: SsrFetchRequest, response: SsrFetchResponse) -> RequestResponse {
        RequestResponse { request, response }
    }

    pub fn status(&self) -> Option<u32> {
        if let SsrFetchResponse::Ok {
            status,
            response: _,
        } = &self.response
        {
            return Some(*status);
        }

        None
    }

    pub fn into<T>(
        self,
        convert: impl Fn(u32, RequestBody) -> Option<Result<T, String>>,
    ) -> Result<T, String> {
        let result: Result<T, String> = match self.response {
            SsrFetchResponse::Ok { status, response } => {
                let data = match response {
                    SsrFetchResponseContent::Json(json_response) => {
                        convert(status, RequestBody::Json(json_response))
                    }
                    SsrFetchResponseContent::Text(_) => {
                        return Err("Tried to decode text/plain reponse".to_string())
                    }
                };

                match data {
                    Some(result) => result,
                    None => Err(format!("Unhandled response code {status}")),
                }
            }
            SsrFetchResponse::Err { message } => Err(message),
        };

        if let Err(err) = &result {
            log::error!(
                "Error fetching {} {}: {}",
                self.request.method.to_str(),
                self.request.url,
                err
            );
        }

        result
    }

    pub fn into_data<T: JsJsonDeserialize>(self) -> Result<T, String> {
        self.into(|_, response_body| Some(response_body.into::<T>()))
    }

    pub fn into_error_message<T>(self) -> Result<T, String> {
        let body = match self.response {
            SsrFetchResponse::Ok { status, response } => {
                format!("API error {status}: {response:#?}")
            }
            SsrFetchResponse::Err { message } => format!("Network error: {message}"),
        };

        Err(body)
    }
}
