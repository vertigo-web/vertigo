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
    let val2 = root.newValue(5);

    let com1: Rc<Computed<i32>> = val1.toComputed();
    let com2: Rc<Computed<i32>> = val2.toComputed();

    let sum = Computed::from2(com1, com2, |a: &i32, b: &i32| -> i32 {
        a + b
    });

    let suma2 = sum.clone().map(|value: &i32| -> i32 {
        2 * value
    });

    let subscription = sum.subscribe(|sum: Rc<i32>| {
        println!("___Suma: {}___", sum);
    });

    let sub2 = suma2    .subscribe(|sum2: Rc<i32>| {
        println!("___Suma2: {}___", sum2);
    });

    val1.setValue(333);
    val2.setValue(888);

    println!("subscription off");

    subscription.off();
    sub2.off();

    val2.setValue(999);
}
