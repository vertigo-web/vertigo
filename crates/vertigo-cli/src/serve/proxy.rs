//! `--proxy`: requests under a path go on to another server, websockets included.

use actix_proxy::IntoHttpResponse;
use actix_web::{
    App, HttpRequest, HttpResponse,
    body::{BodyStream, MessageBody},
    dev::{RequestHead, ServiceFactory, ServiceRequest},
    guard,
    http::header::{self, HeaderName},
    web::{self, Bytes, BytesMut},
};
use futures::StreamExt;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::serve::http_client::http_client;

pub fn install_proxy<T>(app: App<T>, path: String, target: String) -> App<T>
where
    T: ServiceFactory<ServiceRequest, Config = (), Error = actix_web::Error, InitError = ()>,
{
    let websocket = {
        let path = path.clone();
        let target = target.clone();
        move |req: HttpRequest, payload: web::Payload| {
            let target_url = target_url(&path, &target, &req);
            tunnel_websocket(req, payload, target_url)
        }
    };

    app.service(
        web::scope(&path)
            .service(
                web::resource("/{tail:.*}")
                    .guard(guard::fn_guard(|ctx| is_websocket_handshake(ctx.head())))
                    .to(websocket),
            )
            .default_service(web::to({
                move |req: HttpRequest, body: web::Bytes| {
                    let path = path.clone();
                    let target = target.clone();
                    async move {
                        let method = req.method();
                        let target_url = target_url(&path, &target, &req);

                        log::info!("proxy {method} {} -> {target_url}", req.uri().path());

                        let request = http_client().request_from(&target_url, req.head());

                        let response = if !body.is_empty() {
                            request.send_body(body)
                        } else {
                            request.send()
                        };

                        match response.await {
                            Ok(response) => {
                                let mut response = response.into_http_response();

                                let headers = response.headers_mut();

                                // `awc` decompresses the upstream body transparently, but
                                // `into_http_response` copies the upstream headers across verbatim
                                // so remove appropriate headers.
                                if headers.remove(header::CONTENT_ENCODING).next().is_some() {
                                    headers.remove(header::CONTENT_LENGTH);
                                }

                                response
                            }
                            Err(error) => {
                                let message =
                                    format!("Error fetching from url={target_url} error={error}");
                                HttpResponse::InternalServerError().body(message)
                            }
                        }
                    }
                }
            })),
    )
}

/// Where a request under `path` goes: the rest of its path, and its query, after `target`.
fn target_url(path: &str, target: &str, req: &HttpRequest) -> String {
    let uri = req.uri();
    let current_path = uri.path();
    let tail = current_path.strip_prefix(path).unwrap_or(current_path);
    let query = uri.query().map(|q| format!("?{q}")).unwrap_or_default();
    format!("{target}{tail}{query}")
}

fn is_websocket_handshake(head: &RequestHead) -> bool {
    head.headers
        .get(header::UPGRADE)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value.eq_ignore_ascii_case("websocket"))
}

/// Headers the browser's handshake doesn't pass on: awc makes a handshake of its own, and `Host`
/// has to name the target, not this server. Without the extensions both ends speak plain frames,
/// which a tunnel that doesn't look into them needs.
fn is_handshake_header(name: &HeaderName) -> bool {
    [
        header::HOST,
        header::CONNECTION,
        header::UPGRADE,
        header::SEC_WEBSOCKET_KEY,
        header::SEC_WEBSOCKET_VERSION,
        header::SEC_WEBSOCKET_EXTENSIONS,
        header::CONTENT_LENGTH,
        header::TRANSFER_ENCODING,
    ]
    .contains(name)
}

/// Tunnels a websocket to `target_url`: awc makes the handshake with the target, and then the
/// bytes of the frames go through as they are, both ways - text, binary, pings, close codes.
async fn tunnel_websocket(
    req: HttpRequest,
    payload: web::Payload,
    target_url: String,
) -> HttpResponse {
    log::info!("proxy websocket {} -> {target_url}", req.uri().path());

    // The browser's side of the handshake, checked before the target is bothered.
    let mut response = match actix_http::ws::handshake(req.head()) {
        Ok(response) => response,
        Err(error) => return HttpResponse::from_error(error),
    };

    let mut request = http_client().ws(&target_url);
    for (name, value) in req.headers() {
        if !is_handshake_header(name) {
            request = request.header(name.clone(), value.clone());
        }
    }

    let (target_response, framed) = match request.connect().await {
        Ok(connected) => connected,
        Err(error) => {
            let message = format!("Error connecting websocket to url={target_url} error={error}");
            log::error!("{message}");
            return HttpResponse::BadGateway().body(message);
        }
    };

    if let Some(protocol) = target_response
        .headers()
        .get(header::SEC_WEBSOCKET_PROTOCOL)
    {
        response.insert_header((header::SEC_WEBSOCKET_PROTOCOL, protocol.clone()));
    }

    // What the target sent right after its handshake already sits in the read buffer.
    let parts = framed.into_parts();
    let early = parts.read_buf.freeze();
    let (from_target, mut to_target) = tokio::io::split(parts.io);

    actix_web::rt::spawn(async move {
        let mut payload = payload;
        while let Some(Ok(chunk)) = payload.next().await {
            if to_target.write_all(&chunk).await.is_err() {
                return;
            }
        }
        // The browser is gone, so the target gets the end of the stream as well.
        let _ = to_target.shutdown().await;
    });

    let early =
        futures::stream::iter((!early.is_empty()).then_some(Ok::<Bytes, std::io::Error>(early)));
    let rest = futures::stream::unfold(from_target, |mut from_target| async move {
        let mut chunk = BytesMut::with_capacity(16 * 1024);
        match from_target.read_buf(&mut chunk).await {
            Ok(0) | Err(_) => None,
            Ok(_) => Some((Ok(chunk.freeze()), from_target)),
        }
    });

    match response.message_body(BodyStream::new(early.chain(rest)).boxed()) {
        Ok(response) => response.into(),
        Err(error) => HttpResponse::from_error(error),
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use actix_web::{App, HttpRequest, HttpResponse, HttpServer, dev::ServerHandle, web};
    use awc::ws::{CloseCode, CloseReason, Frame, Message};
    use futures::{SinkExt, StreamExt};

    use super::{http_client, install_proxy};

    /// A backend's websocket: says what path and query it got, then echoes text and binary
    /// messages, and answers a close with the same reason.
    async fn echo(req: HttpRequest, body: web::Payload) -> actix_web::Result<HttpResponse> {
        let (response, mut session, mut stream) = actix_ws::handle(&req, body)?;
        let uri = req.uri().to_string();

        actix_web::rt::spawn(async move {
            let _ = session.text(uri).await;
            while let Some(Ok(message)) = stream.next().await {
                match message {
                    actix_ws::Message::Text(text) => {
                        let _ = session.text(text).await;
                    }
                    actix_ws::Message::Binary(bytes) => {
                        let _ = session.binary(bytes).await;
                    }
                    actix_ws::Message::Close(reason) => {
                        let _ = session.close(reason).await;
                        return;
                    }
                    _ => {}
                }
            }
        });

        Ok(response)
    }

    type TestResult = Result<(), Box<dyn std::error::Error>>;

    /// Serves `app` on a free port, and gives its address with a handle to stop it.
    fn serve<F, A>(app: F) -> std::io::Result<(String, ServerHandle)>
    where
        F: Fn() -> App<A> + Send + Clone + 'static,
        A: actix_web::dev::ServiceFactory<
                actix_web::dev::ServiceRequest,
                Config = (),
                Response = actix_web::dev::ServiceResponse,
                Error = actix_web::Error,
                InitError = (),
            > + 'static,
    {
        let server = HttpServer::new(app).workers(1).bind(("127.0.0.1", 0))?;
        let address = server.addrs()[0].to_string();
        let server = server.run();
        let handle = server.handle();
        actix_web::rt::spawn(server);
        Ok((address, handle))
    }

    /// The proxy at `/api` in front of a backend with a websocket at `/api/ws`.
    fn proxy_to_backend() -> std::io::Result<(String, [ServerHandle; 2])> {
        let (backend, backend_handle) = serve(|| {
            App::new()
                .route("/api/ws", web::get().to(echo))
                .route("/api/hello", web::get().to(|| async { "hi" }))
        })?;
        let target = format!("http://{backend}/api");
        let (proxy, proxy_handle) =
            serve(move || install_proxy(App::new(), "/api".to_string(), target.clone()))?;
        Ok((proxy, [proxy_handle, backend_handle]))
    }

    async fn stop(servers: [ServerHandle; 2]) {
        for server in servers {
            server.stop(false).await;
        }
    }

    async fn next<S>(socket: &mut S) -> Frame
    where
        S: futures::Stream<Item = Result<Frame, awc::error::WsProtocolError>> + Unpin,
    {
        match tokio::time::timeout(Duration::from_secs(5), socket.next()).await {
            Ok(Some(Ok(frame))) => frame,
            other => panic!("expected a frame, got {other:?}"),
        }
    }

    #[actix_web::test]
    async fn a_websocket_goes_through_to_the_target() -> TestResult {
        let (proxy, servers) = proxy_to_backend()?;

        let (_, mut socket) = http_client()
            .ws(format!("http://{proxy}/api/ws?world=1"))
            .connect()
            .await?;

        assert_eq!(
            next(&mut socket).await,
            Frame::Text("/api/ws?world=1".into())
        );

        socket.send(Message::Text("hello".into())).await?;
        assert_eq!(next(&mut socket).await, Frame::Text("hello".into()));

        socket.send(Message::Binary(vec![1, 2, 255].into())).await?;
        assert_eq!(
            next(&mut socket).await,
            Frame::Binary(vec![1, 2, 255].into())
        );

        let reason = CloseReason {
            code: CloseCode::Away,
            description: Some("bye".to_string()),
        };
        socket.send(Message::Close(Some(reason.clone()))).await?;
        assert_eq!(next(&mut socket).await, Frame::Close(Some(reason)));

        stop(servers).await;
        Ok(())
    }

    #[actix_web::test]
    async fn plain_requests_still_go_through() -> TestResult {
        let (proxy, servers) = proxy_to_backend()?;

        let mut response = http_client()
            .get(format!("http://{proxy}/api/hello"))
            .send()
            .await?;

        assert_eq!(response.body().await?, "hi");

        stop(servers).await;
        Ok(())
    }

    #[actix_web::test]
    async fn a_target_without_the_websocket_is_a_bad_gateway() -> TestResult {
        let (proxy, servers) = proxy_to_backend()?;

        let result = http_client()
            .ws(format!("http://{proxy}/api/nowhere"))
            .connect()
            .await;

        let Err(awc::error::WsClientError::InvalidResponseStatus(status)) = result else {
            panic!("the handshake should fail");
        };
        assert_eq!(status, 502);

        stop(servers).await;
        Ok(())
    }
}
