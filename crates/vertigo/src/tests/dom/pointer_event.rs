//! The pointer events: what the driver sends, and what the reply asks of the browser.

use std::{cell::RefCell, rc::Rc};

use crate::{self as vertigo, dom};
use crate::{
    DomNode, JsJson, JsJsonNumber, JsJsonSerialize, PointerEvent, PointerType,
    dev::{
        CallbackId,
        command::{CommandForWasm, DriverDomCommand},
        inspect::{log_start, log_take},
    },
    driver_module::api::api_command_wasm,
};

type Got = Rc<RefCell<Vec<(&'static str, PointerEvent)>>>;

/// An element with the four pointer listeners, each keeping the events it gets in `got` and
/// passing each to `react`, and the ids of the callbacks the driver calls, by DOM event name.
fn element_with_pointer_events(
    got: &Got,
    react: impl Fn(&PointerEvent) + 'static,
) -> (DomNode, Vec<(String, CallbackId)>) {
    let react = Rc::new(react);
    let listener = |name: &'static str| {
        let (got, react) = (got.clone(), react.clone());
        move |event: PointerEvent| {
            react(&event);
            got.borrow_mut().push((name, event));
        }
    };

    log_start();
    let element = dom! {
        <div
            on_pointer_down={listener("down")}
            on_pointer_move={listener("move")}
            on_pointer_up={listener("up")}
            on_pointer_cancel={listener("cancel")}
        />
    };
    let commands = log_take();

    let callbacks = commands
        .into_iter()
        .filter_map(|command| match command {
            DriverDomCommand::CallbackAdd {
                event_name,
                callback_id,
                ..
            } => Some((event_name, callback_id)),
            _ => None,
        })
        .collect();

    (element, callbacks)
}

fn callback(callbacks: &[(String, CallbackId)], event_name: &str) -> CallbackId {
    let found = callbacks
        .iter()
        .find_map(|(name, id)| (name == event_name).then_some(*id));
    let Some(id) = found else {
        panic!("no {event_name} listener in {callbacks:?}");
    };
    id
}

fn number(value: f64) -> JsJson {
    JsJson::Number(JsJsonNumber(value))
}

/// What the driver sends for a finger of id 7 and of `pointer_type` touching (30, 40) of
/// a 200 x 100 element, with Shift held.
fn touch(pointer_type: &str) -> JsJson {
    JsJson::List(vec![
        number(7.0),
        JsJson::String(pointer_type.to_string()),
        JsJson::True,
        number(0.0),
        number(1.0),
        number(30.0),
        number(40.0),
        number(200.0),
        number(100.0),
        JsJson::False,
        JsJson::False,
        JsJson::True,
        JsJson::False,
    ])
}

/// The event, delivered as the driver does it, and the reply the driver gets.
fn deliver(callback_id: CallbackId, value: JsJson) -> JsJson {
    let command = CommandForWasm::CallbackCall { callback_id, value };
    api_command_wasm().command_from_js(command.to_json())
}

fn flags(stop_propagation: bool, prevent_default: bool, capture_pointer: bool) -> JsJson {
    JsJson::Object(
        [
            (
                "stop_propagation".to_string(),
                JsJson::from(stop_propagation),
            ),
            ("prevent_default".to_string(), JsJson::from(prevent_default)),
            ("capture_pointer".to_string(), JsJson::from(capture_pointer)),
        ]
        .into_iter()
        .collect(),
    )
}

#[test]
fn each_listener_goes_on_the_dom_event_of_its_name() {
    let got = Rc::new(RefCell::new(Vec::new()));
    let (_element, callbacks) = element_with_pointer_events(&got, |_| {});

    for event_name in ["pointerdown", "pointermove", "pointerup", "pointercancel"] {
        deliver(callback(&callbacks, event_name), touch("touch"));
    }

    let names = got
        .borrow()
        .iter()
        .map(|(name, _)| *name)
        .collect::<Vec<_>>();
    assert_eq!(names, ["down", "move", "up", "cancel"]);
}

#[test]
fn the_callback_gets_which_pointer_it_is_and_where() {
    let got = Rc::new(RefCell::new(Vec::new()));
    let (_element, callbacks) = element_with_pointer_events(&got, |_| {});

    deliver(callback(&callbacks, "pointerdown"), touch("touch"));

    let got = got.borrow();
    let [(_, event)] = got.as_slice() else {
        panic!("one event expected, got {got:?}");
    };
    assert_eq!(event.pointer_id, 7);
    assert_eq!(event.pointer_type, PointerType::Touch);
    assert!(event.is_primary);
    assert_eq!((event.button, event.buttons), (0, 1));
    assert_eq!((event.pointer_x, event.pointer_y), (30.0, 40.0));
    assert_eq!((event.element_width, event.element_height), (200.0, 100.0));
    assert_eq!(
        (
            event.alt_key,
            event.ctrl_key,
            event.shift_key,
            event.meta_key
        ),
        (false, false, true, false)
    );
}

#[test]
fn the_pointer_type_follows_the_browser() {
    let got = Rc::new(RefCell::new(Vec::new()));
    let (_element, callbacks) = element_with_pointer_events(&got, |_| {});

    for pointer_type in ["mouse", "pen", "touch", ""] {
        deliver(callback(&callbacks, "pointermove"), touch(pointer_type));
    }

    let types = got
        .borrow()
        .iter()
        .map(|(_, event)| event.pointer_type)
        .collect::<Vec<_>>();
    assert_eq!(
        types,
        [
            PointerType::Mouse,
            PointerType::Pen,
            PointerType::Touch,
            // A pointer the browser can't tell.
            PointerType::Mouse,
        ]
    );
}

#[test]
fn a_move_without_a_change_of_buttons_has_none() {
    let got = Rc::new(RefCell::new(Vec::new()));
    let (_element, callbacks) = element_with_pointer_events(&got, |_| {});

    let JsJson::List(mut params) = touch("mouse") else {
        unreachable!();
    };
    params[3] = number(-1.0);
    params[4] = number(0.0);
    deliver(callback(&callbacks, "pointermove"), JsJson::List(params));

    let got = got.borrow();
    let [(_, event)] = got.as_slice() else {
        panic!("one event expected, got {got:?}");
    };
    assert_eq!((event.button, event.buttons), (-1, 0));
}

#[test]
fn the_browser_goes_on_as_usual_unless_the_callback_asks_otherwise() {
    let got = Rc::new(RefCell::new(Vec::new()));
    let (_element, callbacks) = element_with_pointer_events(&got, |_| {});

    let reply = deliver(callback(&callbacks, "pointerdown"), touch("touch"));

    assert_eq!(reply, flags(false, false, false));
}

#[test]
fn the_reply_carries_what_the_callback_asked_of_the_browser() {
    let got = Rc::new(RefCell::new(Vec::new()));
    let (_prevents, prevents) = element_with_pointer_events(&got, |event| event.prevent_default());
    let (_stops, stops) = element_with_pointer_events(&got, |event| event.stop_propagation());
    let (_captures, captures) = element_with_pointer_events(&got, |event| event.capture_pointer());

    assert_eq!(
        deliver(callback(&prevents, "pointerdown"), touch("touch")),
        flags(false, true, false)
    );
    assert_eq!(
        deliver(callback(&stops, "pointerup"), touch("touch")),
        flags(true, false, false)
    );
    assert_eq!(
        deliver(callback(&captures, "pointermove"), touch("touch")),
        flags(false, false, true)
    );
}

#[test]
fn an_event_that_is_not_a_pointer_reaches_no_callback() {
    let got = Rc::new(RefCell::new(Vec::new()));
    let (_element, callbacks) = element_with_pointer_events(&got, |_| {});

    let reply = deliver(
        callback(&callbacks, "pointerdown"),
        JsJson::List(vec![number(1.0)]),
    );

    assert_eq!(reply, JsJson::Null);
    assert!(got.borrow().is_empty());
}
