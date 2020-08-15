#![allow(non_snake_case)]

mod lib;
mod tests;

use crate::lib::{
    Value::Value,
    Dependencies::Dependencies,
    Computed::Computed,
};

/*
TODO - Dodać tranzakcyjną aktualizację
    self.deps.triggerChange([id1, id2, id3]);               //to powinno wystarczyc

TODO - Graph - usunac nieuzywane krawedzie (subskrybcje)
*/

struct AppState {
    value1: Value<i32>,
    value2: Value<i32>,
    value3: Value<i32>,
    com1: Computed<i32>,
    #[allow(dead_code)]
    com2: Computed<i32>,
    com3: Computed<i32>,
    suma: Computed<i32>,
}

impl AppState {
    pub fn new(root: &Dependencies) -> std::rc::Rc<AppState> {
        let value1 = root.newValue(1);
        let value2 = root.newValue(2);
        let value3 = root.newValue(3);
        let com1 = value1.toComputed();
        let com2 = value2.toComputed();
        let com3 = value3.toComputed();

        let suma = {
            let com1 = com1.clone();
            let com2 = com2.clone();
            let com3 = com3.clone();

            root.from(move || {
                let val1 = com1.getValue();
                let val2 = com2.getValue();
                let val3 = com3.getValue();
    
                *val1 + *val2 + *val3
            })
        };

        std::rc::Rc::new(AppState {
            value1,
            value2,
            value3,
            com1,
            com2,
            com3,
            suma
        })
    }
}

fn main() {
    let root = Dependencies::new();

    let appState = AppState::new(&root);

    let suma3 = {
        let appState = appState.clone();
        
        root.from(move || -> i32 {
            //let com1: &Computed<i32> = &appState.com1;
            let val1 = appState.com1.getValue();
            let val3 = appState.com3.getValue();

            *val1 + *val3
        })
    };

    let suma3sub = suma3.subscribe(|value| {
        println!("suma 333 ==> {}", value);
    });

    appState.value1.setValue(2);
    appState.value1.setValue(3);
    appState.value2.setValue(4);
    appState.value2.setValue(5);
    appState.value3.setValue(6);
    appState.value3.setValue(7);
    suma3sub.off();
    appState.value3.setValue(8);

    let sumaTotal = appState.suma.clone().subscribe(|value| {
        println!("|||| {}", value);
    });

    appState.value1.setValue(2);
    appState.value2.setValue(3);
    appState.value3.setValue(4);

    sumaTotal.off();
}
