

#[test]
fn basic() {
    use crate::lib::{
        Dependencies::Dependencies,
        BoxValue::BoxValue,
    };

    let root = Dependencies::new();

    let value1 = root.newValue(1);
    let value2 = root.newValue(2);

    let sum = {
        let com1 = value1.toComputed();
        let com2 = value2.toComputed();

        root.from(move || -> i32 {
            let value1 = com1.getValue();
            let value2 = com2.getValue();

            *value1 + *value2
        })
    };

    let sumValue: BoxValue<Option<i32>> = BoxValue::new(None);

    assert_eq!(sumValue.get(), None);

    let sub = {
        let sumValue = sumValue.clone();
        sum.subscribe(move |value| {
            sumValue.set(Some(*value));
        })
    };

    assert_eq!(sumValue.get(), Some(3));

    value1.setValue(4);
    assert_eq!(sumValue.get(), Some(6));

    value2.setValue(5);
    assert_eq!(sumValue.get(), Some(9));

    sub.off();

    value2.setValue(99);
    assert_eq!(sumValue.get(), Some(9));
}

#[test]
fn basic2() {
    use crate::lib::{
        Dependencies::Dependencies,
        Computed::Computed,
        BoxValue::BoxValue,
    };

    let root = Dependencies::new();

    let val1 = root.newValue(4);
    let val2 = root.newValue(5);

    let com1: Computed<i32> = val1.toComputed();
    let com2: Computed<i32> = val2.toComputed();

    let sumBox1: BoxValue<Option<i32>> = BoxValue::new(None);
    let sumBox2: BoxValue<Option<i32>> = BoxValue::new(None);

    let sum = Computed::from2(com1, com2, |a: &i32, b: &i32| -> i32 {
        a + b
    });

    let suma2 = sum.clone().map(|value: &i32| -> i32 {
        2 * value
    });

    let subscription = {
        let sumBox1 = sumBox1.clone();

        sum.subscribe(move |sum: &i32| {
            println!("___Suma: {}___", sum);
            sumBox1.set(Some(*sum));
        })
    };

    let sub2 = {
        let sumBox2 = sumBox2.clone();

        suma2.subscribe(move |sum2: &i32| {
            println!("___Suma2: {}___", sum2);
            sumBox2.set(Some(*sum2));
        })
    };

    assert_eq!(sumBox1.get(), Some(9));
    assert_eq!(sumBox2.get(), Some(18));

    val1.setValue(111);

    assert_eq!(sumBox1.get(), Some(116));
    assert_eq!(sumBox2.get(), Some(232));

    val2.setValue(888);

    assert_eq!(sumBox1.get(), Some(999));
    assert_eq!(sumBox2.get(), Some(1998));

    println!("subscription off");

    subscription.off();
    sub2.off();

    val2.setValue(999);

    assert_eq!(sumBox1.get(), Some(999));
    assert_eq!(sumBox2.get(), Some(1998));

}

#[test]
fn pointers() {
    
    //konwertowanie do wskaźnika

    fn foo1() -> i32 {
        0
    }
    
    fn foo2() -> i32 {
        0
    }

    fn foo3(_yy: i32) -> i32 {
        0
    }
    //println!("aaa {}", std::mem::size_of<* ()>);

    // let aa = std::mem::size_of_val(&foo1);
    // println!("RRRRR {}", aa);
    // println!("RRRRR {}", std::mem::size_of_val(&aa));


    let pointer1: u64 = foo1 as *const () as u64;
    let pointer2: u64 = foo2 as *const () as u64;
    let pointer11: u64 = foo1 as *const () as u64;
    let pointer4: u64 = foo3 as *const () as u64;

    assert_eq!(pointer1 == pointer2, false);
    assert_eq!(pointer1 == pointer11, true);
    assert_eq!(pointer1 == pointer4, false);


    // println!("gg1 {:x}", gg1);
    // println!("gg2 {:x}", gg2);
    // println!("gg3 {:x}", gg3);
    // println!("gg4 {:x}", gg4);

    // println!("tt1 {:x}", tt1);
    // println!("tt2 {:x}", tt2);
    // println!("tt3 {:x}", tt3);
    // println!("tt4 {:x}", tt4);

    // let bb1: u32 = foo1 as u32;
    // let bb2: u32 = foo2 as u32;
    // let bb3: u32 = foo1 as u32;
    // let bb4: u32 = foo3 as u32;

    // println!("bb1 {:x}", bb1);
    // println!("bb2 {:x}", bb2);
    // println!("bb3 {:x}", bb3);
    // println!("bb4 {:x}", bb4);

    // let cc1: u128 = foo1 as u128;
    // let cc2: u128 = foo2 as u128;
    // let cc3: u128 = foo1 as u128;
    // let cc4: u128 = foo3 as u128;

    // println!("cc1 {:x}", cc1);
    // println!("cc2 {:x}", cc2);
    // println!("cc3 {:x}", cc3);
    // println!("cc4 {:x}", cc4);
}