use std::rc::Rc;

use crate::{JsJson, dev::JsJsonListDecoder, struct_mut::ValueMut};

/// Structure passed as a parameter to callback on on_click event.
///
/// The browser's default action for the click - following a link, submitting a form, toggling
/// a checkbox - happens unless the callback calls [`ClickEvent::prevent_default`], and the click
/// reaches the parent elements unless it calls [`ClickEvent::stop_propagation`]. The modifier
/// keys held during the click come along, f. ex. for Ctrl+click adding to a selection, and where
/// the pointer clicked, f. ex. on a `<canvas>`: see [`ClickEvent::position`].
///
/// ```rust
/// use vertigo::{ClickEvent, dom};
///
/// // A link that runs code instead of opening the page it points to
/// let on_click = |event: ClickEvent| {
///     event.prevent_default();
///     vertigo::log::info!("Opened in a dialog instead");
/// };
///
/// dom! {
///     <a href="/details" on_click={on_click}>"Details"</a>
/// };
/// ```
#[derive(Clone, Debug, Default)]
pub struct ClickEvent {
    pub alt_key: bool,
    pub ctrl_key: bool,
    pub shift_key: bool,
    /// ⌘ on a Mac, the Windows key elsewhere.
    pub meta_key: bool,
    /// Where the mouse, a finger or a pen clicked. `None` for a click without a pointer: from the
    /// keyboard, f. ex. Enter on a focused button, or from a script.
    pub position: Option<ClickPosition>,
    inner: Rc<ValueMut<ClickEventInner>>,
}

/// Where a click was, see [`ClickEvent::position`], in CSS pixels from the top-left corner of the
/// element with the `on_click`, also when a child of it was clicked.
///
/// The element's size comes from its box as drawn on the screen (`getBoundingClientRect`): for an
/// element turned by a transform, it's the box around it.
///
/// ```rust
/// use vertigo::{ClickEvent, Value, dom};
///
/// // Where on the picture it was clicked, from 0 to 1 across and down it
/// let picked = Value::new((0.5, 0.5));
/// let on_click = {
///     let picked = picked.clone();
///     move |event: ClickEvent| {
///         if let Some(at) = event.position {
///             picked.set((at.x / at.element_width, at.y / at.element_height));
///         }
///     }
/// };
///
/// dom! {
///     <canvas width="640" height="480" on_click={on_click} />
/// };
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ClickPosition {
    /// From the left edge of the element.
    pub x: f64,
    /// From the top edge of the element.
    pub y: f64,
    pub element_width: f64,
    pub element_height: f64,
}

#[derive(Clone, Debug, Default)]
pub struct ClickEventInner {
    stop_propagation: bool,
    prevent_default: bool,
}

impl ClickEvent {
    /// The click doesn't reach the `on_click` of the parent elements.
    pub fn stop_propagation(&self) {
        self.inner.change(|inner| inner.stop_propagation = true);
    }

    /// Make the browser not do what the click usually would do.
    pub fn prevent_default(&self) {
        self.inner.change(|inner| inner.prevent_default = true);
    }

    /// The event as the driver sends it. The order MUST match `click` in `callbackManager.ts`.
    ///
    /// A click is never dropped: one without the keys, f. ex. from a test, counts as made with
    /// none held, and without a pointer.
    pub(crate) fn from_driver(data: JsJson) -> Self {
        let decoded = data.map_list(|mut params: JsJsonListDecoder| {
            let alt_key = params.get_bool("altKey")?;
            let ctrl_key = params.get_bool("ctrlKey")?;
            let shift_key = params.get_bool("shiftKey")?;
            let meta_key = params.get_bool("metaKey")?;
            let position = match params.get_any("position")? {
                JsJson::Null => None,
                position => Some(position.map_list(|mut params: JsJsonListDecoder| {
                    let position = ClickPosition {
                        x: params.get_f64("pointerX")?,
                        y: params.get_f64("pointerY")?,
                        element_width: params.get_f64("elementWidth")?,
                        element_height: params.get_f64("elementHeight")?,
                    };
                    params.expect_no_more()?;
                    Ok(position)
                })?),
            };
            params.expect_no_more()?;
            Ok(Self {
                alt_key,
                ctrl_key,
                shift_key,
                meta_key,
                position,
                inner: Rc::default(),
            })
        });

        decoded.unwrap_or_default()
    }
}

impl From<ClickEvent> for JsJson {
    fn from(val: ClickEvent) -> JsJson {
        let inner = val.inner.get();
        JsJson::Object(
            [
                (
                    "stop_propagation".to_string(),
                    JsJson::from(inner.stop_propagation),
                ),
                (
                    "prevent_default".to_string(),
                    JsJson::from(inner.prevent_default),
                ),
            ]
            .into_iter()
            .collect(),
        )
    }
}
