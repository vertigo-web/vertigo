#![deny(rust_2018_idioms)]

use vertigo::{start_app};
// use vertigo::{get_driver, JsValue};

mod app;

#[no_mangle]
pub fn start_application() {
    start_app(app::State::component);

    // let driver = get_driver();
    // let ddd = driver.dom_get(&["window", "location"], "hash");
    // log::info!("odczytano: {ddd:#?}");

    // let aaa = driver.dom_call(
    //     &["window", "localStorage"],
    //     "getItem",
    //     vec!(JsValue::str("gggg"))
    // );

    // log::info!("localStorage.gggg = {aaa:#?}");

    // let driver = get_driver();
    // let ddd = driver.dom_set(&["window", "location"], "hash", JsValue::str("nowy_hash_hakis"));
    // log::info!("odczytano: {ddd:#?}");



    //------------------------------------------------------------------------------------------------------------------------------------------



    /*
    let aaa = driver.dom_call(
        &[
             &["get", "window"],
             &["get", "localStorage"],
             &["call", "getItem", "gggg"]
        ],
    );

    let aaa = driver.dom_call(
        &[
             &["get", "window"],
             &["get", "localStorage"],
             &["call", "setItem", "dsadsa", "dsadsadas"]
        ],
    );

    let aaa = driver.dom_call(
        &[
             &["get", 4444],                //jakiś DomElement
             &["call", "get_bounding_client_rect_y"],
             &["get_props", "y", "height"]
        ],
    );

                                    //ustawienie właściwości
    let aaa = driver.dom_call(
        &[
             &["get", 4444],                //jakiś DomElement
             &["set", "scroll_top", 123],
        ],
    );



    driver
        .dom_access()
        .get("window")
        .get("localStorage")
        .call("setItem", JsValue::List(
            vec![
                JsValue::str("dsadsa"),
                JsValue::str("dsadsadas")
            ]
        )
        .exec()

    let result = driver
        .dom_access()
        .element(444)
        .call("getBoundingClientRect")
        .get_props("y", "height")
        .fetch();



    //exec - nic nie zwraca. Sprawdzamy czy js aby na pewno nic nie zwróci
    //fetch() - zwracaj JsValue



    //------------------------------------------------------------------------------------------------------------------------------------------

    //potencjalny sposób na wykonanie kilku rzeczy na raz

    let aaa = driver.dom_call_multi(
        &[
            &[
                &["get", 4444],                //jakiś DomElement
                &["call", "getBoundingClientRect"],
                &["get_props", "y", "height"]
            ],
            &[
                &["get", 5555],                //jakiś DomElement
                &["call", "getBoundingClientRect"],
                &["get_props", "x", "width", "height"]
            ],
    );

    */
}
