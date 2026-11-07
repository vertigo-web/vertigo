use std::collections::BTreeMap;

use vertigo_macro::AutoJsJson;

use crate::{FetchMethod, JsJson};

#[derive(AutoJsJson, Debug, Hash, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum SsrFetchRequestBody {
    None,
    Data { data: JsJson },
}

#[derive(AutoJsJson, Debug, Hash, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct SsrFetchRequest {
    pub method: FetchMethod,
    pub url: String,
    pub headers: BTreeMap<String, String>,
    pub body: SsrFetchRequestBody,
}

/// What the page has in place of a token.
const TOKEN_IN_PAGE: &str = "token";

fn is_authorization(header: &str) -> bool {
    header.trim().eq_ignore_ascii_case("authorization")
}

impl SsrFetchRequest {
    /// Whether the request carries somebody's credentials.
    pub fn has_authorization(&self) -> bool {
        self.headers.keys().any(|name| is_authorization(name))
    }

    /// What identifies the request in the page: a token is replaced by a mark that there was one.
    ///
    /// A mark, not nothing, so that a cache with a token doesn't take what the server fetched
    /// without one, f. ex. for a guest, and fetches for itself instead.
    fn as_in_page(&self) -> SsrFetchRequest {
        let mut request = self.clone();
        for (name, value) in &mut request.headers {
            if is_authorization(name) {
                *value = TOKEN_IN_PAGE.to_string();
            }
        }
        request
    }
}

#[derive(AutoJsJson, Debug, Hash, Clone)]
pub enum SsrFetchResponse {
    Ok {
        status: u32,
        response: SsrFetchResponseContent,
    },
    Err {
        message: String,
    },
}

#[derive(AutoJsJson, Debug, Hash, Clone)]
pub enum SsrFetchResponseContent {
    Json(JsJson),
    Text(String),
}

#[derive(AutoJsJson, Debug, Hash, Clone, Default)]
pub struct SsrFetchCache {
    pub data: BTreeMap<SsrFetchRequest, SsrFetchResponse>,
}

impl SsrFetchCache {
    /// The responses to put in the page, under their requests with the value of the
    /// `Authorization` header replaced: a token has no business in the page's source.
    ///
    /// Requests that only the token tells apart are left out, and the browser fetches them
    /// itself. One of them would otherwise get the response of the other.
    pub fn new(data: &BTreeMap<SsrFetchRequest, SsrFetchResponse>) -> SsrFetchCache {
        let mut responses = BTreeMap::new();
        let mut ambiguous = Vec::new();

        for (request, response) in data {
            let request = request.as_in_page();
            if responses
                .insert(request.clone(), response.clone())
                .is_some()
            {
                ambiguous.push(request);
            }
        }

        for request in &ambiguous {
            responses.remove(request);
        }

        SsrFetchCache { data: responses }
    }

    pub fn empty() -> SsrFetchCache {
        SsrFetchCache {
            data: BTreeMap::new(),
        }
    }

    pub fn get(&self, request: &SsrFetchRequest) -> Option<&SsrFetchResponse> {
        self.data.get(&request.as_in_page())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::{SsrFetchCache, SsrFetchRequest, SsrFetchResponse, SsrFetchResponseContent};
    use crate::{FetchMethod, JsJson, JsJsonSerialize, dev::SsrFetchRequestBody};

    fn request(headers: &[(&str, &str)]) -> SsrFetchRequest {
        SsrFetchRequest {
            method: FetchMethod::GET,
            url: "/api/me".to_string(),
            headers: (headers.iter())
                .map(|(name, value)| (name.to_string(), value.to_string()))
                .collect(),
            body: SsrFetchRequestBody::None,
        }
    }

    fn response(status: u32) -> SsrFetchResponse {
        SsrFetchResponse::Ok {
            status,
            response: SsrFetchResponseContent::Json(JsJson::Null),
        }
    }

    fn status(response: Option<&SsrFetchResponse>) -> Option<u32> {
        match response {
            Some(SsrFetchResponse::Ok { status, .. }) => Some(*status),
            _ => None,
        }
    }

    #[test]
    fn a_token_does_not_reach_the_page() {
        for name in ["Authorization", "authorization", "AUTHORIZATION"] {
            let cache = SsrFetchCache::new(&BTreeMap::from([(
                request(&[(name, "Bearer secret"), ("Accept", "visible")]),
                response(200),
            )]));

            // What goes in the page, before it's encoded: encoded, nothing could be found in it
            let page = format!("{:?}", cache.to_json());
            assert!(page.contains("visible"), "{page}");
            assert!(!page.contains("secret"), "{name}: {page}");
        }
    }

    #[test]
    fn found_with_a_token() {
        let cache = SsrFetchCache::new(&BTreeMap::from([(
            request(&[("Authorization", "Bearer secret")]),
            response(200),
        )]));

        assert_eq!(
            status(cache.get(&request(&[("Authorization", "Bearer secret")]))),
            Some(200)
        );
    }

    #[test]
    fn what_was_fetched_with_a_token_is_not_for_a_guest() {
        let cache = SsrFetchCache::new(&BTreeMap::from([(
            request(&[("Authorization", "Bearer secret")]),
            response(200),
        )]));

        assert_eq!(status(cache.get(&request(&[]))), None);
    }

    #[test]
    fn what_was_fetched_for_a_guest_is_not_for_a_token() {
        let cache = SsrFetchCache::new(&BTreeMap::from([(request(&[]), response(200))]));

        assert_eq!(
            status(cache.get(&request(&[("Authorization", "Bearer secret")]))),
            None
        );
    }

    #[test]
    fn other_headers_still_tell_requests_apart() {
        let cache = SsrFetchCache::new(&BTreeMap::from([
            (request(&[("Accept", "text/plain")]), response(200)),
            (request(&[("Accept", "application/json")]), response(201)),
        ]));

        assert_eq!(
            status(cache.get(&request(&[("Accept", "text/plain")]))),
            Some(200)
        );
        assert_eq!(
            status(cache.get(&request(&[("Accept", "application/json")]))),
            Some(201)
        );
        assert_eq!(status(cache.get(&request(&[]))), None);
    }

    #[test]
    fn requests_only_the_token_tells_apart_are_left_out() {
        let cache = SsrFetchCache::new(&BTreeMap::from([
            (request(&[("Authorization", "Bearer one")]), response(200)),
            (request(&[("Authorization", "Bearer two")]), response(403)),
            (request(&[("Authorization", "Bearer three")]), response(404)),
        ]));

        assert!(cache.data.is_empty(), "{:?}", cache.data);
    }

    #[test]
    fn has_authorization_in_any_case() {
        assert!(request(&[("AuthoriZation", "Bearer secret")]).has_authorization());
        assert!(!request(&[("Accept", "text/plain")]).has_authorization());
    }
}
