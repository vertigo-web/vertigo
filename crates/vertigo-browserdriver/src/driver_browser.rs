
use std::{
    collections::HashMap,
    future::Future,
    pin::Pin,
    rc::Rc
};
use crate::modules::{
    dom::DriverBrowserDom,
    fetch::DriverBrowserFetch,
    hashrouter::DriverBrowserHashrouter,
    instant::js_instant,
    interval::DriverBrowserInterval,
    websocket::DriverWebsocket,
};

use vertigo::Driver;
use vertigo::DriverTrait;
use vertigo::EventCallback;
use vertigo::FetchMethod;
use vertigo::FetchResult;
use vertigo::InstantType;
use vertigo::RealDomId;
use vertigo::RefsContext;
use vertigo::WebcocketMessageDriver;
use vertigo::computed::Dependencies;
use vertigo::utils::DropResource;

struct DriverBrowserInner {
    driver_dom: DriverBrowserDom,
    driver_interval: DriverBrowserInterval,
    driver_hashrouter: DriverBrowserHashrouter,
    driver_fetch: DriverBrowserFetch,
    driver_websocket: DriverWebsocket,
}

impl DriverBrowserInner {
    fn new(dependencies: &Dependencies) -> Self {
        let driver_dom = DriverBrowserDom::new(dependencies);
        let driver_interval = DriverBrowserInterval::new();
        let driver_hashrouter = DriverBrowserHashrouter::new();

        DriverBrowserInner {
            driver_dom,
            driver_interval,
            driver_hashrouter,
            driver_fetch: DriverBrowserFetch::new(),
            driver_websocket: DriverWebsocket::new(),
        }
    }
}

#[derive(Clone)]
pub struct DriverBrowser {
    driver: Rc<DriverBrowserInner>,
}

impl DriverBrowser {
    pub fn new() -> Driver {
        let dependencies = Dependencies::default();

        let driver = DriverBrowserInner::new(&dependencies);

        let dom_driver_browser = DriverBrowser {
            driver: Rc::new(driver),
        };

        Driver::new(dependencies, dom_driver_browser, Box::new(|fut: Pin<Box<dyn Future<Output = ()> + 'static>>| {
            wasm_bindgen_futures::spawn_local(fut);
        }))
    }
}

impl DriverTrait for DriverBrowser {
    fn create_node(&self, id: RealDomId, name: &'static str) {
        self.driver.driver_dom.create_node(id, name);
    }

    fn rename_node(&self, id: RealDomId, new_name: &'static str) {
        self.driver.driver_dom.rename_node(id, new_name);
    }

    fn create_text(&self, id: RealDomId, value: &str) {
        self.driver.driver_dom.create_text(id, value);
    }

    fn update_text(&self, id: RealDomId, value: &str) {
        self.driver.driver_dom.update_text(id, value);
    }

    fn set_attr(&self, id: RealDomId, key: &'static str, value: &str) {
        self.driver.driver_dom.set_attr(id, key, value);
    }

    fn remove_attr(&self, id: RealDomId, name: &'static str) {
        self.driver.driver_dom.remove_attr(id, name);
    }

    fn remove_node(&self, id: RealDomId) {
        self.driver.driver_dom.remove_node(id);
    }

    fn remove_text(&self, id: RealDomId) {
        self.driver.driver_dom.remove_text(id);
    }

    fn insert_before(&self, parent: RealDomId, child: RealDomId, ref_id: Option<RealDomId>) {
        self.driver.driver_dom.insert_before(parent, child, ref_id);
    }

    fn insert_css(&self, selector: &str, value: &str) {
        self.driver.driver_dom.insert_css(selector, value);
    }

    fn set_event(&self, id: RealDomId, callback: EventCallback) {
        self.driver.driver_dom.set_event(id, callback);
    }

    fn fetch(
        &self,
        method: FetchMethod,
        url: String,
        headers: Option<HashMap<String, String>>,
        body: Option<String>
    ) -> Pin<Box<dyn Future<Output=FetchResult> + 'static>> {
        self.driver.driver_fetch.fetch(
            method,
            url,
            headers,
            body
        )
    }

    fn get_hash_location(&self) -> String {
        self.driver.driver_hashrouter.get_hash_location()
    }

    fn push_hash_location(&self, path: &str) {
        self.driver.driver_hashrouter.push_hash_location(path);
    }

    fn on_hash_route_change(&self, on_change: Box<dyn Fn(&String)>) -> DropResource {
        self.driver.driver_hashrouter.on_hash_route_change(on_change)
    }

    fn set_interval(&self, time: u32, func: Box<dyn Fn()>) -> DropResource {
        self.driver.driver_interval.set_interval(time, move |_| {
            func();
        })
    }

    fn now(&self) -> InstantType {
        js_instant::now().round() as InstantType
    }

    fn websocket(&self, host: String, callback: Box<dyn Fn(WebcocketMessageDriver)>) -> DropResource {
        self.driver.driver_websocket.websocket_start(host, callback)
    }

    fn websocket_send_message(&self, callback_id: u64, message: String) {
        self.driver.driver_websocket.websocket_send_message(callback_id, message);
    }

    fn get_bounding_client_rect_x(&self, id: RealDomId) -> f64 {
        self.driver.driver_dom.get_bounding_client_rect_x(id)
    }

    fn get_bounding_client_rect_y(&self, id: RealDomId) -> f64 {
        self.driver.driver_dom.get_bounding_client_rect_y(id)
    }

    fn get_bounding_client_rect_width(&self, id: RealDomId) -> f64 {
        self.driver.driver_dom.get_bounding_client_rect_width(id)
    }

    fn get_bounding_client_rect_height(&self, id: RealDomId) -> f64 {
        self.driver.driver_dom.get_bounding_client_rect_height(id)
    }

    fn scroll_top(&self, id: RealDomId) -> i32 {
        self.driver.driver_dom.scroll_top(id)
    }

    fn set_scroll_top(&self, id: RealDomId, value: i32) {
        self.driver.driver_dom.set_scroll_top(id, value)
    }

    fn scroll_left(&self, id: RealDomId) -> i32 {
        self.driver.driver_dom.scroll_left(id)
    }

    fn set_scroll_left(&self, id: RealDomId, value: i32) {
        self.driver.driver_dom.set_scroll_left(id, value)
    }

    fn scroll_width(&self, id: RealDomId) -> i32 {
        self.driver.driver_dom.scroll_width(id)
    }

    fn scroll_height(&self, id: RealDomId) -> i32 {
        self.driver.driver_dom.scroll_height(id)
    }

    fn push_ref_context(&self, context: RefsContext) {
        self.driver.driver_dom.push_ref_context(context);
    }

    fn flush_update(&self) {
        self.driver.driver_dom.flush_dom_changes();
    }
}
