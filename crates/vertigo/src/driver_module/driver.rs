use std::{future::Future, pin::Pin, rc::Rc};
use vertigo_macro::{AutoJsJson, store};

use crate::{
    Context, Css, DomNode, DropResource, Instant, InstantType, JsJson, WebsocketMessage,
    css::get_css_manager,
    dev::{
        FutureBox,
        command::{LocationSetMode, LocationTarget},
    },
    driver_module::{
        api::{api_browser_command, api_location, api_server_handler, api_timers, api_websocket},
        dom::get_driver_dom,
        utils::futures_spawn::spawn_local,
    },
    fetch::request_builder::{RequestBody, RequestBuilder},
    struct_mut::ValueMut,
};

use super::api::DomAccess;

/// Placeholder where to put public build path at runtime (default /build)
pub const VERTIGO_PUBLIC_BUILD_PATH_PLACEHOLDER: &str = "%%VERTIGO_PUBLIC_BUILD_PATH%%";

/// Placeholder where to put public mount point at runtime (default /)
pub const VERTIGO_MOUNT_POINT_PLACEHOLDER: &str = "%%VERTIGO_MOUNT_POINT%%";

/// Env variable with the public path of the build directory, set by the server both for SSR
/// and for the browser - see [`Driver::public_build_path`].
pub const VERTIGO_PUBLIC_PATH_ENV: &str = "vertigo-public-path";

/// Env variable with the mount point of the app, set by the server both for SSR and for the
/// browser - see [`Driver::route_to_public`].
pub const VERTIGO_MOUNT_POINT_ENV: &str = "vertigo-mount-point";

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

    let subscribe = crate::reactive::on_after_transaction(move || {
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

    /// Hands the mounted tree back, so it can be dropped deliberately.
    ///
    /// In the browser the app owns its tree until the tab closes, so nothing calls this. On
    /// the host it matters: `Driver` lives in a thread-local, and dropping a `DomNode` emits
    /// removal commands through *another* thread-local. Left to run at thread teardown those
    /// two race on destruction order, and reaching an already-dropped store aborts the
    /// process. Tests that mount call this before they finish.
    #[cfg(test)]
    pub(crate) fn take_root(&self) -> Option<DomNode> {
        self.subscription.change(|root| root.take())
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

    /// Create new [RequestBuilder] for GETs
    ///
    /// This is a more complex version of [fetch](struct.Driver.html#method.fetch)
    #[must_use]
    pub fn request_get(&self, url: impl Into<String>) -> RequestBuilder {
        RequestBuilder::get(url)
    }

    /// Create new RequestBuilder for POSTs (more complex version of [fetch](struct.Driver.html#method.fetch))
    #[must_use]
    pub fn request_post(&self, url: impl Into<String>) -> RequestBuilder {
        RequestBuilder::post(url)
    }

    /// Create new RequestBuilder for PATCHes
    #[must_use]
    pub fn request_patch(&self, url: impl Into<String>) -> RequestBuilder {
        RequestBuilder::patch(url)
    }

    /// Create new RequestBuilder for PUTs
    #[must_use]
    pub fn request_put(&self, url: impl Into<String>) -> RequestBuilder {
        RequestBuilder::put(url)
    }

    /// Create new RequestBuilder for DELETEs
    #[must_use]
    pub fn request_delete(&self, url: impl Into<String>) -> RequestBuilder {
        RequestBuilder::delete(url)
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

    /// Fire provided function in a way that all reactive updates made by this function
    /// run once, as if the changes were done all at once.
    pub fn transaction<R, F: FnOnce(&Context) -> R>(&self, func: F) -> R {
        crate::reactive::transaction(func)
    }

    /// Allows to access different objects in the browser (See [js!](crate::js) macro for convenient use).
    pub fn dom_access(&self) -> DomAccess {
        DomAccess::default()
    }

    /// Function added for diagnostic purposes. It allows you to check whether a block with a transaction is missing somewhere.
    pub fn on_after_transaction(&self, callback: impl Fn() + 'static) -> DropResource {
        crate::reactive::on_after_transaction(callback)
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
    ///
    /// The server passes the path in env during SSR as well as to the browser, so the path is
    /// the same on both sides - also where it gets encoded, e.g. into a query string.
    pub fn public_build_path(&self, path: impl Into<String>) -> String {
        let path = path.into();
        match self.env(VERTIGO_PUBLIC_PATH_ENV) {
            Some(public_path) => path.replace(VERTIGO_PUBLIC_BUILD_PATH_PLACEHOLDER, &public_path),
            // A server that doesn't pass it replaces the placeholder in the finished HTML
            None if !self.is_browser() => path,
            // Fallback to default dest_dir
            None => path.replace(VERTIGO_PUBLIC_BUILD_PATH_PLACEHOLDER, "/build"),
        }
    }

    /// Convert relative route to public path (with mount point attached)
    ///
    /// The server passes the mount point in env during SSR as well as to the browser, so the
    /// route is the same on both sides - also where it gets encoded, e.g. as the page to return
    /// to after logging in (`/login?back=%2Fpanel%2F`).
    pub fn route_to_public(&self, path: impl Into<String>) -> String {
        let path = path.into();
        match self.env(VERTIGO_MOUNT_POINT_ENV) {
            Some(mount_point) if mount_point != "/" => [mount_point, path].concat(),
            Some(_) => path,
            // A server that doesn't pass it replaces this placeholder in the finished HTML
            None if !self.is_browser() => [VERTIGO_MOUNT_POINT_PLACEHOLDER, &path].concat(),
            None => path,
        }
    }

    /// Convert path in the url to relative route in the app.
    pub fn route_from_public(&self, path: impl Into<String>) -> String {
        let path: String = path.into();

        if api_browser_command().is_browser() {
            // In the browser use env variable attached during SSR
            let mount_point = api_browser_command()
                .get_env(VERTIGO_MOUNT_POINT_ENV)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::external_api::safe_wrappers::TEST_ENV;

    fn set_env(name: &str, value: &str) {
        TEST_ENV.with(|env| env.borrow_mut().insert(name.into(), value.into()));
    }

    #[test]
    fn public_paths_are_built_from_env_during_ssr_too() {
        let driver = get_driver();
        let style = format!("{VERTIGO_PUBLIC_BUILD_PATH_PLACEHOLDER}/style.css");

        // An older server passes neither, and replaces the placeholders in the finished HTML
        assert_eq!(
            driver.route_to_public("/users/"),
            format!("{VERTIGO_MOUNT_POINT_PLACEHOLDER}/users/")
        );
        assert_eq!(driver.public_build_path(style.clone()), style);

        set_env(VERTIGO_MOUNT_POINT_ENV, "/panel");
        set_env(VERTIGO_PUBLIC_PATH_ENV, "/panel/build");
        assert_eq!(driver.route_to_public("/users/"), "/panel/users/");
        assert_eq!(driver.public_build_path(style), "/panel/build/style.css");

        set_env(VERTIGO_MOUNT_POINT_ENV, "/");
        assert_eq!(driver.route_to_public("/users/"), "/users/");
    }
}
