use std::{future::Future, pin::Pin, rc::Rc};
use vertigo_macro::{store, AutoJsJson};

use crate::{
    computed::{get_dependencies, struct_mut::ValueMut, DropResource},
    css::get_css_manager,
    dev::{
        command::{LocationSetMode, LocationTarget},
        FutureBox,
    },
    driver_module::{
        api::{api_browser_command, api_location, api_server_handler, api_timers, api_websocket},
        dom::get_driver_dom,
        utils::futures_spawn::spawn_local,
    },
    fetch::request_builder::{RequestBody, RequestBuilder},
    Context, Css, DomNode, Instant, InstantType, JsJson, WebsocketMessage,
};

use super::api::DomAccess;

/// Placeholder where to put public build path at runtime (default /build)
pub const VERTIGO_PUBLIC_BUILD_PATH_PLACEHOLDER: &str = "%%VERTIGO_PUBLIC_BUILD_PATH%%";

/// Placeholder where to put public mount point at runtime (default /)
pub const VERTIGO_MOUNT_POINT_PLACEHOLDER: &str = "%%VERTIGO_MOUNT_POINT%%";

#[derive(AutoJsJson, Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
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

/// Result from request made using [RequestBuilder].
///
/// Variants:
/// - `Ok(status_code, response)` if request succeeded,
/// - `Err(response)` if request failed (because of network error for example).
pub type FetchResult = Result<(u32, RequestBody), String>;

/// Getter for [Driver] singleton.
///
/// ```rust
/// use vertigo::get_driver;
///
/// let number = get_driver().get_random(1, 10);
/// ```
#[store]
pub fn get_driver() -> Rc<Driver> {
    let spawn_executor = {
        Rc::new(move |fut: Pin<Box<dyn Future<Output = ()> + 'static>>| {
            spawn_local(fut);
        })
    };

    let subscribe = get_dependencies().hooks.on_after_transaction(move || {
        get_driver_dom().flush_dom_changes();
    });

    Rc::new(Driver {
        spawn_executor,
        _subscribe: subscribe,
        subscription: ValueMut::new(None),
    })
}

/// Do bunch of operations on dependency graph without triggering anything in between.
pub fn transaction<R, F: FnOnce(&Context) -> R>(f: F) -> R {
    get_driver().transaction(f)
}

/// Set of functions to communicate with the browser.
pub struct Driver {
    spawn_executor: Rc<Executable>,
    _subscribe: DropResource,
    subscription: ValueMut<Option<DomNode>>,
}

impl Driver {
    pub(crate) fn set_root(&self, root_view: DomNode) {
        self.subscription.set(Some(root_view));
    }

    /// Gets a cookie by name
    pub fn cookie_get(&self, cname: &str) -> String {
        api_browser_command().cookie_get(cname.into())
    }

    /// Gets a JsJson cookie by name
    pub fn cookie_get_json(&self, cname: &str) -> JsJson {
        api_browser_command().cookie_json_get(cname.into())
    }

    /// Sets a cookie under provided name
    pub fn cookie_set(&self, cname: &str, cvalue: &str, expires_in: u64) {
        api_browser_command().cookie_set(cname.into(), cvalue.into(), expires_in);
    }

    /// Sets a cookie under provided name
    pub fn cookie_set_json(&self, cname: &str, cvalue: JsJson, expires_in: u64) {
        api_browser_command().cookie_json_set(cname.into(), cvalue, expires_in);
    }

    /// Go back in client's (browser's) history
    pub fn history_back(&self) {
        api_browser_command().history_back();
    }

    /// Replace current location
    pub fn history_replace(&self, new_url: &str) {
        api_location().push_location(LocationTarget::History, LocationSetMode::Replace, new_url);
    }

    /// Make `func` fire every `time` seconds.
    #[must_use]
    pub fn set_interval(&self, time: u32, func: impl Fn() + 'static) -> DropResource {
        api_timers().interval(time, func)
    }

    /// Gets current value of monotonic clock.
    pub fn now(&self) -> Instant {
        Instant::now()
    }

    /// Gets current UTC timestamp
    pub fn utc_now(&self) -> InstantType {
        api_browser_command().get_date_now()
    }

    /// Gets browsers time zone offset in seconds
    ///
    /// Compatible with chrono's `FixedOffset::east_opt` method.
    pub fn timezone_offset(&self) -> i32 {
        api_browser_command().timezone_offset()
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

        api_timers().set_timeout_and_detach(time, move || {
            sender.publish(());
        });

        future
    }

    pub fn get_random(&self, min: u32, max: u32) -> u32 {
        api_browser_command().get_random(min, max)
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
        api_websocket().websocket(host, callback)
    }

    /// Spawn a future - thus allowing to fire async functions in, for example, event handler. Handy when fetching resources from internet.
    pub fn spawn(&self, future: impl Future<Output = ()> + 'static) {
        let future = Box::pin(future);
        let spawn_executor = self.spawn_executor.clone();
        spawn_executor(future);
    }

    /// Fire provided function in a way that all changes in [dependency graph](struct.Dependencies.html) made by this function
    /// will trigger only one run of updates, just like the changes were done all at once.
    pub fn transaction<R, F: FnOnce(&Context) -> R>(&self, func: F) -> R {
        get_dependencies().transaction(func)
    }

    /// Allows to access different objects in the browser (See [js!](crate::js) macro for convenient use).
    pub fn dom_access(&self) -> DomAccess {
        DomAccess::default()
    }

    /// Function added for diagnostic purposes. It allows you to check whether a block with a transaction is missing somewhere.
    pub fn on_after_transaction(&self, callback: impl Fn() + 'static) -> DropResource {
        get_dependencies().hooks.on_after_transaction(callback)
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
        api_browser_command().is_browser()
    }

    pub fn is_server(&self) -> bool {
        !self.is_browser()
    }

    /// Get any env variable set upon starting vertigo server.
    pub fn env(&self, name: impl Into<String>) -> Option<String> {
        let name = name.into();
        api_browser_command().get_env(name)
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

        if api_browser_command().is_browser() {
            // In the browser use env variable attached during SSR
            let mount_point = api_browser_command()
                .get_env("vertigo-mount-point")
                .unwrap_or_else(|| "/".to_string());
            if mount_point != "/" {
                path.trim_start_matches(&mount_point).to_string()
            } else {
                path
            }
        } else {
            // On the server no need to do anything
            path
        }
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
    pub fn plains(&self, callback: impl Fn(&str) -> Option<String> + 'static) {
        api_server_handler().plains(callback);
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
            api_browser_command().set_status(status);
        }
    }

    /// Adds this CSS to manager producing a class name, which is returned
    ///
    /// There shouldn't be need to use it manually. It's used by `css!` macro.
    pub fn class_name_for(&self, css: &Css) -> String {
        get_css_manager().get_class_name(css)
    }

    /// Register css bundle
    ///
    /// There shouldn't be need to use it manually. It's used by `main!` macro.
    pub fn register_bundle(&self, bundle: impl Into<String>) {
        get_css_manager().register_bundle(bundle.into())
    }
}
