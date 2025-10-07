use crate::{
    css::css_manager::CssManager,
    fetch::request_builder::{RequestBody, RequestBuilder},
    Context, Css, Dependencies, DropResource, FutureBox, Instant, JsJson, WebsocketMessage,
};
use std::cell::RefCell;
use std::{future::Future, pin::Pin, rc::Rc};

use crate::driver_module::dom::DriverDom;
use crate::{driver_module::api::ApiImport, driver_module::utils::futures_spawn::spawn_local};

use super::api::DomAccess;

/// Placeholder where to put public build path at runtime (default /build)
pub const VERTIGO_PUBLIC_BUILD_PATH_PLACEHOLDER: &str = "%%VERTIGO_PUBLIC_BUILD_PATH%%";

/// Placeholder where to put public mount point at runtime (default /)
pub const VERTIGO_MOUNT_POINT_PLACEHOLDER: &str = "%%VERTIGO_MOUNT_POINT%%";

#[derive(Debug, Clone, Copy)]
pub enum FetchMethod {
    GET,
    HEAD,
    POST,
    PUT,
    DELETE,
    CONNECT,
    OPTIONS,
    TRACE,
    PATCH,
}

impl FetchMethod {
    pub fn to_str(&self) -> String {
        match self {
            Self::GET => "GET",
            Self::HEAD => "HEAD",
            Self::POST => "POST",
            Self::PUT => "PUT",
            Self::DELETE => "DELETE",
            Self::CONNECT => "CONNECT",
            Self::OPTIONS => "OPTIONS",
            Self::TRACE => "TRACE",
            Self::PATCH => "PATCH",
        }
        .into()
    }
}

type Executable = dyn Fn(Pin<Box<dyn Future<Output = ()> + 'static>>);
type PlainHandler = dyn Fn(&str) -> Option<String>;

pub struct DriverInner {
    pub(crate) api: ApiImport,
    pub(crate) dependencies: &'static Dependencies,
    pub(crate) css_manager: CssManager,
    pub(crate) dom: &'static DriverDom,
    spawn_executor: Rc<Executable>,
    _subscribe: DropResource,
    _plains_handler: RefCell<Option<Rc<PlainHandler>>>,
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
            CssManager::new(move |selector, value| driver_dom.insert_css(selector, value))
        };

        let subscribe = dependencies.hooks.on_after_transaction(move || {
            dom.flush_dom_changes();
        });

        Box::leak(Box::new(DriverInner {
            api,
            dependencies,
            css_manager,
            dom,
            spawn_executor,
            _subscribe: subscribe,
            _plains_handler: RefCell::new(None),
        }))
    }
}

/// Result from request made using [RequestBuilder].
///
/// Variants:
/// - `Ok(status_code, response)` if request succeeded,
/// - `Err(response)` if request failed (because of network error for example).
pub type FetchResult = Result<(u32, RequestBody), String>;

/// Set of functions to communicate with the browser.
#[derive(Clone, Copy)]
pub struct Driver {
    pub(crate) inner: &'static DriverInner,
}

impl Default for Driver {
    fn default() -> Self {
        let driver = DriverInner::new();

        Driver { inner: driver }
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

    /// Replace current location
    pub fn history_replace(&self, new_url: &str) {
        self.inner.api.replace_history_location(new_url)
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

    /// Gets current UTC timestamp
    pub fn utc_now(&self) -> i64 {
        self.inner.api.utc_now()
    }

    /// Gets browsers time zone offset in seconds
    ///
    /// Compatible with chrono's `FixedOffset::east_opt` method.
    pub fn timezone_offset(&self) -> i32 {
        self.inner.api.timezone_offset()
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
    pub fn websocket<F: Fn(WebsocketMessage) + 'static>(
        &self,
        host: impl Into<String>,
        callback: F,
    ) -> DropResource {
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

    /// Allows to access different objects in the browser (See [js!](crate::js) macro for convenient use).
    pub fn dom_access(&self) -> DomAccess {
        self.inner.api.dom_access()
    }

    /// Function added for diagnostic purposes. It allows you to check whether a block with a transaction is missing somewhere.
    pub fn on_after_transaction(&self, callback: impl Fn() + 'static) -> DropResource {
        self.inner.dependencies.hooks.on_after_transaction(callback)
    }

    /// Return true if the code is executed client-side (in the browser).
    ///
    /// ```rust
    /// use vertigo::{dom, get_driver};
    ///
    /// let component = if get_driver().is_browser() {
    ///     dom! { <div>"My dynamic component"</div> }
    /// } else {
    ///     dom! { <div>"Loading... (if not loaded check if JavaScript is enabled)"</div> }
    /// };
    /// ```
    pub fn is_browser(&self) -> bool {
        self.inner.api.is_browser()
    }

    pub fn is_server(&self) -> bool {
        !self.is_browser()
    }

    /// Get any env variable set upon starting vertigo server.
    pub fn env(&self, name: impl Into<String>) -> Option<String> {
        let name = name.into();
        self.inner.api.get_env(name)
    }

    /// Get public path to build directory where the browser can access WASM and other build files.
    pub fn public_build_path(&self, path: impl Into<String>) -> String {
        let path = path.into();
        if self.is_browser() {
            // In the browser use env variable attached during SSR
            if let Some(public_path) = self.env("vertigo-public-path") {
                path.replace(VERTIGO_PUBLIC_BUILD_PATH_PLACEHOLDER, &public_path)
            } else {
                // Fallback to default dest_dir
                path.replace(VERTIGO_PUBLIC_BUILD_PATH_PLACEHOLDER, "/build")
            }
        } else {
            // On the server, leave it, it will be replaced during SSR
            path
        }
    }

    /// Convert relative route to public path (with mount point attached)
    pub fn route_to_public(&self, path: impl Into<String>) -> String {
        let path = path.into();
        if self.is_browser() {
            // In the browser use env variable attached during SSR
            let mount_point = self
                .env("vertigo-mount-point")
                .unwrap_or_else(|| "/".to_string());
            if mount_point != "/" {
                [mount_point, path].concat()
            } else {
                path
            }
        } else {
            // On the server, prepend it with mount point token
            [VERTIGO_MOUNT_POINT_PLACEHOLDER, &path].concat()
        }
    }

    /// Convert path in the url to relative route in the app.
    pub fn route_from_public(&self, path: impl Into<String>) -> String {
        let path: String = path.into();
        self.inner.api.route_from_public(path)
    }

    /// Register handler that intercepts defined urls and generates plaintext responses during SSR.
    ///
    /// Should return `None` in the handler if regular HTML should be generated by the App.
    ///
    /// ```rust
    /// use vertigo::get_driver;
    ///
    /// get_driver().plains(|url| {
    ///    if url == "/robots.txt" {
    ///       Some("User-Agent: *\nDisallow: /search".to_string())
    ///    } else {
    ///       None
    ///    }
    /// });
    /// ```
    pub fn plains(&mut self, callback: impl Fn(&str) -> Option<String> + 'static) {
        let mut mut_plains = self.inner._plains_handler.borrow_mut();
        *mut_plains = Some(Rc::new(callback));
    }

    /// Try to intercept urls for generating plaintext responses during SSR (used internally by [main!](crate::main) macro).
    /// See [Driver::plains()] method.
    pub fn try_get_plain(&self) {
        if self.is_server() {
            let url = self.inner.api.get_history_location();
            match self.inner._plains_handler.try_borrow() {
                Ok(callback_ref) => {
                    if let Some(callback) = callback_ref.as_deref() {
                        if let Some(body) = callback(&url) {
                            self.inner.api.plain_response(body)
                        }
                    }
                }
                Err(err) => log::error!("Error invoking plains: {err}"),
            }
        }
    }

    /// Allow to set custom HTTP status code during SSR
    ///
    /// ```rust
    /// use vertigo::get_driver;
    ///
    /// get_driver().set_status(404)
    /// ```
    pub fn set_status(&self, status: u16) {
        if self.is_server() {
            self.inner.api.set_status(status);
        }
    }

    /// Adds this CSS to manager producing a class name, which is returned
    ///
    /// There shouldn't be need to use it manually. It's used by `css!` macro.
    pub fn class_name_for(&mut self, css: &Css) -> String {
        self.inner.css_manager.get_class_name(css)
    }

    /// Register css bundle
    ///
    /// There shouldn't be need to use it manually. It's used by `main!` macro.
    pub fn register_bundle(&self, bundle: impl Into<String>) {
        self.inner.css_manager.register_bundle(bundle.into())
    }
}
