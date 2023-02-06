use vertigo::{start_app, Value, bind, DomElement, dom};

#[derive(Clone, Default)]
pub struct App {
    pub count: Value<i32>,
}

impl App {
    pub fn mount(&self) -> DomElement {
        let state = self.clone();

        let increment = bind!(state, || {
            state.count.change(|value| {
                *value += 1;
            });
        });

        let decrement = bind!(state, || {
            state.count.change(|value| {
                *value -= 1;
            });
        });

        dom! {
            <html>
                <head />
                <body>
                    <div>
                        <p>"Counter: " { state.count }</p>
                        <button on_click={decrement}>"-"</button>
                        <button on_click={increment}>"+"</button>
                    </div>
                </body>
            </html>
        }
    }
}

#[no_mangle]
pub fn start_application() {
    let state = App::default();
    let view = state.mount();
    start_app(state, view);
}
