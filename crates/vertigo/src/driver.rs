use std::{
    collections::HashMap,
    future::Future,
    pin::Pin,
    rc::Rc
};

use crate::{
    Computed, Dependencies, Value, Instant, InstantType, KeyDownEvent, WebsocketConnection, WebsocketMessage,
    dev::WebsocketMessageDriver,
    driver_refs::RefsContext,
    fetch::{fetch_builder::FetchBuilder, request_builder::RequestBuilder},
    DropResource,
    virtualdom::models::{realdom_id::RealDomId},
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

pub trait DriverTrait {
    fn create_text(&self, id: RealDomId, value: &str);
    fn update_text(&self, id: RealDomId, value: &str);
    fn remove_text(&self, id: RealDomId);
    fn create_node(&self, id: RealDomId, name: &'static str);
    fn rename_node(&self, id: RealDomId, new_name: &'static str);
    fn set_attr(&self, id: RealDomId, key: &'static str, value: &str);
    fn remove_attr(&self, id: RealDomId, name: &'static str);
    fn remove_node(&self, id: RealDomId);
    fn insert_before(&self, parent: RealDomId, child: RealDomId, ref_id: Option<RealDomId>);
    fn insert_css(&self, selector: &str, value: &str);
    fn set_event(&self, node: RealDomId, callback: EventCallback);

    fn get_bounding_client_rect_x(&self, id: RealDomId) -> i32;
    fn get_bounding_client_rect_y(&self, id: RealDomId) -> i32;
    fn get_bounding_client_rect_width(&self, id: RealDomId) -> u32;
    fn get_bounding_client_rect_height(&self, id: RealDomId) -> u32;
    fn scroll_top(&self, id: RealDomId) -> i32;
    fn set_scroll_top(&self, id: RealDomId, value: i32);
    fn scroll_left(&self, id: RealDomId) -> i32;
    fn set_scroll_left(&self, id: RealDomId, value: i32);
    fn scroll_width(&self, id: RealDomId) -> u32;
    fn scroll_height(&self, id: RealDomId) -> u32;

    fn fetch(
        &self,
        method: FetchMethod,
        url: String,
        headers: Option<HashMap<String, String>>,
        body: Option<String>,
    ) -> Pin<Box<dyn Future<Output = FetchResult> + 'static>>;

    fn cookie_get(&self, cname: &str) -> String;
    fn cookie_set(&self, cname: &str, cvalue: &str, expires_in: u64);
    fn get_hash_location(&self) -> String;
    fn push_hash_location(&self, path: &str);
    fn on_hash_route_change(&self, on_change: Box<dyn Fn(&String)>) -> DropResource;

    fn set_interval(&self, time: u32, func: Box<dyn Fn()>) -> DropResource;
    fn now(&self) -> InstantType;

    fn websocket(&self, host: String, callback: Box<dyn Fn(WebsocketMessageDriver)>) -> DropResource;
    fn websocket_send_message(&self, callback_id: u32, message: String);

    fn push_ref_context(&self, context: RefsContext);
    fn flush_update(&self);
    fn spawn(&self, fut: Pin<Box<dyn Future<Output = ()> + 'static>>);
}

pub struct DriverInner {
    dependencies: Dependencies,
    driver: Rc<dyn DriverTrait>,
}

impl DriverInner {
    pub fn new(dependencies: Dependencies, driver: impl DriverTrait + 'static) -> DriverInner {
        DriverInner {
            driver: Rc::new(driver),
            dependencies,
        }
    }
}

/// Main connection to vertigo facilities - dependencies and rendering client (the browser).
///
/// This is in fact only a box for inner generic driver.
/// This way a web developer don't need to worry about the specific driver used,
/// though usually it is the [BrowserDriver](../vertigo_browserdriver/struct.DriverBrowser.html)
/// which is used to create a Driver.
///
/// Additionally driver struct wraps [Dependencies] object.
pub struct Driver {
    inner: Rc<DriverInner>,
}

impl Clone for Driver {
    fn clone(&self) -> Driver {
        Driver {
            inner: self.inner.clone(),
        }
    }
}

impl Driver {
    pub fn new(dependencies: Dependencies, driver: impl DriverTrait + 'static) -> Driver {
        Driver {
            inner: Rc::new(DriverInner::new(dependencies, driver)),
        }
    }

    /// Spawn a future - thus allowing to fire async functions in, for example, event handler. Handy when fetching resources from internet.
    pub fn spawn(&self, future: impl Future<Output = ()> + 'static) {
        let future_box = Box::pin(future);
        self.inner.driver.spawn(future_box);
    }

    /// Create new FetchBuilder.
    pub fn fetch(&self, url: impl Into<String>) -> FetchBuilder {
        FetchBuilder::new(self.inner.driver.clone(), url.into())
    }

    /// Gets a cookie by name
    pub fn cookie_get(&self, cname: &str) -> String {
        self.inner.driver.cookie_get(cname)
    }

    /// Sets a cookie under provided name
    pub fn cookie_set(&self, cname: &str, cvalue: &str, expires_in: u64) {
        self.inner.driver.cookie_set(cname, cvalue, expires_in)
    }

    /// Retrieves the hash part of location URL from client (browser)
    pub fn get_hash_location(&self) -> String {
        self.inner.driver.get_hash_location()
    }

    /// Sets the hash part of location URL from client (browser)
    pub fn push_hash_location(&self, path: String) {
        self.inner.driver.push_hash_location(&path)
    }

    /// Set event handler upon hash location change
    pub fn on_hash_route_change(&self, on_change: Box<dyn Fn(&String)>) -> DropResource {
        self.inner.driver.on_hash_route_change(on_change)
    }

    /// Make `func` fire every `time` seconds.
    pub fn set_interval(&self, time: u32, func: impl Fn() + 'static) -> DropResource {
        self.inner.driver.set_interval(time, Box::new(func))
    }

    /// Gets current value of monotonic clock.
    pub fn now(&self) -> Instant {
        Instant::now(self.inner.driver.clone())
    }

    /// Create new RequestBuilder (more complex version of [fetch](struct.Driver.html#method.fetch))
    pub fn request(&self, url: impl Into<String>) -> RequestBuilder {
        RequestBuilder::new(self, url)
    }

    /// Initiate a websocket connection. Provided callback should handle a single [WebsocketMessage].
    pub fn websocket(&self, host: impl Into<String>, callback: Box<dyn Fn(WebsocketMessage)>) -> DropResource {
        let driver = self.clone();
        let host: String = host.into();

        self.inner.driver.websocket(
            host,
            Box::new(move |message: WebsocketMessageDriver| {
                let message = match message {
                    WebsocketMessageDriver::Connection { callback_id } => {
                        let connection = WebsocketConnection::new(callback_id, driver.clone());
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
        self.inner.driver.websocket_send_message(callback_id, message);
    }

    /// Create new reactive value in [dependency graph](struct.Dependencies.html).
    pub fn new_value<T: PartialEq>(&self, value: T) -> Value<T> {
        self.inner.dependencies.new_value(value)
    }

    /// Fire provided function in a way that all changes in [dependency graph](struct.Dependencies.html) made by this function
    /// will trigger only one run of updates, just like the changes were done all at once.
    pub fn transaction<F: FnOnce()>(&self, func: F) {
        self.inner.dependencies.transaction(func);
    }

    /// Create a value that is connected to a generator, where `value` parameter is a starting value, and `create` function takes care of updating it.
    ///
    /// See [game of life](../src/vertigo_demo/app/game_of_life/mod.rs.html#54) example.
    pub fn new_with_connect<T, F>(&self, value: T, create: F) -> Computed<T>
    where
        T: PartialEq,
        F: Fn(&Value<T>) -> DropResource + 'static,
    {
        self.inner.dependencies.new_with_connect(value, create)
    }

    /// Create new computed value calculated using provided function.
    pub fn from<T: PartialEq + 'static, F: Fn() -> T + 'static>(&self, calculate: F) -> Computed<T> {
        self.inner.dependencies.from(calculate)
    }

    // Below - methods to interact with the dom. Used exclusively by vertigo.

    pub(crate) fn create_node(&self, id: RealDomId, name: &'static str) {
        self.inner.driver.create_node(id, name);
    }

    pub(crate) fn rename_node(&self, id: RealDomId, new_name: &'static str) {
        self.inner.driver.rename_node(id, new_name);
    }

    pub(crate) fn create_text(&self, id: RealDomId, value: &str) {
        self.inner.driver.create_text(id, value);
    }

    pub(crate) fn update_text(&self, id: RealDomId, value: &str) {
        self.inner.driver.update_text(id, value);
    }

    pub(crate) fn set_attr(&self, id: RealDomId, key: &'static str, value: &str) {
        self.inner.driver.set_attr(id, key, value);
    }

    pub(crate) fn remove_attr(&self, id: RealDomId, name: &'static str) {
        self.inner.driver.remove_attr(id, name);
    }

    pub(crate) fn remove_node(&self, id: RealDomId) {
        self.inner.driver.remove_node(id);
    }

    pub(crate) fn remove_text(&self, id: RealDomId) {
        self.inner.driver.remove_text(id);
    }

    pub(crate) fn insert_before(&self, parent: RealDomId, child: RealDomId, ref_id: Option<RealDomId>) {
        self.inner.driver.insert_before(parent, child, ref_id);
    }

    pub(crate) fn insert_css(&self, selector: &str, value: &str) {
        self.inner.driver.insert_css(selector, value);
    }

    pub(crate) fn set_event(&self, node: RealDomId, callback: EventCallback) {
        self.inner.driver.set_event(node, callback);
    }

    pub(crate) fn get_bounding_client_rect_x(&self, id: RealDomId) -> i32 {
        self.inner.driver.get_bounding_client_rect_x(id)
    }

    pub(crate) fn get_bounding_client_rect_y(&self, id: RealDomId) -> i32 {
        self.inner.driver.get_bounding_client_rect_y(id)
    }

    pub(crate) fn get_bounding_client_rect_width(&self, id: RealDomId) -> u32 {
        self.inner.driver.get_bounding_client_rect_width(id)
    }

    pub(crate) fn get_bounding_client_rect_height(&self, id: RealDomId) -> u32 {
        self.inner.driver.get_bounding_client_rect_height(id)
    }

    pub(crate) fn scroll_top(&self, id: RealDomId) -> i32 {
        self.inner.driver.scroll_top(id)
    }

    pub(crate) fn set_scroll_top(&self, id: RealDomId, value: i32) {
        self.inner.driver.set_scroll_top(id, value);
    }

    pub(crate) fn scroll_left(&self, id: RealDomId) -> i32 {
        self.inner.driver.scroll_left(id)
    }

    pub(crate) fn set_scroll_left(&self, id: RealDomId, value: i32) {
        self.inner.driver.set_scroll_left(id, value);
    }

    pub(crate) fn scroll_width(&self, id: RealDomId) -> u32 {
        self.inner.driver.scroll_width(id)
    }

    pub(crate) fn scroll_height(&self, id: RealDomId) -> u32 {
        self.inner.driver.scroll_height(id)
    }

    pub(crate) fn push_ref_context(&self, context: RefsContext) {
        self.inner.driver.push_ref_context(context);
    }

    pub(crate) fn flush_update(&self) {
        self.inner.driver.flush_update();
    }
}
