use std::{
    future::Future,
    pin::Pin,
    rc::Rc,
};
use crate::{
    dev::{RealDomId, RefsContext, WebsocketMessageDriver},
    Dependencies, DropResource, FutureBox,
    KeyDownEvent, FetchBuilder, Instant, RequestBuilder, WebsocketMessage, WebsocketConnection, DropFileEvent, virtualdom::models::vdom_element::DropFileItem,
};

use crate::{
    driver_module::api::ApiImport,
    driver_module::utils::futures_spawn::spawn_local,
    driver_module::init_env::init_env
};
use crate::driver_module::modules::{
    dom::DriverBrowserDom,
    fetch::DriverBrowserFetch,
    hashrouter::DriverBrowserHashrouter,
    interval::DriverBrowserInterval,
    websocket::DriverWebsocket,
};

#[derive(Debug)]
pub enum FetchMethod {
    GET,
    POST,
}

impl FetchMethod {
    pub fn to_string(&self) -> &str {
        match self {
            Self::GET => "GET",
            Self::POST => "POST",
        }
    }
}

pub enum EventCallback {
    OnClick {
        callback: Option<Rc<dyn Fn()>>,
    },
    OnInput {
        callback: Option<Rc<dyn Fn(String)>>,
    },
    OnMouseEnter {
        callback: Option<Rc<dyn Fn()>>,
    },
    OnMouseLeave {
        callback: Option<Rc<dyn Fn()>>,
    },
    OnKeyDown {
        callback: Option<Rc<dyn Fn(KeyDownEvent) -> bool>>,
    },
    HookKeyDown {
        callback: Option<Rc<dyn Fn(KeyDownEvent) -> bool>>,
    },
    OnDropFile {
        callback: Option<Rc<dyn Fn(DropFileEvent)>>,
    }
}

impl EventCallback {
    pub fn to_string(&self) -> &str {
        match self {
            EventCallback::OnClick { callback } => {
                if callback.is_some() {
                    "onClick set"
                } else {
                    "onClick clear"
                }
            }
            EventCallback::OnInput { callback } => {
                if callback.is_some() {
                    "on_input set"
                } else {
                    "on_input clear"
                }
            }
            EventCallback::OnMouseEnter { callback } => {
                if callback.is_some() {
                    "onMouseEnter set"
                } else {
                    "onMouseEnter clear"
                }
            }
            EventCallback::OnMouseLeave { callback } => {
                if callback.is_some() {
                    "on_mouse_leave set"
                } else {
                    "on_mouse_leave clear"
                }
            }
            EventCallback::OnKeyDown { callback } => {
                if callback.is_some() {
                    "OnKeyDown set"
                } else {
                    "OnKeyDown clear"
                }
            },
            EventCallback::HookKeyDown { callback } => {
                if callback.is_some() {
                    "HookKeyDown set"
                } else {
                    "HookKeyDown clear"
                }
            },
            EventCallback::OnDropFile { callback } => {
                if callback.is_some() {
                    "OnDropFile set"
                } else {
                    "OnDropFile clear"
                }
            }
        }
    }
}


#[derive(Clone)]
pub struct DriverBrowserInner {
    pub(crate) api: Rc<ApiImport>,
    dependencies: Dependencies,
    driver_dom: DriverBrowserDom,
    driver_interval: DriverBrowserInterval,
    driver_hashrouter: DriverBrowserHashrouter,
    driver_fetch: DriverBrowserFetch,
    driver_websocket: DriverWebsocket,
    spawn_executor: Rc<dyn Fn(Pin<Box<dyn Future<Output = ()> + 'static>>)>,
}

impl DriverBrowserInner {
    pub fn new(api: Rc<ApiImport>) -> Self {
        let dependencies = Dependencies::default();
        let driver_interval = DriverBrowserInterval::new(&api);

        let spawn_executor = {
            let driver_interval = driver_interval.clone();

            Rc::new(move |fut: Pin<Box<dyn Future<Output = ()> + 'static>>| {
                spawn_local(driver_interval.clone(), fut);
            })
        };

        let driver_dom = DriverBrowserDom::new(&dependencies, &api);
        let driver_hashrouter = DriverBrowserHashrouter::new(&api);
        let driver_fetch = DriverBrowserFetch::new(&api);
        let driver_websocket = DriverWebsocket::new(&api);

        DriverBrowserInner {
            api,
            dependencies,
            driver_dom,
            driver_interval,
            driver_hashrouter,
            driver_fetch,
            driver_websocket,
            spawn_executor
        }
    }

    pub fn export_interval_run_callback(&self, callback_id: u32) {
        self.driver_interval.export_interval_run_callback(callback_id);
    }

    pub fn export_timeout_run_callback(&self, callback_id: u32) {
        self.driver_interval.export_timeout_run_callback(callback_id);
    }

    pub fn export_hashrouter_hashchange_callback(&self, list_id: u32) {
        let params = self.api.arguments.unfreeze(list_id);

        let new_hash = params
            .unwrap_or_default()
            .convert::<String, _>(|mut params| {
                let first = params.get_string("first")?;
                params.expect_no_more()?;
                Ok(first)
            })
            .unwrap_or_else(|error| {
                log::error!("export_hashrouter_hashchange_callback -> params decode error -> {error}");
                String::from("")
            });

        self.driver_hashrouter.export_hashrouter_hashchange_callback(new_hash);
    }

    pub fn export_fetch_callback(&self, params_id: u32) {
        let params = self.api.arguments.unfreeze(params_id);

        let params = params
            .unwrap_or_default()
            .convert(|mut params| {
                let request_id = params.get_u32("request_id")?;
                let success = params.get_bool("success")?;
                let status = params.get_u32("status")?;
                let response = params.get_string("response")?;
                params.expect_no_more()?;
                Ok((request_id, success, status, response))
            });

        match params {
            Ok((request_id, success, status, response)) => {
                self.driver_fetch.export_fetch_callback(request_id, success, status, response);
            },
            Err(error) => {
                log::error!("export_fetch_callback -> params decode error -> {error}");
            }
        }
    }

    pub fn export_websocket_callback_socket(&self, callback_id: u32) {
        self.driver_websocket.export_websocket_callback_socket(callback_id);
    }

    pub fn export_websocket_callback_message(&self, params_id: u32) {
        let params = self.api.arguments.unfreeze(params_id);

        let params = params
            .unwrap_or_default()
            .convert(|mut params| {
                let callback_id = params.get_u32("callback_id")?;
                let response = params.get_string("message")?;
                params.expect_no_more()?;
                Ok((callback_id, response))
            });

        match params {
            Ok((callback_id, response)) => {
                self.driver_websocket.export_websocket_callback_message(callback_id, response);
            },
            Err(error) => {
                log::error!("export_websocket_callback_message -> params decode error -> {error}");
            }
        }
    }

    pub fn export_websocket_callback_close(&self, callback_id: u32) {
        self.driver_websocket.export_websocket_callback_close(callback_id);
    }

    pub fn export_dom_keydown(&self, params_id: u32) -> u32 {
        let params = self.api.arguments.unfreeze(params_id);

        let params = params
            .unwrap_or_default()
            .convert(|mut params| {
                let dom_id = params.get_u64_or_null("dom_id")?;
                let key = params.get_string("key")?;
                let code = params.get_string("code")?;
                let alt_key = params.get_bool("altKey")?;
                let ctrl_key = params.get_bool("ctrlKey")?;
                let shift_key = params.get_bool("shiftKey")?;
                let meta_key = params.get_bool("metaKey")?;
                params.expect_no_more()?;

                Ok((dom_id, key, code, alt_key, ctrl_key, shift_key, meta_key))
            });

        match params {
            Ok((dom_id, key, code, alt_key, ctrl_key, shift_key, meta_key)) => {
                let prevent_default = self.driver_dom.export_dom_keydown(
                    dom_id,
                    key,
                    code,
                    alt_key,
                    ctrl_key,
                    shift_key,
                    meta_key
                );

                match prevent_default {
                    true => 1,
                    false => 0
                }
            },
            Err(error) => {
                log::error!("export_websocket_callback_message -> params decode error -> {error}");
                0
            }
        }
    }

    pub fn export_dom_oninput(&self, params_id: u32) {
        let params = self.api.arguments.unfreeze(params_id);

        let params = params
            .unwrap_or_default()
            .convert(|mut params| {
                let dom_id = params.get_u64("dom_id")?;
                let text = params.get_string("text")?;
                params.expect_no_more()?;

                Ok((dom_id, text))
            });

        match params {
            Ok((dom_id, text)) => {
                self.driver_dom.export_dom_oninput(dom_id, text);
            },
            Err(error) => {
                log::error!("export_dom_oninput -> params decode error -> {error}");
            }
        }
    }

    pub fn export_dom_ondropfile(&self, params_id: u32) {
        let params = self.api.arguments.unfreeze(params_id);

        let params = params
            .unwrap_or_default()
            .convert(|mut params| {
                let dom_id = params.get_u64("dom_id")?;
                let files = params.get_list("files", |mut item| {
                    let name = item.get_string("name")?;
                    let data = item.get_buffer("data")?;
                    
                    Ok(DropFileItem::new(name, data))
                })?;
                params.expect_no_more()?;

                Ok((dom_id, DropFileEvent::new(files)))
            });

        match params {
            Ok((dom_id, files)) => {
                self.driver_dom.export_dom_ondropfile(dom_id, files);
            },
            Err(error) => {
                log::error!("export_dom_ondropfile -> params decode error -> {error}");
            }
        }
    }

    pub fn export_dom_mouseover(&self, dom_id: u64) {
        let dom_id = if dom_id == 0 { None } else { Some(dom_id) };
        self.driver_dom.export_dom_mouseover(dom_id);
    }

    pub fn export_dom_mousedown(&self, dom_id: u64) {
        self.driver_dom.export_dom_mousedown(dom_id);
    }

    pub fn init_env(&self) {
        init_env(self.api.clone());
    }
}


/// Result from request made using [FetchBuilder].
///
/// Variants:
/// - `Ok(status_code, response)` if request succeded,
/// - `Err(response)` if request failed (because ofnetwork error for example).
pub type FetchResult = Result<(u32, String), String>;

/// Main connection to vertigo facilities - dependencies and rendering client (the browser).
///
/// This is in fact only a box for inner generic driver.
/// This way a web developer don't need to worry about the specific driver used,
/// though usually it is the [BrowserDriver](../vertigo_browserdriver/struct.DriverBrowser.html)
/// which is used to create a Driver.
///
/// Additionally driver struct wraps [Dependencies] object.
#[derive(Clone)]
pub struct Driver {
    pub driver: Rc<DriverBrowserInner>,
}

impl Driver {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(api: ApiImport) -> Driver {
        let driver = Rc::new(DriverBrowserInner::new(Rc::new(api)));

        Driver {
            driver,
        }
    }
}

impl Driver {
    // Below - methods to interact with the dom. Used exclusively by vertigo.

    pub(crate) fn create_node(&self, id: RealDomId, name: &'static str) {
        self.driver.driver_dom.create_node(id, name);
    }

    pub(crate) fn rename_node(&self, id: RealDomId, new_name: &'static str) {
        self.driver.driver_dom.rename_node(id, new_name);
    }

    pub(crate) fn create_text(&self, id: RealDomId, value: &str) {
        self.driver.driver_dom.create_text(id, value);
    }

    pub(crate) fn update_text(&self, id: RealDomId, value: &str) {
        self.driver.driver_dom.update_text(id, value);
    }

    pub(crate) fn set_attr(&self, id: RealDomId, key: &'static str, value: &str) {
        self.driver.driver_dom.set_attr(id, key, value);
    }

    pub(crate) fn remove_attr(&self, id: RealDomId, name: &'static str) {
        self.driver.driver_dom.remove_attr(id, name);
    }

    pub(crate) fn remove_node(&self, id: RealDomId) {
        self.driver.driver_dom.remove_node(id);
    }

    pub(crate) fn remove_text(&self, id: RealDomId) {
        self.driver.driver_dom.remove_text(id);
    }

    pub(crate) fn insert_before(&self, parent: RealDomId, child: RealDomId, ref_id: Option<RealDomId>) {
        self.driver.driver_dom.insert_before(parent, child, ref_id);
    }

    pub(crate) fn insert_css(&self, selector: &str, value: &str) {
        self.driver.driver_dom.insert_css(selector, value);
    }

    pub(crate) fn set_event(&self, id: RealDomId, callback: EventCallback) {
        self.driver.driver_dom.set_event(id, callback);
    }

    /// Create new FetchBuilder.
    #[must_use]
    pub fn fetch(&self, url: impl Into<String>) -> FetchBuilder {
        FetchBuilder::new(self.driver.driver_fetch.clone(), url.into())
    }

    /// Gets a cookie by name
    pub fn cookie_get(&self, cname: &str) -> String {
        self.driver.api.cookie_get(cname)
    }

    /// Sets a cookie under provided name
    pub fn cookie_set(&self, cname: &str, cvalue: &str, expires_in: u64) {
        self.driver.api.cookie_set(cname, cvalue, expires_in)
    }

    /// Retrieves the hash part of location URL from client (browser)
    pub fn get_hash_location(&self) -> String {
        self.driver.driver_hashrouter.get_hash_location()
    }

    /// Sets the hash part of location URL from client (browser)
    pub fn push_hash_location(&self, path: String) {
        self.driver.driver_hashrouter.push_hash_location(path);
    }

    /// Set event handler upon hash location change
    #[must_use]
    pub fn on_hash_route_change(&self, on_change: Box<dyn Fn(&String)>) -> DropResource {
        self.driver.driver_hashrouter.on_hash_route_change(on_change)
    }

    /// Make `func` fire every `time` seconds.
    #[must_use]
    pub fn set_interval(&self, time: u32, func: impl Fn() + 'static) -> DropResource {
        self.driver.driver_interval.set_interval(time, move |_| {
            func();
        })
    }

    /// Gets current value of monotonic clock.
    pub fn now(&self) -> Instant {
        Instant::now(self.driver.api.clone())
    }

    /// Create new RequestBuilder (more complex version of [fetch](struct.Driver.html#method.fetch))
    #[must_use]
    pub fn request(&self, url: impl Into<String>) -> RequestBuilder {
        RequestBuilder::new(&self.driver.driver_fetch, url)
    }

    pub fn sleep(&self, time: u32) -> FutureBox<()> {
        let (sender, future) = FutureBox::new();
        self.driver.driver_interval.set_timeout_and_detach(time, move |_| {
            sender.publish(());
        });

        future
    }

    /// Initiate a websocket connection. Provided callback should handle a single [WebsocketMessage].
    #[must_use]
    pub fn websocket(&self, host: impl Into<String>, callback: Box<dyn Fn(WebsocketMessage)>) -> DropResource {
        let host: String = host.into();

        self.driver.driver_websocket.websocket_start(
            host,
            Box::new(move |message: WebsocketMessageDriver| {
                let message = match message {
                    WebsocketMessageDriver::Connection { callback_id } => {
                        let connection = WebsocketConnection::new(callback_id);
                        WebsocketMessage::Connection(connection)
                    }
                    WebsocketMessageDriver::Message(message) => WebsocketMessage::Message(message),
                    WebsocketMessageDriver::Close => WebsocketMessage::Close,
                };

                callback(message);
            }),
        )
    }

    pub fn websocket_send_message(&self, callback_id: u32, message: String) {
        self.driver.driver_websocket.websocket_send_message(callback_id, message);
    }

    pub(crate) fn get_bounding_client_rect_x(&self, id: RealDomId) -> i32 {
        self.driver.driver_dom.get_bounding_client_rect_x(id)
    }

    pub(crate) fn get_bounding_client_rect_y(&self, id: RealDomId) -> i32 {
        self.driver.driver_dom.get_bounding_client_rect_y(id)
    }

    pub(crate) fn get_bounding_client_rect_width(&self, id: RealDomId) -> u32 {
        self.driver.driver_dom.get_bounding_client_rect_width(id)
    }

    pub(crate) fn get_bounding_client_rect_height(&self, id: RealDomId) -> u32 {
        self.driver.driver_dom.get_bounding_client_rect_height(id)
    }

    pub(crate) fn scroll_top(&self, id: RealDomId) -> i32 {
        self.driver.driver_dom.scroll_top(id)
    }

    pub(crate) fn set_scroll_top(&self, id: RealDomId, value: i32) {
        self.driver.driver_dom.set_scroll_top(id, value)
    }

    pub(crate) fn scroll_left(&self, id: RealDomId) -> i32 {
        self.driver.driver_dom.scroll_left(id)
    }

    pub(crate) fn set_scroll_left(&self, id: RealDomId, value: i32) {
        self.driver.driver_dom.set_scroll_left(id, value)
    }

    pub(crate) fn scroll_width(&self, id: RealDomId) -> u32 {
        self.driver.driver_dom.scroll_width(id)
    }

    pub(crate) fn scroll_height(&self, id: RealDomId) -> u32 {
        self.driver.driver_dom.scroll_height(id)
    }

    pub(crate) fn push_ref_context(&self, context: RefsContext) {
        self.driver.driver_dom.push_ref_context(context);
    }

    pub(crate) fn flush_update(&self) {
        self.driver.driver_dom.flush_dom_changes();
    }

    /// Spawn a future - thus allowing to fire async functions in, for example, event handler. Handy when fetching resources from internet.
    pub fn spawn(&self, future: impl Future<Output = ()> + 'static) {
        let future = Box::pin(future);
        let spawn_executor = self.driver.spawn_executor.clone();
        spawn_executor(future);
    }

    /// Fire provided function in a way that all changes in [dependency graph](struct.Dependencies.html) made by this function
    /// will trigger only one run of updates, just like the changes were done all at once.
    pub fn transaction<F: FnOnce()>(&self, func: F) {
        self.driver.dependencies.transaction(func);
    }

    pub(crate) fn get_dependencies(&self) -> Dependencies {
        self.driver.dependencies.clone()
    }

    pub(crate) fn external_connections_refresh(&self) {
        self.driver.dependencies.external_connections_refresh();
    }
}

