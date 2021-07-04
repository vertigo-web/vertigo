use web_sys::{Document, Element, HtmlHeadElement, Window};

use vertigo::{RealDomId};



pub fn get_window_elements() -> (Window, Document, HtmlHeadElement) {
    let window = web_sys::window().expect("no global `window` exists");
    let document = window.document().expect("should have a document on window");
    let head = document.head().unwrap();
    (window, document, head)
}

pub fn create_node(document: &Document, id: &RealDomId, name: &'static str) -> Element {
    let node = if name == "path" || name == "svg" {
        document.create_element_ns(Some("http://www.w3.org/2000/svg"), name).unwrap()
    } else {
        document.create_element(name).unwrap()
    };

    let id_str = format!("{}", id.to_u64());
    node.set_attribute("data-id", id_str.as_str()).unwrap();
    node
}

pub fn create_root(document: &Document, root_id: &RealDomId) -> Element {
    let body = document.body().expect("document should have a body");
    let root = create_node(document, root_id, "div");
    body.append_child(&root).unwrap();
    root
}

