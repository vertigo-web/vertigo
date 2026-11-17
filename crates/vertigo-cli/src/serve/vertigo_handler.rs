use actix_web::{
    HttpRequest,
    http::{StatusCode, header},
    web,
};
use std::time::Instant;

use crate::serve::{
    MountConfig, RequestHeaders, html::SSR_FETCH_HEADER, response_state::ResponseState,
};

use super::server_state::ServerState;

/// Directly attach SSR mechanism to the actix web server (no static files mounting)
///
/// To root path:
///
/// ```no_run
/// # use actix_web::web;
/// # use vertigo_cli::serve::{MountConfigBuilder, vertigo_handler};
/// # let mount_config = MountConfigBuilder::new("/", "/").build().unwrap();
/// # let app = web::scope("/");
/// app.default_service(vertigo_handler(&mount_config));
/// ```
///
/// To custom mount point:
///
/// ```no_run
/// # use actix_web::web;
/// # use vertigo_cli::serve::{MountConfigBuilder, vertigo_handler};
/// # let mount_config = MountConfigBuilder::new("/", "/").build().unwrap();
/// # let app = web::scope("/");
/// app.service(
///     web::scope(mount_config.mount_point())
///         .default_service(vertigo_handler(&mount_config)),
/// );
/// ```
pub fn vertigo_handler(mount_config: &MountConfig) -> actix_web::Route {
    let mount_point = mount_config.mount_point().to_string();

    web::route().to(move |req: HttpRequest| {
        let mount_point = mount_point.clone();
        async move {
            if is_ssr_fetch(&req) {
                log::error!(
                    "SSR fetched {}, which only the SSR handler serves",
                    req.uri()
                );
                return actix_web::HttpResponse::NotFound().finish();
            }

            let state = ServerState::global(&mount_point);
            let now = Instant::now();
            let url = req.uri();

            let uri = {
                let path = url.path();
                // Strip mount point to get local url
                let local_url = if state.mount_config.mount_point() != "/" {
                    path.trim_start_matches(state.mount_config.mount_point())
                } else {
                    path
                };

                match url.query() {
                    Some(query) => format!("{local_url}?{query}"),
                    None => local_url.to_string(),
                }
            };

            log::debug!("Incoming request: {uri}");
            let cookie = request_cookie(&req);
            let accept_language = request_accept_language(&req);
            let headers = RequestHeaders {
                cookie: cookie.as_deref(),
                accept_language: accept_language.as_deref(),
            };
            let mut response_state = state.request_with_headers(&uri, headers).await;

            let time = now.elapsed().as_millis();
            let log_level = if time > 1000 {
                log::Level::Warn
            } else {
                log::Level::Info
            };
            log::log!(
                log_level,
                "Response for request: {} {time}ms {uri}",
                response_state.status,
            );

            if let Some(port_watch) = state.port_watch
                && response_state.headers.get("content-type")
                    == Some(&ResponseState::HTML.to_string())
            {
                response_state.add_watch_script(port_watch);
            }

            // Checking for error status to log
            if StatusCode::from_u16(response_state.status)
                .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR)
                .is_server_error()
            {
                log::error!("WASM status: {}", response_state.status);

                match String::from_utf8(response_state.body.clone()) {
                    Ok(messagee) => {
                        log::error!("WASM response: text={}", messagee);
                    }
                    Err(_) => {
                        log::error!("WASM response: bytes={:#?}", response_state.body);
                    }
                }
            }

            actix_web::HttpResponse::from(response_state)
        }
    })
}

/// Whether the request is a fetch the app made while the server rendered a page, sent back to the
/// server's own origin ([`MountConfigBuilder::ssr_fetch_base`]).
///
/// A handler of your own that renders pages with [`ServerState`] should refuse such a request, as
/// [`vertigo_handler`] does: rendering a page for it could fetch the same URL again, endlessly.
/// It comes when the app fetches a URL that only the page handler answers.
///
/// ```
/// use actix_web::{HttpRequest, HttpResponse};
/// use vertigo_cli::serve::{RequestHeaders, ServerState, is_ssr_fetch, request_cookie};
///
/// async fn page(req: HttpRequest) -> HttpResponse {
///     if is_ssr_fetch(&req) {
///         return HttpResponse::NotFound().finish();
///     }
///     let cookie = request_cookie(&req);
///     let headers = RequestHeaders {
///         cookie: cookie.as_deref(),
///         ..RequestHeaders::default()
///     };
///     ServerState::global("/")
///         .request_with_headers(&req.uri().to_string(), headers)
///         .await
///         .into()
/// }
/// ```
///
/// [`MountConfigBuilder::ssr_fetch_base`]: super::MountConfigBuilder::ssr_fetch_base
pub fn is_ssr_fetch(req: &HttpRequest) -> bool {
    req.headers().contains_key(SSR_FETCH_HEADER)
}

/// `Cookie` header of the request, for [`ServerState::request_with_cookie`].
///
/// HTTP/2 may send the cookies as several header fields; they are joined with `; `
/// (RFC 9113, section 8.2.3).
pub fn request_cookie(req: &HttpRequest) -> Option<String> {
    let parts: Vec<&str> = req
        .headers()
        .get_all(header::COOKIE)
        .filter_map(|value| value.to_str().ok())
        .collect();

    (!parts.is_empty()).then(|| parts.join("; "))
}

/// `Accept-Language` header of the request, for [`ServerState::request_with_headers`].
///
/// Several header fields make one list, joined with `, ` (RFC 9110, section 5.3).
pub fn request_accept_language(req: &HttpRequest) -> Option<String> {
    let parts: Vec<&str> = req
        .headers()
        .get_all(header::ACCEPT_LANGUAGE)
        .filter_map(|value| value.to_str().ok())
        .collect();

    (!parts.is_empty()).then(|| parts.join(", "))
}

#[cfg(test)]
mod tests {
    use actix_web::{http::header, test::TestRequest};

    use super::{is_ssr_fetch, request_accept_language, request_cookie};
    use crate::serve::html::SSR_FETCH_HEADER;

    #[test]
    fn ssr_fetch_is_told_by_its_header() {
        let ssr_fetch = TestRequest::get()
            .append_header((SSR_FETCH_HEADER, "1"))
            .to_http_request();

        assert!(is_ssr_fetch(&ssr_fetch));
        assert!(!is_ssr_fetch(&TestRequest::get().to_http_request()));
    }

    #[test]
    fn no_cookie_header() {
        assert_eq!(request_cookie(&TestRequest::get().to_http_request()), None);
    }

    #[test]
    fn split_cookie_headers_are_joined() {
        let req = TestRequest::get()
            .append_header((header::COOKIE, "session=abc"))
            .append_header((header::COOKIE, "theme=dark"))
            .to_http_request();

        assert_eq!(
            request_cookie(&req).as_deref(),
            Some("session=abc; theme=dark")
        );
    }

    #[test]
    fn split_accept_language_headers_are_one_list() {
        let req = TestRequest::get()
            .append_header((header::ACCEPT_LANGUAGE, "pl-PL,pl;q=0.9"))
            .append_header((header::ACCEPT_LANGUAGE, "en;q=0.5"))
            .to_http_request();

        assert_eq!(
            request_accept_language(&req).as_deref(),
            Some("pl-PL,pl;q=0.9, en;q=0.5")
        );
        assert_eq!(
            request_accept_language(&TestRequest::get().to_http_request()),
            None
        );
    }
}
