use vertigo::{
    component, css, dom, get_driver, include_static, store, ClickEvent, Computed, JsJson, Value,
};

mod simple_counter;
use simple_counter::SimpleCounter;

use crate::app::{route::Route, state::state_route};

#[derive(Clone)]
pub struct State {
    counter1: Value<i32>,
    counter2: Value<i32>,
    counter3: Value<i32>,
    counter4: Value<i32>,
    pub sum: Computed<i32>,
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
            sum,
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

    let path = include_static!("./counter.webp");

    let center_base = css! {"
        border: 1px solid black;
        padding: 1px;
        margin: 0 auto;
        display: block;

        cursor: pointer;

        transition: all .2s ease-in-out;
    "};

    let center_css = center_base.clone()
        + css! {"
        box-shadow: 4px 4px 4px #444, 8px 8px 4px #666, 12px 12px 4px #888;

        :hover {
            transform: scale(1.1);
        }
    "};

    let center_css2 = center_base.push_str("
        box-shadow: 4px 4px 4px rgba(0, 0, 0, 0.5), 8px 8px 4px rgba(0, 0, 0, 0.4), 12px 12px 4px rgba(0, 0, 0, 0.3);
        :hover {
            transform: scale(1.5);
            box-shadow: 54px 54px 14px rgba(0, 0, 0, 0.3), 58px 58px 14px rgba(0, 0, 0, 0.2), 62px 62px 14px rgba(0, 0, 0, 0.1);
        }
    ");

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
            <img css={center_css} src={&path} />
            <img css={center_css2} src={path} />
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
            <div on_click={|_| {
                state_route().set(Route::Sudoku);
            }}>
                "Go to Sudoku"
            </div>
            <div on_click={|_| {
                get_driver().cookie_set("test", "test value", 100000000);
            }}>
                "Set cookie"
            </div>

            <div on_click={|_| {
                let value = get_driver().cookie_get("test");
                log::info!("cookie value {:?}", value);
            }}>
                "Get cookie"
            </div>

            <div on_click={|_| {
                let list = vec!(JsJson::String("value1".into()), JsJson::String("value2".into()), JsJson::String("value3".into()));
                let value = JsJson::List(list);
                get_driver().cookie_set_json("test-json", value, 100000000);
            }}>
                "Set json cookie"
            </div>

            <div on_click={|_| {
                let value = get_driver().cookie_get_json("test-json");
                log::info!("cookie value {:?}", value);
            }}>
                "Get json cookie"
            </div>

            <div on_click={|_| {
                let value = get_driver().timezone_offset() ;
                log::info!("timezone_offset {:?}", value);
            }}>
                "Get timezone_offset"
            </div>

            <div on_click={|_| {
                get_driver().history_back();
            }}>
                "History back"
            </div>

            <div on_click={|_| {
                let random = get_driver().get_random(34, 100);
                log::info!("random from 34 to 100 {:?}", random);
            }}>
                "Get random"
            </div>
        </div>
    }
}
