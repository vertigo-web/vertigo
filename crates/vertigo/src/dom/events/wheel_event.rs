use std::rc::Rc;

use crate::{JsJson, dev::JsJsonListDecoder, struct_mut::ValueMut};

/// The unit of the deltas of a [WheelEvent].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum WheelDeltaMode {
    /// CSS pixels, what a touchpad reports, and in most browsers a mouse wheel as well.
    #[default]
    Pixel,
    /// Lines of text.
    Line,
    /// Pages.
    Page,
}

/// Structure passed as a parameter to callback on on_wheel event.
///
/// The wheel scrolls the page, and in Chrome and Firefox with Ctrl held zooms it, unless the
/// callback calls [`WheelEvent::prevent_default`]. There a pinch on a touchpad comes as a wheel
/// with Ctrl held too, while Safari reports a pinch with events of its own. The event reaches
/// the parent elements unless the callback calls [`WheelEvent::stop_propagation`].
///
/// Where the pointer is, and how big the element is, come from the box of the element as drawn on
/// the screen (`getBoundingClientRect`): for an element turned by a transform, it's the box around
/// it. `Default` gives an event of no movement, f. ex. to start from in a test of a handler.
///
/// ```rust
/// use vertigo::{Value, WheelEvent, dom};
///
/// let zoom = Value::new(1.0);
///
/// // Zooms the picture instead of scrolling the page
/// let on_wheel = {
///     let zoom = zoom.clone();
///     move |event: WheelEvent| {
///         event.prevent_default();
///         zoom.change(|zoom| *zoom *= (-event.delta_y / 500.0).exp());
///     }
/// };
///
/// dom! {
///     <div on_wheel={on_wheel}>"Zoom: " {zoom}</div>
/// };
/// ```
#[derive(Clone, Debug, Default)]
pub struct WheelEvent {
    /// How far to scroll rightwards, in [`delta_mode`](Self::delta_mode) units. Negative means
    /// leftwards.
    pub delta_x: f64,
    /// How far to scroll downwards, in [`delta_mode`](Self::delta_mode) units. Negative means
    /// upwards: the wheel turned away from the user.
    pub delta_y: f64,
    /// How far to scroll along the z axis, in [`delta_mode`](Self::delta_mode) units.
    pub delta_z: f64,
    pub delta_mode: WheelDeltaMode,
    /// Where the pointer is, in CSS pixels from the left edge of the element with the
    /// `on_wheel`. With [`element_width`](Self::element_width), f. ex. the point to zoom at.
    pub pointer_x: f64,
    /// Where the pointer is, in CSS pixels from the top edge of the element with the `on_wheel`.
    pub pointer_y: f64,
    /// Width of the element with the `on_wheel`, in CSS pixels, as drawn.
    pub element_width: f64,
    /// Height of the element with the `on_wheel`, in CSS pixels, as drawn.
    pub element_height: f64,
    pub alt_key: bool,
    /// Also set while the user pinches a touchpad, in Chrome and Firefox.
    pub ctrl_key: bool,
    pub shift_key: bool,
    pub meta_key: bool,
    flags: Rc<ValueMut<WheelEventFlags>>,
}

#[derive(Clone, Copy, Debug, Default)]
struct WheelEventFlags {
    stop_propagation: bool,
    prevent_default: bool,
}

impl WheelEvent {
    /// The event doesn't reach the `on_wheel` of the parent elements.
    pub fn stop_propagation(&self) {
        self.flags.change(|flags| flags.stop_propagation = true);
    }

    /// Make the browser not scroll or zoom the page.
    pub fn prevent_default(&self) {
        self.flags.change(|flags| flags.prevent_default = true);
    }

    /// The event as the driver sends it. The order MUST match `wheel` in `callbackManager.ts`.
    pub(crate) fn from_driver(data: JsJson) -> Result<Self, String> {
        data.map_list(|mut params: JsJsonListDecoder| {
            let delta_x = params.get_f64("deltaX")?;
            let delta_y = params.get_f64("deltaY")?;
            let delta_z = params.get_f64("deltaZ")?;
            // `DOM_DELTA_PIXEL` is what browsers report by default, so anything but a line or
            // a page counts as pixels.
            let delta_mode = match params.get_u32("deltaMode")? {
                1 => WheelDeltaMode::Line,
                2 => WheelDeltaMode::Page,
                _ => WheelDeltaMode::Pixel,
            };
            let pointer_x = params.get_f64("pointerX")?;
            let pointer_y = params.get_f64("pointerY")?;
            let element_width = params.get_f64("elementWidth")?;
            let element_height = params.get_f64("elementHeight")?;
            let alt_key = params.get_bool("altKey")?;
            let ctrl_key = params.get_bool("ctrlKey")?;
            let shift_key = params.get_bool("shiftKey")?;
            let meta_key = params.get_bool("metaKey")?;
            params.expect_no_more()?;

            Ok(Self {
                delta_x,
                delta_y,
                delta_z,
                delta_mode,
                pointer_x,
                pointer_y,
                element_width,
                element_height,
                alt_key,
                ctrl_key,
                shift_key,
                meta_key,
                flags: Rc::default(),
            })
        })
    }
}

/// What the callback asked of the browser. The driver reads it as it reads a
/// [`ClickEvent`](crate::ClickEvent).
impl From<WheelEvent> for JsJson {
    fn from(val: WheelEvent) -> JsJson {
        let flags = val.flags.get();
        JsJson::Object(
            [
                (
                    "stop_propagation".to_string(),
                    JsJson::from(flags.stop_propagation),
                ),
                (
                    "prevent_default".to_string(),
                    JsJson::from(flags.prevent_default),
                ),
            ]
            .into_iter()
            .collect(),
        )
    }
}
