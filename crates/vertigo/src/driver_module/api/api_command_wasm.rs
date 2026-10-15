use std::rc::Rc;

use vertigo_macro::store;

use crate::{
    JsJson,
    dev::command::{CommandForWasm, decode_json},
    driver_module::{driver::get_driver, get_driver_dom},
};

use super::{api_fetch::api_fetch, api_location, api_timers, api_websocket};

#[store]
pub fn api_command_wasm() -> Rc<CommandWasmApi> {
    Rc::new(CommandWasmApi {})
}

pub struct CommandWasmApi {}

impl CommandWasmApi {
    /// Every event from the browser comes through here: a DOM event, a websocket message, a fetch
    /// response, a timer - and through the timer every step of async code - or a change of the
    /// location.
    pub fn command_from_js(self: &Rc<CommandWasmApi>, json: JsJson) -> JsJson {
        let command = decode_json::<CommandForWasm>(json)
            .inspect_err(|err| {
                log::error!("command_from_js -> decode error = {err}");
            })
            .ok();

        let Some(command) = command else {
            return JsJson::Null;
        };

        // One event, one transaction: whatever the app writes in it propagates once and reaches
        // the DOM in one batch. F. ex. a websocket message setting a value per row of a table
        // makes one DOM batch, not one per row.
        let response = get_driver().transaction(|_| match command {
            CommandForWasm::FetchExecResponse { response, callback } => {
                api_fetch().callback(callback, response);
                JsJson::Null
            }
            CommandForWasm::Websocket { callback, message } => {
                // After a close, how long the browser should wait before reconnecting.
                api_websocket().callback(callback, message)
            }
            CommandForWasm::TimerCall { callback } => {
                api_timers().callback_timeout(callback);
                JsJson::Null
            }
            CommandForWasm::LocationCall { callback, value } => {
                api_location().callback(callback, value);
                JsJson::Null
            }
            CommandForWasm::CallbackCall { callback_id, value } => {
                use crate::driver_module::api::callbacks::api_callbacks;
                api_callbacks().call(callback_id, value)
            }
        });

        // As after the mount: what `when_connect` queued once the hooks had fired would
        // otherwise wait for the next transaction. A no-op when nothing is queued.
        get_driver_dom().flush_dom_changes();

        response
    }
}
