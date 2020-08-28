use wasm_bindgen::prelude::*;
use std::rc::Rc;

#[wasm_bindgen(module = "/src/driver.js")]
extern "C" {
    fn consoleLog(message: &str);
    fn startDriverLoop(closure: &Closure<dyn FnMut()>);
}

pub struct DomDriverBrowserInner {
}

impl DomDriverBrowserInner {
    fn fromCallback(&self) {
        log::info!("callback z drivera");
    }
}

pub struct DomDriverBrowser {
    driver: Rc<DomDriverBrowserInner>,
    callFromJS: Closure<dyn FnMut()>,
}

impl DomDriverBrowser {
    pub fn new() -> DomDriverBrowser {

        let driver = Rc::new(DomDriverBrowserInner {
        });

        let callFromJS: Closure<dyn FnMut()> = {
            let driver = driver.clone();
            let back = Closure::new(move || {
                driver.fromCallback();
            });
    
            startDriverLoop(&back);
    
            back
        };

        DomDriverBrowser {
            driver,
            callFromJS,
        }
    }

    pub fn consoleLog(&self, message: &str) {
        consoleLog(message);
    }
}

//Moze te bindowania i ten DomDriverBrowser da sie zamknac w jednym module w bibliotece

/*
    jak przyjdzie jakis zdarzenie od uzytkownika
    zapisujemy je w jakiejs tablicy po stronie jsowego drivera
    wysyłamy sygnał zwrotny do modulu rustowego ze czekaja nowe zdarzenia
    modul rustowy pobiera sobie te nowe informaje na temat eventu z jsa i zmienia odpowiedni stan
*/
