//! `on_wheel`: the event as the driver sends it, and what the reply asks of the browser.

use std::{cell::RefCell, rc::Rc};

use crate::{self as vertigo, dom};
use crate::{
    DomNode, JsJson, JsJsonNumber, JsJsonSerialize, WheelDeltaMode, WheelEvent,
    dev::{
        CallbackId,
        command::{CommandForWasm, DriverDomCommand},
        inspect::{log_start, log_take},
    },
    driver_module::api::api_command_wasm,
};

/// An element with `on_wheel` that keeps the events it gets in `got` and passes each to `react`,
/// and the id of the callback the driver calls for it.
fn element_with_on_wheel(
    got: &Rc<RefCell<Vec<WheelEvent>>>,
    react: impl Fn(&WheelEvent) + 'static,
) -> (DomNode, CallbackId) {
    let got = got.clone();
    let on_wheel = move |event: WheelEvent| {
        react(&event);
        got.borrow_mut().push(event);
    };

    log_start();
    let element = dom! { <div on_wheel={on_wheel} /> };
    let commands = log_take();

    let callback_id = commands.into_iter().find_map(|command| match command {
        DriverDomCommand::CallbackAdd {
            event_name,
            callback_id,
            ..
        } if event_name == "wheel" => Some(callback_id),
        _ => None,
    });

    let Some(callback_id) = callback_id else {
        panic!("no wheel listener");
    };

    (element, callback_id)
}

fn number(value: f64) -> JsJson {
    JsJson::Number(JsJsonNumber(value))
}

/// What the driver sends for a wheel turned 120 px down at (30, 40) of a 200 x 100 element,
/// with Ctrl held, and `delta_mode` as `deltaMode` says.
fn wheel_down(delta_mode: u32) -> JsJson {
    JsJson::List(vec![
        number(-2.5),
        number(120.0),
        number(0.0),
        number(delta_mode as f64),
        number(30.0),
        number(40.0),
        number(200.0),
        number(100.0),
        JsJson::False,
        JsJson::True,
        JsJson::False,
        JsJson::False,
    ])
}

/// The event, delivered as the driver does it, and the reply the driver gets.
fn deliver(callback_id: CallbackId, value: JsJson) -> JsJson {
    let command = CommandForWasm::CallbackCall { callback_id, value };
    api_command_wasm().command_from_js(command.to_json())
}

fn flags(stop_propagation: bool, prevent_default: bool) -> JsJson {
    JsJson::Object(
        [
            (
                "stop_propagation".to_string(),
                JsJson::from(stop_propagation),
            ),
            ("prevent_default".to_string(), JsJson::from(prevent_default)),
        ]
        .into_iter()
        .collect(),
    )
}

#[test]
fn the_callback_gets_how_far_the_wheel_turned_and_where_the_pointer_is() {
    let got = Rc::new(RefCell::new(Vec::new()));
    let (_element, callback_id) = element_with_on_wheel(&got, |_| {});

    deliver(callback_id, wheel_down(0));

    let got = got.borrow();
    let [event] = got.as_slice() else {
        panic!("one event expected, got {got:?}");
    };
    assert_eq!(
        (event.delta_x, event.delta_y, event.delta_z),
        (-2.5, 120.0, 0.0)
    );
    assert_eq!(event.delta_mode, WheelDeltaMode::Pixel);
    assert_eq!((event.pointer_x, event.pointer_y), (30.0, 40.0));
    assert_eq!((event.element_width, event.element_height), (200.0, 100.0));
    assert_eq!(
        (
            event.alt_key,
            event.ctrl_key,
            event.shift_key,
            event.meta_key
        ),
        (false, true, false, false)
    );
}

#[test]
fn delta_mode_follows_the_browser() {
    let got = Rc::new(RefCell::new(Vec::new()));
    let (_element, callback_id) = element_with_on_wheel(&got, |_| {});

    for mode in [0, 1, 2, 3] {
        deliver(callback_id, wheel_down(mode));
    }

    let modes = got
        .borrow()
        .iter()
        .map(|event| event.delta_mode)
        .collect::<Vec<_>>();
    assert_eq!(
        modes,
        [
            WheelDeltaMode::Pixel,
            WheelDeltaMode::Line,
            WheelDeltaMode::Page,
            // A mode browsers don't have yet counts as their default.
            WheelDeltaMode::Pixel,
        ]
    );
}

#[test]
fn the_page_scrolls_unless_the_callback_prevents_it() {
    let got = Rc::new(RefCell::new(Vec::new()));
    let (_element, callback_id) = element_with_on_wheel(&got, |_| {});

    assert_eq!(deliver(callback_id, wheel_down(0)), flags(false, false));
}

#[test]
fn the_reply_carries_what_the_callback_asked_of_the_browser() {
    let got = Rc::new(RefCell::new(Vec::new()));
    let (_prevents, prevents) = element_with_on_wheel(&got, |event| event.prevent_default());
    let (_stops, stops) = element_with_on_wheel(&got, |event| event.stop_propagation());

    assert_eq!(deliver(prevents, wheel_down(0)), flags(false, true));
    assert_eq!(deliver(stops, wheel_down(0)), flags(true, false));
}

#[test]
fn an_event_that_is_not_a_wheel_reaches_no_callback() {
    let got = Rc::new(RefCell::new(Vec::new()));
    let (_element, callback_id) = element_with_on_wheel(&got, |_| {});

    let reply = deliver(callback_id, JsJson::List(vec![number(1.0)]));

    assert_eq!(reply, JsJson::Null);
    assert!(got.borrow().is_empty());
}
