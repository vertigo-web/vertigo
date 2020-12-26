
use std::rc::Rc;

use crate::computed::{
    Computed,
    Value
};

#[test]
fn basic() {
    use crate::computed::{
        Dependencies,
    };
    use crate::computed::tests::box_value::BoxValue;

    let root: Dependencies = Dependencies::default();

    let value1: Value<i32> = root.new_value(1);
    let value2: Value<i32> = root.new_value(2);

    let sum: Computed<i32> = {
        let com1 = value1.to_computed();
        let com2 = value2.to_computed();

        root.from(move || -> i32 {
            let value1 = com1.get_value();
            let value2 = com2.get_value();

            *value1 + *value2
        })
    };

    let sum_value: BoxValue<Option<i32>> = BoxValue::new(None);

    assert_eq!(sum_value.get(), None);

    let sub = {
        let sum_value = sum_value.clone();
        sum.subscribe(move |value| {
            sum_value.set(Some(*value));
        })
    };

    assert_eq!(sum_value.get(), Some(3));

    value1.set_value(4);
    assert_eq!(sum_value.get(), Some(6));

    value2.set_value(5);
    assert_eq!(sum_value.get(), Some(9));

    sub.off();

    value2.set_value(99);
    assert_eq!(sum_value.get(), Some(103));
}

#[test]
fn basic2() {
    use crate::computed::{
        Dependencies,
        Computed,
    };
    use crate::computed::tests::box_value::BoxValue;

    let root = Dependencies::default();

    let val1 = root.new_value(4);
    let val2 = root.new_value(5);

    let com1: Computed<i32> = val1.to_computed();
    let com2: Computed<i32> = val2.to_computed();

    let sum_box1: BoxValue<Option<i32>> = BoxValue::new(None);
    let sum_box2: BoxValue<Option<i32>> = BoxValue::new(None);

    let sum = Computed::from2(com1, com2, |a: Rc<i32>, b: Rc<i32>| -> i32 {
        *a + *b
    });

    let suma2 = sum.clone().map_for_render(|value: &Computed<i32>| -> i32 {
        let value = value.get_value();
        2 * (*value)
    });

    let subscription = {
        let sum_box1 = sum_box1.clone();

        sum.subscribe(move |sum: &i32| {
            println!("___Suma: {}___", sum);
            sum_box1.set(Some(*sum));
        })
    };

    let sub2 = {
        let sum_box2 = sum_box2.clone();

        suma2.subscribe(move |sum2: &i32| {
            println!("___Suma2: {}___", sum2);
            sum_box2.set(Some(*sum2));
        })
    };

    assert_eq!(sum_box1.get(), Some(9));
    assert_eq!(sum_box2.get(), Some(18));

    val1.set_value(111);

    assert_eq!(sum_box1.get(), Some(116));
    assert_eq!(sum_box2.get(), Some(232));

    val2.set_value(888);

    assert_eq!(sum_box1.get(), Some(999));
    assert_eq!(sum_box2.get(), Some(1998));

    println!("subscription off");

    subscription.off();
    sub2.off();

    val2.set_value(999);

    assert_eq!(sum_box1.get(), Some(1110));
    assert_eq!(sum_box2.get(), Some(2220));

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

#[test]
fn test_subscription() {
    use crate::computed::{
        Dependencies,
        Computed,
    };
    use crate::computed::tests::box_value::BoxValue;

    let root = Dependencies::default();

    let val1 = root.new_value(1);
    let val2 = root.new_value(2);
    let val3 = root.new_value(3);

    let com1: Computed<i32> = val1.to_computed();
    let com2: Computed<i32> = val2.to_computed();
    #[allow(unused_variables)]
    let com3: Computed<i32> = val3.to_computed();

    #[derive(Copy, Clone, Debug, PartialEq)]
    struct Sum {
        version: u32,
        value: Option<i32>
    }
    impl Sum {
        fn new(version: u32, value: Option<i32>) -> Sum {
            Sum {
                version,
                value,
            }
        }
    }

    let sum_value: BoxValue<Sum> = BoxValue::new(Sum::new(1, None));

    assert_eq!(sum_value.get(), Sum::new(1, None));

    let sum = root.from(move || -> i32 {
        let value1 = com1.get_value();
        let value2 = com2.get_value();

        *value1 + *value2
    });

    let sub = {
        let sum_value = sum_value.clone();
        sum.subscribe(move |value| {
            sum_value.change(move |state| {
                state.version += 1;
                state.value = Some(*value);
            });
        })
    };

    assert_eq!(sum_value.get(), Sum::new(2, Some(3)));
    val1.set_value(2);
    assert_eq!(sum_value.get(), Sum::new(3, Some(4)));
    val2.set_value(10);
    assert_eq!(sum_value.get(), Sum::new(4, Some(12)));
    val3.set_value(10);
    assert_eq!(sum_value.get(), Sum::new(4, Some(12)));
    val2.set_value(20);
    assert_eq!(sum_value.get(), Sum::new(5, Some(22)));

    sub.off();

    val1.set_value(2);
    assert_eq!(sum_value.get(), Sum::new(5, Some(22)));
    val1.set_value(2);
    assert_eq!(sum_value.get(), Sum::new(5, Some(22)));
    val2.set_value(2);
    assert_eq!(sum_value.get(), Sum::new(6, Some(4)));
    val3.set_value(2);
    assert_eq!(sum_value.get(), Sum::new(6, Some(4)));
}

