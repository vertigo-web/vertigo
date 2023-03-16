use vertigo::{main, DomElement, dom, Value};

#[main]
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
