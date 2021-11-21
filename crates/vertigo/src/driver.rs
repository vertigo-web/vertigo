use std::any::Any;
use std::{
    collections::HashMap,
    future::Future,
    pin::Pin,
    rc::Rc,
};
use crate::computed::{Computed, Dependencies, ToRc, Value};
use crate::Instant;
use crate::InstantType;
use crate::KeyDownEvent;
use crate::{WebcocketMessage, WebcocketMessageDriver, WebcocketConnection};
use crate::driver_refs::RefsContext;
use crate::fetch::fetch_builder::FetchBuilder;
use crate::fetch::request_builder::RequestBuilder;
use crate::utils::{DropResource, EqBox};
use crate::virtualdom::models::realdom_id::RealDomId;

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
    }
}

impl EventCallback {
    pub fn to_string(&self) -> &str {
        match self {
            EventCallback::OnClick { callback} => {
                if callback.is_some() {
                    "onClick set"
                } else {
                    "onClick clear"
                }
            },
            EventCallback::OnInput { callback } =>{
                if callback.is_some() {
                    "on_input set"
                } else {
                    "on_input clear"
                }
            },
            EventCallback::OnMouseEnter { callback } =>{
                if callback.is_some() {
                    "onMouseEnter set"
                } else {
                    "onMouseEnter clear"
                }
            },
            EventCallback::OnMouseLeave { callback } =>{
                if callback.is_some() {
                    "on_mouse_leave set"
                } else {
                    "on_mouse_leave clear"
                }
            },
            EventCallback::OnKeyDown { callback } =>{
                if callback.is_some() {
                    "OnKeyDown set"
                } else {
                    "OnKeyDown clear"
                }
            },
        }
    }
}

const SHOW_LOG: bool = false;

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

    fn get_bounding_client_rect_x(&self, id: RealDomId) -> f64;
    fn get_bounding_client_rect_y(&self, id: RealDomId) -> f64;
    fn get_bounding_client_rect_width(&self, id: RealDomId) -> f64;
    fn get_bounding_client_rect_height(&self, id: RealDomId) -> f64;
    fn scroll_top(&self, id: RealDomId) -> i32;
    fn set_scroll_top(&self, id: RealDomId, value: i32);
    fn scroll_left(&self, id: RealDomId) -> i32;
    fn set_scroll_left(&self, id: RealDomId, value: i32);
    fn scroll_width(&self, id: RealDomId) -> i32;
    fn scroll_height(&self, id: RealDomId) -> i32;

    fn fetch(&self, method: FetchMethod, url: String, headers: Option<HashMap<String, String>>, body: Option<String>) -> Pin<Box<dyn Future<Output=FetchResult> + 'static>>;

    fn get_hash_location(&self) -> String;
    fn push_hash_location(&self, path: &str);
    fn on_hash_route_change(&self, on_change: Box<dyn Fn(&String)>) -> DropResource;

    fn set_interval(&self, time: u32, func: Box<dyn Fn()>) -> DropResource;
    fn now(&self) -> InstantType;

    fn websocket(&self, host: String, callback: Box<dyn Fn(WebcocketMessageDriver)>) -> DropResource;
    fn websocket_send_message(&self, callback_id: u64, message: String);

    fn push_ref_context(&self, context: RefsContext);
    fn flush_update(&self);
}

type Executor = Box<dyn Fn(Pin<Box<dyn Future<Output = ()> + 'static>>)>;

pub struct DriverInner {
    dependencies: Dependencies,
    driver: Rc<dyn DriverTrait>,
    spawn_local_executor: Rc<Executor>,
}

impl DriverInner {
    pub fn new(dependencies: Dependencies, driver: impl DriverTrait + 'static, spawn_local: Executor) -> DriverInner {
        DriverInner {
            driver: Rc::new(driver),
            dependencies,
            spawn_local_executor: Rc::new(spawn_local),
        }
    }
}

pub fn show_log(message: String) {
    if SHOW_LOG {
        log::info!("{}", message);
    }
}

#[derive(PartialEq)]
pub struct Driver {
    inner: EqBox<Rc<DriverInner>>,
}

impl Clone for Driver {
    fn clone(&self) -> Driver {
        Driver {
            inner: self.inner.clone(),
        }
    }
}

impl Driver {
    pub fn new(dependencies: Dependencies, driver: impl DriverTrait + 'static, spawn_local: Executor) -> Driver {
        Driver {
            inner: EqBox::new(Rc::new(DriverInner::new(dependencies, driver, spawn_local))),
        }
    }

    pub fn spawn<F>(&self, future: F) where F: Future<Output = ()> + 'static {
        let fur = Box::pin(future);

        let spawn_local_executor = self.inner.spawn_local_executor.clone();
        spawn_local_executor(fur)
    }

    pub fn fetch(&self, url: impl Into<String>) -> FetchBuilder {
        FetchBuilder::new(self.inner.driver.clone(), url.into())
    }

    pub fn get_hash_location(&self) -> String {
        show_log("get_location".to_string());
        self.inner.driver.get_hash_location()
    }

    pub fn push_hash_location(&self, path: String) {
        show_log(format!("set_location {}", path));
        self.inner.driver.push_hash_location(&path)
    }

    pub fn on_hash_route_change(&self, on_change: Box<dyn Fn(&String)>) -> DropResource {
        show_log("on_route_change".to_string());
        self.inner.driver.on_hash_route_change(on_change)
    }

    pub fn set_interval(&self, time: u32, func: impl Fn() + 'static) -> DropResource {
        self.inner.driver.set_interval(time, Box::new(func))
    }

    pub fn now(&self) -> Instant {
        Instant::now(self.inner.driver.clone())
    }

    pub fn request(&self, url: impl Into<String>) -> RequestBuilder {
        RequestBuilder::new(self, url)
    }

    pub fn websocket(&self, host: impl Into<String>, callback: Box<dyn Fn(WebcocketMessage)>) -> DropResource {
        let driver = self.clone();
        let host: String = host.into();

        self.inner.driver.websocket(host, Box::new(move |message: WebcocketMessageDriver| {
            let message = match message {
                WebcocketMessageDriver::Connection{ callback_id} => {
                    let connection = WebcocketConnection::new(callback_id, driver.clone());
                    WebcocketMessage::Connection(connection)
                },
                WebcocketMessageDriver::Message(message) => WebcocketMessage::Message(message),
                WebcocketMessageDriver::Close => WebcocketMessage::Close,
            };

            callback(message);
        }))
    }

    pub(crate) fn websocket_send_message(&self, callback_id: u64, message: String) {
        self.inner.driver.websocket_send_message(callback_id, message);
    }



    pub fn new_value<T: PartialEq>(&self, value: T) -> Value<T> {
        self.inner.dependencies.new_value(value)
    }

    pub fn new_computed_from<T: PartialEq>(&self, value: impl ToRc<T>) -> Computed<T> {
        let value = self.inner.dependencies.new_value(value);
        value.to_computed()
    }

    pub fn transaction<F: FnOnce()>(&self, func: F) {
        self.inner.dependencies.transaction(func);
    }

    pub fn new_with_connect<T: PartialEq, F: Fn(&Value<T>) -> Box<dyn Any> + 'static>(&self, value: T, create: F) -> Computed<T> {
        self.inner.dependencies.new_with_connect(value, create)
    }

    pub fn from<T: PartialEq + 'static, F: Fn() -> T + 'static>(&self, calculate: F) -> Computed<T> {
        self.inner.dependencies.from(calculate)
    }


    //To interact with the dom. Used exclusively by vertigo

    pub(crate) fn create_node(&self, id: RealDomId, name: &'static str) {
        show_log(format!("create_node {} {}", id, name));
        self.inner.driver.create_node(id, name);
    }

    pub(crate) fn rename_node(&self, id: RealDomId, new_name: &'static str) {
        show_log(format!("rename_node {} {}", id, new_name));
        self.inner.driver.rename_node(id, new_name);
    }

    pub(crate) fn create_text(&self, id: RealDomId, value: &str) {
        show_log(format!("create_text {} {}", id, value));
        self.inner.driver.create_text(id, value);
    }

    pub(crate) fn update_text(&self, id: RealDomId, value: &str) {
        show_log(format!("update_text {} {}", id, value));
        self.inner.driver.update_text(id, value);
    }

    pub(crate) fn set_attr(&self, id: RealDomId, key: &'static str, value: &str) {
        show_log(format!("set_attr {} {} {}", id, key, value));
        self.inner.driver.set_attr(id, key, value);
    }

    pub(crate) fn remove_attr(&self, id: RealDomId, name: &'static str) {
        show_log(format!("remove_attr {} {}", id, name));
        self.inner.driver.remove_attr(id, name);
    }

    pub(crate) fn remove_node(&self, id: RealDomId) {
        show_log(format!("remove node {}", id));
        self.inner.driver.remove_node(id);
    }

    pub(crate) fn remove_text(&self, id: RealDomId) {
        show_log(format!("remove text {}", id));
        self.inner.driver.remove_text(id);
    }

    pub(crate) fn insert_before(&self, parent: RealDomId, child: RealDomId, ref_id: Option<RealDomId>) {
        match &ref_id {
            Some(ref_id) => {
                show_log(format!("insert_before child={} refId={}", child, ref_id));
            },
            None => {
                show_log(format!("insert_before child={} refId=None", child));
            }
        }

        self.inner.driver.insert_before(parent, child, ref_id);
    }

    pub(crate) fn insert_css(&self, selector: &str, value: &str) {
        show_log(format!("insert_css selector={} value={}", selector, value));
        self.inner.driver.insert_css(selector, value);
    }



    pub(crate) fn set_event(&self, node: RealDomId, callback: EventCallback) {
        show_log(format!("set_event {} {}", node, callback.to_string()));
        self.inner.driver.set_event(node, callback);
    }

    pub(crate) fn get_bounding_client_rect_x(&self, id: RealDomId) -> f64 {
        self.inner.driver.get_bounding_client_rect_x(id)
    }

    pub(crate) fn get_bounding_client_rect_y(&self, id: RealDomId) -> f64 {
        self.inner.driver.get_bounding_client_rect_y(id)
    }

    pub(crate) fn get_bounding_client_rect_width(&self, id: RealDomId) -> f64 {
        self.inner.driver.get_bounding_client_rect_width(id)
    }

    pub(crate) fn get_bounding_client_rect_height(&self, id: RealDomId) -> f64 {
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

    pub(crate) fn scroll_width(&self, id: RealDomId) -> i32 {
        self.inner.driver.scroll_width(id)
    }

    pub(crate) fn scroll_height(&self, id: RealDomId) -> i32 {
        self.inner.driver.scroll_height(id)
    }

    pub(crate) fn push_ref_context(&self, context: RefsContext) {
        self.inner.driver.push_ref_context(context);
    }

    pub(crate) fn flush_update(&self) {
        self.inner.driver.flush_update();
    }
}
