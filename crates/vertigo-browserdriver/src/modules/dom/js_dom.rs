use wasm_bindgen::prelude::{wasm_bindgen, Closure};

#[wasm_bindgen(module = "/src/modules/dom/js_dom.js")]
extern "C" {

    pub type DriverBrowserDomJs;

    #[wasm_bindgen(constructor)]
    pub fn new(
        mouse_down: &Closure<dyn Fn(u64)>,
        mouse_over: &Closure<dyn Fn(Option<u64>)>,
        keydown: &Closure<dyn Fn(
            Option<u64>,
            String,
            String,
            bool,
            bool,
            bool,
            bool,
        ) -> bool>,
        oninput: &Closure<dyn Fn(u64, String)>,
    ) -> DriverBrowserDomJs;
    #[wasm_bindgen(method)]
    pub fn mount_root(this: &DriverBrowserDomJs, id: u64);
    #[wasm_bindgen(method)]
    pub fn create_node(this: &DriverBrowserDomJs, id: u64, name: &str);
    #[wasm_bindgen(method)]
    pub fn set_attribute(this: &DriverBrowserDomJs, id: u64, attr: &str, value: &str);
    #[wasm_bindgen(method)]
    pub fn remove_attribute(this: &DriverBrowserDomJs, id: u64, attr: &str);
    #[wasm_bindgen(method)]
    pub fn remove_node(this: &DriverBrowserDomJs, id: u64);
    #[wasm_bindgen(method)]
    pub fn create_text(this: &DriverBrowserDomJs, id: u64, value: &str);
    #[wasm_bindgen(method)]
    pub fn remove_text(this: &DriverBrowserDomJs, id: u64);
    #[wasm_bindgen(method)]
    pub fn update_text(this: &DriverBrowserDomJs, id: u64, value: &str);
    #[wasm_bindgen(method)]
    pub fn insert_before(this: &DriverBrowserDomJs, parent: u64, child: u64, ref_id: Option<u64>);
    #[wasm_bindgen(method)]
    pub fn insert_css(this: &DriverBrowserDomJs, selector: &str, value: &str);

    #[wasm_bindgen(method)]
    pub fn get_bounding_client_rect_x(this: &DriverBrowserDomJs, id: u64) -> f64;
    #[wasm_bindgen(method)]
    pub fn get_bounding_client_rect_y(this: &DriverBrowserDomJs, id: u64) -> f64;
    #[wasm_bindgen(method)]
    pub fn get_bounding_client_rect_width(this: &DriverBrowserDomJs, id: u64) -> f64;
    #[wasm_bindgen(method)]
    pub fn get_bounding_client_rect_height(this: &DriverBrowserDomJs, id: u64) -> f64;

    #[wasm_bindgen(method)]
    pub fn scroll_top(this: &DriverBrowserDomJs, node_id: u64) -> i32;
    #[wasm_bindgen(method)]
    pub fn set_scroll_top(this: &DriverBrowserDomJs, node_id: u64, value: i32);
    #[wasm_bindgen(method)]
    pub fn scroll_left(this: &DriverBrowserDomJs, node_id: u64) -> i32;
    #[wasm_bindgen(method)]
    pub fn set_scroll_left(this: &DriverBrowserDomJs, node_id: u64, value: i32);
    #[wasm_bindgen(method)]
    pub fn scroll_width(this: &DriverBrowserDomJs, node_id: u64) -> i32;
    #[wasm_bindgen(method)]
    pub fn scroll_height(this: &DriverBrowserDomJs, node_id: u64) -> i32;
}
