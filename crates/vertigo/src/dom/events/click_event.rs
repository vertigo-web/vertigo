use std::rc::Rc;

use crate::{JsJson, struct_mut::ValueMut};

/// Structure passed as a parameter to callback on on_click event.
///
/// The browser's default action for the click - following a link, submitting a form, toggling
/// a checkbox - happens unless the callback calls [`ClickEvent::prevent_default`], and the click
/// reaches the parent elements unless it calls [`ClickEvent::stop_propagation`].
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
