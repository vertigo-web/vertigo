use std::collections::HashMap;
use crate::{LazyCache, JsJson, get_driver, FetchMethod};
use std::time::Duration;
use super::resource::Resource;
use crate::{from_json, JsJsonSerialize, JsJsonDeserialize};

#[derive(Debug, Clone)]
pub enum RequestBody {
    Text(String),
    Json(JsJson),
    Binary(Vec<u8>),
}

impl RequestBody {
    pub fn into<T: JsJsonDeserialize>(self) -> Resource<T> {
        match self {
            RequestBody::Json(json) => {
                match from_json::<T>(json) {
                    Ok(data) => Resource::Ready(data),
                    Err(err) => Resource::Error(err),
                }
            },
            RequestBody::Text(_) => {
                Resource::Error("FetchBody.into() - expected json, received text".to_string())
            },
            RequestBody::Binary(_) => {
                Resource::Error("FetchBody.into() - expected json, received binary".to_string())
            }
        }
    }
}

/// Builder for typed requests (more complex version of [FetchBuilder](struct.FetchBuilder.html)).
///
/// Unlike in the FetchBuilder, here request and response data is a type implementing [SingleRequestTrait] or [ListRequestTrait].
#[derive(Clone)]
pub struct RequestBuilder {
    method: FetchMethod,
    url: String,
    headers: HashMap<String, String>,
    body: Option<RequestBody>,
    ttl: Option<Duration>,
}

impl RequestBuilder {
    pub fn new(method: FetchMethod, url: impl Into<String>) -> Self {
        Self {
            method,
            url: url.into(),
            headers: HashMap::new(),
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
    pub fn headers(mut self, headers: HashMap<String, String>) -> Self {
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

    pub async fn call(&self) -> RequestResponse {
        let Self { method, url, headers, body, ttl: _ } = self;

        let result = get_driver().inner.api.fetch(
            *method,
            url.clone(),
            Some(headers.clone()),
            body.clone()
        ).await;

        RequestResponse::new(*method, url.clone(), result)
    }

    #[must_use]
    pub fn lazy_cache<T>(
        self,
        map_response: impl Fn(u32, RequestBody) -> Option<Resource<T>> + 'static
    ) -> LazyCache<T> {
        LazyCache::new(self, map_response)
    }
}

/// Result from request made using [RequestBuilder].
#[derive(Debug)]
pub struct RequestResponse {
    method: FetchMethod,
    url: String,
    data: Result<(u32, RequestBody), String>,
}

impl RequestResponse {
    fn new(method: FetchMethod, url: String, data: Result<(u32, RequestBody), String>) -> RequestResponse {
        RequestResponse { method, url, data }
    }

    pub fn status(&self) -> Option<u32> {
        if let Ok((status, _)) = self.data {
            return Some(status);
        }

        None
    }

    pub fn into<T>(self, convert: impl Fn(u32, RequestBody) -> Option<Resource<T>>) -> Resource<T> {
        let result = match self.data {
            Ok((status, body)) => {
                match convert(status, body) {
                    Some(result) => result,
                    None => Resource::Error(format!("Unhandled response code {status}")),
                }
            }
            Err(err) => Resource::Error(err),
        };

        if let Resource::Error(err) = &result {
            log::error!("Error fetching {} {}: {}", self.method.to_str(), self.url, err);
        }

        result
    }

    pub fn into_data<T: JsJsonDeserialize>(self) -> Resource<T> {
        self.into(|_, response_body| {
            Some(response_body.into::<T>())
        })
    }

    pub fn into_error_message<T>(self) -> Resource<T> {
        let body = match self.data {
            Ok((code, body)) => format!("API error {code}: {body:#?}"),
            Err(body) => format!("Network error: {body}"),
        };

        Resource::Error(body)
    }
}
