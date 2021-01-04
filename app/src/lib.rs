mod app;
mod simple_counter;
mod sudoku;
mod input;
mod github_explorer;
pub mod game_of_life;

use wasm_bindgen::prelude::wasm_bindgen;

use vertigo::{
    computed::Dependencies,
    App,
    VDomComponent,
};

use browserdriver::DomDriverBrowser;

#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen(start)]
pub async fn start_app() {
    console_error_panic_hook::set_once();
    wasm_logger::init(wasm_logger::Config::default());

    log::info!("Start rustowego modułu ...");

    let driver = DomDriverBrowser::new();

    let root: Dependencies = Dependencies::default();
    let app_state = app::State::new(&root, &driver);

    let app = App::new(driver.clone(), VDomComponent::new(app_state, app::render));

    app.start_app().await;
}


/*
TODO - dodać jakieś makra które pozwolą na łatwe generowanie html-a (https://docs.rs/maplit/1.0.2/maplit/)
    to wygląda obiecująco
    https://github.com/chinedufn/percy/tree/master/crates/html-macro
    https://github.com/rbalicki2/scoped_css

TODO - makro które wycina białe znaki ?
Css::one("
        margin: 5px;
    ")

TODO - fetch - pozbyć się unwrapow

TODO - updejt nazwy taga ...

TODO - 
    <textarea value="" />
    makro html pewnie będzie mogło przyjmować ten atrybut lub <textarea>value</textarea>


To co jest wywoływane w callbacku eventu powinno być wywoływane w tranzakcji ?
    let on_set3 = {
        let state = state.clone();
        move |new_value: String| {
            let value = state.value.clone();
            value.setValue(new_value);
        }
    };


https://github.com/rustwasm/console_error_panic_hook#readme
https://rustwasm.github.io/wasm-bindgen/reference/passing-rust-closures-to-js.html

*/


