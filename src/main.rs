#![allow(non_snake_case)]

use std::rc::Rc;

mod lib;

use crate::lib::{
    Dependencies::Dependencies,
    Computed::Computed,
};


 
fn main() {
    println!("Hello, world!");

    let root = Dependencies::new();

    let val1 = root.newValue(4);

    println!("ETAP 001");
    let val2 = root.newValue(5);

    println!("ETAP 002");

    let com1: Rc<Computed<i32>> = val1.toComputed();
    println!("ETAP 003");

    let com2: Rc<Computed<i32>> = val2.toComputed();

    println!("ETAP 004");


    let sum = Computed::from2(com1, com2, |a: &i32, b: &i32| -> i32 {
        println!("JESZCZE RAZ LICZE");
        a + b
    });

    println!("ETAP 005");

    let subscription = sum.subscribe(Box::new(|sum: Rc<i32>| {
        println!("___Suma: {}___", sum);
    }));

    println!("ETAP aaa");
    val1.setValue(333);

    println!("ETAP bbb");
    val2.setValue(888);

    println!("ETAP ccc");


    subscription.off();

    val2.setValue(889);
}



/*

            zarzadzanie subskrybcjami



            jesli liczba subskrybcji spadnie do zera, to wtedy trzeba wyczyscic subskrybcje

*/