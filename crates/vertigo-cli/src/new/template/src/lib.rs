use vertigo::{start_app, DomElement, dom, Value};

fn app(state: &Value<u32>) -> DomElement {
    dom! {
        <div>"Hello world"</div>
    }
}

#[no_mangle]
pub fn start_application() {
    let state = Value::new(0);
    let view = app(&state);
    start_app(state, view);
}
