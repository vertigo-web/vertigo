#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

mod app;
mod simple_counter;
mod fetch;
mod sudoku;

use wasm_bindgen::prelude::*;


use std::cell::RefCell;

use virtualdom::{
    computed::{
        Dependencies,
    },
    App,
    VDomComponent,
};

use browserdriver::{
    DomDriverBrowser,
};


#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

thread_local! {
    static APP_STATE: RefCell<App> = RefCell::new({
        let root: Dependencies = Dependencies::default();
        let appStateBox = app::State::new(&root);

        let driver = DomDriverBrowser::default();

        App::new(driver, VDomComponent::new(appStateBox, app::render))
    });
}

#[wasm_bindgen(start)]
pub /*async*/ fn start_app() {
    console_error_panic_hook::set_once();
    wasm_logger::init(wasm_logger::Config::default());

    log::info!("Start rustowego modułu ...");

    APP_STATE.with(|state| state.borrow().start_app());

    wasm_bindgen_futures::spawn_local(async {
        log::info!("test z forka");
    });

    wasm_bindgen_futures::spawn_local(async {
        let aa = fetch::run("rustwasm/wasm-bindgen".into()).await;  //.unwrap();

        match aa {
            Ok(branch) => {
                log::info!("odpowiedź z serwera {:?}", branch);

            },
            Err(err) => {
                log::info!("błąd pobierania danych {:?}", err);
            }
        }
    });
}


/*
TODO - wydzielić computed do osobnego crates

TODO - obadac ten sposób odpalania projektu wasm
    https://github.com/IMI-eRnD-Be/wasm-run

TODO - Graph - usunac nieuzywane krawedzie (subskrybcje)

TODO - Graph - zamienić Clone na Copy

TODO - dodać jakieś makra które pozwolą na łatwe generowanie html-a (https://docs.rs/maplit/1.0.2/maplit/)
    to wygląda obiecująco
    https://github.com/chinedufn/percy/tree/master/crates/html-macro

https://github.com/rustwasm/console_error_panic_hook#readme
https://rustwasm.github.io/wasm-bindgen/reference/passing-rust-closures-to-js.html


TODO - insertAsFirstChild, insertAfter. Wywalić te dwie funkcje.
    trzeba odwrócić kolejność synchronizowania węzłów. Korzystać z metody insertBefore. Najmniejszy narzut pod kątem ilości zmian w domie


TODO - makro które wycina białe znaki ?
Css::one("
        margin: 5px;
    ")


TODO - zrobić analizator Cargo.lock, wyszukiwać biblioteki w rónych wersjach które posiadają zmienne globalne
    przykład tokio ....
*/




/*
#[wasm_bindgen(start)]
pub fn main() {
    future_to_promise(
         Request::new(Method::Get, "example.org/test")
            .header("Accept", "text/plain").send()
            .and_then(|resp_value: JsValue| {
                let resp: Response = resp_value.dyn_into().unwrap();
                resp.text()
            })
            .and_then(|text: Promise| {
                JsFuture::from(text)
            })
            .and_then(|body| {
                println!("Response: {}", body.as_string().unwrap());
                future::ok(JsValue::UNDEFINED)
            })
    );
}
*/