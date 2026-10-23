use std::rc::Rc;

use crate::{JsJson, dev::JsJsonListDecoder, struct_mut::ValueMut};

/// Structure passed as a parameter to callback on on_click event.
///
/// The browser's default action for the click - following a link, submitting a form, toggling
/// a checkbox - happens unless the callback calls [`ClickEvent::prevent_default`], and the click
/// reaches the parent elements unless it calls [`ClickEvent::stop_propagation`]. The modifier
/// keys held during the click come along, f. ex. for Ctrl+click adding to a selection.
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
    inner: Rc<ValueMut<ClickEventInner>>,
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
    /// none held.
    pub(crate) fn from_driver(data: JsJson) -> Self {
        let keys = data.map_list(|mut params: JsJsonListDecoder| {
            let alt_key = params.get_bool("altKey")?;
            let ctrl_key = params.get_bool("ctrlKey")?;
            let shift_key = params.get_bool("shiftKey")?;
            let meta_key = params.get_bool("metaKey")?;
            params.expect_no_more()?;
            Ok((alt_key, ctrl_key, shift_key, meta_key))
        });
        let (alt_key, ctrl_key, shift_key, meta_key) = keys.unwrap_or_default();

        Self {
            alt_key,
            ctrl_key,
            shift_key,
            meta_key,
            inner: Rc::default(),
        }
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
