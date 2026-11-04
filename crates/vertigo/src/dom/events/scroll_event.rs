use crate::{JsJson, dev::JsJsonListDecoder};

/// Structure passed as a parameter to callback on on_scroll event: where the content of the
/// element is scrolled to, read right after the scroll, and how big the content and its visible
/// part are - all in CSS pixels.
///
/// The element's own content scrolled, however it happened: with the wheel, a finger, the scroll
/// bar, the keyboard, the caret moving in a `<textarea>`, or the page setting it. The scroll of
/// a child doesn't come, as the browser passes a scroll on to no parent, and neither does the
/// scroll of the page: that's the document scrolling, not `<html>` or `<body>`. The browser fires
/// it at most once a frame.
///
/// An element of a page rendered on the server, scrolled before the app started, tells where it
/// is right after the app starts, as no scroll would until the next one.
///
/// ```rust
/// use vertigo::{ScrollEvent, Value, bind, dom};
///
/// // A list that wants more of its items once it's scrolled to 100 px from its end
/// let near_end = Value::new(false);
///
/// let on_scroll = bind!(near_end, |event: ScrollEvent| {
///     near_end.set(event.scroll_top + event.client_height >= event.scroll_height - 100.0);
/// });
///
/// dom! {
///     <div style="height: 300px; overflow-y: auto;" {on_scroll}>
///         <ul><li>"Alice's got a cat"</li></ul>
///     </div>
/// };
/// ```
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ScrollEvent {
    /// How far the content is scrolled down (`scrollTop`). Fractional on a zoomed page.
    pub scroll_top: f64,
    /// How far the content is scrolled from where its lines start (`scrollLeft`): rightwards,
    /// or with right-to-left text leftwards, as a negative number.
    pub scroll_left: f64,
    /// Width of the whole content, with the part out of view (`scrollWidth`).
    pub scroll_width: f64,
    /// Height of the whole content, with the part out of view (`scrollHeight`).
    pub scroll_height: f64,
    /// Width of the visible part, without the border and the scroll bar (`clientWidth`).
    pub client_width: f64,
    /// Height of the visible part, without the border and the scroll bar (`clientHeight`).
    pub client_height: f64,
}

impl ScrollEvent {
    /// The event as the driver sends it. The order MUST match `scroll` in `callbackManager.ts`.
    pub(crate) fn from_driver(data: JsJson) -> Result<Self, String> {
        data.map_list(|mut params: JsJsonListDecoder| {
            let scroll_top = params.get_f64("scrollTop")?;
            let scroll_left = params.get_f64("scrollLeft")?;
            let scroll_width = params.get_f64("scrollWidth")?;
            let scroll_height = params.get_f64("scrollHeight")?;
            let client_width = params.get_f64("clientWidth")?;
            let client_height = params.get_f64("clientHeight")?;
            params.expect_no_more()?;

            Ok(Self {
                scroll_top,
                scroll_left,
                scroll_width,
                scroll_height,
                client_width,
                client_height,
            })
        })
    }
}
