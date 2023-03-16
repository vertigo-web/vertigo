use std::sync::Arc;
use std::collections::HashMap;
use std::hash::Hash;
use wasmtime::{
    Caller,
    Engine,
    Func,
    Instance,
    Module,
    Store,
};
use tokio::sync::mpsc::UnboundedSender;

use crate::serve::{
    html::RequestBody,
    request_state::RequestState,
    js_value::{JsValue, JsJson, from_json},
    wasm::data_context::DataContext,
};

use super::get_now;
use super::js_value_match::{Match};

#[derive(Debug)]
pub enum Message {
    TimeoutAndSendResponse,
    DomUpdate(JsJson),
    Panic(Option<String>),
    SetTimeoutZero {
        callback_id: u64
    },
    FetchRequest {
        callback_id: u64,
        request: FetchRequest,
    },
    FetchResponse {
        request: Arc<FetchRequest>,
        response: FetchResponse
    },
}

#[derive(Debug, Eq)]
pub struct FetchRequest {
    pub method: String,
    pub url: String,
    pub headers: HashMap<String, String>,
    pub body: Option<RequestBody>,
}

impl PartialEq for  FetchRequest {
    fn eq(&self, other: &Self) -> bool {
        self.method == other.method &&
        self.url == other.url &&
        self.headers == other.headers  &&
        self.body == other.body
    }
}
impl Hash for FetchRequest {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.method.hash(state);
        self.url.hash(state);
        for (key, value) in self.headers.iter() {
            key.hash(state);
            value.hash(state);
        }
        self.body.hash(state);
    }
}

fn convert_value_to_body(body: JsValue) -> Result<Option<RequestBody>, String> {
    match body {
        JsValue::Json(json) => Ok(Some(RequestBody::Json(json))),
        JsValue::String(text) => Ok(Some(RequestBody::Text(text))),
        JsValue::Vec(buffer) => Ok(Some(RequestBody::Binary(buffer))),
        JsValue::Undefined => Ok(None),
        other => {
            let typename = other.typename();
            let message = format!("expected JsValue::Json or JsValue::Text or JsValue::Binary, received JsValue::{typename}");
            Err(message)
        }
    }
}

fn convert_body_to_value(body: RequestBody) -> JsValue {
    match body {
        RequestBody::Json(json) => JsValue::Json(json),
        RequestBody::Text(text) => JsValue::String(text),
        RequestBody::Binary(buffer) => JsValue::Vec(buffer),
    }
}

#[derive(Clone, Debug)]
pub struct FetchResponse {
    pub success: bool,
    pub status: u32,
    pub body: RequestBody,
}

fn match_is_browser(arg: &JsValue) -> Result<(), ()> {
    let matcher = Match::new(arg)?;
    let matcher = matcher.test_list(&["api"])?;
    let _ = matcher.test_list(&["call", "isBrowser"])?;

    Ok(())
}

fn match_cookie_command(arg: &JsValue) -> Result<(), ()> {
    let matcher = Match::new(arg)?;
    let matcher = matcher.test_list(&["api"])?;
    let _ = matcher.test_list(&["get", "cookie"])?;

    Ok(())
}

fn match_history_router(arg: &JsValue) -> Result<(), ()> {
    let matcher = Match::new(arg)?;
    let matcher = matcher.test_list(&["api"])?;
    let matcher = matcher.test_list(&["get", "historyLocation"])?;
    let matcher = matcher.test_list(&["call", "get"])?;
    matcher.end()?;

    Ok(())
}

fn match_history_router_callback(arg: &JsValue) -> Result<(), ()> {
    let matcher = Match::new(arg)?;
    let matcher = matcher.test_list(&["api"])?;
    let matcher = matcher.test_list(&["get", "historyLocation"])?;
    let (matcher, _) = matcher.test_list_with_fn(|matcher: Match| -> Result<u64, ()> {
        let matcher = matcher.str("call")?;
        let matcher = matcher.str("add")?;
        let (matcher, callback_id) = matcher.u64()?;
        matcher.end()?;

        Ok(callback_id)
    })?;
    matcher.end()?;

    Ok(())
}

fn match_dom_bulk_update(arg: &JsValue) -> Result<JsJson, ()> {
    let matcher = Match::new(arg)?;
    let matcher = matcher.test_list(&["api"])?;
    let matcher = matcher.test_list(&["get", "dom"])?;
    let (matcher, data) = matcher.test_list_with_fn(|matcher: Match| -> Result<JsJson, ()> {
        let matcher = matcher.str("call")?;
        let matcher = matcher.str("dom_bulk_update")?;
        let (matcher, data) = matcher.json()?;
        matcher.end()?;

        Ok(data)
    })?;
    matcher.end()?;

    Ok(data)
}

fn match_log(arg: &JsValue) -> Result<(String, String), ()> {
    let matcher = Match::new(arg)?;

    let matcher = matcher.test_list(&["root", "window"])?;
    let matcher = matcher.test_list(&["get", "console"])?;
    let (matcher, (log_type, log_message)) = matcher.test_list_with_fn(|matcher: Match| -> Result<(String, String), ()> {

        let matcher = matcher.str("call")?;
        let (matcher, log_type) = matcher.string()?;
        let (matcher, log_message) = matcher.string()?;
        let (matcher, _) = matcher.string()?;
        let (matcher, _) = matcher.string()?;
        let (matcher, _) = matcher.string()?;
        matcher.end()?;

        Ok((log_type, log_message))
    })?;

    matcher.end()?;

    Ok((log_type, log_message))
}

fn match_date_now(arg: &JsValue) -> Result<JsValue, ()> {
    let matcher = Match::new(arg)?;

    let matcher = matcher.test_list(&["root", "window"])?;
    let matcher = matcher.test_list(&["get", "Date"])?;
    let matcher = matcher.test_list(&["call", "now"])?;
    matcher.end()?;

    let time = get_now().as_millis();
    Ok(JsValue::I64(time as i64))
}

fn match_websocket(arg: &JsValue) -> Result<(), ()> {
    let matcher = Match::new(arg)?;

    let matcher = matcher.test_list(&["api"])?;
    let _ = matcher.test_list(&["get", "websocket"])?;

    Ok(())
}

enum CallWebsocketResult {
    TimeoutSet {
        time: u32,
        callback_id: u64,
    },
    NoResult,
}

fn match_interval(arg: &JsValue) -> Result<CallWebsocketResult, ()> {
    let matcher = Match::new(arg)?;

    let matcher = matcher.test_list(&["api"])?;
    let matcher = matcher.test_list(&["get", "interval"])?;

    let (matcher, result) = matcher.test_list_with_fn(|matcher| {

        let matcher = matcher.str("call")?;
        if let Ok(matcher) = matcher.str("timeout_set") {
            let (matcher, time) = matcher.u32()?;
            let (_, callback_id) = matcher.u64()?;

            return Ok(CallWebsocketResult::TimeoutSet { time, callback_id });
        }

        Ok(CallWebsocketResult::NoResult)
    })?;

    matcher.end()?;

    Ok(result)
}

fn match_fetch(arg: &JsValue) -> Result<(u64, FetchRequest), ()> {
    let matcher = Match::new(arg)?;

    let matcher = matcher.test_list(&["api"])?;
    let matcher = matcher.test_list(&["get", "fetch"])?;

    let (matcher, result) = matcher.test_list_with_fn(|matcher| {

        let matcher = matcher.str("call")?;
        let matcher = matcher.str("fetch_send_request")?;
        let (matcher, callback_id) = matcher.u64()?;
        let (matcher, method) = matcher.string()?;
        let (matcher, url) = matcher.string()?;
        let (matcher, headers) = matcher.json()?;
        let (matcher, body) = matcher.get_any()?;
        matcher.end()?;

        let headers = from_json::<HashMap<String, String>>(headers).map_err(|error| {
            log::error!("error decode headers: {error}");
        })?;

        let body = convert_value_to_body(body).map_err(|error| {
            log::error!("error decode body: {error}");
        })?;

        Ok((callback_id, FetchRequest {
            method,
            url,
            headers,
            body,
        }))
    })?;

    matcher.end()?;

    Ok(result)
}

pub struct WasmInstance {
    instance: Instance,
    store: Store<RequestState>,
}

impl WasmInstance {
    pub fn new(sender: UnboundedSender<Message>, engine: &Engine, module: &Module, request: RequestState) -> Self {
        let url = request.url.clone();
        let mut store = Store::new(engine, request);

        let import_panic_message = Func::wrap(&mut store, {
            let sender = sender.clone();

            move |caller: Caller<'_, RequestState>, ptr: u32, offset: u32| {
                let mut data_context = DataContext::from_caller(caller);

                let message = data_context.get_string_from(ptr, offset);
                log::error!("panic: {message:?}");

                sender.send(Message::Panic(message)).unwrap();
            }
        });

        let import_dom_access = {
            Func::wrap(
                &mut store,
                move |caller: Caller<'_, RequestState>, ptr: u32, offset: u32| -> u32 {
                    let mut data_context = DataContext::from_caller(caller);

                    let value = data_context.get_value(ptr, offset);

                    // Ignore cookie operations
                    if let Ok(()) = match_cookie_command(&value) {
                        return 0;
                    }

                    //get history router location
                    if let Ok(()) = match_history_router(&value) {
                        let result = JsValue::str(url.clone());
                        return data_context.save_value(result);
                    }

                    //adding callback for hashrouter
                    if match_history_router_callback(&value).is_ok() {
                        return 0;
                    }

                    if let Ok(data) = match_dom_bulk_update(&value) {
                        sender.send(Message::DomUpdate(data)).unwrap();
                        return 0;
                    }

                    if let Ok((log_type, log_message)) = match_log(&value) {
                        if log_type == "error" {
                            log::warn!("{log_message}");
                        } else {
                            log::info!("{log_message}");
                        }
                        return 0;
                    }

                    if let Ok(current_time) = match_date_now(&value) {
                        return data_context.save_value(current_time);
                    }

                    if let Ok(result) = match_interval(&value) {
                        match result {
                            CallWebsocketResult::TimeoutSet { time, callback_id } => {
                                if time == 0 {
                                    sender.send(Message::SetTimeoutZero { callback_id }).unwrap();
                                }

                                let result = JsValue::I32(0); // fake timerId
                                return data_context.save_value(result);
                            },
                            CallWebsocketResult::NoResult => {
                                return 0;
                            }
                        }
                    }

                    if let Ok(()) = match_websocket(&value) {
                        return 0;
                    }

                    if let Ok((callback_id, request)) = match_fetch(&value) {
                        sender.send(Message::FetchRequest { callback_id, request }).unwrap();
                        return 0;
                    }

                    if let Ok(()) = match_is_browser(&value) {
                        let result = JsValue::bool(false);
                        return data_context.save_value(result);
                    }

                    log::error!("unsupported message: {value:#?}");
                    0
                }
            )
        };

        let imports = [
            import_panic_message.into(),
            import_dom_access.into()
        ];
        let instance = Instance::new(&mut store, module, &imports).unwrap();

        WasmInstance {
            instance,
            store
        }
    }

    fn call_function<
        Params: wasmtime::WasmParams,
        Results: wasmtime::WasmResults
    >(&mut self, name: &'static str, params: Params) -> Result<Results, String> {
        let vertigo_entry_function = {
            self.instance.get_typed_func::<Params, Results>(&mut self.store, name).unwrap()
        };

        vertigo_entry_function.call(&mut self.store, params).map_err(|error| {
            format!("{error}")
        })
    }

    pub fn call_vertigo_entry_function(&mut self) {
        self.call_function::<(), ()>("vertigo_entry_function", ()).unwrap();
    }

    pub fn wasm_callback(&mut self, callback_id: u64, params: JsValue) -> JsValue {
        let mut data_context = DataContext::from_store(&mut self.store, self.instance);
        let params_ptr = data_context.save_value(params);

        let result = self.call_function::<(u64, u32), u64>("wasm_callback", (callback_id, params_ptr)).unwrap();

        if result == 0 {
            JsValue::Undefined
        } else {
            //TODO - to implement
            todo!()
        }
    }

    pub fn send_fetch_response(&mut self, callback_id: u64, response: FetchResponse) {
        let params = JsValue::List(vec!(
            JsValue::bool(response.success),
            JsValue::U32(response.status),
            convert_body_to_value(response.body),
        ));

        let result = self.wasm_callback(callback_id, params);
        assert_eq!(result, JsValue::Undefined);
    }
}

