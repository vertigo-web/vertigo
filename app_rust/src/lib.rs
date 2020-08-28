mod application;
mod app_state;
mod view;

use wasm_bindgen::prelude::*;

use std::cell::RefCell;
use crate::application::Application;

#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;


#[wasm_bindgen(module = "/src/driver.js")]
extern "C" {
    fn add(a: u32, b: u32) -> u32;
    fn consoleLog(message: &str);
    fn startDriverLoop(closure: &Closure<dyn FnMut()>);
}

struct DomDriverBrowser {

}

//Moze te bindowania i ten DomDriverBrowser da sie zamknac w jednym module w bibliotece

/*
    jak przyjdzie jakis zdarzenie od uzytkownika
    zapisujemy je w jakiejs tablicy po stronie jsowego drivera
    wysyłamy sygnał zwrotny do modulu rustowego ze czekaja nowe zdarzenia
    modul rustowy pobiera sobie te nowe informaje na temat eventu z jsa i zmienia odpowiedni stan
*/

thread_local! {

    static callFromJS: Closure<dyn FnMut()> = {
        
        let back = Closure::new(move || {
            log::info!("callback z drivera");
        });

        startDriverLoop(&back);

        back
    };

    static appState: RefCell<Application> = {
        println!("Tutaj trzeba będzie powołać do zycia obiekt drivera przegladarkowego który będzie odwoływał się do tych funkcji powyzej");

        RefCell::new(Application::new())
    };
}
#[wasm_bindgen]
pub fn start_app() {
    
    let aa = add(3, 4);
    log::info!("z funkcji add ... {}", aa);
    consoleLog("aaaarrr333");

    appState.with(|state| {
        let state = state.borrow_mut();
        state.start_app();
    });

    callFromJS.with(|_state| {
        log::info!("start callback");
    });
}

#[wasm_bindgen]
pub fn click_button() {

    appState.with(|state| {
        let state = state.borrow_mut();
        state.increment();
        consoleLog("Zwiększyłem licznik");
    });
}
