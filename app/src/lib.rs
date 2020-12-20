#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

mod app;
mod simple_counter;
mod sudoku;

use wasm_bindgen::prelude::wasm_bindgen;

use std::cell::RefCell;

use virtualdom::{
    computed::Dependencies,
    App,
    VDomComponent,
};

use browserdriver::DomDriverBrowser;

#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

thread_local! {
    static APP_STATE: RefCell<App> = RefCell::new({
        let driver = DomDriverBrowser::new();

        let root: Dependencies = Dependencies::default();
        let appStateBox = app::State::new(&root, &driver);

        App::new(driver, VDomComponent::new(appStateBox, app::render))
    });
}

#[wasm_bindgen(start)]
pub async fn start_app() {
    console_error_panic_hook::set_once();
    wasm_logger::init(wasm_logger::Config::default());

    log::info!("Start rustowego modułu ...");

    APP_STATE.with(|state| state.borrow().start_app());
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

TODO - fetch - pozbyć się unwrapow

TODO - dodać do DomDriver asynchronicznego sleep-a
*/

