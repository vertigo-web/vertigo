use std::sync::Arc;
use tokio::sync::mpsc::UnboundedSender;
use vertigo::{JsJson, dev::command::CommandForBrowser};

use crate::serve::{request_state::RequestState, timings::SsrProbe};

use super::message::Message;

/// Everything the imported host functions need, carried as the `Store` payload rather than
/// captured by the closures.
///
/// That is what lets the [`wasmtime::Linker`] be built once at startup: the import functions
/// close over nothing, reaching their per-request collaborators through `Caller::data`
/// instead. Resolving imports by name then happens a single time, into an
/// [`wasmtime::InstancePre`], instead of once per request.
pub struct HostState {
    pub request: RequestState,
    pub sender: UnboundedSender<Message>,
    pub probe: SsrProbe,
    pub handle_command: Arc<dyn Fn(RequestState, CommandForBrowser) -> JsJson + Send + Sync>,
}
