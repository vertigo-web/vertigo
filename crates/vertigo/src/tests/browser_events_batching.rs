//! An event from the browser reaches the DOM as one batch, however many values the app sets in
//! it, without the app opening a transaction of its own.
//!
//! Events come through `command_from_js`, which runs each in one transaction. Before, each
//! `Value::set` in a handler closed an outermost transaction of its own, so every one of them
//! fired the flush hook and went to the browser as a batch of its own.

use std::{cell::RefCell, rc::Rc};

use crate::{self as vertigo, dom};
use crate::{
    JsJson, JsJsonSerialize, Value,
    dev::command::{CommandForWasm, DriverDomCommand},
    driver_module::{
        api::{api_callbacks, api_command_wasm},
        driver::get_driver,
        get_driver_dom,
    },
    exports::mount,
    reactive::on_after_transaction,
    transaction,
};

/// Three values on a mounted page.
fn mounted_labels() -> [Value<String>; 3] {
    let labels = ["a", "b", "c"].map(|label| Value::new(label.to_string()));
    let [a, b, c] = labels.clone();
    mount(move || {
        dom! {
            <html>
                <head />
                <body>
                    <p>{a}</p>
                    <p>{b}</p>
                    <p>{c}</p>
                </body>
            </html>
        }
    });
    labels
}

/// The DOM batches the browser gets while `run` goes - the command stream cut where the flush
/// hook fires, as in `mount_batching`.
fn batches_during(run: impl FnOnce()) -> Vec<Vec<DriverDomCommand>> {
    let pending: Rc<RefCell<Vec<DriverDomCommand>>> = Rc::new(RefCell::new(Vec::new()));
    let batches: Rc<RefCell<Vec<Vec<DriverDomCommand>>>> = Rc::new(RefCell::new(Vec::new()));

    let _tee = get_driver_dom().inspect_command({
        let pending = pending.clone();
        move |command| pending.borrow_mut().push(command)
    });

    let _cut = on_after_transaction({
        let pending = pending.clone();
        let batches = batches.clone();
        move || {
            let batch = pending.borrow_mut().drain(..).collect::<Vec<_>>();
            if !batch.is_empty() {
                batches.borrow_mut().push(batch);
            }
        }
    });

    run();

    let tail = pending.borrow_mut().drain(..).collect::<Vec<_>>();
    if !tail.is_empty() {
        batches.borrow_mut().push(tail);
    }

    batches.borrow().clone()
}

/// A DOM event, as the browser delivers it.
fn dispatch_event(handler: impl Fn(JsJson) -> JsJson + 'static) {
    let (callback_id, _registration) = api_callbacks().register(handler);
    let event = CommandForWasm::CallbackCall {
        callback_id,
        value: JsJson::Null,
    };
    api_command_wasm().command_from_js(event.to_json());
}

/// Mounts three values on a page, delivers one DOM event whose handler gets them, and returns
/// the DOM batches the browser got for the event.
fn batches_for_event(
    handler: impl Fn(&[Value<String>; 3]) + 'static,
) -> Vec<Vec<DriverDomCommand>> {
    let labels = mounted_labels();

    let batches = batches_during(|| {
        dispatch_event(move |_| {
            handler(&labels);
            JsJson::Null
        })
    });

    // Release the tree while the stores it drops into are still alive - see
    // `Driver::take_root` - and before an assertion can fail and leave it to thread teardown.
    drop(get_driver().take_root());
    batches
}

fn text_updates(batch: &[DriverDomCommand]) -> usize {
    batch
        .iter()
        .filter(|command| matches!(command, DriverDomCommand::UpdateText { .. }))
        .count()
}

#[test]
fn an_event_setting_several_values_reaches_the_dom_as_one_batch() {
    let batches = batches_for_event(|labels| {
        for (i, label) in labels.iter().enumerate() {
            label.set(format!("new {i}"));
        }
    });

    assert_eq!(batches.len(), 1, "one event, one batch: {batches:?}");
    assert_eq!(text_updates(&batches[0]), 3);
}

#[test]
fn a_transaction_of_the_apps_own_nests_in_the_events() {
    let batches = batches_for_event(|labels| {
        transaction(|_| {
            labels[0].set("x".to_string());
            labels[1].set("y".to_string());
        });
        labels[2].set("z".to_string());
    });

    assert_eq!(batches.len(), 1, "{batches:?}");
    assert_eq!(text_updates(&batches[0]), 3);
}

#[test]
fn an_event_that_changes_nothing_sends_nothing() {
    let batches = batches_for_event(|_| {});

    assert!(batches.is_empty(), "{batches:?}");
}

/// The callbacks of a frame run in one transaction too, however many there are: each draws its
/// own part of the page, and the browser gets the frame as one batch.
#[test]
fn the_callbacks_of_an_animation_frame_reach_the_dom_as_one_batch() {
    let labels = mounted_labels();

    let _callbacks = labels
        .clone()
        .map(|label| get_driver().on_animation_frame(move |time| label.set(format!("{time}"))));

    let batches = batches_during(|| {
        let frame = CommandForWasm::AnimationFrame { time: 16.0 };
        api_command_wasm().command_from_js(frame.to_json());
    });

    drop(get_driver().take_root());

    assert_eq!(batches.len(), 1, "one frame, one batch: {batches:?}");
    assert_eq!(text_updates(&batches[0]), 3);
}
