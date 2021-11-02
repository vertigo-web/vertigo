
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
    interval::DriverBrowserInterval
};

use vertigo::{
    DomDriver,
    DomDriverTrait,
    EventCallback,
    FetchMethod,
    FetchResult,
    InstantType,
    NodeRefsItem,
    RealDomId,
    computed::Dependencies,
    utils::DropResource
};

struct DriverBrowserInner {
    driver_dom: DriverBrowserDom,
    driver_interval: DriverBrowserInterval,
    driver_hashrouter: DriverBrowserHashrouter,
    driver_fetch: DriverBrowserFetch,
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
        }
    }
}

#[derive(Clone)]
pub struct DriverBrowser {
    driver: Rc<DriverBrowserInner>,
}

impl DriverBrowser {
    pub fn new(dependencies: &Dependencies) -> DomDriver {
        let driver = DriverBrowserInner::new(dependencies);

        let dom_driver_browser = DriverBrowser {
            driver: Rc::new(driver),
        };

        DomDriver::new(dom_driver_browser, Box::new(|fut: Pin<Box<dyn Future<Output = ()> + 'static>>| {
            wasm_bindgen_futures::spawn_local(fut);
        }))
    }
}

impl DomDriverTrait for DriverBrowser {
    fn create_node(&self, id: RealDomId, name: &'static str) {
        self.driver.driver_dom.create_node(id, name);
    }

    fn rename_node(&self, id: RealDomId, new_name: &'static str) {
        self.driver.driver_dom.rename_node(id, new_name);
    }

    fn create_text(&self, id: RealDomId, value: &str) {
        self.driver.driver_dom.create_text(id, value);
    }

    fn get_ref(&self, id: RealDomId) -> Option<NodeRefsItem> {
        self.driver.driver_dom.get_ref(id)
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
}
