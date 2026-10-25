use crate::{JsJson, dev::JsJsonListDecoder};

/// What happened to a `<video>` or an `<audio>`: the media events the driver listens to for
/// `on_media`, under their DOM names.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MediaEventKind {
    /// `loadedmetadata`: the length and the size of the picture are known.
    LoadedMetadata,
    /// `durationchange`
    DurationChange,
    /// `timeupdate`: the position moved, a few times a second while it plays, and after
    /// a jump.
    TimeUpdate,
    /// `progress`: more of it came over the network.
    Progress,
    /// `play`: asked to play, by the page or by the browser's own controls.
    Play,
    /// `playing`: plays, also after waiting for data.
    Playing,
    /// `pause`
    Pause,
    /// `waiting`: stopped for data it doesn't have yet.
    Waiting,
    /// `seeked`: a jump to another position is done.
    Seeked,
    /// `ended`
    Ended,
    /// `volumechange`: the volume, or muting.
    VolumeChange,
    /// `error`: it can't be played, f. ex. the file isn't there anymore.
    Error,
}

impl MediaEventKind {
    /// All of them, by their DOM names. The list MUST match `MEDIA_EVENTS` in
    /// `callbackManager.ts`.
    const ALL: [(&'static str, Self); 12] = [
        ("loadedmetadata", Self::LoadedMetadata),
        ("durationchange", Self::DurationChange),
        ("timeupdate", Self::TimeUpdate),
        ("progress", Self::Progress),
        ("play", Self::Play),
        ("playing", Self::Playing),
        ("pause", Self::Pause),
        ("waiting", Self::Waiting),
        ("seeked", Self::Seeked),
        ("ended", Self::Ended),
        ("volumechange", Self::VolumeChange),
        ("error", Self::Error),
    ];

    fn from_name(name: &str) -> Option<Self> {
        Self::ALL
            .iter()
            .find(|(known, _)| *known == name)
            .map(|(_, kind)| *kind)
    }
}

/// Structure passed as a parameter to callback on on_media event: what happened to
/// a `<video>` or an `<audio>`, and its state right after.
///
/// Every event comes with the whole state, so a player can keep it in one value and draw its
/// controls from it. It changes the media through the element's [`DomElementRef`]:
///
/// ```rust
/// use vertigo::{MediaEvent, Value, bind, dom, dom_element, transaction};
///
/// let playing = Value::new(false);
///
/// let video = dom_element! {
///     <video src="/film.mp4" preload="metadata" on_media={bind!(playing, |event: MediaEvent| {
///         playing.set(!event.paused);
///     })} />
/// };
/// let video_ref = video.get_ref();
///
/// let toggle = bind!(playing, video_ref, |_| {
///     let method = if transaction(|ctx| playing.get(ctx)) { "pause" } else { "play" };
///     video_ref.dom_access().call(method, vec![]).fetch();
/// });
/// let label = playing.map(|playing| if playing { "Pause" } else { "Play" });
///
/// dom! {
///     <div>
///         {video}
///         <button on_click={toggle}>{label}</button>
///     </div>
/// };
/// ```
///
/// [`DomElementRef`]: crate::DomElementRef
#[derive(Clone, Debug, PartialEq)]
pub struct MediaEvent {
    pub kind: MediaEventKind,
    /// The position, in seconds.
    pub current_time: f64,
    /// The length, in seconds. `None` until the metadata comes, and for a stream with no end.
    pub duration: Option<f64>,
    pub paused: bool,
    pub ended: bool,
    pub muted: bool,
    /// From 0 to 1.
    pub volume: f64,
    /// Up to where it's in the browser from the current position on, in seconds - what plays
    /// without waiting for the network. The current position, when nothing ahead is there.
    pub buffered: f64,
}

impl MediaEvent {
    /// The event as the driver sends it. The order MUST match `media` in `callbackManager.ts`.
    pub(crate) fn from_driver(data: JsJson) -> Result<Self, String> {
        data.map_list(|mut params: JsJsonListDecoder| {
            let name = params.get_string("type")?;
            let kind = MediaEventKind::from_name(&name)
                .ok_or_else(|| format!("type -> unknown media event {name}"))?;
            let current_time = params.get_f64("currentTime")?;
            // The driver sends `null` for `NaN` and `Infinity`
            let duration = match params.get_any("duration")? {
                JsJson::Number(duration) => Some(duration.as_f64()),
                JsJson::Null => None,
                other => {
                    return Err(format!(
                        "duration -> number expected, received {}",
                        other.typename()
                    ));
                }
            };
            let paused = params.get_bool("paused")?;
            let ended = params.get_bool("ended")?;
            let muted = params.get_bool("muted")?;
            let volume = params.get_f64("volume")?;
            let buffered = params.get_f64("buffered")?;
            params.expect_no_more()?;

            Ok(Self {
                kind,
                current_time,
                duration,
                paused,
                ended,
                muted,
                volume,
                buffered,
            })
        })
    }
}

impl std::fmt::Display for MediaEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MediaEvent={:?}", self.kind)
    }
}
