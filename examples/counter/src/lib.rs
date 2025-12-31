use vertigo::{DomNode, Value, bind, component, dom, main};

#[component]
fn App(count: Value<i32>) {
    let increment = bind!(count, |_| {
        count.change(|value| {
            *value += 1;
        });
    });

    let decrement = bind!(count, |_| {
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

#[main]
fn render() -> DomNode {
    let count = Value::new(0);

    dom! {
        <App count={&count} />
    }
}
