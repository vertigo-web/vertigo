use vertigo::{start_app, DomElement, dom, Value};

fn app() -> DomElement {
    let message = Value::new("world");
    dom! {
        <html>
            <head />
            <body>
                <div>"Hello " {message}</div>
            </body>
        </html>
    }
}

#[no_mangle]
pub fn start_application() {
    start_app(app);
}
