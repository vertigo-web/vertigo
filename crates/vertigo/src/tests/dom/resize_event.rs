//! `on_resize`: the event as the driver sends it.

use std::{cell::RefCell, rc::Rc};

use crate::{self as vertigo, dom};
use crate::{
    DomNode, JsJson, JsJsonNumber, JsJsonSerialize, ResizeEvent,
    dev::{
        CallbackId,
        command::{CommandForWasm, DriverDomCommand},
        inspect::{log_start, log_take},
    },
    driver_module::api::api_command_wasm,
};

/// An element with `on_resize` that keeps the events it gets in `got`, and the id of the callback
/// the driver calls for it.
fn element_with_on_resize(got: &Rc<RefCell<Vec<ResizeEvent>>>) -> (DomNode, CallbackId) {
    let got = got.clone();
    let on_resize = move |event: ResizeEvent| got.borrow_mut().push(event);

    log_start();
    let element = dom! { <canvas {on_resize} /> };
    let commands = log_take();

    let callback_id = commands.into_iter().find_map(|command| match command {
        DriverDomCommand::CallbackAdd {
            event_name,
            callback_id,
            ..
        } if event_name == "resize" => Some(callback_id),
        _ => None,
    });

    let Some(callback_id) = callback_id else {
        panic!("no resize listener");
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
fn the_callback_gets_the_size_once_on_the_page_and_after_each_change() {
    let got = Rc::new(RefCell::new(Vec::new()));
    let (_element, callback_id) = element_with_on_resize(&got);

    // On the page, then the window grew, on a zoomed page
    assert_eq!(deliver(callback_id, numbers(&[640.0, 400.0])), JsJson::Null);
    assert_eq!(
        deliver(callback_id, numbers(&[1210.5, 756.25])),
        JsJson::Null
    );

    assert_eq!(
        got.borrow().as_slice(),
        [
            ResizeEvent {
                width: 640.0,
                height: 400.0,
            },
            ResizeEvent {
                width: 1210.5,
                height: 756.25,
            },
        ]
    );
}

#[test]
fn an_event_that_is_not_a_size_reaches_no_callback() {
    let got = Rc::new(RefCell::new(Vec::new()));
    let (_element, callback_id) = element_with_on_resize(&got);

    for value in [numbers(&[1.0]), numbers(&[1.0, 2.0, 3.0]), JsJson::Null] {
        assert_eq!(deliver(callback_id, value), JsJson::Null);
    }

    assert!(got.borrow().is_empty());
}
