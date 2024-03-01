use vertigo::{dom, main, DomNode, Value};

#[main]
fn app() -> DomNode {
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
