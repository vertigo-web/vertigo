//! The keys pressed and let go: what the driver sends, and what the reply asks of the browser.

use std::{cell::RefCell, rc::Rc};

use crate::{self as vertigo, dom};
use crate::{
    DomNode, JsJson, JsJsonSerialize, KeyboardEvent,
    dev::{
        CallbackId,
        command::{CommandForWasm, DriverDomCommand},
        inspect::{log_start, log_take},
    },
    driver_module::api::api_command_wasm,
};

type Got = Rc<RefCell<Vec<(&'static str, KeyboardEvent)>>>;

/// An element with the four keyboard listeners, each keeping the events it gets in `got` and
/// replying `handled`, and the ids of the callbacks the driver calls, by their wire names.
fn element_with_keyboard_events(got: &Got, handled: bool) -> (DomNode, Vec<(String, CallbackId)>) {
    let listener = |name: &'static str| {
        let got = got.clone();
        move |event: KeyboardEvent| {
            got.borrow_mut().push((name, event));
            handled
        }
    };

    log_start();
    let element = dom! {
        <div
            on_key_down={listener("down")}
            on_key_up={listener("up")}
            hook_key_down={listener("hook down")}
            hook_key_up={listener("hook up")}
        />
    };
    let callbacks = log_take()
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

/// What the driver sends for the key A with Shift, its press repeated or not.
fn key_a(repeat: bool) -> JsJson {
    JsJson::List(vec![
        JsJson::String("A".to_string()),
        JsJson::String("KeyA".to_string()),
        JsJson::False,
        JsJson::False,
        JsJson::True,
        JsJson::False,
        JsJson::from(repeat),
    ])
}

/// The event, delivered as the driver does it, and the reply the driver gets.
fn deliver(callback_id: CallbackId, value: JsJson) -> JsJson {
    let command = CommandForWasm::CallbackCall { callback_id, value };
    api_command_wasm().command_from_js(command.to_json())
}

#[test]
fn each_listener_goes_on_the_dom_event_of_its_name() {
    let got = Rc::new(RefCell::new(Vec::new()));
    let (_element, callbacks) = element_with_keyboard_events(&got, false);

    for event_name in ["keydown", "keyup", "hook_keydown", "hook_keyup"] {
        deliver(callback(&callbacks, event_name), key_a(false));
    }

    let names = got
        .borrow()
        .iter()
        .map(|(name, _)| *name)
        .collect::<Vec<_>>();
    assert_eq!(names, ["down", "up", "hook down", "hook up"]);
}

#[test]
fn the_callback_gets_the_key_and_whether_its_press_repeats() {
    let got = Rc::new(RefCell::new(Vec::new()));
    let (_element, callbacks) = element_with_keyboard_events(&got, false);

    deliver(callback(&callbacks, "hook_keydown"), key_a(false));
    deliver(callback(&callbacks, "hook_keydown"), key_a(true));

    let expected = |repeat| KeyboardEvent {
        key: "A".to_string(),
        code: "KeyA".to_string(),
        shift_key: true,
        repeat,
        ..KeyboardEvent::default()
    };
    let got = got
        .borrow()
        .iter()
        .map(|(_, event)| event.clone())
        .collect::<Vec<_>>();
    assert_eq!(got, [expected(false), expected(true)]);
}

#[test]
fn the_reply_tells_whether_the_key_was_handled() {
    for handled in [true, false] {
        let got = Rc::new(RefCell::new(Vec::new()));
        let (_element, callbacks) = element_with_keyboard_events(&got, handled);

        for event_name in ["keydown", "keyup", "hook_keydown", "hook_keyup"] {
            let reply = deliver(callback(&callbacks, event_name), key_a(false));
            assert_eq!(reply, JsJson::from(handled), "{event_name}");
        }
    }
}

#[test]
fn an_event_the_listener_cant_read_is_not_handled() {
    let got = Rc::new(RefCell::new(Vec::new()));
    let (_element, callbacks) = element_with_keyboard_events(&got, true);

    // Without `repeat`, as an older driver sent it
    let JsJson::List(mut older) = key_a(false) else {
        unreachable!()
    };
    older.pop();
    let reply = deliver(callback(&callbacks, "keydown"), JsJson::List(older));

    assert_eq!(reply, JsJson::False);
    assert!(got.borrow().is_empty());
}
