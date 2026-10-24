use vertigo::{JsJson, MessageEvent, Value, bind, component, css, dom};

/// The page in the frame: a button that sends a message to the page with the frame.
const FRAME: &str = r#"<button onclick="parent.postMessage({ text: 'Hello from the frame' }, '*')">Send to the page</button>"#;

/// `on_message` on an `<iframe>` gets what the page in it sends with `parent.postMessage` - and
/// only that, not what the page sends itself.
#[component]
pub fn FrameMessage() {
    let received = Value::new("nothing yet".to_string());

    let on_message = bind!(received, |message: MessageEvent| {
        let text = match &message.data {
            JsJson::Object(data) => match data.get("text") {
                Some(JsJson::String(text)) => text.clone(),
                _ => "a message without text".to_string(),
            },
            _ => "a message that isn't an object".to_string(),
        };
        received.set(text);
    });

    let frame_css = css! {"
        width: 180px;
        height: 40px;
        border: 1px solid black;
    "};

    dom! {
        <div tw="flex gap-1 items-center">
            <iframe srcdoc={FRAME} css={frame_css} {on_message} />
            <p>"Message from the frame: " {received}</p>
        </div>
    }
}
