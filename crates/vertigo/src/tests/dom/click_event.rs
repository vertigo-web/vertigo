//! `on_click`: the event as the driver sends it, and what the reply asks of the browser.

use std::{cell::RefCell, rc::Rc};

use crate::{self as vertigo, dom};
use crate::{
    ClickEvent, DomNode, JsJson, JsJsonSerialize,
    dev::{
        CallbackId,
        command::{CommandForWasm, DriverDomCommand},
        inspect::{log_start, log_take},
    },
    driver_module::api::api_command_wasm,
};

/// An element with `on_click` that keeps the events it gets in `got` and passes each to `react`,
/// and the id of the callback the driver calls for it.
fn element_with_on_click(
    got: &Rc<RefCell<Vec<ClickEvent>>>,
    react: impl Fn(&ClickEvent) + 'static,
) -> (DomNode, CallbackId) {
    let got = got.clone();
    let on_click = move |event: ClickEvent| {
        react(&event);
        got.borrow_mut().push(event);
    };

    log_start();
    let element = dom! { <button on_click={on_click} /> };
    let commands = log_take();

    let callback_id = commands.into_iter().find_map(|command| match command {
        DriverDomCommand::CallbackAdd {
            event_name,
            callback_id,
            ..
        } if event_name == "click" => Some(callback_id),
        _ => None,
    });

    let Some(callback_id) = callback_id else {
        panic!("no click listener");
    };

    (element, callback_id)
}

/// What the driver sends for a click with Ctrl and ⌘ held.
fn ctrl_meta_click() -> JsJson {
    JsJson::List(vec![
        JsJson::False,
        JsJson::True,
        JsJson::False,
        JsJson::True,
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

fn keys(event: &ClickEvent) -> (bool, bool, bool, bool) {
    (
        event.alt_key,
        event.ctrl_key,
        event.shift_key,
        event.meta_key,
    )
}

#[test]
fn the_callback_gets_the_modifier_keys() {
    let got = Rc::new(RefCell::new(Vec::new()));
    let (_element, callback_id) = element_with_on_click(&got, |_| {});

    deliver(callback_id, ctrl_meta_click());

    let got = got.borrow();
    let [event] = got.as_slice() else {
        panic!("one event expected, got {got:?}");
    };
    assert_eq!(keys(event), (false, true, false, true));
}

#[test]
fn a_click_without_the_keys_still_reaches_the_callback() {
    let got = Rc::new(RefCell::new(Vec::new()));
    let (_element, callback_id) = element_with_on_click(&got, |_| {});

    deliver(callback_id, JsJson::Null);
    deliver(callback_id, JsJson::List(vec![JsJson::True]));

    let got = got.borrow();
    let all_up = got.iter().map(keys).collect::<Vec<_>>();
    assert_eq!(all_up, [(false, false, false, false); 2]);
}

#[test]
fn the_reply_carries_what_the_callback_asked_of_the_browser() {
    let got = Rc::new(RefCell::new(Vec::new()));
    let (_plain, plain) = element_with_on_click(&got, |_| {});
    let (_prevents, prevents) = element_with_on_click(&got, |event| event.prevent_default());
    let (_stops, stops) = element_with_on_click(&got, |event| event.stop_propagation());

    assert_eq!(deliver(plain, ctrl_meta_click()), flags(false, false));
    assert_eq!(deliver(prevents, ctrl_meta_click()), flags(false, true));
    assert_eq!(deliver(stops, ctrl_meta_click()), flags(true, false));
}
