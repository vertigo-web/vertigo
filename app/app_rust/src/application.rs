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

use std::rc::Rc;
use virtualdom::{
    vdom::{
        startApp::startApp,
        DomDriver::{
            DomDriver::DomDriver,
            DomDriverPrint::DomDriverPrint,
        },
    },
    computed::{
        Dependencies::Dependencies,
        Client::Client,
    }
};

use crate::app_state::AppState;
use crate::view::main_render::main_render;
use crate::driver::DomDriverBrowser;

pub struct Application {
    pub driver: DomDriverBrowser,
    pub appState: Rc<AppState>,
    _subskrybcjaApp: Client,
}

impl Application {
    pub fn new(driver: DomDriverBrowser) -> Application {
        wasm_logger::init(wasm_logger::Config::default());

        log::info!("Start rustowego modułu ...");

        //po wystartowaniu subskrybcjaApp tą zmienną trzeba wpakować w zmienną globalną zeby nie stracić subskrybcji

        let root: Dependencies = Dependencies::new();
        let appState = AppState::new(&root);


        //let driverPrint = DomDriverPrint::new();
        let driverBrowser = driver.driver.clone();
        let domDriver = DomDriver::new(driverBrowser);
        
        
        let subskrybcjaApp = startApp(domDriver, root, appState.clone(), main_render);

        appState.value.setValue(55);
        log::info!("Przestawiam wartość");
        appState.at.setValue(1000);

        log::info!("--- koniec aplikacji, przechodzę do wyłączania subskrybcji ---");

        //subskrybcjaApp.off();

        Application {
            driver: driver,
            appState,
            _subskrybcjaApp: subskrybcjaApp
        }
    }

    pub fn start_app(&self) {
        log::info!("START APP");
    }

    pub fn increment(&self) {
        let rr = self.appState.value.getValue();
        self.appState.value.setValue(*rr + 1);
    }
}