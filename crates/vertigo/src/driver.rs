use std::{
    future::Future,
    rc::Rc
};
use crate::{
    Dependencies, Instant, KeyDownEvent, WebsocketConnection, WebsocketMessage,
    dev::WebsocketMessageDriver,
    driver_refs::RefsContext,
    fetch::{fetch_builder::FetchBuilder, request_builder::RequestBuilder},
    DropResource,
    virtualdom::models::{realdom_id::RealDomId},
    driver_module::driver_browser::DriverBrowser, ApiImport,
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
            }
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
pub struct Driver {
    pub(crate) inner: DriverBrowser,
}

impl Clone for Driver {
    fn clone(&self) -> Driver {
        Driver {
            inner: self.inner.clone(),
        }
    }
}

impl Driver {
    pub fn new(api: ApiImport) -> Driver {
        let driver_browser = DriverBrowser::new(api);

        Driver {
            inner: driver_browser,
        }
    }

    /// Spawn a future - thus allowing to fire async functions in, for example, event handler. Handy when fetching resources from internet.
    pub fn spawn(&self, future: impl Future<Output = ()> + 'static) {
        let future_box = Box::pin(future);
        self.inner.spawn(future_box);
    }

    /// Create new FetchBuilder.
    #[must_use]
    pub fn fetch(&self, url: impl Into<String>) -> FetchBuilder {
        FetchBuilder::new(self.inner.clone(), url.into())
    }

    /// Gets a cookie by name
    pub fn cookie_get(&self, cname: &str) -> String {
        self.inner.cookie_get(cname)
    }

    /// Sets a cookie under provided name
    pub fn cookie_set(&self, cname: &str, cvalue: &str, expires_in: u64) {
        self.inner.cookie_set(cname, cvalue, expires_in)
    }

    /// Retrieves the hash part of location URL from client (browser)
    pub fn get_hash_location(&self) -> String {
        self.inner.get_hash_location()
    }

    /// Sets the hash part of location URL from client (browser)
    pub fn push_hash_location(&self, path: String) {
        self.inner.push_hash_location(&path)
    }

    /// Set event handler upon hash location change
    #[must_use]
    pub fn on_hash_route_change(&self, on_change: Box<dyn Fn(&String)>) -> DropResource {
        self.inner.on_hash_route_change(on_change)
    }

    /// Make `func` fire every `time` seconds.
    #[must_use]
    pub fn set_interval(&self, time: u32, func: impl Fn() + 'static) -> DropResource {
        self.inner.set_interval(time, Box::new(func))
    }

    pub async fn sleep(&self, time: u32) {
        self.inner.sleep(time).await;
    }

    /// Gets current value of monotonic clock.
    pub fn now(&self) -> Instant {
        Instant::now(self.inner.clone())
    }

    /// Create new RequestBuilder (more complex version of [fetch](struct.Driver.html#method.fetch))
    #[must_use]
    pub fn request(&self, url: impl Into<String>) -> RequestBuilder {
        RequestBuilder::new(self, url)
    }

    /// Initiate a websocket connection. Provided callback should handle a single [WebsocketMessage].
    #[must_use]
    pub fn websocket(&self, host: impl Into<String>, callback: Box<dyn Fn(WebsocketMessage)>) -> DropResource {
        let host: String = host.into();

        self.inner.websocket(
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

    pub(crate) fn websocket_send_message(&self, callback_id: u32, message: String) {
        self.inner.websocket_send_message(callback_id, message);
    }

    /// Fire provided function in a way that all changes in [dependency graph](struct.Dependencies.html) made by this function
    /// will trigger only one run of updates, just like the changes were done all at once.
    pub fn transaction<F: FnOnce()>(&self, func: F) {
        self.inner.driver.dependencies.transaction(func);
    }

    // Below - methods to interact with the dom. Used exclusively by vertigo.

    pub(crate) fn create_node(&self, id: RealDomId, name: &'static str) {
        self.inner.create_node(id, name);
    }

    pub(crate) fn rename_node(&self, id: RealDomId, new_name: &'static str) {
        self.inner.rename_node(id, new_name);
    }

    pub(crate) fn create_text(&self, id: RealDomId, value: &str) {
        self.inner.create_text(id, value);
    }

    pub(crate) fn update_text(&self, id: RealDomId, value: &str) {
        self.inner.update_text(id, value);
    }

    pub(crate) fn set_attr(&self, id: RealDomId, key: &'static str, value: &str) {
        self.inner.set_attr(id, key, value);
    }

    pub(crate) fn remove_attr(&self, id: RealDomId, name: &'static str) {
        self.inner.remove_attr(id, name);
    }

    pub(crate) fn remove_node(&self, id: RealDomId) {
        self.inner.remove_node(id);
    }

    pub(crate) fn remove_text(&self, id: RealDomId) {
        self.inner.remove_text(id);
    }

    pub(crate) fn insert_before(&self, parent: RealDomId, child: RealDomId, ref_id: Option<RealDomId>) {
        self.inner.insert_before(parent, child, ref_id);
    }

    pub(crate) fn insert_css(&self, selector: &str, value: &str) {
        self.inner.insert_css(selector, value);
    }

    pub(crate) fn set_event(&self, node: RealDomId, callback: EventCallback) {
        self.inner.set_event(node, callback);
    }

    pub(crate) fn get_bounding_client_rect_x(&self, id: RealDomId) -> i32 {
        self.inner.get_bounding_client_rect_x(id)
    }

    pub(crate) fn get_bounding_client_rect_y(&self, id: RealDomId) -> i32 {
        self.inner.get_bounding_client_rect_y(id)
    }

    pub(crate) fn get_bounding_client_rect_width(&self, id: RealDomId) -> u32 {
        self.inner.get_bounding_client_rect_width(id)
    }

    pub(crate) fn get_bounding_client_rect_height(&self, id: RealDomId) -> u32 {
        self.inner.get_bounding_client_rect_height(id)
    }

    pub(crate) fn scroll_top(&self, id: RealDomId) -> i32 {
        self.inner.scroll_top(id)
    }

    pub(crate) fn set_scroll_top(&self, id: RealDomId, value: i32) {
        self.inner.set_scroll_top(id, value);
    }

    pub(crate) fn scroll_left(&self, id: RealDomId) -> i32 {
        self.inner.scroll_left(id)
    }

    pub(crate) fn set_scroll_left(&self, id: RealDomId, value: i32) {
        self.inner.set_scroll_left(id, value);
    }

    pub(crate) fn scroll_width(&self, id: RealDomId) -> u32 {
        self.inner.scroll_width(id)
    }

    pub(crate) fn scroll_height(&self, id: RealDomId) -> u32 {
        self.inner.scroll_height(id)
    }

    pub(crate) fn push_ref_context(&self, context: RefsContext) {
        self.inner.push_ref_context(context);
    }

    pub(crate) fn flush_update(&self) {
        self.inner.flush_update();
    }

    pub(crate) fn get_dependencies(&self) -> Dependencies {
        self.inner.driver.dependencies.clone()
    }

    pub(crate) fn external_connections_refresh(&self) {
        self.inner.driver.dependencies.external_connections_refresh();
    }

    pub fn spawn_bind2<
        T1: Clone,
        T2: Clone,
        Fut: Future<Output=()> + 'static,
        F: Fn(T1, T2) -> Fut
    >(&self, param1: &T1, param2: &T2, fun: F) -> impl Fn() {
        let driver = self.clone();

        let param1 = param1.clone();
        let param2 = param2.clone();

        move || {
            let param1 = param1.clone();
            let param2 = param2.clone();
            let future = fun(param1, param2);
            driver.spawn(future);
        }
    }
}
