#![allow(non_snake_case)]

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

TODO - sprawdzić czy da się coś ciekawego uzyskać z rollupem:
    https://github.com/wasm-tool/rollup-plugin-rust

TODO - niezmienne struktury danych, https://docs.rs/im/15.0.0/im/


https://github.com/rustwasm/console_error_panic_hook#readme
https://rustwasm.github.io/wasm-bindgen/reference/passing-rust-closures-to-js.html
*/

use virtualdom::{
    vdom::{
        App::App,
        DomDriver::{
            DomDriver::DomDriver,
        },
    },
    computed::{
        Dependencies::Dependencies,
    }
};

use crate::app_state::AppState;
use crate::view::main_render::main_render;
use browserdriver::DomDriverBrowser;

pub struct Application {
    _app: App,
}

impl Application {
    pub fn new() -> Application {
        console_error_panic_hook::set_once();
        wasm_logger::init(wasm_logger::Config::default());

        log::info!("Start rustowego modułu ...");

        let root: Dependencies = Dependencies::new();
        let appState = AppState::new(&root);

        let driver = DomDriverBrowser::new();
        let domDriver = DomDriver::new(driver);

        let app = App::new(domDriver, root, appState.clone(), main_render);

        Application {
            _app: app
        }
    }

    pub fn start_app(&self) {
        log::info!("START APP");
    }

}