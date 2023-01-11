use vertigo::{Computed, Value, DomElement, dom, include_static, css, component};

mod simple_counter;
use simple_counter::SimpleCounter;

#[derive(Clone)]
pub struct State {
    counter1: Value<u32>,
    counter2: Value<u32>,
    counter3: Value<u32>,
    counter4: Value<u32>,
    pub sum: Computed<u32>,
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

        State {
            counter1,
            counter2,
            counter3,
            counter4,
            sum
        }
    }
}



#[component]
fn Sum(sum: Computed<u32>) -> DomElement {
    dom! {
        <div>
            "sum = " {sum}
        </div>
    }
}

#[component]
pub fn CountersDemo(state: State) -> DomElement {
    let path = include_static!("./counter.webp");

    let center_css = css!("
        border: 1px solid black;
        padding: 1px;
        margin: 0 auto;
        display: block;

        cursor: pointer;
        box-shadow: 4px 4px 4px #444, 8px 8px 4px #666, 12px 12px 4px #888;

        transition: all .2s ease-in-out;
        :hover {
            transform: scale(1.1);
        }
    ");

    let center_css2 = css!("
        border: 1px solid black;
        padding: 1px;
        margin: 0 auto;
        display: block;

        cursor: pointer;

        box-shadow: 4px 4px 4px rgba(0, 0, 0, 0.5), 8px 8px 4px rgba(0, 0, 0, 0.4), 12px 12px 4px rgba(0, 0, 0, 0.3);
        transition: all .2s ease-in-out;
        :hover {
            transform: scale(1.5);
            box-shadow: 54px 54px 14px rgba(0, 0, 0, 0.3), 58px 58px 14px rgba(0, 0, 0, 0.2), 62px 62px 14px rgba(0, 0, 0, 0.1);
        }
    ");

    dom! {
        <div>
            <SimpleCounter label="counter1 value" value={&state.counter1} />
            <SimpleCounter label="counter2 value" value={&state.counter2} />
            <SimpleCounter label="counter3 value" value={&state.counter3} />
            <SimpleCounter label="counter4 value" value={&state.counter4} />
            <Sum sum={&state.sum} />
            <img css={center_css} src={path} />
            <img css={center_css2} src={path} />
        </div>
    }
}
