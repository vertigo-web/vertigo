use vertigo::{start_app, Value, bind, DomElement, dom};

#[derive(Default)]
pub struct App {
    pub count: Value<i32>,
}

impl App {
    pub fn mount(self) -> DomElement {
        let increment = bind(&self.count).call(|context, count| {
            count.set(count.get(context) + 1)
        });

        let decrement = bind(&self.count).call(|context, count| {
            count.set(count.get(context) - 1)
        });

        dom! {
            <div>
                <p>"Counter: " { self.count }</p>
                <button on_click={decrement}>"-"</button>
                <button on_click={increment}>"+"</button>
            </div>
        }
    }
}

#[no_mangle]
pub fn start_application() {
    start_app(|| App::default().mount());
}
