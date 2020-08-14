#![allow(non_snake_case)]

mod lib;

use crate::lib::{
    Value::Value,
    Dependencies::Dependencies,
    Computed::Computed,
};

/*
TODO - Dodać tranzakcyjną aktualizację

TODO - Uprościć wyciąganie wartośći:
            let val1RR = appState.com1.getValue();
            let val1 = val1RR.as_ref();
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
                let val1RR = com1.getValue();
                let val1 = val1RR.as_ref();
    
                let val2RR = com2.getValue();
                let val2 = val2RR.as_ref();

                let val3RR = com3.getValue();
                let val3 = val3RR.as_ref();
    
                val1 + val2 + val3
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
    println!("Hello, world!");

    let root = Dependencies::new();

    let val1 = root.newValue(4);
    let val2 = root.newValue(5);

    let com1: Computed<i32> = val1.toComputed();
    let com2: Computed<i32> = val2.toComputed();

    let sum = Computed::from2(com1, com2, |a: &i32, b: &i32| -> i32 {
        a + b
    });

    let suma2 = sum.clone().map(|value: &i32| -> i32 {
        2 * value
    });

    let subscription = sum.subscribe(|sum: &i32| {
        println!("___Suma: {}___", sum);
    });

    let sub2 = suma2.subscribe(|sum2: &i32| {
        println!("___Suma2: {}___", sum2);
    });

    val1.setValue(111);
    val2.setValue(888);

    println!("subscription off");

    subscription.off();
    sub2.off();

    val2.setValue(999);

    let appState = AppState::new(&root);

    let suma3 = {
        let appState = appState.clone();
        
        root.from(move || -> i32 {
            let val1RR = appState.com1.getValue();
            let val1 = val1RR.as_ref();

            let val3RR = appState.com3.getValue();
            let val3 = val3RR.as_ref();

            val1 + val3
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

    println!("=============");

    let sumaTotal = appState.suma.clone().subscribe(|value| {
        println!("|||| {}", value);
    });

    appState.value1.setValue(2);
    appState.value2.setValue(3);
    appState.value3.setValue(4);

    sumaTotal.off();
}
