use std::{
    future::Future,
    pin::Pin,
    rc::Rc,
};
use crate::JsValue;
use crate::{
    WebsocketMessageDriver,
    Dependencies, DropResource, FutureBox,
    FetchBuilder, Instant, RequestBuilder, WebsocketMessage, WebsocketConnection,
    get_driver, css::css_manager::CssManager, Css, Context,
};

use crate::{
    driver_module::api::ApiImport,
    driver_module::utils::futures_spawn::spawn_local,
    driver_module::init_env::init_env
};
use crate::driver_module::modules::{
    dom::DriverDom,
    fetch::DriverFetch,
    hashrouter::DriverHashrouter,
    interval::DriverInterval,
    websocket::DriverWebsocket,
};

use super::DomAccess;
use super::callbacks::{CallbackStore, CallbackId};

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

type Executable = dyn Fn(Pin<Box<dyn Future<Output = ()> + 'static>>);

#[derive(Clone)]
pub struct DriverInner {
    pub(crate) api: Rc<ApiImport>,
    pub(crate) dependencies: Dependencies,
    css_manager: CssManager,
    pub(crate) dom: Rc<DriverDom>,
    interval: DriverInterval,
    hashrouter: DriverHashrouter,
    fetch: DriverFetch,
    websocket: DriverWebsocket,
    spawn_executor: Rc<Executable>,
    pub(crate) callback_store: Rc<CallbackStore>,
}

impl DriverInner {
    pub fn new(api: Rc<ApiImport>) -> Self {
        let dependencies = Dependencies::default();
        let interval = DriverInterval::new(&api);

        let spawn_executor = {
            let driver_interval = interval.clone();

            Rc::new(move |fut: Pin<Box<dyn Future<Output = ()> + 'static>>| {
                spawn_local(driver_interval.clone(), fut);
            })
        };

        let dom = Rc::new(DriverDom::new(&api));
        let hashrouter = DriverHashrouter::new(&api);
        let driver_fetch = DriverFetch::new(&api);
        let websocket = DriverWebsocket::new(&api);

        let css_manager = {
            let driver_dom = dom.clone();
            CssManager::new(move |selector: &str, value: &str| {
                driver_dom.insert_css(selector, value);
            })
        };

        dependencies.on_after_transaction({
            let dom = dom.clone();

            move || {
                dom.flush_dom_changes();
            }
        });

        DriverInner {
            api,
            dependencies,
            css_manager,
            dom,
            interval,
            hashrouter,
            fetch: driver_fetch,
            websocket,
            spawn_executor,
            callback_store: Rc::new(CallbackStore::new()),
        }
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
    pub(crate) inner: Rc<DriverInner>,
}

impl Driver {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(api: ApiImport) -> Driver {
        let driver = Rc::new(DriverInner::new(Rc::new(api)));

        Driver {
            inner: driver,
        }
    }
}

impl Driver {
    /// Create new FetchBuilder.
    #[must_use]
    pub fn fetch(&self, url: impl Into<String>) -> FetchBuilder {
        FetchBuilder::new(self.inner.fetch.clone(), url.into())
    }

    /// Gets a cookie by name
    pub fn cookie_get(&self, cname: &str) -> String {
        self.inner.api.cookie_get(cname)
    }

    /// Sets a cookie under provided name
    pub fn cookie_set(&self, cname: &str, cvalue: &str, expires_in: u64) {
        self.inner.api.cookie_set(cname, cvalue, expires_in)
    }

    /// Retrieves the hash part of location URL from client (browser)
    pub fn get_hash_location(&self) -> String {
        self.inner.hashrouter.get_hash_location()
    }

    /// Sets the hash part of location URL from client (browser)
    pub fn push_hash_location(&self, path: String) {
        self.inner.hashrouter.push_hash_location(path);
    }

    /// Set event handler upon hash location change
    #[must_use]
    pub fn on_hash_route_change(&self, on_change: Box<dyn Fn(&String)>) -> DropResource {
        self.inner.hashrouter.on_hash_route_change(on_change)
    }

    /// Make `func` fire every `time` seconds.
    #[must_use]
    pub fn set_interval(&self, time: u32, func: impl Fn() + 'static) -> DropResource {
        self.inner.interval.set_interval(time, move |_| {
            func();
        })
    }

    /// Gets current value of monotonic clock.
    pub fn now(&self) -> Instant {
        Instant::now(self.inner.api.clone())
    }

    /// Create new RequestBuilder (more complex version of [fetch](struct.Driver.html#method.fetch))
    #[must_use]
    pub fn request(&self, url: impl Into<String>) -> RequestBuilder {
        RequestBuilder::new(&self.inner.fetch, url)
    }

    pub fn sleep(&self, time: u32) -> FutureBox<()> {
        let (sender, future) = FutureBox::new();
        self.inner.interval.set_timeout_and_detach(time, move |_| {
            sender.publish(());
        });

        future
    }

    /// Initiate a websocket connection. Provided callback should handle a single [WebsocketMessage].
    #[must_use]
    pub fn websocket(&self, host: impl Into<String>, callback: Box<dyn Fn(WebsocketMessage)>) -> DropResource {
        let host: String = host.into();

        self.inner.websocket.websocket_start(
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
        self.inner.websocket.websocket_send_message(callback_id, message);
    }

    /// Spawn a future - thus allowing to fire async functions in, for example, event handler. Handy when fetching resources from internet.
    pub fn spawn(&self, future: impl Future<Output = ()> + 'static) {
        let future = Box::pin(future);
        let spawn_executor = self.inner.spawn_executor.clone();
        spawn_executor(future);
    }

    /// Fire provided function in a way that all changes in [dependency graph](struct.Dependencies.html) made by this function
    /// will trigger only one run of updates, just like the changes were done all at once.
    pub fn transaction<F: FnOnce(&Context)>(&self, func: F) {
        self.inner.dependencies.transaction(func);
    }

    pub (crate) fn get_class_name(&self, css: &Css) -> String {
        self.inner.css_manager.get_class_name(css)
    }

    pub fn dom_access(&self) -> DomAccess {
        self.inner.api.dom_access()
    }

    pub(crate) fn init_env(&self) {
        init_env(self.inner.api.clone());
    }

    pub(crate) fn export_interval_run_callback(&self, callback_id: u32) {
        self.inner.interval.export_interval_run_callback(callback_id);
    }

    pub(crate) fn export_timeout_run_callback(&self, callback_id: u32) {
        self.inner.interval.export_timeout_run_callback(callback_id);
    }

    pub(crate) fn export_hashrouter_hashchange_callback(&self, ptr: u32) {
        let params = self.inner.api.arguments.get_by_ptr(ptr);

        let new_hash = params
            .convert::<String, _>(|mut params| {
                let first = params.get_string("first")?;
                params.expect_no_more()?;
                Ok(first)
            })
            .unwrap_or_else(|error| {
                log::error!("export_hashrouter_hashchange_callback -> params decode error -> {error}");
                String::from("")
            });

        get_driver().transaction(|_|{
            self.inner.hashrouter.export_hashrouter_hashchange_callback(new_hash);
        });
    }

    pub(crate) fn export_fetch_callback(&self, ptr: u32) {
        let params = self.inner.api.arguments.get_by_ptr(ptr);

        let params = params
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
                get_driver().transaction(|_|{
                    self.inner.fetch.export_fetch_callback(request_id, success, status, response);
                });
            },
            Err(error) => {
                log::error!("export_fetch_callback -> params decode error -> {error}");
            }
        }
    }

    pub(crate) fn export_websocket_callback_socket(&self, callback_id: u32) {
        self.inner.websocket.export_websocket_callback_socket(callback_id);
    }

    pub(crate) fn export_websocket_callback_message(&self, ptr: u32) {
        let params = self.inner.api.arguments.get_by_ptr(ptr);

        let params = params
            .convert(|mut params| {
                let callback_id = params.get_u32("callback_id")?;
                let response = params.get_string("message")?;
                params.expect_no_more()?;
                Ok((callback_id, response))
            });

        match params {
            Ok((callback_id, response)) => {
                get_driver().transaction(|_|{
                    self.inner.websocket.export_websocket_callback_message(callback_id, response);
                });
            },
            Err(error) => {
                log::error!("export_websocket_callback_message -> params decode error -> {error}");
            }
        }
    }

    pub(crate) fn export_websocket_callback_close(&self, callback_id: u32) {
        get_driver().transaction(|_| {
            self.inner.websocket.export_websocket_callback_close(callback_id);
        });
    }

    pub(crate) fn export_dom_callback(&self, callback_id: u64, value_ptr: u32) -> (u32, u32) {
        let value = self.inner.api.arguments.get_by_ptr(value_ptr);
        let callback_id = CallbackId::from_u64(callback_id);

        let driver = get_driver();
        let mut result = JsValue::Undefined;

        driver.transaction(|_| {
            result = self.inner.callback_store.call(callback_id, value);
        });

        if result == JsValue::Undefined {
            return (0, 0);
        }

        let memory_block = result.to_snapshot();
        let (ptr, size) = memory_block.get_ptr_and_size();
        self.inner.api.arguments.set(memory_block);

        (ptr, size)
    }

}

