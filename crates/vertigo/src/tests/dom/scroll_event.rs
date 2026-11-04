//! `on_scroll`: the event as the driver sends it.

use std::{cell::RefCell, rc::Rc};

use crate::{self as vertigo, dom};
use crate::{
    DomNode, JsJson, JsJsonNumber, JsJsonSerialize, ScrollEvent,
    dev::{
        CallbackId,
        command::{CommandForWasm, DriverDomCommand},
        inspect::{log_start, log_take},
    },
    driver_module::api::api_command_wasm,
};

/// An element with `on_scroll` that keeps the events it gets in `got`, and the id of the callback
/// the driver calls for it.
fn element_with_on_scroll(got: &Rc<RefCell<Vec<ScrollEvent>>>) -> (DomNode, CallbackId) {
    let got = got.clone();
    let on_scroll = move |event: ScrollEvent| got.borrow_mut().push(event);

    log_start();
    let element = dom! { <div {on_scroll} /> };
    let commands = log_take();

    let callback_id = commands.into_iter().find_map(|command| match command {
        DriverDomCommand::CallbackAdd {
            event_name,
            callback_id,
            ..
        } if event_name == "scroll" => Some(callback_id),
        _ => None,
    });

    let Some(callback_id) = callback_id else {
        panic!("no scroll listener");
    };

    (element, callback_id)
}

fn numbers(values: &[f64]) -> JsJson {
    JsJson::List(
        values
            .iter()
            .map(|value| JsJson::Number(JsJsonNumber(*value)))
            .collect(),
    )
}

/// The event, delivered as the driver does it, and the reply the driver gets.
fn deliver(callback_id: CallbackId, value: JsJson) -> JsJson {
    let command = CommandForWasm::CallbackCall { callback_id, value };
    api_command_wasm().command_from_js(command.to_json())
}

#[test]
fn the_callback_gets_where_the_content_is_scrolled_to_and_how_big_it_is() {
    let got = Rc::new(RefCell::new(Vec::new()));
    let (_element, callback_id) = element_with_on_scroll(&got);

    // 120.5 px down a 900 px tall content, of which 300 px show, on a zoomed page
    let reply = deliver(
        callback_id,
        numbers(&[120.5, 0.0, 400.0, 900.0, 385.0, 300.0]),
    );

    assert_eq!(reply, JsJson::Null);
    assert_eq!(
        got.borrow().as_slice(),
        [ScrollEvent {
            scroll_top: 120.5,
            scroll_left: 0.0,
            scroll_width: 400.0,
            scroll_height: 900.0,
            client_width: 385.0,
            client_height: 300.0,
        }]
    );
}

#[test]
fn right_to_left_content_scrolls_to_negative_left() {
    let got = Rc::new(RefCell::new(Vec::new()));
    let (_element, callback_id) = element_with_on_scroll(&got);

    deliver(
        callback_id,
        numbers(&[0.0, -250.0, 1000.0, 100.0, 400.0, 100.0]),
    );

    let lefts = got
        .borrow()
        .iter()
        .map(|event| event.scroll_left)
        .collect::<Vec<_>>();
    assert_eq!(lefts, [-250.0]);
}

#[test]
fn an_event_that_is_not_a_scroll_reaches_no_callback() {
    let got = Rc::new(RefCell::new(Vec::new()));
    let (_element, callback_id) = element_with_on_scroll(&got);

    for value in [
        numbers(&[1.0]),
        numbers(&[1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0]),
        JsJson::Null,
    ] {
        assert_eq!(deliver(callback_id, value), JsJson::Null);
    }

    assert!(got.borrow().is_empty());
}
