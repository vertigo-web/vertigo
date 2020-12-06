#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

mod app_state;
mod simple_counter;
mod fetch;

use wasm_bindgen::{JsCast, prelude::*};
use web_sys::EventTarget;

use std::cell::RefCell;

use virtualdom::{
    computed::{
        Dependencies::Dependencies,
    },
    vdom::{
        App::App,
    }
};

use browserdriver::{
    DomDriverBrowser,
};

use crate::app_state::app_state::AppState;
use crate::app_state::app_state_render::main_render;


#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

thread_local! {
    static APP_STATE: RefCell<App> = RefCell::new({
        let root: Dependencies = Dependencies::new();
        let appStateBox = AppState::new(&root);
    
        let driver = DomDriverBrowser::new();
    
        App::new(driver, appStateBox.render(main_render))
    });
}

#[wasm_bindgen(start)]
pub async fn start_app() {
    console_error_panic_hook::set_once();
    wasm_logger::init(wasm_logger::Config::default());

    log::info!("Start rustowego modułu ...");

    APP_STATE.with(|state| state.borrow().start_app());

    let window = web_sys::window().expect("no global `window` exists");
    let document = window.document().expect("should have a document on window");
    let body = document.body().expect("document should have a body");

    // Manufacture the element we're gonna append
    let val1 = document.create_element("p").unwrap();
    val1.set_inner_html("Hello from Rust!");


    let closure: Closure<dyn FnMut(_)> = {
        let val1 = val1.clone();
    
        Closure::new(move |event: web_sys::Event| {
            log::info!("click ...");
            // let target = event.related_target();

            let target2 = event.target();

            if let Some(target) = target2 {
                let ta  = target.dyn_ref::<web_sys::Element>().unwrap();
                log::info!("sprawdzam target {}", ta /* as web_sys::Element*/ == &val1);
            } else {
                log::info!("brak targeta");
            }


            let kon = event.dyn_ref::<web_sys::MouseEvent>();
            log::info!("skonwertowanie na event myszy {:?}", kon);
        })
    };


    (&body).add_event_listener_with_callback(
        "mousedown",
        closure.as_ref().unchecked_ref()
    ).unwrap();

    closure.forget();




    let closure: Closure<dyn FnMut(_)> = Closure::new(move |event: web_sys::KeyboardEvent| {
        log::info!("keydown ... {} {}", event.char_code(), event.key());
    });

    (&body).add_event_listener_with_callback(
        "keydown",
        closure.as_ref().unchecked_ref()
    ).unwrap();

    closure.forget();





    body.append_child(&val1).unwrap();
    log::info!("po dodaniu");

    val1.set_attribute("debug1", "debug1").unwrap();
    let val2 = val1.clone();
    val2.set_attribute("debug2", "debug2").unwrap();
    val1.set_attribute("debug3", "debug3").unwrap();
    //web_sys::set_timeout_with_callback(this, handler)

    log::info!("eeeq {}", val1 == val2);

    wasm_bindgen_futures::spawn_local(async {
        log::info!("test z forka");
    });

    let aa = fetch::run("rustwasm/wasm-bindgen".into()).await;  //.unwrap();

    match aa {
        Ok(branch) => {
            log::info!("odpowiedź z serwera {:?}", branch);

        },
        Err(err) => {
            log::info!("błąd pobierania danych {:?}", err);
        }
    }
}


/*

TODO - obadac ten sposób odpalania projektu wasm
    https://github.com/IMI-eRnD-Be/wasm-run


TODO - Dodać tranzakcyjną aktualizację
    self.deps.triggerChange([id1, id2, id3]);               //to powinno wystarczyc

TODO - Graph - usunac nieuzywane krawedzie (subskrybcje)

TODO - Graph - zamienić Clone na Copy

TODO - dodać jakieś makra które pozwolą na łatwe generowanie html-a (https://docs.rs/maplit/1.0.2/maplit/)
    to wygląda obiecująco
    https://github.com/chinedufn/percy/tree/master/crates/html-macro

TODO - Będąc w bloku computed, albo subskrybcji, całkowicie ignorować wszelkie akcje które będą chciały zmienić wartość
       rzucać standardowy strumień błędów informację o incydencie. Dzięki temu nowa wadliwa funkcjonalność nie zepsuje tej juz dobrze ulezanej funkcjonalności
    
TODO - poprawić mechanizm synchronizowania listy dzieci
    1. brać listę istniejących juz nodów (bez odczepiania od parenta)
    2. wygenerowac nowa liste
    3. wyszukac pierwszy usuniety node, za nim podczapiac te nowe we własciwej kolejnośli
        lub wyszukac pierwszy który istnieje i za nim podczepiać te kolejne

TODO - niezmienne struktury danych, https://docs.rs/im/15.0.0/im/

https://github.com/rustwasm/console_error_panic_hook#readme
https://rustwasm.github.io/wasm-bindgen/reference/passing-rust-closures-to-js.html
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