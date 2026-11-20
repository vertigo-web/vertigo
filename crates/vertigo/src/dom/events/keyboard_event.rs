/// A key pressed or let go: what `on_key_down`, `on_key_up`, `hook_key_down` and `hook_key_up`
/// get.
///
/// A key held down repeats its press, which [`repeat`](Self::repeat) tells from the first one,
/// and comes up once. A game which acts once a press, and goes on moving while a key is held,
/// keeps the keys held:
///
/// ```rust
/// use std::collections::BTreeSet;
/// use vertigo::{KeyboardEvent, Value, bind, dom};
///
/// let moves = Value::new(0);
/// let held = Value::new(BTreeSet::<String>::new());
///
/// let down = bind!(moves, held, |event: KeyboardEvent| {
///     if event.code == "ArrowLeft" && !event.repeat {
///         moves.change(|moves| *moves += 1);
///     }
///     held.change(|held| {
///         held.insert(event.code);
///     });
///     true
/// });
/// let up = bind!(held, |event: KeyboardEvent| {
///     held.change(|held| {
///         held.remove(&event.code);
///     });
///     true
/// });
///
/// dom! {
///     <div hook_key_down={down} hook_key_up={up}>
///         "Moved " {moves} " times"
///     </div>
/// };
/// ```
///
/// `Default` gives an event of no key, f. ex. to start from in a test of a handler.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct KeyboardEvent {
    /// What the key gives, by the keyboard's layout and the modifiers held, f. ex. `"a"`, `"A"`
    /// or `"ArrowLeft"`.
    pub key: String,
    /// Which key it is wherever it is on the keyboard, whatever its layout, f. ex. `"KeyA"` or
    /// `"ArrowLeft"`: what a game reads, as W, A, S and D are in the same place on every keyboard.
    pub code: String,
    pub alt_key: bool,
    pub ctrl_key: bool,
    pub shift_key: bool,
    pub meta_key: bool,
    /// A press repeated by the key held down, not the first one. Never in a key-up.
    pub repeat: bool,
}

/// The older name of [`KeyboardEvent`], from when only the presses of keys were told.
pub type KeyDownEvent = KeyboardEvent;

impl std::fmt::Display for KeyboardEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "KeyboardEvent={}", self.key)
    }
}
