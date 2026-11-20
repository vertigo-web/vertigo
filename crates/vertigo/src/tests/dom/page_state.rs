//! The hooks of a state of the page, either on or off: what the driver sends, and what the app
//! gets.

use std::{cell::RefCell, rc::Rc};

use crate::{self as vertigo, dom};
use crate::{
    DomNode, JsJson, JsJsonSerialize,
    dev::{
        CallbackId,
        command::{CommandForWasm, DriverDomCommand},
        inspect::{log_start, log_take},
    },
    driver_module::api::api_command_wasm,
};

type Got = Rc<RefCell<Vec<(&'static str, bool)>>>;

/// An element with the hooks of the page's state, each keeping what it gets in `got`, and the
/// ids of the callbacks the driver calls, by their wire names.
fn element_with_state_hooks(got: &Got) -> (DomNode, Vec<(String, CallbackId)>) {
    let hook = |name: &'static str| {
        let got = got.clone();
        move |state: bool| got.borrow_mut().push((name, state))
    };

    log_start();
    let element = dom! {
        <div
            hook_window_focus={hook("focus")}
            hook_visibility={hook("visibility")}
            hook_pointer_lock={hook("pointer lock")}
            hook_fullscreen={hook("fullscreen")}
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

fn deliver(callback_id: CallbackId, value: JsJson) {
    let command = CommandForWasm::CallbackCall { callback_id, value };
    api_command_wasm().command_from_js(command.to_json());
}

#[test]
fn the_hooks_tell_the_state_the_driver_sends() {
    let got = Rc::new(RefCell::new(Vec::new()));
    let (_element, callbacks) = element_with_state_hooks(&got);

    deliver(callback(&callbacks, "hook_window_focus"), JsJson::False);
    deliver(callback(&callbacks, "hook_window_focus"), JsJson::True);
    deliver(callback(&callbacks, "hook_visibility"), JsJson::False);
    deliver(callback(&callbacks, "hook_pointerlock"), JsJson::True);
    deliver(callback(&callbacks, "hook_fullscreen"), JsJson::True);

    assert_eq!(
        *got.borrow(),
        [
            ("focus", false),
            ("focus", true),
            ("visibility", false),
            ("pointer lock", true),
            ("fullscreen", true)
        ]
    );
}

#[test]
fn a_state_the_hook_cant_read_is_not_told() {
    let got = Rc::new(RefCell::new(Vec::new()));
    let (_element, callbacks) = element_with_state_hooks(&got);

    deliver(
        callback(&callbacks, "hook_visibility"),
        JsJson::String("hidden".to_string()),
    );

    assert!(got.borrow().is_empty());
}
