use std::{
    future::Future,
    pin::Pin,
    rc::Rc,
};
use crate::JsValue;
use crate::{
    Dependencies, DropResource, FutureBox,
    FetchBuilder, Instant, RequestBuilder, WebsocketMessage,
    get_driver, css::css_manager::CssManager, Context,
};

use crate::{
    driver_module::api::ApiImport,
    driver_module::utils::futures_spawn::spawn_local,
    driver_module::init_env::init_env
};
use crate::driver_module::dom::DriverDom;

use super::DomAccess;
use super::callbacks::{CallbackId};

#[derive(Debug)]
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

#[derive(Clone)]
pub struct DriverInner {
    pub(crate) api: ApiImport,
    pub(crate) dependencies: Dependencies,
    pub(crate) css_manager: CssManager,
    pub(crate) dom: Rc<DriverDom>,
    spawn_executor: Rc<Executable>,
}

impl DriverInner {
    pub fn new(api: ApiImport) -> Self {
        let dependencies = Dependencies::default();

        let spawn_executor = {
            let api = api.clone();

            Rc::new(move |fut: Pin<Box<dyn Future<Output = ()> + 'static>>| {
                spawn_local(api.clone(), fut);
            })
        };

        let dom = Rc::new(DriverDom::new(&api));
        let css_manager = {
            let driver_dom = dom.clone();
            CssManager::new(move |selector: &str, value: &str| {
                driver_dom.insert_css(selector, value);
            })
        };

        dependencies.transaction_state.hooks.on_after_transaction({
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
            spawn_executor,
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
        let driver = Rc::new(DriverInner::new(api));

        Driver {
            inner: driver,
        }
    }
}

impl Driver {
    /// Create new FetchBuilder.
    #[must_use]
    pub fn fetch(&self, url: impl Into<String>) -> FetchBuilder {
        FetchBuilder::new(self.inner.api.clone(), url.into())
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
        self.inner.api.get_hash_location()
    }

    /// Sets the hash part of location URL from client (browser)
    pub fn push_hash_location(&self, path: String) {
        self.inner.api.push_hash_location(&path);
    }

    /// Set event handler upon hash location change
    #[must_use]
    pub fn on_hash_route_change(&self, on_change: Box<dyn Fn(String)>) -> DropResource {
        self.inner.api.on_hash_route_change(on_change)
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

    /// Create new RequestBuilder (more complex version of [fetch](struct.Driver.html#method.fetch))
    #[must_use]
    pub fn request(&self, url: impl Into<String>) -> RequestBuilder {
        RequestBuilder::new(&self.inner.api, url)
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
    pub fn on_after_transaction(&self, callback: impl Fn() + 'static) {
        self.inner.dependencies.transaction_state.hooks.on_after_transaction(callback);
    }

    pub(crate) fn init_env(&self) {
        init_env(self.inner.api.clone());
    }
    
    pub(crate) fn wasm_callback(&self, callback_id: u64, value_ptr: u32) -> (u32, u32) {
        let value = self.inner.api.arguments.get_by_ptr(value_ptr);
        let callback_id = CallbackId::from_u64(callback_id);

        let driver = get_driver();
        let mut result = JsValue::Undefined;

        driver.transaction(|_| {
            result = self.inner.api.callback_store.call(callback_id, value);
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

