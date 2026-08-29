//! What the app can ask of the environment it is running in, through `get_driver()`.
//!
//! Cookies, the timezone, a random number, the history stack, the router - and whether any of
//! this is happening in a browser at all, which is what the panel at the bottom turns on.
//!
//! Each control reports what it got rather than only logging it, so the tab shows something
//! without the console open.

use vertigo::{Css, JsJson, Value, bind, component, css, dom, get_driver};

use crate::app::{route::Route, state::state_route};

mod ssr_test;
use ssr_test::SsrTest;

const COOKIE: &str = "test";
const JSON_COOKIE: &str = "test-json";

#[component]
pub fn DriverDemo() {
    let last = Value::new(String::new());

    let set_cookie = bind!(last, |_| {
        get_driver().cookie_set(COOKIE, "test value", 100000000);
        last.set(format!("set the cookie {COOKIE:?}"));
    });

    let get_cookie = bind!(last, |_| {
        let value = get_driver().cookie_get(COOKIE);
        last.set(format!("cookie {COOKIE:?} = {value:?}"));
    });

    let set_json_cookie = bind!(last, |_| {
        let list = vec![
            JsJson::String("value1".into()),
            JsJson::String("value2".into()),
            JsJson::String("value3".into()),
        ];
        get_driver().cookie_set_json(JSON_COOKIE, JsJson::List(list), 100000000);
        last.set(format!("set the json cookie {JSON_COOKIE:?}"));
    });

    let get_json_cookie = bind!(last, |_| {
        let value = get_driver().cookie_get_json(JSON_COOKIE);
        last.set(format!("json cookie {JSON_COOKIE:?} = {value:?}"));
    });

    let timezone = bind!(last, |_| {
        let offset = get_driver().timezone_offset();
        last.set(format!("timezone offset = {offset} minutes"));
    });

    let random = bind!(last, |_| {
        let value = get_driver().get_random(34, 100);
        last.set(format!("random between 34 and 100 = {value}"));
    });

    let go_to_sudoku = |_| {
        state_route().set(Route::Sudoku);
    };

    let history_back = |_| {
        get_driver().history_back();
    };

    dom! {
        <div>
            <p>
                "Each of these is a plain "<code>"div"</code>" with an "<code>"on_click"</code>
                " - vertigo attaches handlers with "<code>"addEventListener"</code>
                ", so a control does not have to be a button."
            </p>

            <div css={css_action()} on_click={set_cookie}>"Set cookie"</div>
            <div css={css_action()} on_click={get_cookie}>"Get cookie"</div>
            <div css={css_action()} on_click={set_json_cookie}>"Set json cookie"</div>
            <div css={css_action()} on_click={get_json_cookie}>"Get json cookie"</div>
            <div css={css_action()} on_click={timezone}>"Get timezone_offset"</div>
            <div css={css_action()} on_click={random}>"Get random"</div>

            <hr/>

            <div css={css_action()} on_click={go_to_sudoku}>"Go to Sudoku"</div>
            <div css={css_action()} on_click={history_back}>"History back"</div>

            <div css={css_result()}>
                "Last result: " { last }
            </div>

            <hr/>

            <SsrTest />
        </div>
    }
}

fn css_action() -> Css {
    css! {"
        width: 220px;
        padding: 4px 6px;
        margin: 2px 0;
        border: 1px solid #ccc;
        cursor: pointer;

        :hover {
            background-color: #eee;
        }
    "}
}

/// A fixed width, so a longer result does not resize anything around it.
fn css_result() -> Css {
    css! {"
        width: 420px;
        min-height: 1.4em;
        margin: 10px 0;
        padding: 4px 6px;
        background-color: #f4f4f4;
        font-family: monospace;
    "}
}
