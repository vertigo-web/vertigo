use vertigo::{ClickEvent, Computed, Value, component, css, dom, store};

mod simple_counter;
use simple_counter::SimpleCounter;

#[derive(Clone)]
pub struct State {
    counter1: Value<i32>,
    counter2: Value<i32>,
    counter3: Value<i32>,
    counter4: Value<i32>,
    pub sum: Computed<i32>,
    pub double: Computed<i32>,
}

impl State {
    pub fn new() -> State {
        let counter1 = Value::new(1);
        let counter2 = Value::new(2);
        let counter3 = Value::new(3);
        let counter4 = Value::new(4);

        let sum = {
            let counter1 = counter1.clone();
            let counter2 = counter2.clone();
            let counter3 = counter3.clone();
            let counter4 = counter4.clone();

            Computed::from(move |context| {
                let value1 = counter1.get(context);
                let value2 = counter2.get(context);
                let value3 = counter3.get(context);
                let value4 = counter4.get(context);

                value1 + value2 + value3 + value4
            })
        };

        let double = Computed::from({
            let sum = sum.clone();

            move |context| {
                let sub_comp = Computed::from({
                    let sum = sum.clone();
                    move |context| sum.get(context)
                });

                sub_comp.get(context) * 2
            }
        });

        State {
            counter1,
            counter2,
            counter3,
            counter4,
            sum,
            double,
        }
    }
}

#[store]
pub fn state_counters() -> State {
    State::new()
}

#[component]
fn Sum(sum: Computed<i32>) {
    dom! {
        <div>
            "sum = " {sum}
        </div>
    }
}

#[component]
pub fn CountersDemo() {
    let state = state_counters();

    let outer_click_css = css! {"
        border: solid 1px black;
        width: 200px;
        margin: 5px;
        padding: 5px;
    "};

    dom! {
        <div>
            <SimpleCounter label="counter1 value" value={&state.counter1} />
            <SimpleCounter label="counter2 value" value={&state.counter2} />
            <SimpleCounter label="counter3 value" value={&state.counter3} />
            <SimpleCounter label="counter4 value" value={&state.counter4} />
            <Sum sum={&state.sum} />
            <div>
                { &state.double }
            </div>
            <div>
                "Stop propagation test:"
                <div css={outer_click_css} on_click={|_| log::info!("Outer")}>
                    "outer click"<br/>
                    <button on_click={|evt: ClickEvent| {
                        evt.stop_propagation();
                        log::info!("Inner");
                    }}>"Inner click"</button>
                </div>
            </div>
        </div>
    }
}
