#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

mod app_state;
mod simple_counter;

use wasm_bindgen::prelude::*;

use std::cell::RefCell;

use virtualdom::{
    computed::{
        Dependencies::Dependencies,
    },
    vdom::{
        App::App,
        StateBox::StateBox,
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
        let appStateBox = StateBox::new(&root, AppState::new(&root));
    
        let driver = DomDriverBrowser::new();
    
        App::new(driver, appStateBox.render(main_render))
    });
}

#[wasm_bindgen(start)]
pub fn start_app() {
    console_error_panic_hook::set_once();
    wasm_logger::init(wasm_logger::Config::default());

    log::info!("Start rustowego modułu ...");

    APP_STATE.with(|state| state.borrow().start_app());
}


/*
TODO - Dodać tranzakcyjną aktualizację
    self.deps.triggerChange([id1, id2, id3]);               //to powinno wystarczyc

TODO - Graph - usunac nieuzywane krawedzie (subskrybcje)

TODO - Graph - zamienić Clone na Copy

TODO - dodać jakieś makra które pozwolą na łatwe generowanie html-a (https://docs.rs/maplit/1.0.2/maplit/)

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
