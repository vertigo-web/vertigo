use crate::{JsJson, dev::JsJsonListDecoder};

/// Structure passed as a parameter to callback on on_resize event: how big the element is as
/// drawn, in CSS pixels.
///
/// It comes once the element is on the page, and again whenever its size changes, for whatever
/// reason: the window, the content, a style or a sibling. The size comes from the box of the
/// element as drawn on the screen (`getBoundingClientRect`), with its padding and border, as the
/// size in a [`PointerEvent`](crate::PointerEvent) or a [`WheelEvent`](crate::WheelEvent): for an
/// element turned by a transform, it's the box around it. The browser tells of a change at most
/// once a frame, before the frame is drawn.
///
/// Nothing comes while rendering on the server, so the size to start with is the app's own guess.
///
/// ```rust
/// use vertigo::{ResizeEvent, Value, bind, dom};
///
/// // A canvas that draws as much as its size holds, rather than the same picture scaled
/// let size = Value::new((640.0, 400.0));
///
/// let on_resize = bind!(size, |event: ResizeEvent| size.set((event.width, event.height)));
///
/// dom! {
///     <canvas style="width: 100%; height: 50vh;" {on_resize} />
/// };
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ResizeEvent {
    /// Width of the element, in CSS pixels, as drawn.
    pub width: f64,
    /// Height of the element, in CSS pixels, as drawn.
    pub height: f64,
}

impl ResizeEvent {
    /// The event as the driver sends it. The order MUST match `resizeAdd` in `callbackManager.ts`.
    pub(crate) fn from_driver(data: JsJson) -> Result<Self, String> {
        data.map_list(|mut params: JsJsonListDecoder| {
            let width = params.get_f64("width")?;
            let height = params.get_f64("height")?;
            params.expect_no_more()?;

            Ok(Self { width, height })
        })
    }
}
