use axum::http::StatusCode;

#[derive(PartialEq)]
pub enum ContentType {
    Plain,
    Html,
}

impl std::fmt::Display for ContentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Self::Html => f.write_str("text/html; charset=utf-8"),
            Self::Plain => f.write_str("text/plain"),
        }
    }
}

pub struct ResponseState {
    pub status: StatusCode,
    pub content_type: ContentType,
    pub body: String,
}

impl ResponseState {
    pub const HTML: &'static str = "text/html; charset=utf-8";
    pub const PLAIN: &'static str = "text/plain";

    pub fn html(status: StatusCode, body: impl Into<String>) -> Self {
        Self {
            status,
            content_type: ContentType::Html,
            body: body.into(),
        }
    }

    pub fn plain(status: StatusCode, body: impl Into<String>) -> Self {
        Self {
            status,
            content_type: ContentType::Plain,
            body: body.into(),
        }
    }

    pub fn internal_error(body: impl Into<String>) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            content_type: ContentType::Plain,
            body: body.into(),
        }
    }

    pub fn is_html(&self) -> bool {
        self.content_type == ContentType::Html
    }

    pub fn add_watch_script(&mut self, port_watch: u16) {
        if self.status.is_success() && self.is_html() {
            let watch = include_str!("./watch.js");

            let start = format!("start_watch('http://127.0.0.1:{port_watch}/events');");

            let chunks = [
                &self.body,
                "<script>",
                watch,
                &start,
                "</script>",
                "\n</body>",
            ];

            // Usually body tag is in the response, but better be prepared
            if self.body.contains("</body>") {
                let script = chunks[1..6].join("\n");
                self.body = self.body.replace("</body>", &script);
            } else {
                self.body = chunks[0..5].join("\n");
            }
        }
    }
}

impl From<ResponseState> for axum::response::Response<String> {
    fn from(value: ResponseState) -> Self {
        axum::response::Response::builder()
            .status(value.status)
            .header(
                "cache-control",
                "private, no-cache, no-store, must-revalidate, max-age=0",
            )
            .header("content-type", value.content_type.to_string())
            .body(value.body)
            .inspect_err(|err| log::error!("Error reading response: {err}"))
            .unwrap_or_default()
    }
}
