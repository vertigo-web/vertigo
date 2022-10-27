use vertigo::{start_app, DomElement, dom};

fn app() -> DomElement {
    dom! {
        <div>"Hello world"</div>
    }
}

#[no_mangle]
pub fn start_application() {
    start_app(app);
}
