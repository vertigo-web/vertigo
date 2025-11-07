use axum::body::Body;
use axum::http::StatusCode;
use std::collections::HashMap;

fn content_type(content_type: &str) -> HashMap<String, String> {
    let mut headers = HashMap::new();
    headers.insert("content-type".into(), content_type.into());
    headers
}

fn content_type_html() -> HashMap<String, String> {
    content_type("text/html; charset=utf-8")
}

fn content_type_plain() -> HashMap<String, String> {
    content_type("text/plain")
}

pub struct ResponseState {
    pub status: StatusCode,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
}

impl ResponseState {
    pub const HTML: &'static str = "text/html; charset=utf-8";
    pub const PLAIN: &'static str = "text/plain";

    pub fn html(status: StatusCode, body: impl Into<String>) -> Self {
        Self {
            status,
            headers: content_type_html(),
            body: body.into().into_bytes(),
        }
    }

    pub fn plain(status: StatusCode, body: impl Into<String>) -> Self {
        Self {
            status,
            headers: content_type_plain(),
            body: body.into().into_bytes(),
        }
    }

    pub fn internal_error(body: impl Into<String>) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            headers: content_type_plain(),
            body: body.into().into_bytes(),
        }
    }

    pub fn add_watch_script(&mut self, port_watch: u16) {
        let watch = include_str!("./watch.js");

        let start = format!("start_watch('http://127.0.0.1:{port_watch}/events');");

        let chunks = ["<script>", watch, &start, "</script>"];

        let script = chunks.join("\n").into_bytes();
        self.body.extend(script);
    }
}

impl From<ResponseState> for axum::response::Response<Body> {
    fn from(value: ResponseState) -> Self {
        let mut builder = axum::response::Response::builder().status(value.status);

        for (name, value) in value.headers {
            builder = builder.header(name, value);
        }

        builder
            .body(Body::from(value.body))
            .inspect_err(|err| log::error!("Error reading response: {err}"))
            .unwrap_or_default()
    }
}
