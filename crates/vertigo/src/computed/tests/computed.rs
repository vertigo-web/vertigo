use std::rc::Rc;

use crate::DropResource;
use crate::computed::{Computed, Dependencies, Value};

use crate::computed::tests::box_value_version::SubscribeValueVer;
use crate::struct_mut::ValueMut;

#[test]
fn basic() {
    use crate::computed::Dependencies;

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

    let mut sum_value = SubscribeValueVer::new(sum);

    assert_eq!(sum_value.get(), (3, 1));

    value1.set_value(4);
    assert_eq!(sum_value.get(), (6, 2));

    value2.set_value(5);
    assert_eq!(sum_value.get(), (9, 3));

    sum_value.off();

    value2.set_value(99);
    assert_eq!(sum_value.get(), (9, 3));
}

#[test]
fn basic2() {
    let root = Dependencies::default();

    let val1 = root.new_value(4);
    let val2 = root.new_value(5);

    let com1: Computed<i32> = val1.to_computed();
    let com2: Computed<i32> = val2.to_computed();

    let sum = root.from(move || {
        let a = com1.get_value();
        let b = com2.get_value();
        *a + *b
    });

    let suma2 = sum.clone().map_for_render(|value: &Computed<i32>| -> i32 {
        let value = value.get_value();
        2 * (*value)
    });

    let mut sum_box1 = SubscribeValueVer::new(sum);
    let mut sum_box2 = SubscribeValueVer::new(suma2);

    assert_eq!(sum_box1.get(), (9, 1));
    assert_eq!(sum_box2.get(), (18, 1));

    val1.set_value(111);

    assert_eq!(sum_box1.get(), (116, 2));
    assert_eq!(sum_box2.get(), (232, 2));

    val2.set_value(888);

    assert_eq!(sum_box1.get(), (999, 3));
    assert_eq!(sum_box2.get(), (1998, 3));

    println!("subscription off");

    sum_box1.off();
    sum_box2.off();

    val2.set_value(999);

    assert_eq!(sum_box1.get(), (999, 3));
    assert_eq!(sum_box2.get(), (1998, 3));
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
    let root = Dependencies::default();

    let val1 = root.new_value(1);
    let val2 = root.new_value(2);
    let val3 = root.new_value(3);

    let com1: Computed<i32> = val1.to_computed();
    let com2: Computed<i32> = val2.to_computed();
    #[allow(unused_variables)]
    let com3: Computed<i32> = val3.to_computed();

    let sum = root.from(move || -> i32 {
        let value1 = com1.get_value();
        let value2 = com2.get_value();

        *value1 + *value2
    });

    let mut sum_value = SubscribeValueVer::new(sum);

    assert_eq!(sum_value.get(), (3, 1));
    val1.set_value(2);
    assert_eq!(sum_value.get(), (4, 2));
    val2.set_value(10);
    assert_eq!(sum_value.get(), (12, 3));
    val3.set_value(10);
    assert_eq!(sum_value.get(), (12, 3));
    val2.set_value(20);
    assert_eq!(sum_value.get(), (22, 4));

    sum_value.off();

    val1.set_value(2);
    assert_eq!(sum_value.get(), (22, 4));
    val1.set_value(2);
    assert_eq!(sum_value.get(), (22, 4));
    val2.set_value(2);
    assert_eq!(sum_value.get(), (22, 4));
    val3.set_value(2);
    assert_eq!(sum_value.get(), (22, 4));
}

#[test]
fn test_computed_cache() {
    let root = Dependencies::default();

    assert_eq!(root.all_connections_len(), 0);

    {
        //a
        //b
        //c = a + b
        //d = c % 2;

        let a = root.new_value(1);
        let b = root.new_value(2);

        let c: Computed<u32> = {
            let a = a.clone();

            root.from(move || {
                let a_val = a.get_value();
                let b_val = b.get_value();

                *a_val + *b_val
            })
        };

        let d: Computed<bool> = {
            //is even
            let c = c.clone();
            root.from(move || -> bool {
                let c_value = c.get_value();

                *c_value % 2 == 0
            })
        };

        let mut c = SubscribeValueVer::new(c);
        let mut d = SubscribeValueVer::new(d);

        assert_eq!(c.get(), (3, 1));
        assert_eq!(d.get(), (false, 1));

        a.set_value(2);

        assert_eq!(c.get(), (4, 2));
        assert_eq!(d.get(), (true, 2));

        a.set_value(2);

        assert_eq!(c.get(), (4, 2));
        assert_eq!(d.get(), (true, 2));

        a.set_value(4);

        assert_eq!(c.get(), (6, 3));
        assert_eq!(d.get(), (true, 2));

        assert_eq!(root.all_connections_len(), 5);

        c.off();
        d.off();

        assert_eq!(root.all_connections_len(), 0);
    }

    assert_eq!(root.all_connections_len(), 0);
}

#[test]
fn test_computed_new_value() {
    /*
        a
        b
        c
        d = a + b
        e = d + c
    */

    #![allow(clippy::many_single_char_names)]

    let root = Dependencies::default();

    let a = root.new_value(0);
    let b = root.new_value(0);
    let c = root.new_value(0);

    let d: Computed<u32> = {
        let a = a.clone();

        root.from(move || {
            let a_val = a.get_value();
            let b_val = b.get_value();

            *a_val + *b_val
        })
    };

    let e: Computed<u32> = {
        //is even
        let d = d.clone();
        let c = c.clone();
        root.from(move || -> u32 {
            let d_val = d.get_value();
            let c_val = c.get_value();

            *d_val + *c_val
        })
    };

    let mut d = SubscribeValueVer::new(d);
    let mut e = SubscribeValueVer::new(e);

    assert_eq!(d.get(), (0, 1));
    assert_eq!(e.get(), (0, 1));

    a.set_value(33);
    assert_eq!(d.get(), (33, 2));
    assert_eq!(e.get(), (33, 2));

    c.set_value(66);
    assert_eq!(d.get(), (33, 2));
    assert_eq!(e.get(), (99, 3));

    d.off();
    e.off();
    assert_eq!(root.all_connections_len(), 0);
}

#[test]
fn test_computed_switch_subscription() {
    #[derive(PartialEq)]
    enum Switch {
        Ver1,
        Ver2,
        Ver3,
    }

    //a, b, c

    let root = Dependencies::default();

    let switch = root.new_value(Switch::Ver1);
    let a = root.new_value(0);
    let b = root.new_value(0);
    let c = root.new_value(0);

    // println!("s {:?}", switch.id());
    // println!("a {:?}", a.id());
    // println!("b {:?}", b.id());
    // println!("c {:?}", c.id());

    let sum: Computed<u32> = {
        let switch = switch.clone();
        let a = a.clone();
        let b = b.clone();
        let c = c.clone();

        root.from(move || -> u32 {
            let switch_value = switch.get_value();

            match *switch_value {
                Switch::Ver1 => {
                    let a_value = a.get_value();
                    *a_value
                }
                Switch::Ver2 => {
                    let a_value = a.get_value();
                    let b_value = b.get_value();
                    *a_value + *b_value
                }
                Switch::Ver3 => {
                    let a_value = a.get_value();
                    let b_value = b.get_value();
                    let c_value = c.get_value();
                    *a_value + *b_value + *c_value
                }
            }
        })
    };

    assert_eq!(root.all_connections_len(), 0);
    let mut sum = SubscribeValueVer::new(sum);
    assert_eq!(root.all_connections_len(), 3);

    assert_eq!(sum.get(), (0, 1));
    assert_eq!(root.all_connections_len(), 3);

    a.set_value(1);
    assert_eq!(sum.get(), (1, 2));
    assert_eq!(root.all_connections_len(), 3);

    b.set_value(1);
    assert_eq!(sum.get(), (1, 2));
    c.set_value(1);
    assert_eq!(sum.get(), (1, 2));

    a.set_value(0);
    b.set_value(0);
    c.set_value(0);
    assert_eq!(sum.get(), (0, 3));

    assert_eq!(root.all_connections_len(), 3);
    switch.set_value(Switch::Ver2);
    assert_eq!(root.all_connections_len(), 4);

    assert_eq!(sum.get(), (0, 3)); //no rerender

    a.set_value(1);

    assert_eq!(root.all_connections_len(), 4);

    assert_eq!(sum.get(), (1, 4));
    b.set_value(1);
    assert_eq!(sum.get(), (2, 5));
    c.set_value(1);
    assert_eq!(sum.get(), (2, 5));

    a.set_value(0);
    b.set_value(0);
    c.set_value(0);
    assert_eq!(sum.get(), (0, 7));

    switch.set_value(Switch::Ver3);
    assert_eq!(sum.get(), (0, 7)); //no rerender

    assert_eq!(root.all_connections_len(), 5);

    a.set_value(1);
    assert_eq!(sum.get(), (1, 8));
    b.set_value(1);
    assert_eq!(sum.get(), (2, 9));
    c.set_value(1);
    assert_eq!(sum.get(), (3, 10));

    root.transaction(|| {
        a.set_value(0);
        b.set_value(0);
        c.set_value(0);
    });

    assert_eq!(sum.get(), (0, 11));

    sum.off();
    assert_eq!(root.all_connections_len(), 0);
}


#[test]
fn test_connect() {

    let root = Dependencies::default();

    let is_subscribe = Rc::new(ValueMut::new(false));

    let value = root.new_with_connect(10, {
        let is_subscribe = is_subscribe.clone();

        move |_value| {
            is_subscribe.set(true);

            Box::new(DropResource::new({
                let is_subscribe = is_subscribe.clone();
                move || {
                    is_subscribe.set(false);
                }
            }))
        }
    });

    assert_eq!(is_subscribe.get(), false);

    let current_value = Rc::new(ValueMut::new(0));

    println!("Subskrybcja 1 {:?}", value.id());

    let client = value.clone().subscribe({
        let current_value = current_value.clone();
        move |val| {
            current_value.set(*val);
        }
    });

    println!("Subskrybcja 2");

    assert_eq!(is_subscribe.get(), true);

    drop(client);

    assert_eq!(is_subscribe.get(), false);

    println!("Subskrybcja 3 {:?}", value.id());

    let client = value.subscribe({
        let current_value = current_value.clone();
        move |val| {
            current_value.set(*val);
        }
    });

    assert_eq!(is_subscribe.get(), true);

    drop(client);

    assert_eq!(is_subscribe.get(), false);

    println!("Subskrybcja 4");
}