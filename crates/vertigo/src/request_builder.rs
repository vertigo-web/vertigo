use std::{collections::HashMap};
use crate::Driver;
use crate::resource::Resource;

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

//dodać makro, które będzie automatycznie implementowało te traity wraz z serde ...

pub trait RequestTrait: Sized {
    fn into_string(self) -> Result<String, String>;
    fn from_string(data: &str) -> Result<Self, String>;
}

pub enum RequestBuilder {
    ErrorInput(String),
    Data {
        driver: Driver,
        url: String,
        headers: Option<HashMap<String, String>>,
        body: Option<String>,
    }
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

    pub fn body(self, body: String) -> RequestBuilder {
        match self {
            RequestBuilder::ErrorInput(message) => RequestBuilder::ErrorInput(message),
            RequestBuilder::Data { driver , url, headers, body: _ } => {
                RequestBuilder::Data {
                    driver,
                    url,
                    headers,
                    body: Some(body)
                }
            }
        }
    }

    fn set_header(&mut self, name: String, value: String) {
        if let RequestBuilder::Data { headers, .. } = self {
            if let Some(headers) = headers {
                headers.insert(name, value);
                return;
            }

            let mut new_headers = HashMap::new();
            new_headers.insert(name, value);
            *headers = Some(new_headers);
        }
    }

    pub fn body_json(self, body: impl RequestTrait) -> RequestBuilder {
        let body_string: Result<String, String> = body.into_string();

        match body_string {
            Ok(body) => {         
                let mut request = self.body(body);
                request.set_header("Content-Type".into(), "application/json".into());
                request
            },
            Err(message) => {
                RequestBuilder::ErrorInput(message)
            },
        }
    }

    pub fn headers(self, headers: HashMap<String, String>) -> RequestBuilder {
        match self {
            RequestBuilder::ErrorInput(message) => RequestBuilder::ErrorInput(message),
            RequestBuilder::Data { driver, url, headers: _, body } => {
                RequestBuilder::Data {
                    driver,
                    url,
                    headers: Some(headers),
                    body
                }
            }
        }
    }

    async fn call(self, method: Method) -> RequestResponse {
        let (driver, url, headers, body) = match self {
            RequestBuilder::ErrorInput(message) => {
                return RequestResponse::new(None, Err(message));
            },
            RequestBuilder::Data { driver, url, headers, body } => (driver, url, headers, body)
        };

        let builder = driver.fetch(url.clone());

        let builder = match body {
            None => builder,
            Some(body) => builder.set_body(body)
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
        RequestResponseBody {
            body
        }
    }

    pub fn into<T: PartialEq + RequestTrait>(self) -> Resource<T> {
        match T::from_string(self.body.as_str()) {
            Ok(data) => Resource::Ready(data),
            Err(err) => Resource::Error(err)
        }
    }
}

#[derive(Debug)]
pub struct RequestResponse {
    request_details: Option<(Method, String)>,
    data: Result<(u32, String), String>,
}

impl RequestResponse {
    fn new(request_details: Option<(Method, String)>, data: Result<(u32, String), String>) -> RequestResponse {
        RequestResponse {
            request_details,
            data
        }
    }

    pub fn into<T: PartialEq + RequestTrait>(self, convert: fn(u32, RequestResponseBody) -> Option<Resource<T>>) -> Resource<T> {
        let result = match self.data {
            Ok((status, body)) => {
                let body = RequestResponseBody::new(body);
                match convert(status, body) {
                    Some(result) => result,
                    None => Resource::Error(format!("Unhandled response code {}", status)),
                }
            },
            Err(err) => {
                Resource::Error(err)
            }
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
}
