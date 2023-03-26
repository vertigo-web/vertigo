use std::{
    future::Future,
    pin::Pin,
    rc::Rc,
};
use crate::fetch::request_builder::{RequestBuilder, RequestBody};
use crate::{
    Dependencies, DropResource, FutureBox,
    Instant, WebsocketMessage, JsJson,
    css::css_manager::CssManager, Context,
};

use crate::{
    driver_module::api::ApiImport,
    driver_module::utils::futures_spawn::spawn_local,
};
use crate::driver_module::dom::DriverDom;

use super::api::DomAccess;

#[derive(Debug, Clone, Copy)]
pub enum FetchMethod {
    GET,
    POST,
}

impl FetchMethod {
    pub fn to_str(&self) -> String {
        match self {
            Self::GET => "GET",
            Self::POST => "POST",
        }.into()
    }
}

type Executable = dyn Fn(Pin<Box<dyn Future<Output = ()> + 'static>>);

pub struct DriverInner {
    pub(crate) api: ApiImport,
    pub(crate) dependencies: &'static Dependencies,
    pub(crate) css_manager: CssManager,
    pub(crate) dom: &'static DriverDom,
    spawn_executor: Rc<Executable>,
    _subscribe: DropResource,
}

impl DriverInner {
    pub fn new() -> &'static Self {
        let dependencies: &'static Dependencies = Box::leak(Box::default());

        let api = ApiImport::default();

        let spawn_executor = {
            let api = api.clone();

            Rc::new(move |fut: Pin<Box<dyn Future<Output = ()> + 'static>>| {
                spawn_local(api.clone(), fut);
            })
        };

        let dom = DriverDom::new(&api);
        let css_manager = {
            let driver_dom = dom;
            CssManager::new(move |selector: &str, value: &str| {
                driver_dom.insert_css(selector, value);
            })
        };

        let subscribe = dependencies.transaction_state.hooks.on_after_transaction(move || {
            dom.flush_dom_changes();
        });

        Box::leak(Box::new(DriverInner {
            api,
            dependencies,
            css_manager,
            dom,
            spawn_executor,
            _subscribe: subscribe,
        }))
    }

}


/// Result from request made using [RequestBuilder].
///
/// Variants:
/// - `Ok(status_code, response)` if request succeeded,
/// - `Err(response)` if request failed (because of network error for example).
pub type FetchResult = Result<(u32, RequestBody), String>;

/// Main connection to vertigo facilities - dependencies and rendering client (the browser).
#[derive(Clone, Copy)]
pub struct Driver {
    pub(crate) inner: &'static DriverInner,
}

impl Default for Driver {
    fn default() -> Self {
        let driver = DriverInner::new();

        Driver {
            inner: driver,
        }
    }
}

impl Driver {
    /// Gets a cookie by name
    pub fn cookie_get(&self, cname: &str) -> String {
        self.inner.api.cookie_get(cname)
    }

    /// Gets a JsJson cookie by name
    pub fn cookie_get_json(&self, cname: &str) -> JsJson {
        self.inner.api.cookie_get_json(cname)
    }

    /// Sets a cookie under provided name
    pub fn cookie_set(&self, cname: &str, cvalue: &str, expires_in: u64) {
        self.inner.api.cookie_set(cname, cvalue, expires_in)
    }

    /// Sets a cookie under provided name
    pub fn cookie_set_json(&self, cname: &str, cvalue: JsJson, expires_in: u64) {
        self.inner.api.cookie_set_json(cname, cvalue, expires_in)
    }

    /// Go back in client's (browser's) history
    pub fn history_back(&self) {
        self.inner.api.history_back();
    }

    /// Make `func` fire every `time` seconds.
    #[must_use]
    pub fn set_interval(&self, time: u32, func: impl Fn() + 'static) -> DropResource {
        self.inner.api.interval_set(time, func)
    }

    /// Gets current value of monotonic clock.
    pub fn now(&self) -> Instant {
        Instant::now(self.inner.api.clone())
    }

    /// Create new RequestBuilder for GETs (more complex version of [fetch](struct.Driver.html#method.fetch))
    #[must_use]
    pub fn request_get(&self, url: impl Into<String>) -> RequestBuilder {
        RequestBuilder::get(url)
    }

    /// Create new RequestBuilder for POSTs (more complex version of [fetch](struct.Driver.html#method.fetch))
    #[must_use]
    pub fn request_post(&self, url: impl Into<String>) -> RequestBuilder {
        RequestBuilder::post(url)
    }

    #[must_use]
    pub fn sleep(&self, time: u32) -> FutureBox<()> {
        let (sender, future) = FutureBox::new();
        self.inner.api.set_timeout_and_detach(time, move || {
            sender.publish(());
        });

        future
    }

    pub fn get_random(&self, min: u32, max: u32) -> u32 {
        self.inner.api.get_random(min, max)
    }

    pub fn get_random_from<K: Clone>(&self, list: &[K]) -> Option<K> {
        let len = list.len();

        if len < 1 {
            return None;
        }

        let max_index = len - 1;

        let index = self.get_random(0, max_index as u32);
        Some(list[index as usize].clone())
    }

    /// Initiate a websocket connection. Provided callback should handle a single [WebsocketMessage].
    #[must_use]
    pub fn websocket<F: Fn(WebsocketMessage) + 'static>(&self, host: impl Into<String>, callback: F) -> DropResource {
        self.inner.api.websocket(host, callback)
    }

    /// Spawn a future - thus allowing to fire async functions in, for example, event handler. Handy when fetching resources from internet.
    pub fn spawn(&self, future: impl Future<Output = ()> + 'static) {
        let future = Box::pin(future);
        let spawn_executor = self.inner.spawn_executor.clone();
        spawn_executor(future);
    }

    /// Fire provided function in a way that all changes in [dependency graph](struct.Dependencies.html) made by this function
    /// will trigger only one run of updates, just like the changes were done all at once.
    pub fn transaction<R, F: FnOnce(&Context) -> R>(&self, func: F) -> R {
        self.inner.dependencies.transaction(func)
    }

    pub fn dom_access(&self) -> DomAccess {
        self.inner.api.dom_access()
    }

    ///Function added for diagnostic purposes. It allows you to check whether a block with a transaction is missing somewhere.
    pub fn on_after_transaction(&self, callback: impl Fn() + 'static) -> DropResource {
        self.inner.dependencies.transaction_state.hooks.on_after_transaction(callback)
    }

    pub fn is_browser(&self) -> bool {
        self.inner.api.is_browser()
    }

    pub fn is_server(&self) -> bool {
        !self.is_browser()
    }

}

