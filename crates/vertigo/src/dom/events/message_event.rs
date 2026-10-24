use crate::{JsJson, dev::JsJsonListDecoder};

/// Structure passed as a parameter to callback on on_message event: a message the page in a frame
/// (`<iframe>`) sent to the page with the frame, with `parent.postMessage`.
///
/// Only messages from the page in this frame come - not from other frames, the page itself or
/// a worker - but that page may be any page the frame went to. So check [`MessageEvent::origin`]
/// before trusting what it says.
///
/// ```rust
/// use vertigo::{JsJson, MessageEvent, Value, bind, dom};
///
/// // A player in a frame tells how tall it wants to be
/// let height = Value::new(150);
///
/// let on_message = bind!(height, |message: MessageEvent| {
///     if message.origin != "https://player.example.com" {
///         return;
///     }
///     if let JsJson::Object(data) = &message.data
///         && let Some(JsJson::Number(wanted)) = data.get("height")
///     {
///         height.set(wanted.as_f64() as u32);
///     }
/// });
///
/// let style = height.map(|height| format!("height: {height}px"));
///
/// dom! {
///     <iframe src="https://player.example.com/" {style} {on_message} />
/// };
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct MessageEvent {
    /// Scheme, host and port of the page that sent it, f. ex. `https://player.example.com`, or
    /// `null` for a page without one, f. ex. from `srcdoc` in a sandboxed frame.
    pub origin: String,
    /// The message, as far as JSON can hold it: a `Date` comes as text, and what JSON has no
    /// room for at all (a `BigInt`, `undefined`, a cycle) as [`JsJson::Null`].
    pub data: JsJson,
}

impl MessageEvent {
    /// The event as the driver sends it. The order MUST match `message` in `callbackManager.ts`.
    pub(crate) fn from_driver(data: JsJson) -> Result<Self, String> {
        data.map_list(|mut params: JsJsonListDecoder| {
            let origin = params.get_string("origin")?;
            let data = params.get_any("data")?;
            params.expect_no_more()?;

            Ok(Self { origin, data })
        })
    }
}

impl std::fmt::Display for MessageEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MessageEvent={}", self.origin)
    }
}
