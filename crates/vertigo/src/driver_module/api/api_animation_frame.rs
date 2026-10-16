use std::rc::Rc;

use vertigo_macro::store;

use crate::{
    DropResource,
    dev::command::CommandForBrowser,
    struct_mut::{BTreeMapMut, ValueMut},
};

use super::api_browser_command::exec_command;

#[store]
pub fn api_animation_frame() -> Rc<ApiAnimationFrame> {
    Rc::new(ApiAnimationFrame {
        frames: Rc::new(Frames {
            next_id: ValueMut::new(0),
            callbacks: BTreeMapMut::new(),
            #[cfg(test)]
            asked: ValueMut::new(Vec::new()),
        }),
    })
}

/// The app's callbacks for the frames the browser draws.
///
/// The browser is asked for frames only while there is a callback, and each frame comes as one
/// call for all of them. So they run in one transaction, and what they change reaches the DOM in
/// one batch - not one per callback.
pub struct ApiAnimationFrame {
    frames: Rc<Frames>,
}

struct Frames {
    next_id: ValueMut<u64>,
    /// By id, so in the order they came.
    callbacks: BTreeMapMut<u64, Rc<dyn Fn(f64)>>,
    /// Whether the browser was told to start (`true`) or to stop (`false`), in order.
    #[cfg(test)]
    asked: ValueMut<Vec<bool>>,
}

impl Frames {
    fn ask_for_frames(&self, frames: bool) {
        #[cfg(test)]
        self.asked.change(|asked| asked.push(frames));

        exec_command(match frames {
            true => CommandForBrowser::AnimationFrameStart,
            false => CommandForBrowser::AnimationFrameStop,
        });
    }
}

impl ApiAnimationFrame {
    pub fn on_frame(&self, callback: impl Fn(f64) + 'static) -> DropResource {
        let frames = self.frames.clone();

        let id = frames.next_id.change(|next_id| {
            *next_id += 1;
            *next_id
        });

        let first = frames.callbacks.is_empty();
        frames.callbacks.insert(id, Rc::new(callback));

        if first {
            frames.ask_for_frames(true);
        }

        DropResource::new(move || {
            let removed = frames.callbacks.remove(&id);

            if removed.is_some() && frames.callbacks.is_empty() {
                frames.ask_for_frames(false);
            }

            // Only now: the callback may own others, and the last of them to go stops the
            // frames itself.
            drop(removed);
        })
    }

    /// A frame from the browser, for the callbacks there were when it came, in the order they
    /// came. One that an earlier one dropped doesn't run, and one added waits for the next frame,
    /// as with `requestAnimationFrame` itself.
    pub fn frame(&self, time: f64) {
        let ids = self
            .frames
            .callbacks
            .map(|callbacks| callbacks.keys().copied().collect::<Vec<_>>());

        for id in ids {
            if let Some(callback) = self.frames.callbacks.get_and_clone(&id) {
                callback(time);
            }
        }
    }

    /// Whether the browser was told to start (`true`) or to stop (`false`) sending frames, in
    /// order.
    #[cfg(test)]
    pub fn asked(&self) -> Vec<bool> {
        self.frames.asked.get()
    }
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, rc::Rc};

    use crate::{
        JsJson, JsJsonNumber, JsJsonSerialize,
        dev::command::{CommandForBrowser, CommandForWasm, decode_json},
        driver_module::api::{api_animation_frame, api_command_wasm},
    };

    /// A frame, as the browser delivers it.
    fn frame(time: f64) {
        let command = CommandForWasm::AnimationFrame { time };
        api_command_wasm().command_from_js(command.to_json());
    }

    /// A callback that writes its name and the time of every frame it gets to `log`.
    fn recording(log: &Rc<RefCell<Vec<String>>>, name: &'static str) -> impl Fn(f64) + 'static {
        let log = log.clone();
        move |time| log.borrow_mut().push(format!("{name}@{time}"))
    }

    #[test]
    fn every_frame_calls_each_callback_with_its_time_in_the_order_they_came() {
        let log = Rc::new(RefCell::new(Vec::new()));
        let _a = api_animation_frame().on_frame(recording(&log, "a"));
        let _b = api_animation_frame().on_frame(recording(&log, "b"));

        frame(16.5);
        frame(33.25);

        assert_eq!(*log.borrow(), ["a@16.5", "b@16.5", "a@33.25", "b@33.25"]);
    }

    #[test]
    fn the_browser_is_asked_for_frames_only_while_there_is_a_callback() {
        let first = api_animation_frame().on_frame(|_| {});
        let second = api_animation_frame().on_frame(|_| {});
        assert_eq!(
            api_animation_frame().asked(),
            [true],
            "the first one starts the frames"
        );

        drop(first);
        assert_eq!(api_animation_frame().asked(), [true], "one is still there");

        drop(second);
        assert_eq!(
            api_animation_frame().asked(),
            [true, false],
            "the last one stops them"
        );

        let _again = api_animation_frame().on_frame(|_| {});
        assert_eq!(api_animation_frame().asked(), [true, false, true]);
    }

    #[test]
    fn a_dropped_callback_gets_no_more_frames() {
        let log = Rc::new(RefCell::new(Vec::new()));
        let _a = api_animation_frame().on_frame(recording(&log, "a"));
        let b = api_animation_frame().on_frame(recording(&log, "b"));

        frame(1.0);
        drop(b);
        frame(2.0);

        assert_eq!(*log.borrow(), ["a@1", "b@1", "a@2"]);
    }

    #[test]
    fn a_callback_dropped_by_an_earlier_one_in_the_same_frame_does_not_run() {
        let log = Rc::new(RefCell::new(Vec::new()));
        let second: Rc<RefCell<Option<crate::DropResource>>> = Rc::new(RefCell::new(None));

        let _first = api_animation_frame().on_frame({
            let log = log.clone();
            let second = second.clone();
            move |time| {
                log.borrow_mut().push(format!("first@{time}"));
                second.borrow_mut().take();
            }
        });
        *second.borrow_mut() = Some(api_animation_frame().on_frame(recording(&log, "second")));

        frame(1.0);

        assert_eq!(*log.borrow(), ["first@1"]);
    }

    #[test]
    fn a_callback_added_during_a_frame_waits_for_the_next() {
        let log = Rc::new(RefCell::new(Vec::new()));
        let added: Rc<RefCell<Option<crate::DropResource>>> = Rc::new(RefCell::new(None));

        let _adding = api_animation_frame().on_frame({
            let log = log.clone();
            let added = added.clone();
            move |_| {
                let mut added = added.borrow_mut();
                if added.is_none() {
                    *added = Some(api_animation_frame().on_frame(recording(&log, "added")));
                }
            }
        });

        frame(1.0);
        assert!(log.borrow().is_empty(), "{:?}", log.borrow());

        frame(2.0);
        assert_eq!(*log.borrow(), ["added@2"]);
    }

    #[test]
    fn a_callback_can_drop_itself() {
        let log = Rc::new(RefCell::new(Vec::new()));
        let own: Rc<RefCell<Option<crate::DropResource>>> = Rc::new(RefCell::new(None));

        *own.borrow_mut() = Some(api_animation_frame().on_frame({
            let log = log.clone();
            let own = own.clone();
            move |time| {
                log.borrow_mut().push(format!("once@{time}"));
                own.borrow_mut().take();
            }
        }));

        frame(1.0);
        frame(2.0);

        assert_eq!(*log.borrow(), ["once@1"]);
        assert_eq!(
            api_animation_frame().asked(),
            [true, false],
            "and with it the frames stop"
        );
    }

    #[test]
    fn a_callback_owning_the_last_other_one_stops_the_frames_once() {
        let inner = api_animation_frame().on_frame(|_| {});
        let outer = api_animation_frame().on_frame(move |_| {
            let _owned = &inner;
        });

        drop(outer);

        assert_eq!(api_animation_frame().asked(), [true, false]);
    }

    fn object(key: &str, value: JsJson) -> JsJson {
        JsJson::Object([(key.to_string(), value)].into_iter().collect())
    }

    /// `api.ts` reads the commands as these strings, and `animationFrames.ts` sends a frame as
    /// `{ AnimationFrame: { time } }`.
    #[test]
    fn the_commands_are_the_ones_the_driver_reads_and_sends() {
        assert_eq!(
            CommandForBrowser::AnimationFrameStart.to_json(),
            JsJson::String("AnimationFrameStart".to_string())
        );
        assert_eq!(
            CommandForBrowser::AnimationFrameStop.to_json(),
            JsJson::String("AnimationFrameStop".to_string())
        );

        let sent = object(
            "AnimationFrame",
            object("time", JsJson::Number(JsJsonNumber(16.5))),
        );
        let frame = decode_json::<CommandForWasm>(sent);
        assert!(
            matches!(frame, Ok(CommandForWasm::AnimationFrame { time }) if time == 16.5),
            "{frame:?}"
        );
    }
}
