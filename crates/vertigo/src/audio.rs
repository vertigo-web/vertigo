//! Sound with Web Audio: short sounds played over each other ([`Sound`]), music ([`Music`]), and
//! the volume of all of it ([`set_volume`], [`set_muted`]).
//!
//! A browser lets a page make sound only once the user has clicked, pressed a key or touched it.
//! Until then the page is silent: a sound played isn't heard, and music asked to play starts with
//! the first click or key. [`allowed`] tells when sound goes through.
//!
//! A sound is fetched and decoded once, when it's made, so it plays at once when asked, as often
//! as asked, over itself and anything else. One played before it came isn't heard. Music streams
//! as it plays, from an `<audio>` the browser keeps, so a long piece takes no memory. Both come
//! from the page's own server, or one which lets the page read them (CORS). The browser's code
//! for it is a chunk of its own, loaded when a page first makes a sound or music.
//!
//! ```rust
//! use vertigo::{audio::{Music, Sound}, bind, dom};
//!
//! let click = Sound::new("/sounds/click.ogg");
//! let theme = Music::new("/music/menu.ogg");
//! theme.set_looping(true);
//! theme.set_volume(0.5);
//! // At once, or with the first click or key when the browser waits for one
//! theme.play();
//!
//! let start = bind!(click, theme, |_| {
//!     click.play();
//!     theme.stop();
//! });
//!
//! dom! {
//!     <button on_click={start}>"Start"</button>
//! };
//! ```

use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
    rc::{Rc, Weak},
};

use crate::{
    Computed, DropResource, JsJson, Value,
    dev::command::AudioOp,
    driver_module::{
        api::{api_browser_command, api_callbacks},
        event_emitter::EventEmitter,
    },
};

/// What the browser tells, see [`AudioOp::Listen`].
mod event {
    /// Sound goes through.
    pub const ALLOWED: f64 = 0.0;
    /// A music played to its end, by its id.
    pub const ENDED: f64 = 1.0;
}

struct Audio {
    allowed: Value<bool>,
    /// The musics, by their ids, to tell them of their ends.
    musics: RefCell<HashMap<u32, Weak<MusicState>>>,
    last_id: Cell<u32>,
    /// The callback the browser tells through, for as long as the page is.
    _callback: DropResource,
}

thread_local! {
    /// Made with the first sound, music or call, which so tells the browser where to tell its
    /// events before anything else.
    static AUDIO: Audio = Audio::new();
}

impl Audio {
    fn new() -> Self {
        let (callback, drop) = api_callbacks().register(|data| {
            told(data);
            JsJson::Null
        });
        api_browser_command().audio(AudioOp::Listen { callback });

        Self {
            allowed: Value::new(false),
            musics: RefCell::new(HashMap::new()),
            last_id: Cell::new(0),
            _callback: drop,
        }
    }
}

fn told(data: JsJson) {
    let JsJson::List(event) = &data else {
        log::error!("audio: an event of the browser not a list: {data:?}");
        return;
    };

    match event.as_slice() {
        [JsJson::Number(kind)] if kind.0 == event::ALLOWED => {
            AUDIO.with(|audio| audio.allowed.set(true));
        }
        [JsJson::Number(kind), JsJson::Number(id)] if kind.0 == event::ENDED => {
            let id = id.0 as u32;
            let music = AUDIO.with(|audio| audio.musics.borrow().get(&id).and_then(Weak::upgrade));
            if let Some(music) = music {
                music.ended.trigger(&());
            }
        }
        _ => log::error!("audio: an unknown event of the browser: {data:?}"),
    }
}

/// A call to the browser, after the one telling it where to tell its events.
fn send(op: AudioOp) {
    AUDIO.with(|_| ());
    api_browser_command().audio(op);
}

fn next_id() -> u32 {
    AUDIO.with(|audio| {
        let id = audio.last_id.get() + 1;
        audio.last_id.set(id);
        id
    })
}

/// A volume as the browser takes it: no less than silence.
fn volume(volume: f32) -> f32 {
    volume.max(0.0)
}

/// A sound or a music in the browser, let go when its last handle is dropped.
struct Object {
    id: u32,
}

impl Drop for Object {
    fn drop(&mut self) {
        // Not when the page goes, whose thread-locals may be gone already
        if AUDIO.try_with(|_| ()).is_ok() {
            api_browser_command().audio(AudioOp::Forget { id: self.id });
        }
    }
}

/// A short sound, f. ex. of a click or of a piece landing, decoded once and played at once when
/// asked, as often as asked, over itself and anything else. Clones play the same sound, which the
/// browser lets go with the last of them.
///
/// ```rust
/// use vertigo::audio::Sound;
///
/// let landed = Sound::new("/sounds/landed.ogg");
/// landed.play();
/// landed.play_at(0.3);
/// ```
#[derive(Clone)]
pub struct Sound {
    object: Rc<Object>,
}

impl Sound {
    /// The sound at `url`, which the browser fetches and decodes now.
    pub fn new(url: impl Into<String>) -> Self {
        let id = next_id();
        send(AudioOp::LoadSound {
            id,
            url: url.into(),
        });
        Self {
            object: Rc::new(Object { id }),
        }
    }

    /// Plays it once, as loud as it is. Not heard before it came, nor before the browser lets
    /// the page make sound (see [`allowed`]).
    pub fn play(&self) {
        self.play_at(1.0);
    }

    /// Plays it once, `volume` times as loud as it is: 0.5 at half, 0 not at all.
    pub fn play_at(&self, volume: f32) {
        send(AudioOp::PlaySound {
            id: self.object.id,
            volume: self::volume(volume),
        });
    }
}

struct MusicState {
    object: Object,
    ended: EventEmitter<()>,
}

impl Drop for MusicState {
    fn drop(&mut self) {
        let id = self.object.id;
        let _ = AUDIO.try_with(|audio| audio.musics.borrow_mut().remove(&id));
    }
}

/// Music, streamed as it plays: played, paused, stopped and looped. Clones are the same music,
/// which stops when the last of them is dropped.
///
/// ```rust
/// use vertigo::{Value, audio::Music, dom_element};
///
/// let bonus = Music::new("/music/bonus.ogg");
/// let over = Value::new(false);
/// let element = dom_element! { <div /> };
/// element.append_drop_resource(bonus.on_end({
///     let over = over.clone();
///     move || over.set(true)
/// }));
/// bonus.play();
/// ```
#[derive(Clone)]
pub struct Music {
    state: Rc<MusicState>,
}

impl Music {
    /// The music at `url`, which the browser starts fetching now.
    pub fn new(url: impl Into<String>) -> Self {
        let id = next_id();
        send(AudioOp::LoadMusic {
            id,
            url: url.into(),
        });
        let state = Rc::new(MusicState {
            object: Object { id },
            ended: EventEmitter::default(),
        });
        AUDIO.with(|audio| audio.musics.borrow_mut().insert(id, Rc::downgrade(&state)));
        Self { state }
    }

    fn id(&self) -> u32 {
        self.state.object.id
    }

    /// Plays from where it is. When the browser waits for the user to let the page make sound,
    /// it starts with their first click or key (see [`allowed`]).
    pub fn play(&self) {
        send(AudioOp::PlayMusic { id: self.id() });
    }

    pub fn pause(&self) {
        send(AudioOp::PauseMusic { id: self.id() });
    }

    /// Pauses it and goes back to its start.
    pub fn stop(&self) {
        send(AudioOp::StopMusic { id: self.id() });
    }

    /// Whether it starts again at its end, instead of ending. Not at first.
    pub fn set_looping(&self, looping: bool) {
        send(AudioOp::LoopMusic {
            id: self.id(),
            looping,
        });
    }

    /// How loud it plays: 1 as loud as it is, at first, 0.5 at half, 0 not at all.
    pub fn set_volume(&self, volume: f32) {
        send(AudioOp::MusicVolume {
            id: self.id(),
            volume: self::volume(volume),
        });
    }

    /// `callback` when it played to its end, f. ex. to go back to the music before it. Never
    /// while it loops. Told for as long as the returned `DropResource` is kept.
    pub fn on_end(&self, callback: impl Fn() + 'static) -> DropResource {
        self.state.ended.add(move |()| callback())
    }
}

/// How loud all sound and music is: 1 as loud as they are, at first, 0.5 at half, 0 not at all.
/// The volumes they play at are times this.
pub fn set_volume(volume: f32) {
    send(AudioOp::Volume {
        volume: self::volume(volume),
    });
}

/// Silences all sound and music, or lets them be heard again, at the volume they had. Music
/// plays on while muted.
pub fn set_muted(muted: bool) {
    send(AudioOp::Muted { muted });
}

/// Whether the browser lets the page make sound: once the user clicked, pressed a key or touched
/// it, or at once when the browser doesn't wait for that. F. ex. to ask for a click to start.
///
/// ```rust
/// use vertigo::{audio, dom};
///
/// let hint = audio::allowed().map(|allowed| (!allowed).then(|| "Click for sound".to_string()));
///
/// dom! { <p>{hint.map(|hint| hint.unwrap_or_default())}</p> };
/// ```
pub fn allowed() -> Computed<bool> {
    AUDIO.with(|audio| audio.allowed.to_computed())
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, rc::Rc};

    use super::{Music, Sound, allowed, set_muted, set_volume};
    use crate::{
        JsJson, JsJsonNumber, JsJsonSerialize,
        dev::command::{AudioOp, CommandForWasm},
        driver_module::api::api_command_wasm,
        external_api::safe_wrappers::TEST_AUDIO,
        transaction,
    };

    /// The calls the browser got since the last time asked.
    fn calls() -> Vec<AudioOp> {
        TEST_AUDIO.with(|calls| calls.take())
    }

    /// The browser telling `event`, as it does.
    fn tell(event: &[f64]) {
        let Some(callback) = listened() else {
            panic!("the browser wasn't told where to tell");
        };
        let value = JsJson::List(
            event
                .iter()
                .map(|number| JsJson::Number(JsJsonNumber(*number)))
                .collect(),
        );
        let command = CommandForWasm::CallbackCall {
            callback_id: callback,
            value,
        };
        api_command_wasm().command_from_js(command.to_json());
    }

    thread_local! {
        static LISTENED: RefCell<Option<crate::dev::CallbackId>> = const { RefCell::new(None) };
    }

    /// Where the browser was told to tell its events, by the first call.
    fn listened() -> Option<crate::dev::CallbackId> {
        LISTENED.with(|listened| {
            if listened.borrow().is_none() {
                let first = TEST_AUDIO.with(|calls| calls.borrow().first().cloned());
                if let Some(AudioOp::Listen { callback }) = first {
                    *listened.borrow_mut() = Some(callback);
                }
            }
            *listened.borrow()
        })
    }

    #[test]
    fn the_first_call_tells_where_to_tell_and_a_sound_loads_and_plays() {
        let click = Sound::new("/click.ogg");
        listened();
        click.play();
        click.play_at(0.25);
        click.play_at(-1.0);

        let calls = calls();
        assert!(matches!(calls.first(), Some(AudioOp::Listen { .. })));
        assert_eq!(
            calls[1..],
            [
                AudioOp::LoadSound {
                    id: 1,
                    url: "/click.ogg".to_string()
                },
                AudioOp::PlaySound { id: 1, volume: 1.0 },
                AudioOp::PlaySound {
                    id: 1,
                    volume: 0.25
                },
                AudioOp::PlaySound { id: 1, volume: 0.0 },
            ]
        );
    }

    #[test]
    fn music_is_played_paused_stopped_looped_and_let_go_with_its_last_clone() {
        let music = Music::new("/theme.ogg");
        listened();
        calls();
        let clone = music.clone();
        music.set_looping(true);
        music.set_volume(0.5);
        music.play();
        music.pause();
        music.stop();
        drop(music);
        assert_eq!(
            calls(),
            [
                AudioOp::LoopMusic {
                    id: 1,
                    looping: true
                },
                AudioOp::MusicVolume { id: 1, volume: 0.5 },
                AudioOp::PlayMusic { id: 1 },
                AudioOp::PauseMusic { id: 1 },
                AudioOp::StopMusic { id: 1 },
            ]
        );

        drop(clone);
        assert_eq!(calls(), [AudioOp::Forget { id: 1 }]);
    }

    #[test]
    fn the_volume_and_muting_are_of_all_sound() {
        set_volume(0.8);
        set_muted(true);

        let calls = calls();
        assert_eq!(
            calls[1..],
            [
                AudioOp::Volume { volume: 0.8 },
                AudioOp::Muted { muted: true }
            ]
        );
    }

    #[test]
    fn sound_is_allowed_when_the_browser_tells() {
        let allowed = allowed();
        listened();
        assert!(!transaction(|context| allowed.get(context)));

        tell(&[0.0]);

        assert!(transaction(|context| allowed.get(context)));
    }

    #[test]
    fn a_music_is_told_of_its_end_while_the_resource_is_kept() {
        let first = Music::new("/first.ogg");
        let second = Music::new("/second.ogg");
        listened();
        let ended = Rc::new(RefCell::new(Vec::new()));
        let told = |name: &'static str| {
            let ended = ended.clone();
            move || ended.borrow_mut().push(name)
        };
        let first_end = first.on_end(told("first"));
        let _second_end = second.on_end(told("second"));

        tell(&[1.0, 2.0]);
        drop(first_end);
        tell(&[1.0, 1.0]);
        drop(second);
        tell(&[1.0, 2.0]);

        assert_eq!(*ended.borrow(), ["second"]);
    }
}
