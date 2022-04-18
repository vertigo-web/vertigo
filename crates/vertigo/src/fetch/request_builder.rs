use std::collections::HashMap;

use crate::Driver;

use super::resource::Resource;

#[derive(Debug)]
enum Method {
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
    fn into_string(self) -> Result<String, String>;
    fn from_string(data: &str) -> Result<Self, String>;
}

/// Ensures that vector of objects of this type is serializable and deserializable so can be used for defining a resource.
pub trait ListRequestTrait: Sized {
    fn list_into_string(vec: Vec<Self>) -> Result<String, String>;
    fn list_from_string(data: &str) -> Result<Vec<Self>, String>;
}

/// Builder for typed requests (more complex version of [FetchBuilder](struct.FetchBuilder.html)).
///
/// Unlike in the FetchBuilder, here request and response data is a type implementing [SingleRequestTrait] or [ListRequestTrait].
pub enum RequestBuilder {
    ErrorInput(String),
    Data {
        driver: Driver,
        url: String,
        headers: Option<HashMap<String, String>>,
        body: Option<String>,
    },
}

impl RequestBuilder {
    pub fn new(driver: &Driver, url: impl Into<String>) -> RequestBuilder {
        RequestBuilder::Data {
            driver: driver.clone(),
            url: url.into(),
            headers: None,
            body: None,
        }
    }

    #[must_use]
    pub fn body(self, body: String) -> RequestBuilder {
        match self {
            RequestBuilder::ErrorInput(message) => RequestBuilder::ErrorInput(message),
            RequestBuilder::Data { driver, url, headers, .. } =>
                RequestBuilder::Data {
                    driver,
                    url,
                    headers,
                    body: Some(body),
                },
        }
    }

    #[must_use]
    pub fn bearer_auth(self, token: impl Into<String>) -> RequestBuilder {
        let token: String = token.into();
        self.set_header("Authorization", format!("Bearer {}", token))
    }

    #[must_use]
    pub fn set_header(self, name: impl Into<String>, value: impl Into<String>) -> RequestBuilder {
        let name: String = name.into();
        let value: String = value.into();

        if let RequestBuilder::Data { headers, driver, url, body} = self {
            if let Some(mut headers) = headers {
                headers.insert(name, value);
                return RequestBuilder::Data { headers: Some(headers), driver, url, body };
            }

            let mut new_headers = HashMap::new();
            new_headers.insert(name, value);
            return RequestBuilder::Data { headers: Some(new_headers), driver, url, body };
        }

        self
    }

    #[must_use]
    pub fn body_json(self, body: impl SingleRequestTrait) -> RequestBuilder {
        let body_string: Result<String, String> = body.into_string();

        match body_string {
            Ok(body) => self.body(body).set_header("Content-Type", "application/json"),
            Err(message) => RequestBuilder::ErrorInput(message),
        }
    }

    #[must_use]
    pub fn headers(self, headers: HashMap<String, String>) -> RequestBuilder {
        match self {
            RequestBuilder::ErrorInput(message) => RequestBuilder::ErrorInput(message),
            RequestBuilder::Data { driver, url, body, .. } => RequestBuilder::Data {
                driver,
                url,
                headers: Some(headers),
                body,
            },
        }
    }

    async fn call(self, method: Method) -> RequestResponse {
        let (driver, url, headers, body) = match self {
            RequestBuilder::ErrorInput(message) => return RequestResponse::new(None, Err(message)),
            RequestBuilder::Data { driver, url, headers, body } => (driver, url, headers, body),
        };

        let builder = driver.fetch(url.clone());

        let builder = match body {
            None => builder,
            Some(body) => builder.set_body(body),
        };

        let builder = match headers {
            Some(headers) => builder.set_headres(headers),
            None => builder,
        };

        let result = match method {
            Method::Get => builder.get().await,
            Method::Post => builder.post().await,
        };

        RequestResponse::new(Some((method, url)), result)
    }

    pub async fn get(self) -> RequestResponse {
        self.call(Method::Get).await
    }

    pub async fn post(self) -> RequestResponse {
        self.call(Method::Post).await
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
    request_details: Option<(Method, String)>,
    data: Result<(u32, String), String>,
}

impl RequestResponse {
    fn new(request_details: Option<(Method, String)>, data: Result<(u32, String), String>) -> RequestResponse {
        RequestResponse { request_details, data }
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
                    None => Resource::Error(format!("Unhandled response code {}", status)),
                }
            }
            Err(err) => Resource::Error(err),
        };

        if let Resource::Error(err) = &result {
            if let Some((method, url)) = self.request_details {
                log::error!("Error fetching {} {}: {}", method.get_str(), url, err);
            } else {
                log::error!("Error fetching {}", err);
            }
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
            Ok((code, body)) => format!("API error {}: {}", code, body),
            Err(body) => format!("Network error: {}", body),
        };

        Resource::Error(body)
    }
}
