//! The media events: one listener for all of them, and what the driver sends.

use std::{cell::RefCell, rc::Rc};

use crate::{self as vertigo, dom};
use crate::{
    DomNode, JsJson, JsJsonNumber, JsJsonSerialize, MediaEvent, MediaEventKind,
    dev::{
        CallbackId,
        command::{CommandForWasm, DriverDomCommand},
        inspect::{DomDebugFragment, log_start, log_take},
    },
    driver_module::api::api_command_wasm,
};

type Got = Rc<RefCell<Vec<MediaEvent>>>;

/// A video keeping the events it gets in `got`, and the id of the callback the driver calls.
fn video_with_media_events(got: &Got) -> (DomNode, CallbackId) {
    let got = got.clone();

    log_start();
    let element = dom! {
        <video src="/film.mp4" on_media={move |event: MediaEvent| got.borrow_mut().push(event)} />
    };
    let commands = log_take();

    let added = commands
        .into_iter()
        .filter_map(|command| match command {
            DriverDomCommand::CallbackAdd {
                event_name,
                callback_id,
                ..
            } => Some((event_name, callback_id)),
            _ => None,
        })
        .collect::<Vec<_>>();
    let [(event_name, callback_id)] = added.as_slice() else {
        panic!("one listener expected, got {added:?}");
    };
    assert_eq!(event_name, "media");
    (element, *callback_id)
}

fn number(value: f64) -> JsJson {
    JsJson::Number(JsJsonNumber(value))
}

/// What the driver sends: the event's name, then the state of the media after it.
fn state(name: &str, duration: JsJson) -> JsJson {
    JsJson::List(vec![
        JsJson::String(name.to_string()),
        number(12.5),
        duration,
        JsJson::False,
        JsJson::False,
        JsJson::True,
        number(0.8),
        number(20.0),
    ])
}

fn deliver(callback_id: CallbackId, value: JsJson) {
    let command = CommandForWasm::CallbackCall { callback_id, value };
    api_command_wasm().command_from_js(command.to_json());
}

#[test]
fn one_listener_for_all_the_media_events() {
    log_start();
    let _element = dom! {
        <audio src="/audycja.mp3" on_media={|_: MediaEvent| ()} />
    };
    let html = DomDebugFragment::from_log().to_pseudo_html();

    assert_eq!(html, "<audio src='/audycja.mp3' media=1 />");
}

#[test]
fn the_callback_gets_what_happened_and_the_state_after() {
    let got = Rc::new(RefCell::new(Vec::new()));
    let (_video, callback_id) = video_with_media_events(&got);

    deliver(callback_id, state("timeupdate", number(60.0)));

    assert_eq!(
        got.borrow().as_slice(),
        [MediaEvent {
            kind: MediaEventKind::TimeUpdate,
            current_time: 12.5,
            duration: Some(60.0),
            paused: false,
            ended: false,
            muted: true,
            volume: 0.8,
            buffered: 20.0,
        }]
    );
}

#[test]
fn each_kind_by_its_dom_name() {
    let got = Rc::new(RefCell::new(Vec::new()));
    let (_video, callback_id) = video_with_media_events(&got);
    let kinds = [
        ("loadedmetadata", MediaEventKind::LoadedMetadata),
        ("durationchange", MediaEventKind::DurationChange),
        ("timeupdate", MediaEventKind::TimeUpdate),
        ("progress", MediaEventKind::Progress),
        ("play", MediaEventKind::Play),
        ("playing", MediaEventKind::Playing),
        ("pause", MediaEventKind::Pause),
        ("waiting", MediaEventKind::Waiting),
        ("seeked", MediaEventKind::Seeked),
        ("ended", MediaEventKind::Ended),
        ("volumechange", MediaEventKind::VolumeChange),
        ("error", MediaEventKind::Error),
    ];

    for (name, _) in kinds {
        deliver(callback_id, state(name, JsJson::Null));
    }

    let got = got
        .borrow()
        .iter()
        .map(|event| event.kind)
        .collect::<Vec<_>>();
    assert_eq!(got, kinds.map(|(_, kind)| kind));
}

#[test]
fn no_length_while_unknown() {
    let got = Rc::new(RefCell::new(Vec::new()));
    let (_video, callback_id) = video_with_media_events(&got);

    deliver(callback_id, state("loadedmetadata", JsJson::Null));

    assert_eq!(got.borrow()[0].duration, None);
}

#[test]
fn an_unknown_event_goes_nowhere() {
    let got = Rc::new(RefCell::new(Vec::new()));
    let (_video, callback_id) = video_with_media_events(&got);

    deliver(callback_id, state("suspend", JsJson::Null));

    assert!(got.borrow().is_empty());
}
