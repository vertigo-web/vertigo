use vertigo::{start_app, Value, bind, DomElement, dom};

#[derive(Clone, Default)]
pub struct App {
    pub count: Value<i32>,
}

impl App {
    pub fn mount(self) -> DomElement {
        let state = self;

        let increment = bind!(|state| {
            state.count.change(|value| {
                *value += 1;
            });
        });

        let decrement = bind!(|state| {
            state.count.change(|value| {
                *value -= 1;
            });
        });

        dom! {
            <div>
                <p>"Counter: " { state.count }</p>
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
