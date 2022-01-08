use std::{
    collections::HashMap,
    future::Future,
    pin::Pin,
    rc::Rc, cell::RefCell
};
use vertigo::{
    dev::{DriverTrait, EventCallback, FetchMethod, RealDomId, RefsContext, WebsocketMessageDriver},
    Dependencies, DropResource, Driver, FetchResult, InstantType, Client,
};

use crate::{api::ApiImport, utils::futures_spawn::spawn_local, init_env::init_env};
use crate::modules::{
    dom::DriverBrowserDom,
    fetch::DriverBrowserFetch,
    hashrouter::DriverBrowserHashrouter,
    interval::DriverBrowserInterval,
    websocket::DriverWebsocket,
};

#[derive(Clone)]
pub struct DriverBrowserInner {
    api: Rc<ApiImport>,
    dependencies: Dependencies,
    driver_dom: DriverBrowserDom,
    driver_interval: DriverBrowserInterval,
    driver_hashrouter: DriverBrowserHashrouter,
    driver_fetch: DriverBrowserFetch,
    driver_websocket: DriverWebsocket,
    spawn_executor: Rc<dyn Fn(Pin<Box<dyn Future<Output = ()> + 'static>>)>,
}

impl DriverBrowserInner {
    pub fn new(api: Rc<ApiImport>) -> Self {
        let dependencies = Dependencies::default();
        let driver_dom = DriverBrowserDom::new(&dependencies, &api);
        let driver_interval = DriverBrowserInterval::new(&api);
        let driver_hashrouter = DriverBrowserHashrouter::new(&api);
        let driver_fetch = DriverBrowserFetch::new(&api);
        let driver_websocket = DriverWebsocket::new(&api);
        let spawn_executor = {
            let driver_interval = driver_interval.clone();

            Rc::new(move |fut: Pin<Box<dyn Future<Output = ()> + 'static>>| {
                spawn_local(driver_interval.clone(), fut);
            })
        };

        DriverBrowserInner {
            api,
            dependencies,
            driver_dom,
            driver_interval,
            driver_hashrouter,
            driver_fetch,
            driver_websocket,
            spawn_executor
        }
    }

    fn pop_string(&self) -> String {
        self.api.stack.pop()
    }

    pub fn alloc(&self, len: u64) -> u64 {
        self.api.stack.alloc(len as usize) as u64
    }

    pub fn alloc_empty_string(&self) {
        self.api.stack.alloc_empty_string()
    }

    pub fn export_interval_run_callback(&self, callback_id: u32) {
        self.driver_interval.export_interval_run_callback(callback_id);
    }

    pub fn export_timeout_run_callback(&self, callback_id: u32) {
        self.driver_interval.export_timeout_run_callback(callback_id);
    }

    pub fn export_hashrouter_hashchange_callback(&self) {
        let new_hash = self.pop_string();
        self.driver_hashrouter.export_hashrouter_hashchange_callback(new_hash);
    }

    pub fn export_fetch_callback(&self, request_id: u32, success: u32, status: u32) {
        let success = success > 0;
        let response = self.pop_string();
        self.driver_fetch.export_fetch_callback(request_id, success, status, response);
    }

    pub fn export_websocket_callback_socket(&self, callback_id: u32) {
        self.driver_websocket.export_websocket_callback_socket(callback_id);
    }

    pub fn export_websocket_callback_message(&self, callback_id: u32) {
        let message = self.pop_string();
        self.driver_websocket.export_websocket_callback_message(callback_id, message);
    }

    pub fn export_websocket_callback_close(&self, callback_id: u32) {
        self.driver_websocket.export_websocket_callback_close(callback_id);
    }

    pub fn export_dom_keydown(
        &self,
        dom_id: u64,                                                                         // 0 - null
        alt_key: u32,                                                                        // 0 - false, >0 - true
        ctrl_key: u32,                                                                       // 0 - false, >0 - true
        shift_key: u32,                                                                      // 0 - false, >0 - true
        meta_key: u32                                                                        // 0 - false, >0 - true
    ) -> u32 {
        let code = self.pop_string();
        let key = self.pop_string();
    
        let dom_id = if dom_id == 0 { None } else { Some(dom_id) };
        let alt_key = alt_key > 0;
        let ctrl_key = ctrl_key > 0;
        let shift_key = shift_key > 0;
        let meta_key = meta_key > 0;
    
        let prevent_default = self.driver_dom.export_dom_keydown(
            dom_id,
            key,
            code,
            alt_key,
            ctrl_key,
            shift_key,
            meta_key
        );
        
        match prevent_default {
            true => 1,
            false => 0
        }
    }
    
    pub fn export_dom_oninput(&self, dom_id: u64) {
        let text = self.pop_string();
        self.driver_dom.export_dom_oninput(dom_id, text);
    }

    pub fn export_dom_mouseover(&self, dom_id: u64) {
        let dom_id = if dom_id == 0 { None } else { Some(dom_id) };
        self.driver_dom.export_dom_mouseover(dom_id);
    }

    pub fn export_dom_mousedown(&self, dom_id: u64) {
        self.driver_dom.export_dom_mousedown(dom_id);
    }

    pub fn init_env(&self) {
        init_env(self.api.logger.clone());
    }
}

pub struct DriverConstruct {
    pub driver_inner: Rc<DriverBrowserInner>,
    pub driver: Driver,
    pub subscription: RefCell<Option<Client>>,
}

impl DriverConstruct {
    pub fn new(api: ApiImport) -> DriverConstruct {
        let (driver_inner, driver) = DriverBrowser::new(api);

        DriverConstruct {
            driver_inner,
            driver,
            subscription: RefCell::new(None),
        }
    }
}

/// Implementation of vertigo driver for web browsers.
#[derive(Clone)]
pub struct DriverBrowser {
    driver: Rc<DriverBrowserInner>,
}

impl DriverBrowser {
    #[allow(clippy::new_ret_no_self)]
    fn new(api: ApiImport) -> (Rc<DriverBrowserInner>, Driver) {
        let driver_inner = Rc::new(DriverBrowserInner::new(Rc::new(api)));
        let dependencies = driver_inner.dependencies.clone();

        let dom_driver_browser = DriverBrowser {
            driver: driver_inner.clone(),
        };

        let driver = Driver::new(
            dependencies,
            dom_driver_browser,
        );

        (driver_inner, driver)
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
        body: Option<String>,
    ) -> Pin<Box<dyn Future<Output = FetchResult> + 'static>> {
        self.driver.driver_fetch.fetch(method, url, headers, body)
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
        self.driver.api.instant_now()
    }

    fn websocket(&self, host: String, callback: Box<dyn Fn(WebsocketMessageDriver)>) -> DropResource {
        self.driver.driver_websocket.websocket_start(host, callback)
    }

    fn websocket_send_message(&self, callback_id: u32, message: String) {
        self.driver.driver_websocket.websocket_send_message(callback_id, message);
    }

    fn get_bounding_client_rect_x(&self, id: RealDomId) -> i32 {
        self.driver.driver_dom.get_bounding_client_rect_x(id)
    }

    fn get_bounding_client_rect_y(&self, id: RealDomId) -> i32 {
        self.driver.driver_dom.get_bounding_client_rect_y(id)
    }

    fn get_bounding_client_rect_width(&self, id: RealDomId) -> u32 {
        self.driver.driver_dom.get_bounding_client_rect_width(id)
    }

    fn get_bounding_client_rect_height(&self, id: RealDomId) -> u32 {
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

    fn scroll_width(&self, id: RealDomId) -> u32 {
        self.driver.driver_dom.scroll_width(id)
    }

    fn scroll_height(&self, id: RealDomId) -> u32 {
        self.driver.driver_dom.scroll_height(id)
    }

    fn push_ref_context(&self, context: RefsContext) {
        self.driver.driver_dom.push_ref_context(context);
    }

    fn flush_update(&self) {
        self.driver.driver_dom.flush_dom_changes();
    }

    fn spawn(&self, fut: Pin<Box<dyn Future<Output = ()> + 'static>>) {
        let spawn_executor = self.driver.spawn_executor.clone();
        spawn_executor(fut);
    }
}
