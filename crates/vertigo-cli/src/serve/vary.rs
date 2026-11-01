use actix_web::{
    Error,
    body::MessageBody,
    dev::{ServiceRequest, ServiceResponse},
    http::{
        StatusCode,
        header::{self, HeaderMap, HeaderValue},
    },
    middleware::Next,
    mime::{self, Mime},
};

/// Middleware making every response `Compress` would compress for a client accepting it say
/// `Vary: accept-encoding`, also when sent as it is.
///
/// `Compress` says it only on what it compresses. Sending the same response as it is, to a client
/// without `Accept-Encoding` (a crawler, curl), it says nothing, and a cache on the way (nginx,
/// a CDN) gives that version to everyone. `vertigo serve` uses it, a back-end with its own
/// `Compress` wraps it around that one, i.e. after it:
///
/// ```no_run
/// use actix_web::{App, middleware::{Compress, from_fn}};
/// use vertigo_cli::serve::vary_accept_encoding;
///
/// let app = App::new()
///     .wrap(Compress::default())
///     .wrap(from_fn(vary_accept_encoding));
/// ```
pub async fn vary_accept_encoding(
    req: ServiceRequest,
    next: Next<impl MessageBody>,
) -> Result<ServiceResponse<impl MessageBody>, Error> {
    let mut res = next.call(req).await?;
    if compressible(&res) && !varies_on_encoding(res.headers()) {
        res.headers_mut()
            .append(header::VARY, HeaderValue::from_static("accept-encoding"));
    }
    Ok(res)
}

/// What `Compress` compresses: a response not encoded already, with content, not an image (but
/// SVG) or a video. A revalidation (304) or a range (206) of it too, the response they stand for
/// is compressed.
fn compressible<B>(res: &ServiceResponse<B>) -> bool {
    if matches!(
        res.status(),
        StatusCode::SWITCHING_PROTOCOLS | StatusCode::NO_CONTENT
    ) || res.headers().contains_key(header::CONTENT_ENCODING)
    {
        return false;
    }
    let content_type = res
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.parse::<Mime>().ok());
    match content_type {
        Some(mime) if mime.type_() == mime::IMAGE => mime.subtype() == mime::SVG,
        Some(mime) => mime.type_() != mime::VIDEO,
        None => true,
    }
}

/// Already says it, f. ex. the precompressed files or what `Compress` compressed
fn varies_on_encoding(headers: &HeaderMap) -> bool {
    headers
        .get_all(header::VARY)
        .filter_map(|value| value.to_str().ok())
        .flat_map(|value| value.split(','))
        .map(str::trim)
        .any(|name| name == "*" || name.eq_ignore_ascii_case("accept-encoding"))
}

#[cfg(test)]
mod tests {
    use actix_web::{
        App, HttpResponse,
        http::{StatusCode, header},
        middleware::{Compress, from_fn},
        test, web,
    };

    use super::vary_accept_encoding;

    /// `Vary` and `Content-Encoding` of the response to a GET of `uri`
    async fn get(uri: &str, accept_encoding: Option<&str>) -> (Vec<String>, Option<String>) {
        let respond = |status: StatusCode, content_type: &'static str| {
            move || async move {
                HttpResponse::build(status)
                    .content_type(content_type)
                    .body("<p>Some content</p>")
            }
        };
        let app = test::init_service(
            App::new()
                .wrap(Compress::default())
                .wrap(from_fn(vary_accept_encoding))
                .route("/page", web::get().to(respond(StatusCode::OK, "text/html")))
                .route(
                    "/page-revalidated",
                    web::get().to(|| async {
                        HttpResponse::NotModified()
                            .content_type("text/html")
                            .finish()
                    }),
                )
                .route(
                    "/page-range",
                    web::get().to(respond(StatusCode::PARTIAL_CONTENT, "text/html")),
                )
                .route(
                    "/logo.png",
                    web::get().to(respond(StatusCode::OK, "image/png")),
                )
                .route(
                    "/logo.svg",
                    web::get().to(respond(StatusCode::OK, "image/svg+xml")),
                )
                .route(
                    "/intro.mp4",
                    web::get().to(respond(StatusCode::OK, "video/mp4")),
                )
                .route(
                    "/proxied",
                    web::get().to(|| async {
                        HttpResponse::Ok()
                            .content_type("text/html")
                            .insert_header((header::CONTENT_ENCODING, "gzip"))
                            .body("gzipped upstream")
                    }),
                )
                .route(
                    "/per-origin",
                    web::get().to(|| async {
                        HttpResponse::Ok()
                            .content_type("application/json")
                            .insert_header((header::VARY, "Origin"))
                            .body("{}")
                    }),
                )
                .route(
                    "/nothing",
                    web::get().to(|| async { HttpResponse::NoContent().finish() }),
                ),
        )
        .await;

        let mut req = test::TestRequest::get().uri(uri);
        if let Some(accept_encoding) = accept_encoding {
            req = req.insert_header((header::ACCEPT_ENCODING, accept_encoding));
        }
        let res = test::call_service(&app, req.to_request()).await;
        let header = |name| {
            res.headers()
                .get_all(name)
                .map(|value| String::from_utf8_lossy(value.as_bytes()).into_owned())
                .collect::<Vec<_>>()
        };
        (header(header::VARY), header(header::CONTENT_ENCODING).pop())
    }

    #[actix_web::test]
    async fn compressed_or_not_the_response_depends_on_accept_encoding() {
        let vary = |values: &[&str]| values.iter().map(|value| value.to_string()).collect();

        // Sent as it is
        for accept_encoding in [None, Some("identity"), Some("gzip;q=0")] {
            assert_eq!(
                get("/page", accept_encoding).await,
                (vary(&["accept-encoding"]), None),
                "{accept_encoding:?}"
            );
        }
        // Compressed, `Compress` says it once
        assert_eq!(
            get("/page", Some("gzip")).await,
            (vary(&["accept-encoding"]), Some("gzip".to_string()))
        );

        // The revalidation and the range of a compressed response
        assert_eq!(
            get("/page-revalidated", None).await,
            (vary(&["accept-encoding"]), None)
        );
        assert_eq!(
            get("/page-range", Some("gzip")).await,
            (vary(&["accept-encoding"]), None)
        );

        assert_eq!(
            get("/logo.svg", None).await,
            (vary(&["accept-encoding"]), None)
        );

        // To the other `Vary`
        assert_eq!(
            get("/per-origin", None).await,
            (vary(&["Origin", "accept-encoding"]), None)
        );
        assert_eq!(
            get("/per-origin", Some("gzip")).await,
            (
                vary(&["Origin", "accept-encoding"]),
                Some("gzip".to_string())
            )
        );
    }

    #[actix_web::test]
    async fn never_compressed_is_left_as_it_is() {
        for uri in ["/logo.png", "/intro.mp4", "/nothing"] {
            for accept_encoding in [None, Some("gzip")] {
                assert_eq!(
                    get(uri, accept_encoding).await,
                    (vec![], None),
                    "{uri} {accept_encoding:?}"
                );
            }
        }
        // Encoded by the back-end, which says what it depends on
        assert_eq!(
            get("/proxied", Some("gzip")).await,
            (vec![], Some("gzip".to_string()))
        );
    }
}
