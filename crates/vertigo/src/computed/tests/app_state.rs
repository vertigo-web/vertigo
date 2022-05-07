use crate::computed::{
    Computed, Value,
    tests::box_value_version::SubscribeValueVer,
};

struct AppState {
    value1: Value<i32>,
    value2: Value<i32>,
    value3: Value<i32>,
    sum: Computed<i32>,
}

impl AppState {
    pub fn new() -> std::rc::Rc<AppState> {
        let value1 = Value::new(1);
        let value2 = Value::new(2);
        let value3 = Value::new(3);

        let sum = {
            let com1 = value1.to_computed();
            let com2 = value2.to_computed();
            let com3 = value3.to_computed();

            Computed::from(move || {
                let val1 = com1.get_value();
                let val2 = com2.get_value();
                let val3 = com3.get_value();

                *val1 + *val2 + *val3
            })
        };

        std::rc::Rc::new(AppState {
            value1,
            value2,
            value3,
            sum,
        })
    }
}

#[test]
fn test_app_state() {
    let app_state = AppState::new();

    let sum3 = {
        let app_state = app_state.clone();

        Computed::from(move || -> i32 {
            let val1 = app_state.value1.get_value();
            let val3 = app_state.value3.get_value();

            *val1 + *val3
        })
    };

    let mut sum3_box = SubscribeValueVer::new(sum3);

    assert_eq!(sum3_box.get(), (4, 1)); // 1 _ 3

    app_state.value1.set_value(2); // 2 _ 3

    assert_eq!(sum3_box.get(), (5, 2));

    app_state.value1.set_value(3); // 3 _ 3
    assert_eq!(sum3_box.get(), (6, 3));

    app_state.value2.set_value(4); // 3 _ 3
    assert_eq!(sum3_box.get(), (6, 3));

    app_state.value2.set_value(5); // 3 _ 3
    assert_eq!(sum3_box.get(), (6, 3));

    app_state.value3.set_value(6); // 3 _ 6
    assert_eq!(sum3_box.get(), (9, 4));

    app_state.value3.set_value(7); // 3 _ 7
    assert_eq!(sum3_box.get(), (10, 5));

    sum3_box.off();

    app_state.value3.set_value(8);

    assert_eq!(sum3_box.get(), (10, 5));

    let mut sum_total = SubscribeValueVer::new(app_state.sum.clone());

    assert_eq!((sum3_box.get(), sum_total.get()), ((10, 5), (16, 1)));

    app_state.value1.set_value(2);
    assert_eq!((sum3_box.get(), sum_total.get()), ((10, 5), (15, 2)));

    app_state.value2.set_value(3);
    assert_eq!((sum3_box.get(), sum_total.get()), ((10, 5), (13, 3)));

    app_state.value3.set_value(4);
    assert_eq!((sum3_box.get(), sum_total.get()), ((10, 5), (9, 4)));

    app_state.value3.set_value(4);
    assert_eq!((sum3_box.get(), sum_total.get()), ((10, 5), (9, 4)));

    sum_total.off();
}
