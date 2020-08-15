#![allow(non_snake_case)]

mod lib;

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
    println!("Hello, world!");

    let root = Dependencies::new();

    println!("=============1=============");

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

    println!("=============2=============");

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

    println!("=============3=============");

    let sumaTotal = appState.suma.clone().subscribe(|value| {
        println!("|||| {}", value);
    });

    appState.value1.setValue(2);
    appState.value2.setValue(3);
    appState.value3.setValue(4);

    sumaTotal.off();


    //konwertowanie do wskaźnika

    fn foo1() -> i32 {
        0
    }
    
    fn foo2() -> i32 {
        0
    }

    fn foo3(yy: i32) -> i32 {
        0
    }

    //println!("aaa {}", std::mem::size_of<* ()>);

    let pointer1 = foo1 as *const ();
    let pointer2 = foo2 as *const ();
    let pointer3 = foo1 as *const ();
    let pointer4 = foo3 as *const ();

    let gg1: u64 = pointer1 as u64;
    let gg2: u64 = pointer2 as u64;
    let gg3: u64 = pointer3 as u64;
    let gg4: u64 = pointer4 as u64;

    let tt1: u64 = foo1 as u64;
    let tt2: u64 = foo2 as u64;
    let tt3: u64 = foo1 as u64;
    let tt4: u64 = foo3 as u64;

    println!("sadas {}", pointer1 == pointer2);
    println!("sadas {}", pointer1 == pointer3);

    println!("gg1 {:x}", gg1);
    println!("gg2 {:x}", gg2);
    println!("gg3 {:x}", gg3);
    println!("gg4 {:x}", gg4);

    println!("tt1 {:x}", tt1);
    println!("tt2 {:x}", tt2);
    println!("tt3 {:x}", tt3);
    println!("tt4 {:x}", tt4);

    let bb1: u32 = foo1 as u32;
    let bb2: u32 = foo2 as u32;
    let bb3: u32 = foo1 as u32;
    let bb4: u32 = foo3 as u32;

    println!("bb1 {:x}", bb1);
    println!("bb2 {:x}", bb2);
    println!("bb3 {:x}", bb3);
    println!("bb4 {:x}", bb4);

    let cc1: u128 = foo1 as u128;
    let cc2: u128 = foo2 as u128;
    let cc3: u128 = foo1 as u128;
    let cc4: u128 = foo3 as u128;

    println!("cc1 {:x}", cc1);
    println!("cc2 {:x}", cc2);
    println!("cc3 {:x}", cc3);
    println!("cc4 {:x}", cc4);
}
