use vertigo::{start_app, Value, bind, DomElement, dom, component};

#[component]
fn App(count: Value<i32>) -> DomElement {
    let increment = bind!(count, || {
        count.change(|value| {
            *value += 1;
        });
    });

    let decrement = bind!(count, || {
        count.change(|value| {
            *value -= 1;
        });
    });

    dom! {
        <html>
            <head />
            <body>
                <div>
                    <p>"Counter: " { count }</p>
                    <button on_click={decrement}>"-"</button>
                    <button on_click={increment}>"+"</button>
                </div>
            </body>
        </html>
    }
}

#[no_mangle]
pub fn start_application() {
    let count = Value::new(0);

    let view = dom! {
        <App count={&count} />
    };

    start_app(count, view);
}
