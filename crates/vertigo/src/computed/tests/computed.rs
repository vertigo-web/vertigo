use std::rc::Rc;

use crate::computed::{get_dependencies, Computed, DropResource, Value};
use crate::transaction;

use crate::computed::tests::box_value_version::SubscribeValueVer;
use crate::struct_mut::ValueMut;

#[test]
fn basic() {
    let value1: Value<i32> = Value::new(1);
    let value2: Value<i32> = Value::new(2);

    let sum: Computed<i32> = {
        let com1 = value1.to_computed();
        let com2 = value2.to_computed();

        Computed::from(move |context| -> i32 {
            let value1 = com1.get(context);
            let value2 = com2.get(context);

            value1 + value2
        })
    };

    let mut sum_value = SubscribeValueVer::new(sum);

    assert_eq!(sum_value.get(), (3, 1));

    value1.set(4);
    assert_eq!(sum_value.get(), (6, 2));

    value2.set(5);
    assert_eq!(sum_value.get(), (9, 3));

    sum_value.off();

    value2.set(99);
    assert_eq!(sum_value.get(), (9, 3));
}

#[test]
fn basic2() {
    let val1 = Value::new(4);
    let val2 = Value::new(5);

    let com1: Computed<i32> = val1.to_computed();
    let com2: Computed<i32> = val2.to_computed();

    let sum = Computed::from(move |context| {
        let a = com1.get(context);
        let b = com2.get(context);
        a + b
    });

    let sum2 = sum.map(|value: i32| -> i32 { 2 * (value) });

    let mut sum_box1 = SubscribeValueVer::new(sum);
    let mut sum_box2 = SubscribeValueVer::new(sum2);

    assert_eq!(sum_box1.get(), (9, 1));
    assert_eq!(sum_box2.get(), (18, 1));

    val1.set(111);

    assert_eq!(sum_box1.get(), (116, 2));
    assert_eq!(sum_box2.get(), (232, 2));

    val2.set(888);

    assert_eq!(sum_box1.get(), (999, 3));
    assert_eq!(sum_box2.get(), (1998, 3));

    sum_box1.off();
    sum_box2.off();

    val2.set(999);

    assert_eq!(sum_box1.get(), (999, 3));
    assert_eq!(sum_box2.get(), (1998, 3));
}

#[test]
fn pointers() {
    // pointer conversion

    fn foo1() -> i32 {
        1
    }

    fn foo2() -> i32 {
        2
    }

    fn foo3(_yy: i32) -> i32 {
        3
    }

    let pointer1: u64 = foo1 as *const () as u64;
    let pointer2: u64 = foo2 as *const () as u64;
    let pointer11: u64 = foo1 as *const () as u64;
    let pointer4: u64 = foo3 as *const () as u64;

    assert!(pointer1 != pointer2);
    assert!(pointer1 == pointer11);
    assert!(pointer1 != pointer4);
}

#[test]
fn test_subscription() {
    let val1 = Value::new(1);
    let val2 = Value::new(2);
    let val3 = Value::new(3);

    let com1: Computed<i32> = val1.to_computed();
    let com2: Computed<i32> = val2.to_computed();
    #[allow(unused_variables)]
    let com3: Computed<i32> = val3.to_computed();

    let sum = Computed::from(move |context| -> i32 {
        let value1 = com1.get(context);
        let value2 = com2.get(context);

        value1 + value2
    });

    let mut sum_value = SubscribeValueVer::new(sum);

    assert_eq!(sum_value.get(), (3, 1));
    val1.set(2);
    assert_eq!(sum_value.get(), (4, 2));
    val2.set(10);
    assert_eq!(sum_value.get(), (12, 3));
    val3.set(10);
    assert_eq!(sum_value.get(), (12, 3));
    val2.set(20);
    assert_eq!(sum_value.get(), (22, 4));

    sum_value.off();

    val1.set(2);
    assert_eq!(sum_value.get(), (22, 4));
    val1.set(2);
    assert_eq!(sum_value.get(), (22, 4));
    val2.set(2);
    assert_eq!(sum_value.get(), (22, 4));
    val3.set(2);
    assert_eq!(sum_value.get(), (22, 4));
}

#[test]
fn test_computed_cache() {
    let root = get_dependencies();

    assert_eq!(root.graph.connections.all_connections_len(), 0);

    {
        //a
        //b
        //c = a + b
        //d = c % 2;

        let a = Value::new(1);
        let b = Value::new(2);

        let c: Computed<u32> = {
            let a = a.clone();

            Computed::from(move |context| {
                let a_val = a.get(context);
                let b_val = b.get(context);

                a_val + b_val
            })
        };

        let d: Computed<bool> = {
            //is even
            let c = c.clone();
            Computed::from(move |context| -> bool {
                let c_value = c.get(context);

                c_value.is_multiple_of(2)
            })
        };

        let mut c = SubscribeValueVer::new(c);
        let mut d = SubscribeValueVer::new(d);

        assert_eq!(c.get(), (3, 1));
        assert_eq!(d.get(), (false, 1));

        a.set(2);

        assert_eq!(c.get(), (4, 2));
        assert_eq!(d.get(), (true, 2));

        a.set(2);

        assert_eq!(c.get(), (4, 2));
        assert_eq!(d.get(), (true, 2));

        a.set(4);

        assert_eq!(c.get(), (6, 3));
        assert_eq!(d.get(), (true, 2));

        assert_eq!(root.graph.connections.all_connections_len(), 5);

        c.off();
        d.off();

        assert_eq!(root.graph.connections.all_connections_len(), 0);
    }

    assert_eq!(root.graph.connections.all_connections_len(), 0);
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

    let root = get_dependencies();

    let a = Value::new(0);
    let b = Value::new(0);
    let c = Value::new(0);

    let d: Computed<u32> = {
        let a = a.clone();

        Computed::from(move |context| {
            let a_val = a.get(context);
            let b_val = b.get(context);

            a_val + b_val
        })
    };

    let e: Computed<u32> = {
        //is even
        let d = d.clone();
        let c = c.clone();
        Computed::from(move |context| -> u32 {
            let d_val = d.get(context);
            let c_val = c.get(context);

            d_val + c_val
        })
    };

    let mut d = SubscribeValueVer::new(d);
    let mut e = SubscribeValueVer::new(e);

    assert_eq!(d.get(), (0, 1));
    assert_eq!(e.get(), (0, 1));

    a.set(33);
    assert_eq!(d.get(), (33, 2));
    assert_eq!(e.get(), (33, 2));

    c.set(66);
    assert_eq!(d.get(), (33, 2));
    assert_eq!(e.get(), (99, 3));

    d.off();
    e.off();
    assert_eq!(root.graph.connections.all_connections_len(), 0);
}

#[test]
fn test_computed_new_value2() {
    #![allow(clippy::many_single_char_names)]

    let root = get_dependencies();

    let a = Value::new(0);
    let b = Value::new(0);

    let d: Computed<u32> = {
        let a = a.clone();
        let b = b.clone();
        Computed::from(move |context| a.get(context) + b.get(context))
    };

    let mut d = SubscribeValueVer::new(d);

    assert_eq!(d.get(), (0, 1));
    a.set(2);
    assert_eq!(d.get(), (2, 2));
    b.set(9);
    assert_eq!(d.get(), (11, 3));
    a.set(3);
    assert_eq!(d.get(), (12, 4));
    a.set(4);
    assert_eq!(d.get(), (13, 5));

    d.off();
    assert_eq!(root.graph.connections.all_connections_len(), 0);
}

#[test]
fn test_computed_switch_subscription() {
    #[derive(Clone, PartialEq)]
    enum Switch {
        Ver1,
        Ver2,
        Ver3,
    }

    //a, b, c

    let root = get_dependencies();

    let switch = Value::new(Switch::Ver1);
    let a = Value::new(0);
    let b = Value::new(0);
    let c = Value::new(0);

    let sum: Computed<u32> = {
        let switch = switch.clone();
        let a = a.clone();
        let b = b.clone();
        let c = c.clone();

        Computed::from(move |context| -> u32 {
            let switch_value = switch.get(context);

            match switch_value {
                Switch::Ver1 => a.get(context),
                Switch::Ver2 => {
                    let a_value = a.get(context);
                    let b_value = b.get(context);
                    a_value + b_value
                }
                Switch::Ver3 => {
                    let a_value = a.get(context);
                    let b_value = b.get(context);
                    let c_value = c.get(context);
                    a_value + b_value + c_value
                }
            }
        })
    };

    assert_eq!(root.graph.connections.all_connections_len(), 0);
    let mut sum = SubscribeValueVer::new(sum);
    assert_eq!(root.graph.connections.all_connections_len(), 3);

    assert_eq!(sum.get(), (0, 1));
    assert_eq!(root.graph.connections.all_connections_len(), 3);

    a.set(1);
    assert_eq!(sum.get(), (1, 2));
    assert_eq!(root.graph.connections.all_connections_len(), 3);

    b.set(1);
    assert_eq!(sum.get(), (1, 2));
    c.set(1);
    assert_eq!(sum.get(), (1, 2));

    a.set(0);
    b.set(0);
    c.set(0);
    assert_eq!(sum.get(), (0, 3));

    assert_eq!(root.graph.connections.all_connections_len(), 3);
    switch.set(Switch::Ver2);
    assert_eq!(root.graph.connections.all_connections_len(), 4);

    assert_eq!(sum.get(), (0, 3)); //no rerender

    a.set(1);

    assert_eq!(root.graph.connections.all_connections_len(), 4);

    assert_eq!(sum.get(), (1, 4));
    b.set(1);
    assert_eq!(sum.get(), (2, 5));
    c.set(1);
    assert_eq!(sum.get(), (2, 5));

    a.set(0);
    b.set(0);
    c.set(0);
    assert_eq!(sum.get(), (0, 7));

    switch.set(Switch::Ver3);
    assert_eq!(sum.get(), (0, 7)); //no rerender

    assert_eq!(root.graph.connections.all_connections_len(), 5);

    a.set(1);
    assert_eq!(sum.get(), (1, 8));
    b.set(1);
    assert_eq!(sum.get(), (2, 9));
    c.set(1);
    assert_eq!(sum.get(), (3, 10));

    root.transaction(|_| {
        a.set(0);
        b.set(0);
        c.set(0);
    });

    assert_eq!(sum.get(), (0, 11));

    sum.off();
    assert_eq!(root.graph.connections.all_connections_len(), 0);
}

#[test]
fn test_transaction() {
    let root = get_dependencies();
    assert_eq!(root.graph.connections.all_connections_len(), 0);

    let val1 = Value::new(1);
    let val2 = Value::new(2);

    let val3 = Computed::from({
        let val1 = val1.clone();
        let val2 = val2.clone();

        move |context| val1.get(context) + val2.get(context)
    });

    let mut val2sub = SubscribeValueVer::new(val3);

    assert_eq!(val2sub.get(), (3, 1));

    val1.set(444);

    assert_eq!(val2sub.get(), (446, 2));

    root.transaction(|context| {
        assert_eq!(val2sub.get(), (446, 2));
        val1.set(222);
        assert_eq!(val1.get(context), 222);
        assert_eq!(val2sub.get(), (446, 2));
        val2.set(333);
        assert_eq!(val2.get(context), 333);
        assert_eq!(val2sub.get(), (446, 2));
    });

    root.transaction(|context| {
        assert_eq!(val1.get(context), 222);
        assert_eq!(val2.get(context), 333);
    });

    assert_eq!(val2sub.get(), (555, 3));

    val2sub.off();
    assert_eq!(root.graph.connections.all_connections_len(), 0);
}

#[test]
#[allow(clippy::bool_assert_comparison)]
fn test_connect() {
    let is_subscribe = Rc::new(ValueMut::new(false));

    let value = Value::with_connect(10, {
        let is_subscribe = is_subscribe.clone();

        move |_value| {
            is_subscribe.set(true);

            DropResource::new({
                let is_subscribe = is_subscribe.clone();
                move || {
                    is_subscribe.set(false);
                }
            })
        }
    });

    assert_eq!(is_subscribe.get(), false);

    let current_value = Rc::new(ValueMut::new(0));

    let client = value.clone().subscribe({
        let current_value = current_value.clone();
        move |val| {
            current_value.set(val);
        }
    });

    assert_eq!(is_subscribe.get(), true);

    drop(client);

    assert_eq!(is_subscribe.get(), false);

    let client = value.subscribe({
        move |val| {
            current_value.set(val);
        }
    });

    assert_eq!(is_subscribe.get(), true);

    drop(client);

    assert_eq!(is_subscribe.get(), false);
}

#[test]
fn test_without_subscription() {
    let value = Value::new(2);

    let comp_2 = {
        let v = value.clone();
        Computed::from(move |context| v.get(context) * 2)
    };

    transaction(|context| {
        assert_eq!(comp_2.get(context), 4);
    });

    value.set(6);

    transaction(|context| {
        assert_eq!(comp_2.get(context), 12);
    });
}

#[test]
fn test_set_if_changed() {
    let value = Value::new(2);

    let value_com = value.to_computed();

    let value_com = value_com.map(|item| item);

    fn build(value: &Computed<i32>) -> (Rc<ValueMut<i32>>, DropResource) {
        let boxy = Rc::new(ValueMut::new(0));

        let router = Computed::from({
            let value = value.clone();

            move |context| value.get(context)
        });

        let router = Computed::from(move |context| router.get(context));

        let router = router.map(|item| item);

        let router = router.map(|item| item);

        let router = router.map(|item| item);

        let client = router.subscribe({
            let boxy = boxy.clone();
            move |sub_value| {
                println!("callback");
                boxy.set(sub_value);
            }
        });

        (boxy, client)
    }

    let (boxy, client) = build(&value_com);
    let (boxy2, client2) = build(&value_com);
    let (boxy3, client3) = build(&value_com);

    assert_eq!(boxy.get(), 2);

    value.set_force(3);
    assert_eq!(boxy.get(), 3);

    value.set(4);
    assert_eq!(boxy.get(), 4);
    value.set(4);
    assert_eq!(boxy.get(), 4);

    value.set(5);
    assert_eq!(boxy.get(), 5);
    value.set(5);
    assert_eq!(boxy.get(), 5);

    value.set_force(6);
    assert_eq!(boxy.get(), 6);
    assert_eq!(boxy2.get(), 6);
    assert_eq!(boxy3.get(), 6);

    drop(client);
    drop(client2);
    drop(client3);
}
