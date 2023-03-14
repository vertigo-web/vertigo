use vertigo::{start_app, DomElement, dom, Value};

fn app(state: &Value<u32>) -> DomElement {
    dom! {
        <div>"Hello world"</div>
    }
}

fn render() -> DomElement {
    let state = Value::new(0);
    app(&state)
}

#[no_mangle]
pub fn start_application() {
    start_app(render);
}
