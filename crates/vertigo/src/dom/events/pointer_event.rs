use std::rc::Rc;

use crate::{JsJson, dev::JsJsonListDecoder, struct_mut::ValueMut};

/// What a [PointerEvent] comes from.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PointerType {
    /// A mouse, or a pointer the browser can't tell.
    #[default]
    Mouse,
    Pen,
    Touch,
}

/// Structure passed as a parameter to callback on `on_pointer_down`, `on_pointer_move`,
/// `on_pointer_up` and `on_pointer_cancel` events.
///
/// A pointer is a mouse, a pen or a finger on a touch screen; each finger is a pointer of its
/// own, with its own [`pointer_id`](Self::pointer_id). The browser cancels a pointer it takes
/// over, f. ex. to scroll the page with a finger, unless the element's CSS `touch-action` says
/// otherwise.
///
/// Once the element calls [`PointerEvent::capture_pointer`], the events of that pointer go to it
/// until it's released, even when it leaves the element. That's what dragging needs:
///
/// ```rust
/// use std::{cell::Cell, rc::Rc};
/// use vertigo::{PointerEvent, Value, dom};
///
/// let position = Value::new(0.0);
/// // Where the dragging pointer was in its last event.
/// let grabbed = Rc::new(Cell::new(None::<f64>));
///
/// let down = {
///     let grabbed = grabbed.clone();
///     move |event: PointerEvent| {
///         event.capture_pointer();
///         grabbed.set(Some(event.pointer_x));
///     }
/// };
/// let moved = {
///     let (grabbed, position) = (grabbed.clone(), position.clone());
///     move |event: PointerEvent| {
///         if let Some(last) = grabbed.get() {
///             position.change(|position| *position += event.pointer_x - last);
///             grabbed.set(Some(event.pointer_x));
///         }
///     }
/// };
/// let up = move |_: PointerEvent| grabbed.set(None);
///
/// dom! {
///     <div
///         style="touch-action: none"
///         on_pointer_down={down}
///         on_pointer_move={moved}
///         on_pointer_up={up.clone()}
///         on_pointer_cancel={up}
///     >
///         "Moved by " {position}
///     </div>
/// };
/// ```
///
/// Where the pointer is, and how big the element is, come from the box of the element as drawn on
/// the screen (`getBoundingClientRect`), as with a [`WheelEvent`](crate::WheelEvent). `Default`
/// gives an event with every number 0 and every flag false, f. ex. to start from in a test of
/// a handler, setting the fields it reads.
#[derive(Clone, Debug, Default)]
pub struct PointerEvent {
    /// Tells the pointers apart: each finger touching the screen has an id of its own.
    pub pointer_id: i32,
    pub pointer_type: PointerType,
    /// The mouse, or the first finger to touch the screen while no other did.
    pub is_primary: bool,
    /// The button pressed or released: 0 the main one (left), 1 the middle one, 2 the secondary
    /// one (right). -1 when none was, as in a move.
    pub button: i16,
    /// The buttons held, one bit each: 1 the main one, 2 the secondary one, 4 the middle one. A
    /// finger or a pen touching the screen holds the main one.
    pub buttons: u16,
    /// Where the pointer is, in CSS pixels from the left edge of the element with the listener,
    /// also when it's over a child of it or, captured, outside of it.
    pub pointer_x: f64,
    /// Where the pointer is, in CSS pixels from the top edge of the element with the listener.
    pub pointer_y: f64,
    /// Width of the element with the listener, in CSS pixels, as drawn.
    pub element_width: f64,
    /// Height of the element with the listener, in CSS pixels, as drawn.
    pub element_height: f64,
    pub alt_key: bool,
    pub ctrl_key: bool,
    pub shift_key: bool,
    pub meta_key: bool,
    flags: Rc<ValueMut<PointerEventFlags>>,
}

#[derive(Clone, Copy, Debug, Default)]
struct PointerEventFlags {
    stop_propagation: bool,
    prevent_default: bool,
    capture_pointer: bool,
}

impl PointerEvent {
    /// The event doesn't reach the listeners of the parent elements.
    pub fn stop_propagation(&self) {
        self.flags.change(|flags| flags.stop_propagation = true);
    }

    /// Make the browser not do what it usually would. In `on_pointer_down`, the browser then
    /// doesn't send the mouse events (`mousedown`, `mousemove`, `mouseup`) that would follow, but
    /// it still sends a click.
    pub fn prevent_default(&self) {
        self.flags.change(|flags| flags.prevent_default = true);
    }

    /// The events of this pointer go to the element with the listener until the pointer is
    /// released, even when it leaves the element - f. ex. to go on dragging. Only for a pointer
    /// that's down.
    ///
    /// Browsers following Pointer Events 3 send the click after the release to the capturing
    /// element too, so a child's `on_click` doesn't get it. To keep the children clickable,
    /// capture once the pointer has moved some way, rather than when it's pressed.
    ///
    /// The capture ends with the element: one that renders anew mid-drag, f. ex. after a change
    /// of what it shows, has to ask for it again.
    pub fn capture_pointer(&self) {
        self.flags.change(|flags| flags.capture_pointer = true);
    }

    /// The event as the driver sends it. The order MUST match `pointer` in `callbackManager.ts`.
    pub(crate) fn from_driver(data: JsJson) -> Result<Self, String> {
        data.map_list(|mut params: JsJsonListDecoder| {
            let pointer_id = params.get_f64("pointerId")? as i32;
            let pointer_type = match params.get_string("pointerType")?.as_str() {
                "pen" => PointerType::Pen,
                "touch" => PointerType::Touch,
                _ => PointerType::Mouse,
            };
            let is_primary = params.get_bool("isPrimary")?;
            let button = params.get_f64("button")? as i16;
            let buttons = params.get_f64("buttons")? as u16;
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
                pointer_id,
                pointer_type,
                is_primary,
                button,
                buttons,
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

/// What the callback asked of the browser: the flags of a [`ClickEvent`](crate::ClickEvent),
/// and whether to capture the pointer.
impl From<PointerEvent> for JsJson {
    fn from(val: PointerEvent) -> JsJson {
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
                (
                    "capture_pointer".to_string(),
                    JsJson::from(flags.capture_pointer),
                ),
            ]
            .into_iter()
            .collect(),
        )
    }
}
