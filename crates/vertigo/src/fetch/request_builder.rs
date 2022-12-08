use std::collections::HashMap;
use crate::{FetchBuilder, LazyCache};
use std::time::Duration;
use super::resource::Resource;

#[derive(Clone, Debug)]
pub enum Method {
    Get,
    Post,
}

impl Method {
    fn get_str(&self) -> &str {
        match self {
            Method::Get => "get",
            &Method::Post => "post",
        }
    }
}

/// Ensures that this type is serializable and deserializable so can be used for defining a resource.
pub trait SingleRequestTrait: Sized {
    fn into_string(self) -> String;
    fn from_string(data: &str) -> Result<Self, String>;
}

/// Ensures that vector of objects of this type is serializable and deserializable so can be used for defining a resource.
pub trait ListRequestTrait: Sized {
    //TODO - To consider whether this returned string will cause problems in the case of ill-defined structures. Perhaps it is worth replacing the serde with something smaller ?
    fn list_into_string(vec: Vec<Self>) -> String;
    //TODO - To consider whether this returned string will cause problems in the case of ill-defined structures. Perhaps it is worth replacing the serde with something smaller ?
    fn list_from_string(data: &str) -> Result<Vec<Self>, String>;
}

/// Builder for typed requests (more complex version of [FetchBuilder](struct.FetchBuilder.html)).
///
/// Unlike in the FetchBuilder, here request and response data is a type implementing [SingleRequestTrait] or [ListRequestTrait].
#[derive(Clone)]
pub struct RequestBuilder {
    method: Method,
    url: String,
    headers: HashMap<String, String>,
    body: Option<String>,
    ttl: Option<Duration>,
}

impl RequestBuilder {
    pub fn new(method: Method, url: impl Into<String>) -> Self {
        Self {
            method,
            url: url.into(),
            headers: HashMap::new(),
            body: None,
            ttl: None,
        }
    }

    pub fn get(url: impl Into<String>) -> Self {
        Self::new(Method::Get, url)
    }

    pub fn post(url: impl Into<String>) -> Self {
        Self::new(Method::Post, url)
    }

    #[must_use]
    pub fn body(mut self, body: String) -> Self {
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
    pub fn body_json(self, body: impl SingleRequestTrait) -> Self {
        let body: String = body.into_string();
        self.body(body).set_header("Content-Type", "application/json")
    }

    #[must_use]
    pub fn headers(mut self, headers: HashMap<String, String>) -> Self {
        self.headers = headers;
        self
    }

    pub fn ttl_seconds(mut self, seconds: u64) -> Self {
        self.ttl = Some(Duration::from_secs(seconds));
        self
    }

    pub fn ttl_minutes(mut self, minutes: u64) -> Self {
        self.ttl = Some(Duration::from_secs(minutes * 60));
        self
    }

    pub fn ttl_hours(mut self, hours: u64) -> Self {
        self.ttl = Some(Duration::from_secs(hours * 60 * 60));
        self
    }

    pub fn ttl_days(mut self, days: u64) -> Self {
        self.ttl = Some(Duration::from_secs(days * 24 * 60 * 60));
        self
    }

    pub fn get_ttl(&self) -> Option<Duration> {
        self.ttl
    }

    pub async fn call(&self) -> RequestResponse {
        let Self { method, url, headers, body, ttl: _ } = self;

        let builder = FetchBuilder::new(url.clone());

        let builder = match body {
            None => builder,
            Some(body) => builder.set_body(body.clone()),
        };

        let builder = builder.set_headres(headers.clone());

        let result = match method {
            Method::Get => builder.get().await,
            Method::Post => builder.post().await,
        };

        RequestResponse::new(method.clone(), url.clone(), result)
    }

    pub fn lazy_cache<T>(
        self,
        map_response: impl Fn(u32, RequestResponseBody) -> Option<Resource<T>> + 'static
    ) -> LazyCache<T> {
        LazyCache::new(self, map_response)
    }
}


#[derive(Debug)]
pub struct RequestResponseBody {
    body: String,
}

impl RequestResponseBody {
    fn new(body: String) -> RequestResponseBody {
        RequestResponseBody { body }
    }

    pub fn into<T: SingleRequestTrait>(self) -> Resource<T> {
        match T::from_string(self.body.as_str()) {
            Ok(data) => Resource::Ready(data),
            Err(err) => Resource::Error(err),
        }
    }

    pub fn into_vec<T: ListRequestTrait>(self) -> Resource<Vec<T>> {
        match T::list_from_string(self.body.as_str()) {
            Ok(data) => Resource::Ready(data),
            Err(err) => Resource::Error(err),
        }
    }
}

/// Result from request made using [RequestBuilder].
#[derive(Debug)]
pub struct RequestResponse {
    method: Method,
    url: String,
    data: Result<(u32, String), String>,
}

impl RequestResponse {
    fn new(method: Method, url: String, data: Result<(u32, String), String>) -> RequestResponse {
        RequestResponse { method, url, data }
    }

    pub fn status(&self) -> Option<u32> {
        if let Ok((status, _)) = self.data {
            return Some(status);
        }

        None
    }

    pub fn into<T>(self, convert: impl Fn(u32, RequestResponseBody) -> Option<Resource<T>>) -> Resource<T> {
        let result = match self.data {
            Ok((status, body)) => {
                let body = RequestResponseBody::new(body);
                match convert(status, body) {
                    Some(result) => result,
                    None => Resource::Error(format!("Unhandled response code {status}")),
                }
            }
            Err(err) => Resource::Error(err),
        };

        if let Resource::Error(err) = &result {
            log::error!("Error fetching {} {}: {}", self.method.get_str(), self.url, err);
        }

        result
    }

    pub fn into_data<T: SingleRequestTrait>(self) -> Resource<T> {
        self.into(|_, response_body| {
            Some(response_body.into::<T>())
        })
    }

    pub fn into_error_message<T>(self) -> Resource<T> {
        let body = match self.data {
            Ok((code, body)) => format!("API error {code}: {body}"),
            Err(body) => format!("Network error: {body}"),
        };

        Resource::Error(body)
    }
}
