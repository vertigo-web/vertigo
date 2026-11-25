use std::sync::Arc;

use vertigo_macro::AutoJsJson;

use crate::{
    DomId, JsJson, JsJsonContext, JsJsonDeserialize,
    dev::{SsrFetchRequest, SsrFetchResponse, callback_id::CallbackId},
    driver_module::StaticString,
};

pub fn decode_json<T: JsJsonDeserialize>(json: JsJson) -> Result<T, JsJsonContext> {
    T::from_json(JsJsonContext::new(""), json)
}

#[derive(AutoJsJson, Debug)]
pub enum CommandForBrowser {
    FetchCacheGet,
    FetchExec {
        request: SsrFetchRequest,
        callback: CallbackId,
    },
    SetStatus {
        status: u16,
    },
    IsBrowser,
    GetDateNow,

    WebsocketRegister {
        host: String,
        callback: CallbackId,
        /// Milliseconds an attempt may take to connect.
        connect_timeout: u32,
    },
    WebsocketUnregister {
        callback: CallbackId,
    },
    /// Sends `message` as JSON in a text frame, or a [JsJson::Vec] as a binary frame.
    WebsocketSendMessage {
        callback: CallbackId,
        message: JsJson,
    },

    TimerSet {
        callback: CallbackId,
        duration: u32,
        kind: TimerKind,
    },
    TimerClear {
        callback: CallbackId,
    },

    /// From now on, before every frame the browser draws, send it through the export
    /// `vertigo_export_animation_frame` - until [`CommandForBrowser::AnimationFrameStop`].
    AnimationFrameStart,
    AnimationFrameStop,

    LocationCallback {
        target: LocationTarget,
        mode: LocationCallbackMode,
        callback: CallbackId,
    },
    LocationSet {
        target: LocationTarget,
        mode: LocationSetMode,
        value: String,
    },
    LocationGet {
        target: LocationTarget,
    },

    CookieGet {
        name: String,
    },
    CookieSet {
        name: String,
        value: String,
        expires_in: u64,
    },
    CookieJsonGet {
        name: String,
    },
    CookieJsonSet {
        name: String,
        value: JsJson,
        expires_in: u64,
    },

    LocalStorageGet {
        name: String,
    },
    LocalStorageSet {
        name: String,
        value: String,
    },
    LocalStorageRemove {
        name: String,
    },

    GetEnv {
        name: String,
    },

    Log {
        kind: ConsoleLogLevel,
        message: String,
        arg2: String,
        arg3: String,
        arg4: String,
    },

    TimezoneOffset,
    /// The languages the user prefers; see [`crate::Driver::languages`].
    Languages,
    HistoryBack,
    GetRandom {
        min: u32,
        max: u32,
    },

    /// Load the image at `url` for drawings on canvases, see [`crate::Driver::preload_image`].
    ImagePreload {
        url: String,
    },

    /// Lock the pointer to the element `dom_id`, see
    /// [`crate::DomElementRef::request_pointer_lock`], or let it go with `None`, see
    /// [`crate::Driver::exit_pointer_lock`].
    PointerLock {
        dom_id: Option<u64>,
    },

    /// Show the element `dom_id` on the whole screen, see
    /// [`crate::DomElementRef::request_fullscreen`], or leave the whole screen with `None`, see
    /// [`crate::Driver::exit_fullscreen`].
    Fullscreen {
        dom_id: Option<u64>,
    },

    /// Remember the element with the focus as `id`, see [`crate::Driver::remember_focus`].
    FocusRemember {
        id: u64,
    },

    /// Give the focus back to the element remembered as `id`, if it got lost meanwhile, see
    /// [`crate::Driver::remember_focus`].
    FocusReturn {
        id: u64,
    },

    /// A call of [`crate::audio`]: an [`AudioOp`], as the JSON it is serialized to where sound
    /// is asked for. Were it an `AudioOp` here, serializing this enum would link the code
    /// serializing every operation of sound into every app, also one without sound - about
    /// 2 KB of wasm. The same goes for the two below.
    Audio {
        op: JsJson,
    },

    /// A call of [`crate::image`]: an [`ImageOp`] as JSON, see [`CommandForBrowser::Audio`].
    Image {
        op: JsJson,
    },

    /// Execute JavaScript API calls (for js! macro): a list of [`JsApiCommand`] as JSON, see
    /// [`CommandForBrowser::Audio`].
    JsApiCall {
        commands: JsJson,
    },

    /// A batch of DOM commands, flat-encoded. The browser and vertigo-cli get it through the
    /// `dom_bulk_update` import, not `dom_access`, but vertigo-cli handles it as this command.
    ///
    /// Not a `Vec<DriverDomCommand>`: as `JsJson` that becomes two `BTreeMap`s and three
    /// heap `String`s per command, which dominated the cost of a large update. Encode with
    /// [`encode_dom_commands`](crate::dev::command_wire::encode_dom_commands) and read it
    /// back with [`decode_dom_commands`](crate::dev::command_wire::decode_dom_commands).
    DomBulkUpdate {
        commands: Vec<u8>,
    },
}

#[derive(AutoJsJson, Debug, Clone)]
pub enum JsApiCommand {
    Root { name: String },
    RootElement { dom_id: u64 },
    Get { property: String },
    Set { property: String, value: JsJson },
    Call { method: String, args: Vec<JsJson> },
}

#[derive(AutoJsJson, Debug, PartialEq)]
pub enum ConsoleLogLevel {
    Debug,
    Info,
    Log,
    Warn,
    Error,
}

#[derive(AutoJsJson, Debug, Clone, Copy, PartialEq)]
pub enum LocationTarget {
    Hash,
    History,
}

#[derive(AutoJsJson, Debug, Clone, Copy)]
pub enum LocationCallbackMode {
    Add,
    Remove,
}

#[derive(AutoJsJson, Debug, Clone, Copy)]
pub enum LocationSetMode {
    Push,
    Replace,
}

/// A call of [`crate::image`], which `image.ts` in the browser does.
#[derive(AutoJsJson, Clone, Debug, PartialEq)]
pub enum ImageOp {
    /// Read the picture in `data` (its file, as bytes), scale it down to fit `max_width` x
    /// `max_height`, encode it as `mime` and tell an [`ImageAnswer`] through `callback`.
    Resize {
        callback: CallbackId,
        data: JsJson,
        max_width: u32,
        max_height: u32,
        mime: String,
        quality: f32,
        /// A CSS color under the picture, for a format without transparency.
        background: String,
        /// Answer with no data, and encode nothing, when the picture fits as it is.
        keep_fitting: bool,
    },
}

/// What the browser tells of an [`ImageOp`].
#[derive(AutoJsJson, Clone, Debug, PartialEq)]
pub enum ImageAnswer {
    /// The picture made, in `data` as bytes - `null` when it was kept as it is. Its size, and
    /// that of the picture given.
    Resized {
        width: u32,
        height: u32,
        original_width: u32,
        original_height: u32,
        data: JsJson,
    },
    /// What the browser said, f. ex. that it can't read the picture.
    Failed { message: String },
}

/// A call of [`crate::audio`], which `audio.ts` in the browser does, in the order they come.
#[derive(AutoJsJson, Clone, Debug, PartialEq)]
pub enum AudioOp {
    /// Tell the events of sound through `callback`: `[0]` once the browser lets sound through,
    /// `[1, id]` when the music `id` played to its end. Always the first call.
    Listen {
        callback: CallbackId,
    },
    /// Fetch the sound at `url` and decode it, to play as `id`.
    LoadSound {
        id: u32,
        url: String,
    },
    /// Play the sound `id` once, over whatever plays, `volume` times as loud as it is.
    PlaySound {
        id: u32,
        volume: f32,
    },
    /// Take the music at `url`, to play as `id`, streamed as it plays.
    LoadMusic {
        id: u32,
        url: String,
    },
    PlayMusic {
        id: u32,
    },
    PauseMusic {
        id: u32,
    },
    /// Pause the music `id` and go back to its start.
    StopMusic {
        id: u32,
    },
    LoopMusic {
        id: u32,
        looping: bool,
    },
    MusicVolume {
        id: u32,
        volume: f32,
    },
    /// The volume of all sound.
    Volume {
        volume: f32,
    },
    Muted {
        muted: bool,
    },
    /// Let go of the sound or the music `id`, which stops.
    Forget {
        id: u32,
    },
}

#[derive(AutoJsJson, Debug)]
pub enum TimerKind {
    Timeout,
    Interval,
}

pub mod browser_response {
    use vertigo_macro::AutoJsJson;

    use crate::{InstantType, JsJson};

    #[derive(AutoJsJson)]
    pub struct FetchCacheGet {
        pub data: Option<String>,
    }

    #[derive(AutoJsJson)]
    pub struct IsBrowser {
        pub value: bool,
    }

    #[derive(AutoJsJson)]
    pub struct GetDateNow {
        pub value: InstantType,
    }

    #[derive(AutoJsJson)]
    pub struct LocationGet {
        pub value: String,
    }

    #[derive(AutoJsJson)]
    pub struct CookieGet {
        pub value: String,
    }

    #[derive(AutoJsJson)]
    pub struct CookieJsonGet {
        pub value: JsJson,
    }

    #[derive(AutoJsJson)]
    pub struct LocalStorageGet {
        pub value: Option<String>,
    }

    #[derive(AutoJsJson)]
    pub struct GetEnv {
        pub value: Option<String>,
    }

    #[derive(AutoJsJson)]
    pub struct TimezoneOffset {
        pub value: i32,
    }

    #[derive(AutoJsJson)]
    pub struct Languages {
        pub value: Vec<String>,
    }

    #[derive(AutoJsJson)]
    pub struct GetRandom {
        pub value: u32,
    }
}

#[derive(AutoJsJson, Debug)]
pub enum WebsocketMessageFromBrowser {
    Connected,
    /// A text frame parsed as JSON, or a binary frame as [JsJson::Vec].
    Message {
        message: JsJson,
    },
    /// A connection closed (`connected`), or an attempt to connect failed.
    Disconnected {
        code: u16,
        reason: String,
        connected: bool,
    },
}

#[derive(AutoJsJson, Debug)]
pub enum CommandForWasm {
    FetchExecResponse {
        response: SsrFetchResponse,
        callback: CallbackId,
    },

    /// A [`WebsocketMessageFromBrowser`] as JSON, read where the app's websockets are: see
    /// [`CommandForBrowser::Audio`].
    Websocket {
        callback: CallbackId,
        message: JsJson,
    },

    TimerCall {
        callback: CallbackId,
    },

    LocationCall {
        callback: CallbackId,
        value: String,
    },

    CallbackCall {
        callback_id: CallbackId,
        value: JsJson,
    },
}

#[derive(AutoJsJson, Clone, Debug)]
pub enum DriverDomCommand {
    CreateNode {
        id: DomId,
        name: StaticString,
    },
    CreateText {
        id: DomId,
        value: String,
    },
    UpdateText {
        id: DomId,
        value: String,
    },
    SetAttr {
        id: DomId,
        name: StaticString,
        value: String,
    },
    RemoveAttr {
        id: DomId,
        name: StaticString,
    },
    RemoveNode {
        id: DomId,
    },
    RemoveText {
        id: DomId,
    },
    InsertBefore {
        parent: DomId,
        child: DomId,
        ref_id: Option<DomId>,
    },
    InsertCss {
        selector: Option<String>,
        value: String,
    },

    CreateComment {
        id: DomId,
        value: String,
    },
    RemoveComment {
        id: DomId,
    },
    CallbackAdd {
        id: DomId,
        event_name: String,
        callback_id: CallbackId,
    },
    CallbackRemove {
        id: DomId,
        event_name: String,
        callback_id: CallbackId,
    },
    /// A new picture for a `<canvas>`: the bytes of a [`Drawing`](crate::Drawing), shared with
    /// it rather than copied.
    CanvasDraw {
        id: DomId,
        drawing: Arc<Vec<u8>>,
    },
    /// Calls of the WebGL context of a `<canvas>`, in the format of
    /// [`webgl`](crate::webgl#format).
    WebGl {
        id: DomId,
        commands: Vec<u8>,
    },
    /// Calls of the WebGPU context of a `<canvas>`, in the format of
    /// [`webgpu`](crate::webgpu#format).
    WebGpu {
        id: DomId,
        commands: Vec<u8>,
    },
}

impl DriverDomCommand {
    pub fn is_event(&self) -> bool {
        matches!(
            self,
            Self::RemoveNode { .. } | Self::RemoveText { .. } | Self::RemoveComment { .. }
        )
    }
}
